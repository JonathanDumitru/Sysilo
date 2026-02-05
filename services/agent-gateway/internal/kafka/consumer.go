package kafka

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/IBM/sarama"
	"go.uber.org/zap"
)

// TaskMessage represents a task received from Kafka
type TaskMessage struct {
	TaskID        string                 `json:"task_id"`
	IntegrationID string                 `json:"integration_id"`
	TenantID      string                 `json:"tenant_id"`
	AgentID       string                 `json:"agent_id,omitempty"`
	TaskType      string                 `json:"task_type"`
	Config        map[string]interface{} `json:"config"`
	Priority      int                    `json:"priority"`
	Timeout       int                    `json:"timeout_seconds"`
}

// TaskDispatcher is called when a task needs to be sent to an agent
type TaskDispatcher interface {
	DispatchTask(ctx context.Context, msg *TaskMessage) error
}

// Consumer consumes task messages from Kafka
type Consumer struct {
	logger     *zap.Logger
	client     sarama.ConsumerGroup
	topics     []string
	dispatcher TaskDispatcher
}

// ConsumerConfig holds Kafka consumer configuration
type ConsumerConfig struct {
	Brokers   []string
	GroupID   string
	TaskTopic string
}

// NewConsumer creates a new Kafka consumer
func NewConsumer(logger *zap.Logger, cfg ConsumerConfig, dispatcher TaskDispatcher) (*Consumer, error) {
	config := sarama.NewConfig()
	config.Consumer.Group.Rebalance.GroupStrategies = []sarama.BalanceStrategy{sarama.NewBalanceStrategyRoundRobin()}
	config.Consumer.Offsets.Initial = sarama.OffsetNewest
	config.Version = sarama.V3_0_0_0

	client, err := sarama.NewConsumerGroup(cfg.Brokers, cfg.GroupID, config)
	if err != nil {
		return nil, fmt.Errorf("failed to create consumer group: %w", err)
	}

	return &Consumer{
		logger:     logger.Named("kafka-consumer"),
		client:     client,
		topics:     []string{cfg.TaskTopic},
		dispatcher: dispatcher,
	}, nil
}

// Start begins consuming messages
func (c *Consumer) Start(ctx context.Context) error {
	handler := &consumerHandler{
		logger:     c.logger,
		dispatcher: c.dispatcher,
	}

	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			if err := c.client.Consume(ctx, c.topics, handler); err != nil {
				c.logger.Error("Consumer error", zap.Error(err))
			}
		}
	}
}

// Close closes the consumer
func (c *Consumer) Close() error {
	return c.client.Close()
}

// consumerHandler implements sarama.ConsumerGroupHandler
type consumerHandler struct {
	logger     *zap.Logger
	dispatcher TaskDispatcher
}

func (h *consumerHandler) Setup(sarama.ConsumerGroupSession) error {
	h.logger.Info("Consumer session setup")
	return nil
}

func (h *consumerHandler) Cleanup(sarama.ConsumerGroupSession) error {
	h.logger.Info("Consumer session cleanup")
	return nil
}

func (h *consumerHandler) ConsumeClaim(session sarama.ConsumerGroupSession, claim sarama.ConsumerGroupClaim) error {
	for {
		select {
		case message, ok := <-claim.Messages():
			if !ok {
				return nil
			}

			h.logger.Debug("Received message",
				zap.String("topic", message.Topic),
				zap.Int32("partition", message.Partition),
				zap.Int64("offset", message.Offset),
			)

			var taskMsg TaskMessage
			if err := json.Unmarshal(message.Value, &taskMsg); err != nil {
				h.logger.Error("Failed to unmarshal task message",
					zap.Error(err),
					zap.ByteString("value", message.Value),
				)
				session.MarkMessage(message, "")
				continue
			}

			// Dispatch the task to an agent
			if err := h.dispatcher.DispatchTask(session.Context(), &taskMsg); err != nil {
				h.logger.Error("Failed to dispatch task",
					zap.String("task_id", taskMsg.TaskID),
					zap.Error(err),
				)
				// Don't mark as processed - will be retried
				continue
			}

			session.MarkMessage(message, "")

		case <-session.Context().Done():
			return nil
		}
	}
}
