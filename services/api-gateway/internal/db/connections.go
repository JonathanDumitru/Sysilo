package db

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
)

// ConnectionRepository handles connection database operations
type ConnectionRepository struct {
	db *DB
}

// CreateConnectionInput holds the data needed to create a connection
type CreateConnectionInput struct {
	TenantID             string
	Name                 string
	Description          string
	ConnectionType       string
	Config               map[string]interface{}
	CredentialsEncrypted []byte
	AgentID              string
}

// UpdateConnectionInput holds the data needed to update a connection
type UpdateConnectionInput struct {
	Name                 *string
	Description          *string
	Config               map[string]interface{}
	CredentialsEncrypted []byte
	AgentID              *string
	Status               *string
}

// List returns connections for a tenant with pagination
func (r *ConnectionRepository) List(ctx context.Context, tenantID string, opts ListOptions) (*ListResult[Connection], error) {
	// Count total
	var total int64
	countQuery := `SELECT COUNT(*) FROM connections WHERE tenant_id = $1`
	if err := r.db.queryRow(ctx, countQuery, tenantID).Scan(&total); err != nil {
		return nil, fmt.Errorf("failed to count connections: %w", err)
	}

	// Query connections
	query := `
		SELECT id, tenant_id, name, description, connection_type, config,
		       agent_id, status, last_tested_at, last_test_status, created_at, updated_at
		FROM connections
		WHERE tenant_id = $1
		ORDER BY name ASC
		LIMIT $2 OFFSET $3
	`

	rows, err := r.db.query(ctx, query, tenantID, opts.Limit(), opts.Offset())
	if err != nil {
		return nil, fmt.Errorf("failed to query connections: %w", err)
	}
	defer rows.Close()

	connections := make([]Connection, 0)
	for rows.Next() {
		var c Connection
		err := rows.Scan(
			&c.ID, &c.TenantID, &c.Name, &c.Description, &c.ConnectionType, &c.Config,
			&c.AgentID, &c.Status, &c.LastTestedAt, &c.LastTestStatus, &c.CreatedAt, &c.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan connection: %w", err)
		}
		c.Normalize()
		connections = append(connections, c)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating connections: %w", err)
	}

	limit := opts.Limit()
	totalPages := int(total) / limit
	if int(total)%limit > 0 {
		totalPages++
	}

	return &ListResult[Connection]{
		Items:      connections,
		Total:      total,
		Page:       opts.Page,
		PageSize:   limit,
		TotalPages: totalPages,
	}, nil
}

// GetByID returns a connection by ID
func (r *ConnectionRepository) GetByID(ctx context.Context, tenantID, connectionID string) (*Connection, error) {
	query := `
		SELECT id, tenant_id, name, description, connection_type, config,
		       agent_id, status, last_tested_at, last_test_status, created_at, updated_at
		FROM connections
		WHERE tenant_id = $1 AND id = $2
	`

	var c Connection
	err := r.db.queryRow(ctx, query, tenantID, connectionID).Scan(
		&c.ID, &c.TenantID, &c.Name, &c.Description, &c.ConnectionType, &c.Config,
		&c.AgentID, &c.Status, &c.LastTestedAt, &c.LastTestStatus, &c.CreatedAt, &c.UpdatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get connection: %w", err)
	}

	c.Normalize()
	return &c, nil
}

// Create creates a new connection
func (r *ConnectionRepository) Create(ctx context.Context, input CreateConnectionInput) (*Connection, error) {
	configJSON, err := json.Marshal(input.Config)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal config: %w", err)
	}

	query := `
		INSERT INTO connections (tenant_id, name, description, connection_type, config, credentials_encrypted, agent_id)
		VALUES ($1, $2, $3, $4, $5, $6, $7)
		RETURNING id, tenant_id, name, description, connection_type, config,
		          agent_id, status, last_tested_at, last_test_status, created_at, updated_at
	`

	var agentID interface{}
	if input.AgentID != "" {
		agentID = input.AgentID
	}

	var c Connection
	err = r.db.queryRow(ctx, query,
		input.TenantID, input.Name, input.Description, input.ConnectionType,
		configJSON, input.CredentialsEncrypted, agentID,
	).Scan(
		&c.ID, &c.TenantID, &c.Name, &c.Description, &c.ConnectionType, &c.Config,
		&c.AgentID, &c.Status, &c.LastTestedAt, &c.LastTestStatus, &c.CreatedAt, &c.UpdatedAt,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create connection: %w", err)
	}

	c.Normalize()
	return &c, nil
}

// Update updates an existing connection
func (r *ConnectionRepository) Update(ctx context.Context, tenantID, connectionID string, input UpdateConnectionInput) (*Connection, error) {
	// Build dynamic update query
	setClauses := []string{}
	args := []interface{}{}
	argNum := 1

	if input.Name != nil {
		setClauses = append(setClauses, fmt.Sprintf("name = $%d", argNum))
		args = append(args, *input.Name)
		argNum++
	}
	if input.Description != nil {
		setClauses = append(setClauses, fmt.Sprintf("description = $%d", argNum))
		args = append(args, *input.Description)
		argNum++
	}
	if input.Config != nil {
		configJSON, err := json.Marshal(input.Config)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal config: %w", err)
		}
		setClauses = append(setClauses, fmt.Sprintf("config = $%d", argNum))
		args = append(args, configJSON)
		argNum++
	}
	if input.CredentialsEncrypted != nil {
		setClauses = append(setClauses, fmt.Sprintf("credentials_encrypted = $%d", argNum))
		args = append(args, input.CredentialsEncrypted)
		argNum++
	}
	if input.AgentID != nil {
		var agentVal interface{}
		if *input.AgentID != "" {
			agentVal = *input.AgentID
		}
		setClauses = append(setClauses, fmt.Sprintf("agent_id = $%d", argNum))
		args = append(args, agentVal)
		argNum++
	}
	if input.Status != nil {
		setClauses = append(setClauses, fmt.Sprintf("status = $%d", argNum))
		args = append(args, *input.Status)
		argNum++
	}

	if len(setClauses) == 0 {
		// Nothing to update, just return current
		return r.GetByID(ctx, tenantID, connectionID)
	}

	// Add WHERE clause params
	args = append(args, tenantID, connectionID)

	query := fmt.Sprintf(`
		UPDATE connections
		SET %s, updated_at = NOW()
		WHERE tenant_id = $%d AND id = $%d
		RETURNING id, tenant_id, name, description, connection_type, config,
		          agent_id, status, last_tested_at, last_test_status, created_at, updated_at
	`, joinStrings(setClauses, ", "), argNum, argNum+1)

	var c Connection
	err := r.db.queryRow(ctx, query, args...).Scan(
		&c.ID, &c.TenantID, &c.Name, &c.Description, &c.ConnectionType, &c.Config,
		&c.AgentID, &c.Status, &c.LastTestedAt, &c.LastTestStatus, &c.CreatedAt, &c.UpdatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to update connection: %w", err)
	}

	c.Normalize()
	return &c, nil
}

// Delete removes a connection by ID
func (r *ConnectionRepository) Delete(ctx context.Context, tenantID, connectionID string) error {
	query := `DELETE FROM connections WHERE tenant_id = $1 AND id = $2`
	result, err := r.db.execStatement(ctx, query, tenantID, connectionID)
	if err != nil {
		return fmt.Errorf("failed to delete connection: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}
	if rows == 0 {
		return sql.ErrNoRows
	}

	return nil
}

// UpdateTestStatus updates the test status of a connection
func (r *ConnectionRepository) UpdateTestStatus(ctx context.Context, tenantID, connectionID, status string) error {
	query := `
		UPDATE connections
		SET last_tested_at = NOW(), last_test_status = $3, updated_at = NOW()
		WHERE tenant_id = $1 AND id = $2
	`
	_, err := r.db.execStatement(ctx, query, tenantID, connectionID, status)
	if err != nil {
		return fmt.Errorf("failed to update connection test status: %w", err)
	}
	return nil
}

// helper function
func joinStrings(strs []string, sep string) string {
	if len(strs) == 0 {
		return ""
	}
	result := strs[0]
	for i := 1; i < len(strs); i++ {
		result += sep + strs[i]
	}
	return result
}
