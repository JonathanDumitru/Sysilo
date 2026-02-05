package db

import (
	"context"
	"database/sql"
	"fmt"
	"time"

	_ "github.com/lib/pq"
	"github.com/sysilo/sysilo/services/api-gateway/internal/config"
	"go.uber.org/zap"
)

// DB wraps the database connection and provides access to repositories
type DB struct {
	conn   *sql.DB
	logger *zap.Logger

	Agents       *AgentRepository
	Connections  *ConnectionRepository
	Integrations *IntegrationRepository
	Runs         *RunRepository
	Users        *UserRepository
}

// New creates a new database connection and initializes repositories
func New(cfg config.DatabaseConfig, logger *zap.Logger) (*DB, error) {
	conn, err := sql.Open("postgres", cfg.DSN())
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %w", err)
	}

	// Configure connection pool
	conn.SetMaxOpenConns(cfg.MaxOpenConns)
	conn.SetMaxIdleConns(cfg.MaxIdleConns)
	conn.SetConnMaxLifetime(5 * time.Minute)

	// Verify connection
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := conn.PingContext(ctx); err != nil {
		conn.Close()
		return nil, fmt.Errorf("failed to ping database: %w", err)
	}

	logger.Info("Database connection established",
		zap.String("host", cfg.Host),
		zap.Int("port", cfg.Port),
		zap.String("database", cfg.Database),
	)

	db := &DB{
		conn:   conn,
		logger: logger,
	}

	// Initialize repositories
	db.Agents = &AgentRepository{db: db}
	db.Connections = &ConnectionRepository{db: db}
	db.Integrations = &IntegrationRepository{db: db}
	db.Runs = &RunRepository{db: db}
	db.Users = &UserRepository{db: db}

	return db, nil
}

// Close closes the database connection
func (db *DB) Close() error {
	return db.conn.Close()
}

// Ping verifies the database connection is alive
func (db *DB) Ping(ctx context.Context) error {
	return db.conn.PingContext(ctx)
}

// queryRow is a helper for single-row queries
func (db *DB) queryRow(ctx context.Context, query string, args ...interface{}) *sql.Row {
	return db.conn.QueryRowContext(ctx, query, args...)
}

// query is a helper for multi-row queries
func (db *DB) query(ctx context.Context, query string, args ...interface{}) (*sql.Rows, error) {
	return db.conn.QueryContext(ctx, query, args...)
}

// execStatement is a helper for non-query statements
func (db *DB) execStatement(ctx context.Context, query string, args ...interface{}) (sql.Result, error) {
	return db.conn.ExecContext(ctx, query, args...)
}
