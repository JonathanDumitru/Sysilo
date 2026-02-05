package db

import (
	"context"
	"database/sql"
	"fmt"
)

// RunRepository handles integration run database operations
type RunRepository struct {
	db *DB
}

// CreateRunInput holds the data needed to create an integration run
type CreateRunInput struct {
	TenantID           string
	IntegrationID      string
	IntegrationVersion int
	TriggerType        string
	TriggeredBy        string
}

// ListByIntegration returns runs for a specific integration with pagination
func (r *RunRepository) ListByIntegration(ctx context.Context, tenantID, integrationID string, opts ListOptions) (*ListResult[IntegrationRun], error) {
	// Count total
	var total int64
	countQuery := `SELECT COUNT(*) FROM integration_runs WHERE tenant_id = $1 AND integration_id = $2`
	if err := r.db.queryRow(ctx, countQuery, tenantID, integrationID).Scan(&total); err != nil {
		return nil, fmt.Errorf("failed to count runs: %w", err)
	}

	// Query runs
	query := `
		SELECT id, tenant_id, integration_id, integration_version, status, trigger_type,
		       triggered_by, agent_id, started_at, completed_at, error_message, error_details,
		       metrics, created_at
		FROM integration_runs
		WHERE tenant_id = $1 AND integration_id = $2
		ORDER BY created_at DESC
		LIMIT $3 OFFSET $4
	`

	rows, err := r.db.query(ctx, query, tenantID, integrationID, opts.Limit(), opts.Offset())
	if err != nil {
		return nil, fmt.Errorf("failed to query runs: %w", err)
	}
	defer rows.Close()

	runs := make([]IntegrationRun, 0)
	for rows.Next() {
		var run IntegrationRun
		err := rows.Scan(
			&run.ID, &run.TenantID, &run.IntegrationID, &run.IntegrationVersion,
			&run.Status, &run.TriggerType, &run.TriggeredBy, &run.AgentID,
			&run.StartedAt, &run.CompletedAt, &run.ErrorMessage, &run.ErrorDetails,
			&run.Metrics, &run.CreatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan run: %w", err)
		}
		run.Normalize()
		runs = append(runs, run)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating runs: %w", err)
	}

	limit := opts.Limit()
	totalPages := int(total) / limit
	if int(total)%limit > 0 {
		totalPages++
	}

	return &ListResult[IntegrationRun]{
		Items:      runs,
		Total:      total,
		Page:       opts.Page,
		PageSize:   limit,
		TotalPages: totalPages,
	}, nil
}

// GetByID returns a run by ID
func (r *RunRepository) GetByID(ctx context.Context, tenantID, runID string) (*IntegrationRun, error) {
	query := `
		SELECT id, tenant_id, integration_id, integration_version, status, trigger_type,
		       triggered_by, agent_id, started_at, completed_at, error_message, error_details,
		       metrics, created_at
		FROM integration_runs
		WHERE tenant_id = $1 AND id = $2
	`

	var run IntegrationRun
	err := r.db.queryRow(ctx, query, tenantID, runID).Scan(
		&run.ID, &run.TenantID, &run.IntegrationID, &run.IntegrationVersion,
		&run.Status, &run.TriggerType, &run.TriggeredBy, &run.AgentID,
		&run.StartedAt, &run.CompletedAt, &run.ErrorMessage, &run.ErrorDetails,
		&run.Metrics, &run.CreatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get run: %w", err)
	}

	run.Normalize()
	return &run, nil
}

// Create creates a new integration run
func (r *RunRepository) Create(ctx context.Context, input CreateRunInput) (*IntegrationRun, error) {
	query := `
		INSERT INTO integration_runs (tenant_id, integration_id, integration_version, trigger_type, triggered_by)
		VALUES ($1, $2, $3, $4, $5)
		RETURNING id, tenant_id, integration_id, integration_version, status, trigger_type,
		          triggered_by, agent_id, started_at, completed_at, error_message, error_details,
		          metrics, created_at
	`

	var triggeredBy interface{}
	if input.TriggeredBy != "" {
		triggeredBy = input.TriggeredBy
	}

	var run IntegrationRun
	err := r.db.queryRow(ctx, query,
		input.TenantID, input.IntegrationID, input.IntegrationVersion,
		input.TriggerType, triggeredBy,
	).Scan(
		&run.ID, &run.TenantID, &run.IntegrationID, &run.IntegrationVersion,
		&run.Status, &run.TriggerType, &run.TriggeredBy, &run.AgentID,
		&run.StartedAt, &run.CompletedAt, &run.ErrorMessage, &run.ErrorDetails,
		&run.Metrics, &run.CreatedAt,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create run: %w", err)
	}

	run.Normalize()
	return &run, nil
}

// UpdateStatus updates a run's status
func (r *RunRepository) UpdateStatus(ctx context.Context, tenantID, runID, status string) error {
	query := `UPDATE integration_runs SET status = $3 WHERE tenant_id = $1 AND id = $2`
	_, err := r.db.execStatement(ctx, query, tenantID, runID, status)
	if err != nil {
		return fmt.Errorf("failed to update run status: %w", err)
	}
	return nil
}

// Cancel marks a run as cancelled
func (r *RunRepository) Cancel(ctx context.Context, tenantID, runID string) error {
	query := `
		UPDATE integration_runs
		SET status = 'cancelled', completed_at = NOW()
		WHERE tenant_id = $1 AND id = $2 AND status IN ('pending', 'running')
	`
	result, err := r.db.execStatement(ctx, query, tenantID, runID)
	if err != nil {
		return fmt.Errorf("failed to cancel run: %w", err)
	}

	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to get rows affected: %w", err)
	}
	if rows == 0 {
		return sql.ErrNoRows // Run not found or not in cancellable state
	}

	return nil
}

// GetLogs returns logs for a run (from tasks table for now)
func (r *RunRepository) GetLogs(ctx context.Context, tenantID, runID string, opts ListOptions) (*ListResult[Task], error) {
	// Count total tasks for this run
	var total int64
	countQuery := `SELECT COUNT(*) FROM tasks WHERE tenant_id = $1 AND integration_run_id = $2`
	if err := r.db.queryRow(ctx, countQuery, tenantID, runID).Scan(&total); err != nil {
		return nil, fmt.Errorf("failed to count tasks: %w", err)
	}

	// Query tasks
	query := `
		SELECT id, tenant_id, integration_run_id, agent_id, task_type, priority, config,
		       status, retry_count, max_retries, timeout_seconds, scheduled_at,
		       started_at, completed_at, result, error_message, error_code, created_at
		FROM tasks
		WHERE tenant_id = $1 AND integration_run_id = $2
		ORDER BY created_at ASC
		LIMIT $3 OFFSET $4
	`

	rows, err := r.db.query(ctx, query, tenantID, runID, opts.Limit(), opts.Offset())
	if err != nil {
		return nil, fmt.Errorf("failed to query tasks: %w", err)
	}
	defer rows.Close()

	tasks := make([]Task, 0)
	for rows.Next() {
		var t Task
		err := rows.Scan(
			&t.ID, &t.TenantID, &t.IntegrationRunID, &t.AgentID, &t.TaskType, &t.Priority,
			&t.Config, &t.Status, &t.RetryCount, &t.MaxRetries, &t.TimeoutSeconds,
			&t.ScheduledAt, &t.StartedAt, &t.CompletedAt, &t.Result, &t.ErrorMessage,
			&t.ErrorCode, &t.CreatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan task: %w", err)
		}
		t.Normalize()
		tasks = append(tasks, t)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating tasks: %w", err)
	}

	limit := opts.Limit()
	totalPages := int(total) / limit
	if int(total)%limit > 0 {
		totalPages++
	}

	return &ListResult[Task]{
		Items:      tasks,
		Total:      total,
		Page:       opts.Page,
		PageSize:   limit,
		TotalPages: totalPages,
	}, nil
}
