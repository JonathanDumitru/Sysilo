package tunnel

import (
	"context"
	"io"
	"sync"
	"time"

	agentv1 "github.com/sysilo/sysilo/proto/agent/v1"
	"github.com/sysilo/sysilo/services/agent-gateway/internal/config"
	"github.com/sysilo/sysilo/services/agent-gateway/internal/kafka"
	"github.com/sysilo/sysilo/services/agent-gateway/internal/registry"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// Server implements the agent gRPC service
type Server struct {
	agentv1.UnimplementedAgentServiceServer
	logger   *zap.Logger
	config   *config.Config
	registry *registry.Registry
	producer *kafka.Producer // optional Kafka producer for forwarding results/logs

	// Active streams for sending tasks to agents
	streams   map[string]*agentStream // agentID -> stream
	streamsMu sync.RWMutex
}

// agentStream wraps a connected agent's stream
type agentStream struct {
	stream agentv1.AgentService_ConnectServer
	sendMu sync.Mutex
	agent  *registry.Agent
}

// NewServer creates a new tunnel server
func NewServer(logger *zap.Logger, cfg *config.Config, reg *registry.Registry, producer *kafka.Producer) *Server {
	return &Server{
		logger:   logger.Named("tunnel"),
		config:   cfg,
		registry: reg,
		producer: producer,
		streams:  make(map[string]*agentStream),
	}
}

// Register registers the server with a gRPC server
func (s *Server) Register(grpcServer *grpc.Server) {
	agentv1.RegisterAgentServiceServer(grpcServer, s)
}

// Connect handles the bidirectional streaming connection from agents
func (s *Server) Connect(stream agentv1.AgentService_ConnectServer) error {
	ctx := stream.Context()

	// Wait for registration message
	msg, err := stream.Recv()
	if err != nil {
		return status.Errorf(codes.Internal, "failed to receive registration: %v", err)
	}

	reg, ok := msg.Message.(*agentv1.AgentMessage_Registration)
	if !ok {
		return status.Error(codes.InvalidArgument, "first message must be registration")
	}

	// Validate registration
	if reg.Registration.AgentId == "" {
		return status.Error(codes.InvalidArgument, "agent_id is required")
	}
	if reg.Registration.TenantId == "" {
		return status.Error(codes.InvalidArgument, "tenant_id is required")
	}

	// Authorization is handled via mTLS certificate validation (when TLS is enabled)
	// or by validating the agent's registration token against the API gateway.
	// For production: Add call to API gateway to verify tenant/agent authorization
	// Example: s.apiClient.VerifyAgent(ctx, reg.Registration.TenantId, reg.Registration.AgentId)

	agent := &registry.Agent{
		ID:       reg.Registration.AgentId,
		TenantID: reg.Registration.TenantId,
		Name:     reg.Registration.AgentName,
		Version:  reg.Registration.Version,
		Labels:   reg.Registration.Labels,
	}

	if reg.Registration.Capabilities != nil {
		agent.Capabilities = registry.AgentCapabilities{
			SupportedAdapters:  reg.Registration.Capabilities.SupportedAdapters,
			MaxConcurrentTasks: int(reg.Registration.Capabilities.MaxConcurrentTasks),
			SupportsStreaming:  reg.Registration.Capabilities.SupportsStreaming,
		}
	}

	// Register the agent
	s.registry.Register(agent)

	// Store the stream
	as := &agentStream{
		stream: stream,
		agent:  agent,
	}
	s.streamsMu.Lock()
	s.streams[agent.ID] = as
	s.streamsMu.Unlock()

	// Send registration acknowledgment
	ack := &agentv1.GatewayMessage{
		Message: &agentv1.GatewayMessage_RegistrationAck{
			RegistrationAck: &agentv1.RegistrationAck{
				Success:                  true,
				Message:                  "Registration successful",
				HeartbeatIntervalSeconds: int32(s.config.Server.HeartbeatTimeout / 3),
			},
		},
	}
	if err := stream.Send(ack); err != nil {
		s.cleanup(agent.ID)
		return status.Errorf(codes.Internal, "failed to send ack: %v", err)
	}

	s.logger.Info("Agent connected",
		zap.String("agent_id", agent.ID),
		zap.String("tenant_id", agent.TenantID),
		zap.String("name", agent.Name),
		zap.String("version", agent.Version),
	)

	// Process incoming messages
	if err := s.receiveLoop(ctx, agent.ID, stream); err != nil {
		s.logger.Error("Receive loop error",
			zap.String("agent_id", agent.ID),
			zap.Error(err),
		)
	}

	// Cleanup on disconnect
	s.cleanup(agent.ID)

	s.logger.Info("Agent disconnected",
		zap.String("agent_id", agent.ID),
		zap.String("tenant_id", agent.TenantID),
	)

	return nil
}

// receiveLoop processes incoming messages from an agent
func (s *Server) receiveLoop(ctx context.Context, agentID string, stream agentv1.AgentService_ConnectServer) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		msg, err := stream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}

		switch m := msg.Message.(type) {
		case *agentv1.AgentMessage_Heartbeat:
			s.handleHeartbeat(agentID, m.Heartbeat)
		case *agentv1.AgentMessage_TaskResult:
			s.handleTaskResult(agentID, m.TaskResult)
		case *agentv1.AgentMessage_Logs:
			s.handleLogs(agentID, m.Logs)
		}
	}
}

// handleHeartbeat processes a heartbeat from an agent
func (s *Server) handleHeartbeat(agentID string, heartbeat *agentv1.AgentHeartbeat) {
	runningTasks := make([]string, 0, len(heartbeat.RunningTasks))
	for _, task := range heartbeat.RunningTasks {
		runningTasks = append(runningTasks, task.TaskId)
	}

	s.registry.UpdateHeartbeat(agentID, runningTasks)

	s.logger.Debug("Received heartbeat",
		zap.String("agent_id", agentID),
		zap.Int("running_tasks", len(runningTasks)),
	)
}

// handleTaskResult processes a task result from an agent
func (s *Server) handleTaskResult(agentID string, result *agentv1.TaskResult) {
	// Get agent info for tenant context
	s.streamsMu.RLock()
	as, ok := s.streams[agentID]
	s.streamsMu.RUnlock()

	var tenantID string
	if ok && as.agent != nil {
		tenantID = as.agent.TenantID
	}

	s.logger.Info("Received task result",
		zap.String("agent_id", agentID),
		zap.String("tenant_id", tenantID),
		zap.String("task_id", result.TaskId),
		zap.String("status", result.Status.String()),
	)

	// Forward result to integration service via Kafka
	if s.producer != nil {
		resultMsg := s.convertTaskResultToKafkaMessage(agentID, tenantID, result)
		if err := s.producer.PublishResult(resultMsg); err != nil {
			s.logger.Error("Failed to publish task result to Kafka",
				zap.String("task_id", result.TaskId),
				zap.Error(err),
			)
		}
	}
}

// handleLogs processes log entries from an agent
func (s *Server) handleLogs(agentID string, logs *agentv1.LogBatch) {
	// Get agent info for tenant context
	s.streamsMu.RLock()
	as, ok := s.streams[agentID]
	s.streamsMu.RUnlock()

	var tenantID string
	if ok && as.agent != nil {
		tenantID = as.agent.TenantID
	}

	s.logger.Debug("Received logs",
		zap.String("agent_id", agentID),
		zap.String("tenant_id", tenantID),
		zap.Int("entries", len(logs.Entries)),
	)

	// Forward logs to Kafka for log aggregation
	if s.producer != nil {
		logMsg := s.convertLogBatchToKafkaMessage(agentID, tenantID, logs)
		if err := s.producer.PublishLogs(logMsg); err != nil {
			s.logger.Error("Failed to publish logs to Kafka",
				zap.String("agent_id", agentID),
				zap.Error(err),
			)
		}
	}
}

// SendTask sends a task to a specific agent
func (s *Server) SendTask(agentID string, task *agentv1.Task) error {
	s.streamsMu.RLock()
	as, ok := s.streams[agentID]
	s.streamsMu.RUnlock()

	if !ok {
		return status.Error(codes.NotFound, "agent not connected")
	}

	msg := &agentv1.GatewayMessage{
		Message: &agentv1.GatewayMessage_Task{
			Task: task,
		},
	}

	as.sendMu.Lock()
	defer as.sendMu.Unlock()

	return as.stream.Send(msg)
}

// SendCommand sends a command to a specific agent
func (s *Server) SendCommand(agentID string, cmd *agentv1.AgentCommand) error {
	s.streamsMu.RLock()
	as, ok := s.streams[agentID]
	s.streamsMu.RUnlock()

	if !ok {
		return status.Error(codes.NotFound, "agent not connected")
	}

	msg := &agentv1.GatewayMessage{
		Message: &agentv1.GatewayMessage_Command{
			Command: cmd,
		},
	}

	as.sendMu.Lock()
	defer as.sendMu.Unlock()

	return as.stream.Send(msg)
}

// cleanup removes an agent from the streams map and registry
func (s *Server) cleanup(agentID string) {
	s.streamsMu.Lock()
	delete(s.streams, agentID)
	s.streamsMu.Unlock()

	s.registry.Unregister(agentID)
}

// ReportTaskResult handles direct task result reporting (non-streaming)
func (s *Server) ReportTaskResult(ctx context.Context, result *agentv1.TaskResult) (*agentv1.TaskResultAck, error) {
	// Get tenant from agent registry
	agent, ok := s.registry.Get(result.AgentId)
	var tenantID string
	if ok && agent != nil {
		tenantID = agent.TenantID
	}

	s.logger.Info("Received task result (direct)",
		zap.String("task_id", result.TaskId),
		zap.String("agent_id", result.AgentId),
		zap.String("tenant_id", tenantID),
		zap.String("status", result.Status.String()),
	)

	// Forward result to Kafka
	if s.producer != nil {
		resultMsg := s.convertTaskResultToKafkaMessage(result.AgentId, tenantID, result)
		if err := s.producer.PublishResult(resultMsg); err != nil {
			s.logger.Error("Failed to publish task result to Kafka",
				zap.String("task_id", result.TaskId),
				zap.Error(err),
			)
			return &agentv1.TaskResultAck{
				Success: false,
				Message: "Failed to process result",
			}, nil
		}
	}

	return &agentv1.TaskResultAck{
		Success: true,
		Message: "Result received",
	}, nil
}

// convertTaskResultToKafkaMessage converts a protobuf TaskResult to Kafka ResultMessage
func (s *Server) convertTaskResultToKafkaMessage(agentID, tenantID string, result *agentv1.TaskResult) *kafka.ResultMessage {
	msg := &kafka.ResultMessage{
		TaskID:   result.TaskId,
		AgentID:  agentID,
		TenantID: tenantID,
		Status:   result.Status.String(),
		Metrics:  make(map[string]interface{}),
	}

	// Convert timestamps
	if result.StartedAt != nil {
		msg.StartedAt = result.StartedAt.AsTime()
	}
	if result.CompletedAt != nil {
		msg.CompletedAt = result.CompletedAt.AsTime()
	}

	// Convert error if present
	if result.Error != nil {
		msg.Error = &kafka.ErrorDetail{
			Code:      result.Error.Code,
			Message:   result.Error.Message,
			Details:   result.Error.Details,
			Retryable: result.Error.Retryable,
		}
	}

	// Convert metrics if present
	if result.Metrics != nil {
		msg.Metrics["records_read"] = result.Metrics.RecordsRead
		msg.Metrics["records_written"] = result.Metrics.RecordsWritten
		msg.Metrics["bytes_processed"] = result.Metrics.BytesProcessed
		msg.Metrics["duration_ms"] = result.Metrics.DurationMs
	}

	return msg
}

// convertLogBatchToKafkaMessage converts a protobuf LogBatch to Kafka LogMessage
func (s *Server) convertLogBatchToKafkaMessage(agentID, tenantID string, batch *agentv1.LogBatch) *kafka.LogMessage {
	entries := make([]kafka.LogEntry, 0, len(batch.Entries))

	for _, entry := range batch.Entries {
		logEntry := kafka.LogEntry{
			Level:   entry.Level.String(),
			Message: entry.Message,
			TaskID:  entry.TaskId,
			Fields:  entry.Fields,
		}
		if entry.Timestamp != nil {
			logEntry.Timestamp = entry.Timestamp.AsTime()
		}
		entries = append(entries, logEntry)
	}

	return &kafka.LogMessage{
		AgentID:   agentID,
		TenantID:  tenantID,
		Entries:   entries,
		Timestamp: time.Now(),
	}
}
