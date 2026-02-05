package discovery

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

// Handler discovers database schemas, tables, views, and columns
type Handler struct {
	logger *zap.Logger
}

// NewHandler creates a new discovery handler
func NewHandler(logger *zap.Logger) *Handler {
	return &Handler{
		logger: logger.Named("discovery"),
	}
}

// Type returns the handler type identifier
func (h *Handler) Type() string {
	return "discovery"
}

// DiscoveryConfig holds the task configuration
type DiscoveryConfig struct {
	Connection    ConnectionConfig `json:"connection"`
	DiscoveryType string           `json:"discovery_type"`
	ResourceTypes []string         `json:"resource_types"`
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

// DiscoveredAsset matches the Rust consumer's expected structure
type DiscoveredAsset struct {
	Name        string                 `json:"name"`
	AssetType   string                 `json:"asset_type"`
	Description *string                `json:"description,omitempty"`
	Vendor      *string                `json:"vendor,omitempty"`
	Version     *string                `json:"version,omitempty"`
	Metadata    map[string]interface{} `json:"metadata,omitempty"`
}

// ColumnInfo describes a discovered column
type ColumnInfo struct {
	Name         string  `json:"name"`
	DataType     string  `json:"data_type"`
	IsNullable   bool    `json:"is_nullable"`
	DefaultValue *string `json:"default_value,omitempty"`
}

// Execute runs the discovery task
func (h *Handler) Execute(ctx context.Context, task *executor.Task) (*executor.TaskResult, error) {
	config, err := h.parseConfig(task.Config)
	if err != nil {
		return nil, fmt.Errorf("invalid discovery config: %w", err)
	}

	h.logger.Info("Starting database discovery",
		zap.String("task_id", task.ID),
		zap.String("host", config.Connection.Host),
		zap.String("database", config.Connection.Database),
		zap.String("discovery_type", config.DiscoveryType),
	)

	db, err := h.connect(ctx, config.Connection)
	if err != nil {
		return nil, fmt.Errorf("failed to connect for discovery: %w", err)
	}
	defer db.Close()

	assets, err := h.discoverAssets(ctx, db, config)
	if err != nil {
		return nil, fmt.Errorf("discovery failed: %w", err)
	}

	h.logger.Info("Discovery complete",
		zap.String("task_id", task.ID),
		zap.Int("assets_found", len(assets)),
	)

	output := map[string]interface{}{
		"discovered_assets": assets,
	}

	return &executor.TaskResult{
		Output: output,
		Metrics: executor.TaskMetrics{
			RecordsRead: int64(len(assets)),
		},
	}, nil
}

func (h *Handler) parseConfig(raw map[string]interface{}) (*DiscoveryConfig, error) {
	data, err := json.Marshal(raw)
	if err != nil {
		return nil, err
	}

	var config DiscoveryConfig
	if err := json.Unmarshal(data, &config); err != nil {
		return nil, err
	}

	if config.Connection.Port == 0 {
		config.Connection.Port = 5432
	}
	if config.Connection.SSLMode == "" {
		config.Connection.SSLMode = "disable"
	}
	if config.DiscoveryType == "" {
		config.DiscoveryType = "full"
	}

	return &config, nil
}

func (h *Handler) connect(ctx context.Context, config ConnectionConfig) (*sql.DB, error) {
	connStr := fmt.Sprintf(
		"host=%s port=%d user=%s password=%s dbname=%s sslmode=%s",
		config.Host, config.Port, config.User, config.Password,
		config.Database, config.SSLMode,
	)

	db, err := sql.Open("postgres", connStr)
	if err != nil {
		return nil, fmt.Errorf("failed to open connection: %w", err)
	}

	db.SetMaxOpenConns(5)
	db.SetMaxIdleConns(2)
	db.SetConnMaxLifetime(2 * time.Minute)

	if err := db.PingContext(ctx); err != nil {
		db.Close()
		return nil, fmt.Errorf("failed to ping database: %w", err)
	}

	return db, nil
}

func (h *Handler) discoverAssets(ctx context.Context, db *sql.DB, config *DiscoveryConfig) ([]DiscoveredAsset, error) {
	// Get PostgreSQL version for vendor info
	var version string
	if err := db.QueryRowContext(ctx, "SELECT version()").Scan(&version); err != nil {
		version = "PostgreSQL"
	}

	// Discover schemas (exclude system schemas)
	schemas, err := h.discoverSchemas(ctx, db)
	if err != nil {
		return nil, fmt.Errorf("schema discovery failed: %w", err)
	}

	var assets []DiscoveredAsset

	for _, schema := range schemas {
		// Discover tables and views in this schema
		schemaAssets, err := h.discoverTablesAndViews(ctx, db, schema, version, config)
		if err != nil {
			h.logger.Warn("Failed to discover tables in schema",
				zap.String("schema", schema),
				zap.Error(err),
			)
			continue
		}
		assets = append(assets, schemaAssets...)
	}

	return assets, nil
}

func (h *Handler) discoverSchemas(ctx context.Context, db *sql.DB) ([]string, error) {
	rows, err := db.QueryContext(ctx, `
		SELECT schema_name
		FROM information_schema.schemata
		WHERE schema_name NOT LIKE 'pg_%'
		  AND schema_name != 'information_schema'
		ORDER BY schema_name
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var schemas []string
	for rows.Next() {
		var name string
		if err := rows.Scan(&name); err != nil {
			return nil, err
		}
		schemas = append(schemas, name)
	}
	return schemas, rows.Err()
}

func (h *Handler) discoverTablesAndViews(
	ctx context.Context,
	db *sql.DB,
	schema string,
	version string,
	config *DiscoveryConfig,
) ([]DiscoveredAsset, error) {
	// Get tables and views
	rows, err := db.QueryContext(ctx, `
		SELECT table_name, table_type
		FROM information_schema.tables
		WHERE table_schema = $1
		ORDER BY table_name
	`, schema)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	type tableInfo struct {
		name      string
		tableType string
	}

	var tables []tableInfo
	for rows.Next() {
		var t tableInfo
		if err := rows.Scan(&t.name, &t.tableType); err != nil {
			return nil, err
		}
		tables = append(tables, t)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}

	var assets []DiscoveredAsset

	for _, t := range tables {
		// Get columns for this table
		columns, err := h.discoverColumns(ctx, db, schema, t.name)
		if err != nil {
			h.logger.Warn("Failed to discover columns",
				zap.String("schema", schema),
				zap.String("table", t.name),
				zap.Error(err),
			)
			columns = nil
		}

		assetType := "table"
		if t.tableType == "VIEW" {
			assetType = "view"
		}

		qualifiedName := fmt.Sprintf("%s.%s", schema, t.name)
		desc := fmt.Sprintf("%s '%s' with %d columns in schema '%s'",
			assetType, t.name, len(columns), schema)
		vendor := "PostgreSQL"

		assets = append(assets, DiscoveredAsset{
			Name:        qualifiedName,
			AssetType:   assetType,
			Description: &desc,
			Vendor:      &vendor,
			Version:     &version,
			Metadata: map[string]interface{}{
				"schema":        schema,
				"table_name":    t.name,
				"table_type":    t.tableType,
				"columns":       columns,
				"column_count":  len(columns),
				"discovered_at": time.Now().UTC().Format(time.RFC3339),
				"database":      config.Connection.Database,
			},
		})
	}

	return assets, nil
}

func (h *Handler) discoverColumns(
	ctx context.Context,
	db *sql.DB,
	schema, table string,
) ([]ColumnInfo, error) {
	rows, err := db.QueryContext(ctx, `
		SELECT column_name, data_type, is_nullable, column_default
		FROM information_schema.columns
		WHERE table_schema = $1 AND table_name = $2
		ORDER BY ordinal_position
	`, schema, table)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var columns []ColumnInfo
	for rows.Next() {
		var col ColumnInfo
		var nullable string
		var defaultVal sql.NullString

		if err := rows.Scan(&col.Name, &col.DataType, &nullable, &defaultVal); err != nil {
			return nil, err
		}

		col.IsNullable = nullable == "YES"
		if defaultVal.Valid {
			col.DefaultValue = &defaultVal.String
		}

		columns = append(columns, col)
	}
	return columns, rows.Err()
}
