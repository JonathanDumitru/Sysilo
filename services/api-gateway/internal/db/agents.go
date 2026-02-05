package db

import (
	"context"
	"database/sql"
	"fmt"
)

// AgentRepository handles agent database operations
type AgentRepository struct {
	db *DB
}

// List returns agents for a tenant with pagination
func (r *AgentRepository) List(ctx context.Context, tenantID string, opts ListOptions) (*ListResult[Agent], error) {
	// Count total
	var total int64
	countQuery := `SELECT COUNT(*) FROM agents WHERE tenant_id = $1`
	if err := r.db.queryRow(ctx, countQuery, tenantID).Scan(&total); err != nil {
		return nil, fmt.Errorf("failed to count agents: %w", err)
	}

	// Query agents
	query := `
		SELECT id, tenant_id, name, description, status, version, capabilities, labels,
		       last_heartbeat_at, last_connected_at, config, created_at, updated_at
		FROM agents
		WHERE tenant_id = $1
		ORDER BY created_at DESC
		LIMIT $2 OFFSET $3
	`

	rows, err := r.db.query(ctx, query, tenantID, opts.Limit(), opts.Offset())
	if err != nil {
		return nil, fmt.Errorf("failed to query agents: %w", err)
	}
	defer rows.Close()

	agents := make([]Agent, 0)
	for rows.Next() {
		var a Agent
		err := rows.Scan(
			&a.ID, &a.TenantID, &a.Name, &a.Description, &a.Status, &a.Version,
			&a.Capabilities, &a.Labels, &a.LastHeartbeatAt, &a.LastConnectedAt,
			&a.Config, &a.CreatedAt, &a.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan agent: %w", err)
		}
		a.Normalize()
		agents = append(agents, a)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating agents: %w", err)
	}

	limit := opts.Limit()
	totalPages := int(total) / limit
	if int(total)%limit > 0 {
		totalPages++
	}

	return &ListResult[Agent]{
		Items:      agents,
		Total:      total,
		Page:       opts.Page,
		PageSize:   limit,
		TotalPages: totalPages,
	}, nil
}

// GetByID returns an agent by ID
func (r *AgentRepository) GetByID(ctx context.Context, tenantID, agentID string) (*Agent, error) {
	query := `
		SELECT id, tenant_id, name, description, status, version, capabilities, labels,
		       last_heartbeat_at, last_connected_at, config, created_at, updated_at
		FROM agents
		WHERE tenant_id = $1 AND id = $2
	`

	var a Agent
	err := r.db.queryRow(ctx, query, tenantID, agentID).Scan(
		&a.ID, &a.TenantID, &a.Name, &a.Description, &a.Status, &a.Version,
		&a.Capabilities, &a.Labels, &a.LastHeartbeatAt, &a.LastConnectedAt,
		&a.Config, &a.CreatedAt, &a.UpdatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get agent: %w", err)
	}

	a.Normalize()
	return &a, nil
}

// Delete removes an agent by ID
func (r *AgentRepository) Delete(ctx context.Context, tenantID, agentID string) error {
	query := `DELETE FROM agents WHERE tenant_id = $1 AND id = $2`
	result, err := r.db.execStatement(ctx, query, tenantID, agentID)
	if err != nil {
		return fmt.Errorf("failed to delete agent: %w", err)
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

// UpdateStatus updates an agent's status
func (r *AgentRepository) UpdateStatus(ctx context.Context, tenantID, agentID, status string) error {
	query := `UPDATE agents SET status = $3, updated_at = NOW() WHERE tenant_id = $1 AND id = $2`
	_, err := r.db.execStatement(ctx, query, tenantID, agentID, status)
	if err != nil {
		return fmt.Errorf("failed to update agent status: %w", err)
	}
	return nil
}
