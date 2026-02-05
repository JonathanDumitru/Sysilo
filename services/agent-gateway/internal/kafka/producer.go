package kafka

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/IBM/sarama"
	"go.uber.org/zap"
)

// ResultMessage represents a task result to be published to Kafka
type ResultMessage struct {
	TaskID        string                 `json:"task_id"`
	AgentID       string                 `json:"agent_id"`
	IntegrationID string                 `json:"integration_id"`
	TenantID      string                 `json:"tenant_id"`
	Status        string                 `json:"status"`
	StartedAt     time.Time              `json:"started_at"`
	CompletedAt   time.Time              `json:"completed_at"`
	Output        interface{}            `json:"output,omitempty"`
	Error         *ErrorDetail           `json:"error,omitempty"`
	Metrics       map[string]interface{} `json:"metrics"`
}

// ErrorDetail contains error information
type ErrorDetail struct {
	Code      string `json:"code"`
	Message   string `json:"message"`
	Details   string `json:"details,omitempty"`
	Retryable bool   `json:"retryable"`
}

// Producer publishes messages to Kafka
type Producer struct {
	logger      *zap.Logger
	producer    sarama.SyncProducer
	resultTopic string
	logsTopic   string
}

// LogMessage represents a log batch to be published to Kafka
type LogMessage struct {
	AgentID   string     `json:"agent_id"`
	TenantID  string     `json:"tenant_id"`
	Entries   []LogEntry `json:"entries"`
	Timestamp time.Time  `json:"timestamp"`
}

// LogEntry represents a single log entry
type LogEntry struct {
	Timestamp time.Time         `json:"timestamp"`
	Level     string            `json:"level"`
	Message   string            `json:"message"`
	TaskID    string            `json:"task_id,omitempty"`
	Fields    map[string]string `json:"fields,omitempty"`
}

// ProducerConfig holds Kafka producer configuration
type ProducerConfig struct {
	Brokers     []string
	ResultTopic string
	LogsTopic   string
}

// NewProducer creates a new Kafka producer
func NewProducer(logger *zap.Logger, cfg ProducerConfig) (*Producer, error) {
	config := sarama.NewConfig()
	config.Producer.RequiredAcks = sarama.WaitForAll
	config.Producer.Retry.Max = 3
	config.Producer.Return.Successes = true
	config.Version = sarama.V3_0_0_0

	producer, err := sarama.NewSyncProducer(cfg.Brokers, config)
	if err != nil {
		return nil, fmt.Errorf("failed to create producer: %w", err)
	}

	return &Producer{
		logger:      logger.Named("kafka-producer"),
		producer:    producer,
		resultTopic: cfg.ResultTopic,
		logsTopic:   cfg.LogsTopic,
	}, nil
}

// PublishResult publishes a task result to Kafka
func (p *Producer) PublishResult(result *ResultMessage) error {
	data, err := json.Marshal(result)
	if err != nil {
		return fmt.Errorf("failed to marshal result: %w", err)
	}

	msg := &sarama.ProducerMessage{
		Topic: p.resultTopic,
		Key:   sarama.StringEncoder(result.TaskID),
		Value: sarama.ByteEncoder(data),
		Headers: []sarama.RecordHeader{
			{Key: []byte("tenant_id"), Value: []byte(result.TenantID)},
			{Key: []byte("task_id"), Value: []byte(result.TaskID)},
		},
	}

	partition, offset, err := p.producer.SendMessage(msg)
	if err != nil {
		return fmt.Errorf("failed to send message: %w", err)
	}

	p.logger.Debug("Published result",
		zap.String("task_id", result.TaskID),
		zap.Int32("partition", partition),
		zap.Int64("offset", offset),
	)

	return nil
}

// PublishLogs publishes a log batch to Kafka
func (p *Producer) PublishLogs(logs *LogMessage) error {
	if p.logsTopic == "" {
		// Logs topic not configured, skip publishing
		return nil
	}

	data, err := json.Marshal(logs)
	if err != nil {
		return fmt.Errorf("failed to marshal logs: %w", err)
	}

	msg := &sarama.ProducerMessage{
		Topic: p.logsTopic,
		Key:   sarama.StringEncoder(logs.AgentID),
		Value: sarama.ByteEncoder(data),
		Headers: []sarama.RecordHeader{
			{Key: []byte("tenant_id"), Value: []byte(logs.TenantID)},
			{Key: []byte("agent_id"), Value: []byte(logs.AgentID)},
		},
	}

	partition, offset, err := p.producer.SendMessage(msg)
	if err != nil {
		return fmt.Errorf("failed to send logs: %w", err)
	}

	p.logger.Debug("Published logs",
		zap.String("agent_id", logs.AgentID),
		zap.Int("entries", len(logs.Entries)),
		zap.Int32("partition", partition),
		zap.Int64("offset", offset),
	)

	return nil
}

// Close closes the producer
func (p *Producer) Close() error {
	return p.producer.Close()
}
