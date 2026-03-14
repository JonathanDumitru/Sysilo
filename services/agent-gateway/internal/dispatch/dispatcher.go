package dispatch

import (
	"context"
	"fmt"

	agentv1 "github.com/sysilo/sysilo/proto/agent/v1"
	"github.com/sysilo/sysilo/services/agent-gateway/internal/kafka"
	"github.com/sysilo/sysilo/services/agent-gateway/internal/registry"
	"go.uber.org/zap"
	"google.golang.org/protobuf/types/known/structpb"
)

// TaskSender abstracts sending a task to a connected agent. This is
// satisfied by *tunnel.Server.
type TaskSender interface {
	SendTask(agentID string, task *agentv1.Task) error
}

// Dispatcher implements kafka.TaskDispatcher. It receives task messages from
// the Kafka consumer, selects the best available agent via the registry, and
// forwards the task over the agent's gRPC tunnel stream.
type Dispatcher struct {
	logger   *zap.Logger
	registry *registry.Registry
	sender   TaskSender
}

// New creates a new Dispatcher.
func New(logger *zap.Logger, reg *registry.Registry, sender TaskSender) *Dispatcher {
	return &Dispatcher{
		logger:   logger.Named("dispatcher"),
		registry: reg,
		sender:   sender,
	}
}

// DispatchTask implements kafka.TaskDispatcher. It is called for every task
// message consumed from Kafka.
func (d *Dispatcher) DispatchTask(ctx context.Context, msg *kafka.TaskMessage) error {
	log := d.logger.With(
		zap.String("task_id", msg.TaskID),
		zap.String("tenant_id", msg.TenantID),
		zap.String("task_type", msg.TaskType),
		zap.String("integration_id", msg.IntegrationID),
	)

	// --- 1. Validate required fields -------------------------------------------
	if msg.TenantID == "" {
		log.Error("Task message missing tenant_id, dropping message")
		return fmt.Errorf("task message missing tenant_id")
	}
	if msg.TaskID == "" {
		log.Error("Task message missing task_id, dropping message")
		return fmt.Errorf("task message missing task_id")
	}

	// --- 2. Find the best agent ------------------------------------------------
	// TaskType is used as the required adapter name (e.g. "postgresql", "mysql",
	// "rest"). If the TaskType doesn't directly map to an adapter, an empty
	// string lets us match any agent for the tenant.
	requiredAdapter := msg.TaskType

	agent := d.registry.FindBestAgent(msg.TenantID, requiredAdapter, msg.AgentID)
	if agent == nil {
		log.Warn("No available agent found for task",
			zap.String("required_adapter", requiredAdapter),
			zap.String("preferred_agent_id", msg.AgentID),
		)
		// Return an error so the Kafka consumer does NOT mark the offset.
		// The message will be re-delivered on the next poll, giving agents
		// time to connect or free up capacity.
		return fmt.Errorf("no available agent for tenant %s with adapter %q", msg.TenantID, requiredAdapter)
	}

	log = log.With(zap.String("agent_id", agent.ID))

	// --- 3. Convert the Kafka TaskMessage to a protobuf Task -------------------
	protoTask, err := d.toProtoTask(msg)
	if err != nil {
		log.Error("Failed to convert task message to protobuf", zap.Error(err))
		return fmt.Errorf("converting task to protobuf: %w", err)
	}

	// --- 4. Send via the tunnel stream -----------------------------------------
	if err := d.sender.SendTask(agent.ID, protoTask); err != nil {
		log.Error("Failed to send task to agent",
			zap.Error(err),
		)
		return fmt.Errorf("sending task to agent %s: %w", agent.ID, err)
	}

	log.Info("Task dispatched to agent",
		zap.String("agent_name", agent.Name),
		zap.Int("agent_running_tasks", len(agent.RunningTasks)),
	)

	return nil
}

// toProtoTask converts a kafka.TaskMessage into an agentv1.Task protobuf.
func (d *Dispatcher) toProtoTask(msg *kafka.TaskMessage) (*agentv1.Task, error) {
	// Convert the free-form config map into a protobuf Struct.
	var cfgStruct *structpb.Struct
	if msg.Config != nil {
		var err error
		cfgStruct, err = structpb.NewStruct(msg.Config)
		if err != nil {
			return nil, fmt.Errorf("converting config to protobuf Struct: %w", err)
		}
	}

	task := &agentv1.Task{
		TaskId:         msg.TaskID,
		IntegrationId:  msg.IntegrationID,
		TenantId:       msg.TenantID,
		TaskType:       mapTaskType(msg.TaskType),
		Config:         cfgStruct,
		Priority:       mapPriority(msg.Priority),
		TimeoutSeconds: int32(msg.Timeout),
	}

	return task, nil
}

// mapTaskType converts a string task type to the protobuf TaskType enum.
func mapTaskType(taskType string) agentv1.TaskType {
	switch taskType {
	case "query", "postgresql", "mysql", "bigquery":
		return agentv1.TaskType_TASK_TYPE_QUERY
	case "api_call", "rest", "graphql":
		return agentv1.TaskType_TASK_TYPE_API_CALL
	case "file_transfer", "sftp", "s3":
		return agentv1.TaskType_TASK_TYPE_FILE_TRANSFER
	case "discovery":
		return agentv1.TaskType_TASK_TYPE_DISCOVERY
	case "health_check":
		return agentv1.TaskType_TASK_TYPE_HEALTH_CHECK
	default:
		return agentv1.TaskType_TASK_TYPE_UNSPECIFIED
	}
}

// mapPriority converts a numeric priority to the protobuf TaskPriority enum.
func mapPriority(priority int) agentv1.TaskPriority {
	switch {
	case priority <= 0:
		return agentv1.TaskPriority_TASK_PRIORITY_NORMAL
	case priority == 1:
		return agentv1.TaskPriority_TASK_PRIORITY_LOW
	case priority == 2:
		return agentv1.TaskPriority_TASK_PRIORITY_NORMAL
	case priority == 3:
		return agentv1.TaskPriority_TASK_PRIORITY_HIGH
	case priority >= 4:
		return agentv1.TaskPriority_TASK_PRIORITY_CRITICAL
	default:
		return agentv1.TaskPriority_TASK_PRIORITY_NORMAL
	}
}
