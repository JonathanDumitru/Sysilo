package postgresql

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"time"

	_ "github.com/lib/pq"
	"go.uber.org/zap"

	"github.com/sysilo/sysilo/agent/internal/executor"
)

// Adapter handles PostgreSQL database operations
type Adapter struct {
	logger *zap.Logger
}

// NewAdapter creates a new PostgreSQL adapter
func NewAdapter(logger *zap.Logger) *Adapter {
	return &Adapter{
		logger: logger.Named("postgresql"),
	}
}

// Type returns the adapter type identifier
func (a *Adapter) Type() string {
	return "postgresql"
}

// Execute runs a PostgreSQL task
func (a *Adapter) Execute(ctx context.Context, task *executor.Task) (*executor.TaskResult, error) {
	config, err := parseConfig(task.Config)
	if err != nil {
		return nil, fmt.Errorf("invalid task config: %w", err)
	}

	a.logger.Info("Executing PostgreSQL task",
		zap.String("task_id", task.ID),
		zap.String("operation", config.Operation),
	)

	// Connect to database
	db, err := a.connect(ctx, config.Connection)
	if err != nil {
		return nil, fmt.Errorf("failed to connect: %w", err)
	}
	defer db.Close()

	// Execute based on operation type
	switch config.Operation {
	case "query":
		return a.executeQuery(ctx, db, config)
	case "execute":
		return a.executeStatement(ctx, db, config)
	case "health_check":
		return a.healthCheck(ctx, db)
	default:
		return nil, fmt.Errorf("unknown operation: %s", config.Operation)
	}
}

// TaskConfig holds the configuration for a PostgreSQL task
type TaskConfig struct {
	Operation  string           `json:"operation"`
	Connection ConnectionConfig `json:"connection"`
	Query      string           `json:"query"`
	Parameters []interface{}    `json:"parameters"`
	BatchSize  int              `json:"batch_size"`
}

// ConnectionConfig holds database connection details
type ConnectionConfig struct {
	Host     string `json:"host"`
	Port     int    `json:"port"`
	Database string `json:"database"`
	User     string `json:"user"`
	Password string `json:"password"`
	SSLMode  string `json:"ssl_mode"`
}

func parseConfig(raw map[string]interface{}) (*TaskConfig, error) {
	data, err := json.Marshal(raw)
	if err != nil {
		return nil, err
	}

	var config TaskConfig
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, err
	}

	// Defaults
	if config.Connection.Port == 0 {
		config.Connection.Port = 5432
	}
	if config.Connection.SSLMode == "" {
		config.Connection.SSLMode = "disable"
	}
	if config.BatchSize == 0 {
		config.BatchSize = 1000
	}

	return &config, nil
}

func (a *Adapter) connect(ctx context.Context, config ConnectionConfig) (*sql.DB, error) {
	connStr := fmt.Sprintf(
		"host=%s port=%d user=%s password=%s dbname=%s sslmode=%s",
		config.Host,
		config.Port,
		config.User,
		config.Password,
		config.Database,
		config.SSLMode,
	)

	db, err := sql.Open("postgres", connStr)
	if err != nil {
		return nil, fmt.Errorf("failed to open connection: %w", err)
	}

	// Configure connection pool
	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(5)
	db.SetConnMaxLifetime(5 * time.Minute)

	// Verify connection
	if err := db.PingContext(ctx); err != nil {
		db.Close()
		return nil, fmt.Errorf("failed to ping database: %w", err)
	}

	return db, nil
}

func (a *Adapter) executeQuery(ctx context.Context, db *sql.DB, config *TaskConfig) (*executor.TaskResult, error) {
	rows, err := db.QueryContext(ctx, config.Query, config.Parameters...)
	if err != nil {
		return nil, fmt.Errorf("query failed: %w", err)
	}
	defer rows.Close()

	// Get column information
	columns, err := rows.Columns()
	if err != nil {
		return nil, fmt.Errorf("failed to get columns: %w", err)
	}

	columnTypes, err := rows.ColumnTypes()
	if err != nil {
		return nil, fmt.Errorf("failed to get column types: %w", err)
	}

	// Build column metadata
	columnMeta := make([]map[string]interface{}, len(columns))
	for i, col := range columns {
		nullable, _ := columnTypes[i].Nullable()
		columnMeta[i] = map[string]interface{}{
			"name":     col,
			"type":     columnTypes[i].DatabaseTypeName(),
			"nullable": nullable,
		}
	}

	// Fetch rows
	var resultRows [][]interface{}
	var recordsRead int64

	for rows.Next() {
		// Create a slice of interface{} to hold the row values
		values := make([]interface{}, len(columns))
		valuePtrs := make([]interface{}, len(columns))
		for i := range values {
			valuePtrs[i] = &values[i]
		}

		if err := rows.Scan(valuePtrs...); err != nil {
			return nil, fmt.Errorf("failed to scan row: %w", err)
		}

		// Convert values to JSON-safe types
		row := make([]interface{}, len(values))
		for i, v := range values {
			row[i] = convertValue(v)
		}

		resultRows = append(resultRows, row)
		recordsRead++

		// Check batch size limit
		if config.BatchSize > 0 && int(recordsRead) >= config.BatchSize {
			break
		}
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("row iteration error: %w", err)
	}

	// Build result
	output := map[string]interface{}{
		"columns":  columnMeta,
		"rows":     resultRows,
		"has_more": config.BatchSize > 0 && int(recordsRead) >= config.BatchSize,
	}

	return &executor.TaskResult{
		Output: output,
		Metrics: executor.TaskMetrics{
			RecordsRead: recordsRead,
		},
	}, nil
}

func (a *Adapter) executeStatement(ctx context.Context, db *sql.DB, config *TaskConfig) (*executor.TaskResult, error) {
	result, err := db.ExecContext(ctx, config.Query, config.Parameters...)
	if err != nil {
		return nil, fmt.Errorf("execute failed: %w", err)
	}

	rowsAffected, _ := result.RowsAffected()

	output := map[string]interface{}{
		"rows_affected": rowsAffected,
	}

	return &executor.TaskResult{
		Output: output,
		Metrics: executor.TaskMetrics{
			RecordsWritten: rowsAffected,
		},
	}, nil
}

func (a *Adapter) healthCheck(ctx context.Context, db *sql.DB) (*executor.TaskResult, error) {
	start := time.Now()

	var version string
	err := db.QueryRowContext(ctx, "SELECT version()").Scan(&version)
	if err != nil {
		return nil, fmt.Errorf("health check failed: %w", err)
	}

	latency := time.Since(start)

	output := map[string]interface{}{
		"healthy":    true,
		"version":    version,
		"latency_ms": latency.Milliseconds(),
	}

	return &executor.TaskResult{
		Output: output,
	}, nil
}

// convertValue converts database values to JSON-safe types
func convertValue(v interface{}) interface{} {
	if v == nil {
		return nil
	}

	switch val := v.(type) {
	case []byte:
		return string(val)
	case time.Time:
		return val.Format(time.RFC3339)
	default:
		return val
	}
}
