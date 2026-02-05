package tunnel

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"io"
	"os"
	"runtime"
	"sync"
	"syscall"
	"time"

	agentv1 "github.com/sysilo/sysilo/proto/agent/v1"

	"github.com/sysilo/sysilo/agent/internal/config"
	"github.com/sysilo/sysilo/agent/internal/executor"
	"github.com/sysilo/sysilo/agent/pkg/version"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/keepalive"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// Client manages the connection to the agent gateway
type Client struct {
	logger   *zap.Logger
	config   *config.Config
	executor *executor.Executor

	conn   *grpc.ClientConn
	client agentv1.AgentServiceClient
	stream agentv1.AgentService_ConnectClient

	mu        sync.RWMutex
	closed    bool
	closeCh   chan struct{}
	restartCh chan RestartRequest
}

// RestartRequest carries restart command parameters
type RestartRequest struct {
	Reason       string
	DelaySeconds int32
}

// NewClient creates a new tunnel client
func NewClient(logger *zap.Logger, cfg *config.Config, exec *executor.Executor) (*Client, error) {
	return &Client{
		logger:    logger.Named("tunnel"),
		config:    cfg,
		executor:  exec,
		closeCh:   make(chan struct{}),
		restartCh: make(chan RestartRequest, 1),
	}, nil
}

// RestartCh returns a channel that receives restart requests from the gateway
func (c *Client) RestartCh() <-chan RestartRequest {
	return c.restartCh
}

// Connect establishes and maintains the connection to the gateway
func (c *Client) Connect(ctx context.Context) error {
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-c.closeCh:
			return nil
		default:
		}

		if err := c.connectOnce(ctx); err != nil {
			c.logger.Error("Connection failed, will retry",
				zap.Error(err),
				zap.Int("retry_interval_seconds", c.config.Gateway.ReconnectInterval),
			)

			select {
			case <-ctx.Done():
				return ctx.Err()
			case <-c.closeCh:
				return nil
			case <-time.After(time.Duration(c.config.Gateway.ReconnectInterval) * time.Second):
				continue
			}
		}
	}
}

// connectOnce establishes a single connection attempt
func (c *Client) connectOnce(ctx context.Context) error {
	c.logger.Info("Connecting to gateway", zap.String("address", c.config.Gateway.Address))

	// Set up connection options
	opts := []grpc.DialOption{
		grpc.WithKeepaliveParams(keepalive.ClientParameters{
			Time:                30 * time.Second,
			Timeout:             10 * time.Second,
			PermitWithoutStream: true,
		}),
	}

	// Configure TLS
	if c.config.TLS.Enabled {
		tlsConfig, err := c.loadTLSConfig()
		if err != nil {
			return fmt.Errorf("failed to load TLS config: %w", err)
		}
		opts = append(opts, grpc.WithTransportCredentials(credentials.NewTLS(tlsConfig)))
	} else {
		c.logger.Warn("TLS is disabled - this should only be used for development")
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}

	// Establish connection
	conn, err := grpc.DialContext(ctx, c.config.Gateway.Address, opts...)
	if err != nil {
		return fmt.Errorf("failed to connect: %w", err)
	}

	c.mu.Lock()
	c.conn = conn
	c.client = agentv1.NewAgentServiceClient(conn)
	c.mu.Unlock()

	// Start the bidirectional stream
	stream, err := c.client.Connect(ctx)
	if err != nil {
		conn.Close()
		return fmt.Errorf("failed to establish stream: %w", err)
	}

	c.mu.Lock()
	c.stream = stream
	c.mu.Unlock()

	// Send registration
	if err := c.sendRegistration(); err != nil {
		stream.CloseSend()
		conn.Close()
		return fmt.Errorf("failed to register: %w", err)
	}

	// Start heartbeat goroutine
	heartbeatCtx, cancelHeartbeat := context.WithCancel(ctx)
	defer cancelHeartbeat()

	go c.heartbeatLoop(heartbeatCtx)

	// Process incoming messages
	if err := c.receiveLoop(ctx); err != nil {
		if err != io.EOF {
			return fmt.Errorf("receive loop error: %w", err)
		}
	}

	return nil
}

// loadTLSConfig creates TLS configuration for mTLS
func (c *Client) loadTLSConfig() (*tls.Config, error) {
	// Load client certificate and key
	cert, err := tls.LoadX509KeyPair(c.config.TLS.CertFile, c.config.TLS.KeyFile)
	if err != nil {
		return nil, fmt.Errorf("failed to load client certificate: %w", err)
	}

	// Load CA certificate
	caCert, err := os.ReadFile(c.config.TLS.CACertFile)
	if err != nil {
		return nil, fmt.Errorf("failed to load CA certificate: %w", err)
	}

	caCertPool := x509.NewCertPool()
	if !caCertPool.AppendCertsFromPEM(caCert) {
		return nil, fmt.Errorf("failed to parse CA certificate")
	}

	return &tls.Config{
		Certificates: []tls.Certificate{cert},
		RootCAs:      caCertPool,
		ServerName:   c.config.TLS.ServerName,
	}, nil
}

// sendRegistration sends the agent registration message
func (c *Client) sendRegistration() error {
	c.mu.RLock()
	stream := c.stream
	c.mu.RUnlock()

	reg := &agentv1.AgentMessage{
		Message: &agentv1.AgentMessage_Registration{
			Registration: &agentv1.AgentRegistration{
				AgentId:   c.config.Agent.ID,
				TenantId:  c.config.Agent.TenantID,
				AgentName: c.config.Agent.Name,
				Version:   version.Version,
				Capabilities: &agentv1.AgentCapabilities{
					SupportedAdapters:  []string{"postgresql", "mysql", "rest"},
					MaxConcurrentTasks: int32(c.config.Agent.MaxConcurrentTasks),
					SupportsStreaming:  true,
				},
				Labels: c.config.Agent.Labels,
			},
		},
	}

	if err := stream.Send(reg); err != nil {
		return fmt.Errorf("failed to send registration: %w", err)
	}

	c.logger.Info("Sent agent registration",
		zap.String("agent_id", c.config.Agent.ID),
		zap.String("tenant_id", c.config.Agent.TenantID),
	)

	return nil
}

// heartbeatLoop sends periodic heartbeats
func (c *Client) heartbeatLoop(ctx context.Context) {
	ticker := time.NewTicker(time.Duration(c.config.Gateway.HeartbeatInterval) * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-c.closeCh:
			return
		case <-ticker.C:
			if err := c.sendHeartbeat(); err != nil {
				c.logger.Error("Failed to send heartbeat", zap.Error(err))
			}
		}
	}
}

// sendHeartbeat sends a heartbeat message
func (c *Client) sendHeartbeat() error {
	c.mu.RLock()
	stream := c.stream
	c.mu.RUnlock()

	if stream == nil {
		return fmt.Errorf("stream not connected")
	}

	// Collect system metrics
	cpuPercent, memPercent, diskFree := c.collectSystemMetrics()

	heartbeat := &agentv1.AgentMessage{
		Message: &agentv1.AgentMessage_Heartbeat{
			Heartbeat: &agentv1.AgentHeartbeat{
				AgentId:   c.config.Agent.ID,
				Timestamp: timestamppb.Now(),
				Health: &agentv1.AgentHealth{
					Status:        agentv1.HealthStatus_HEALTH_STATUS_HEALTHY,
					CpuPercent:    cpuPercent,
					MemoryPercent: memPercent,
					DiskFreeBytes: diskFree,
				},
				RunningTasks: c.getRunningTasks(),
			},
		},
	}

	return stream.Send(heartbeat)
}

// collectSystemMetrics gathers CPU, memory, and disk usage metrics
func (c *Client) collectSystemMetrics() (cpuPercent, memPercent float64, diskFree int64) {
	// Memory metrics using runtime
	var memStats runtime.MemStats
	runtime.ReadMemStats(&memStats)

	// Calculate memory percent based on allocated vs system memory
	// Note: This is Go process memory, not total system memory
	memPercent = float64(memStats.Alloc) / float64(memStats.Sys) * 100.0
	if memPercent > 100 {
		memPercent = 100
	}

	// CPU percent approximation using goroutine count relative to available CPUs
	// This is a rough estimate - for production use, consider gopsutil
	numCPU := float64(runtime.NumCPU())
	numGoroutine := float64(runtime.NumGoroutine())
	cpuPercent = (numGoroutine / (numCPU * 100)) * 100
	if cpuPercent > 100 {
		cpuPercent = 100
	}

	// Disk free space using syscall.Statfs (works on Unix-like systems)
	var stat syscall.Statfs_t
	if err := syscall.Statfs("/", &stat); err == nil {
		diskFree = int64(stat.Bavail) * int64(stat.Bsize)
	}

	return cpuPercent, memPercent, diskFree
}

// getRunningTasks returns proto messages for running tasks
func (c *Client) getRunningTasks() []*agentv1.RunningTask {
	taskInfos := c.executor.RunningTasksInfo()
	tasks := make([]*agentv1.RunningTask, 0, len(taskInfos))

	for _, info := range taskInfos {
		tasks = append(tasks, &agentv1.RunningTask{
			TaskId:        info.TaskID,
			IntegrationId: info.IntegrationID,
			StartedAt:     timestamppb.New(info.StartedAt),
		})
	}

	return tasks
}

// receiveLoop processes incoming messages from the gateway
func (c *Client) receiveLoop(ctx context.Context) error {
	c.mu.RLock()
	stream := c.stream
	c.mu.RUnlock()

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-c.closeCh:
			return nil
		default:
		}

		msg, err := stream.Recv()
		if err != nil {
			return err
		}

		switch m := msg.Message.(type) {
		case *agentv1.GatewayMessage_RegistrationAck:
			c.handleRegistrationAck(m.RegistrationAck)
		case *agentv1.GatewayMessage_Task:
			go c.handleTask(ctx, m.Task)
		case *agentv1.GatewayMessage_Command:
			c.handleCommand(m.Command)
		}
	}
}

// handleRegistrationAck processes registration acknowledgment
func (c *Client) handleRegistrationAck(ack *agentv1.RegistrationAck) {
	if ack.Success {
		c.logger.Info("Registration successful",
			zap.Int32("heartbeat_interval", ack.HeartbeatIntervalSeconds),
		)
	} else {
		c.logger.Error("Registration failed", zap.String("message", ack.Message))
	}
}

// handleTask processes an incoming task
func (c *Client) handleTask(ctx context.Context, protoTask *agentv1.Task) {
	c.logger.Info("Received task",
		zap.String("task_id", protoTask.TaskId),
		zap.String("type", protoTask.TaskType.String()),
	)

	// Convert proto task to internal task
	task := &executor.Task{
		ID:            protoTask.TaskId,
		IntegrationID: protoTask.IntegrationId,
		TenantID:      protoTask.TenantId,
		Type:          protoTask.TaskType.String(),
		Config:        protoTask.Config.AsMap(),
		Priority:      int(protoTask.Priority),
		Timeout:       time.Duration(protoTask.TimeoutSeconds) * time.Second,
		RetryCount:    int(protoTask.RetryCount),
	}

	// Execute the task
	result := c.executor.ExecuteSync(ctx, task)

	// Send result back
	if err := c.sendTaskResult(result); err != nil {
		c.logger.Error("Failed to send task result", zap.Error(err))
	}
}

// sendTaskResult sends task execution result to gateway
func (c *Client) sendTaskResult(result *executor.TaskResult) error {
	c.mu.RLock()
	stream := c.stream
	c.mu.RUnlock()

	if stream == nil {
		return fmt.Errorf("stream not connected")
	}

	protoResult := &agentv1.TaskResult{
		TaskId:      result.TaskID,
		AgentId:     c.config.Agent.ID,
		Status:      convertStatus(result.Status),
		StartedAt:   timestamppb.New(result.StartedAt),
		CompletedAt: timestamppb.New(result.CompletedAt),
		Metrics: &agentv1.TaskMetrics{
			RecordsRead:    result.Metrics.RecordsRead,
			RecordsWritten: result.Metrics.RecordsWritten,
			BytesProcessed: result.Metrics.BytesProcessed,
			DurationMs:     result.Metrics.DurationMs,
		},
	}

	if result.Error != nil {
		protoResult.Error = &agentv1.TaskError{
			Code:      result.Error.Code,
			Message:   result.Error.Message,
			Details:   result.Error.Details,
			Retryable: result.Error.Retryable,
		}
	}

	if result.Output != nil {
		protoResult.Output = c.convertTaskOutput(result.Output)
	}

	msg := &agentv1.AgentMessage{
		Message: &agentv1.AgentMessage_TaskResult{
			TaskResult: protoResult,
		},
	}

	return stream.Send(msg)
}

// convertStatus converts internal status to proto status
func convertStatus(status executor.TaskStatus) agentv1.TaskStatus {
	switch status {
	case executor.TaskStatusPending:
		return agentv1.TaskStatus_TASK_STATUS_PENDING
	case executor.TaskStatusRunning:
		return agentv1.TaskStatus_TASK_STATUS_RUNNING
	case executor.TaskStatusCompleted:
		return agentv1.TaskStatus_TASK_STATUS_COMPLETED
	case executor.TaskStatusFailed:
		return agentv1.TaskStatus_TASK_STATUS_FAILED
	case executor.TaskStatusCancelled:
		return agentv1.TaskStatus_TASK_STATUS_CANCELLED
	case executor.TaskStatusTimeout:
		return agentv1.TaskStatus_TASK_STATUS_TIMEOUT
	default:
		return agentv1.TaskStatus_TASK_STATUS_UNSPECIFIED
	}
}

// convertTaskOutput converts executor output to proto TaskOutput
func (c *Client) convertTaskOutput(output interface{}) *agentv1.TaskOutput {
	switch v := output.(type) {
	case map[string]interface{}:
		return c.convertQueryResultOutput(v)
	case *executor.QueryResultOutput:
		return c.convertStructuredQueryResult(v)
	default:
		// For unknown types, try to convert to struct
		if outputMap, ok := output.(map[string]interface{}); ok {
			return c.convertQueryResultOutput(outputMap)
		}
		c.logger.Warn("Unknown output type, skipping conversion",
			zap.String("type", fmt.Sprintf("%T", output)))
		return nil
	}
}

// convertQueryResultOutput converts a map-based query result to proto
func (c *Client) convertQueryResultOutput(outputMap map[string]interface{}) *agentv1.TaskOutput {
	queryResult := &agentv1.QueryResult{}

	// Extract columns if present
	if cols, ok := outputMap["columns"].([]interface{}); ok {
		for _, col := range cols {
			if colMap, ok := col.(map[string]interface{}); ok {
				column := &agentv1.Column{
					Name:     getString(colMap, "name"),
					DataType: getString(colMap, "data_type"),
					Nullable: getBool(colMap, "nullable"),
				}
				queryResult.Columns = append(queryResult.Columns, column)
			}
		}
	}

	// Extract rows if present
	if rows, ok := outputMap["rows"].([]interface{}); ok {
		for _, row := range rows {
			if rowVals, ok := row.([]interface{}); ok {
				protoRow := &agentv1.Row{}
				for _, val := range rowVals {
					protoVal, err := structpb.NewValue(val)
					if err != nil {
						protoVal = structpb.NewNullValue()
					}
					protoRow.Values = append(protoRow.Values, protoVal)
				}
				queryResult.Rows = append(queryResult.Rows, protoRow)
			}
		}
	}

	// Extract metadata
	if affected, ok := outputMap["rows_affected"].(int64); ok {
		queryResult.RowsAffected = affected
	}
	if hasMore, ok := outputMap["has_more"].(bool); ok {
		queryResult.HasMore = hasMore
	}
	if cursor, ok := outputMap["cursor"].(string); ok {
		queryResult.Cursor = cursor
	}

	return &agentv1.TaskOutput{
		Output: &agentv1.TaskOutput_QueryResult{
			QueryResult: queryResult,
		},
	}
}

// convertStructuredQueryResult converts a structured QueryResultOutput to proto
func (c *Client) convertStructuredQueryResult(result *executor.QueryResultOutput) *agentv1.TaskOutput {
	queryResult := &agentv1.QueryResult{
		RowsAffected: result.RowsAffected,
		HasMore:      result.HasMore,
		Cursor:       result.Cursor,
	}

	for _, col := range result.Columns {
		queryResult.Columns = append(queryResult.Columns, &agentv1.Column{
			Name:     col.Name,
			DataType: col.DataType,
			Nullable: col.Nullable,
		})
	}

	for _, row := range result.Rows {
		protoRow := &agentv1.Row{}
		for _, val := range row {
			protoVal, err := structpb.NewValue(val)
			if err != nil {
				protoVal = structpb.NewNullValue()
			}
			protoRow.Values = append(protoRow.Values, protoVal)
		}
		queryResult.Rows = append(queryResult.Rows, protoRow)
	}

	return &agentv1.TaskOutput{
		Output: &agentv1.TaskOutput_QueryResult{
			QueryResult: queryResult,
		},
	}
}

// Helper functions for safe type conversion
func getString(m map[string]interface{}, key string) string {
	if v, ok := m[key].(string); ok {
		return v
	}
	return ""
}

func getBool(m map[string]interface{}, key string) bool {
	if v, ok := m[key].(bool); ok {
		return v
	}
	return false
}

// handleCommand processes commands from the gateway
func (c *Client) handleCommand(cmd *agentv1.AgentCommand) {
	switch m := cmd.Command.(type) {
	case *agentv1.AgentCommand_CancelTask:
		c.logger.Info("Received cancel task command",
			zap.String("task_id", m.CancelTask.TaskId),
			zap.String("reason", m.CancelTask.Reason),
		)
		c.executor.CancelTask(m.CancelTask.TaskId)

	case *agentv1.AgentCommand_UpdateConfig:
		c.logger.Info("Received config update command")
		c.applyConfigUpdate(m.UpdateConfig.Config)

	case *agentv1.AgentCommand_Restart:
		c.logger.Info("Received restart command",
			zap.String("reason", m.Restart.Reason),
			zap.Int32("delay_seconds", m.Restart.DelaySeconds),
		)
		c.handleRestart(m.Restart.Reason, m.Restart.DelaySeconds)
	}
}

// applyConfigUpdate applies configuration changes from the gateway
func (c *Client) applyConfigUpdate(configUpdates map[string]string) {
	c.mu.Lock()
	defer c.mu.Unlock()

	for key, value := range configUpdates {
		c.logger.Info("Applying config update",
			zap.String("key", key),
			zap.String("value", value),
		)

		// Apply supported runtime configuration changes
		switch key {
		case "heartbeat_interval_seconds":
			// Parse and apply heartbeat interval
			if interval, err := parseIntConfig(value); err == nil && interval > 0 {
				c.config.Gateway.HeartbeatInterval = interval
				c.logger.Info("Updated heartbeat interval", zap.Int("seconds", interval))
			}
		case "reconnect_interval_seconds":
			// Parse and apply reconnect interval
			if interval, err := parseIntConfig(value); err == nil && interval > 0 {
				c.config.Gateway.ReconnectInterval = interval
				c.logger.Info("Updated reconnect interval", zap.Int("seconds", interval))
			}
		case "log_level":
			// Log level changes would require logger reconfiguration
			c.logger.Info("Log level change requested (requires restart)", zap.String("level", value))
		default:
			c.logger.Warn("Unknown config key, ignoring", zap.String("key", key))
		}
	}
}

// handleRestart initiates a graceful restart after the specified delay
func (c *Client) handleRestart(reason string, delaySeconds int32) {
	go func() {
		if delaySeconds > 0 {
			c.logger.Info("Delaying restart",
				zap.Int32("delay_seconds", delaySeconds),
				zap.String("reason", reason),
			)
			time.Sleep(time.Duration(delaySeconds) * time.Second)
		}

		// Signal restart through the restart channel
		// The main goroutine should listen on this and perform the actual restart
		select {
		case c.restartCh <- RestartRequest{Reason: reason, DelaySeconds: delaySeconds}:
			c.logger.Info("Restart signal sent")
		default:
			c.logger.Warn("Restart channel full, restart may already be in progress")
		}
	}()
}

// parseIntConfig safely parses an integer from a config string
func parseIntConfig(value string) (int, error) {
	var result int
	_, err := fmt.Sscanf(value, "%d", &result)
	return result, err
}

// Close closes the tunnel connection
func (c *Client) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.closed {
		return nil
	}

	c.closed = true
	close(c.closeCh)

	if c.stream != nil {
		c.stream.CloseSend()
	}

	if c.conn != nil {
		return c.conn.Close()
	}

	return nil
}
