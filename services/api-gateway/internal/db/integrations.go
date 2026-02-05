package db

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
)

// IntegrationRepository handles integration database operations
type IntegrationRepository struct {
	db *DB
}

// CreateIntegrationInput holds the data needed to create an integration
type CreateIntegrationInput struct {
	TenantID    string
	Name        string
	Description string
	Definition  map[string]interface{}
	Schedule    map[string]interface{}
	Config      map[string]interface{}
	CreatedBy   string
}

// UpdateIntegrationInput holds the data needed to update an integration
type UpdateIntegrationInput struct {
	Name        *string
	Description *string
	Definition  map[string]interface{}
	Schedule    map[string]interface{}
	Config      map[string]interface{}
	Status      *string
	UpdatedBy   string
}

// List returns integrations for a tenant with pagination
func (r *IntegrationRepository) List(ctx context.Context, tenantID string, opts ListOptions) (*ListResult[Integration], error) {
	// Count total
	var total int64
	countQuery := `SELECT COUNT(*) FROM integrations WHERE tenant_id = $1`
	if err := r.db.queryRow(ctx, countQuery, tenantID).Scan(&total); err != nil {
		return nil, fmt.Errorf("failed to count integrations: %w", err)
	}

	// Query integrations
	query := `
		SELECT id, tenant_id, name, description, definition, version, status,
		       schedule, config, created_by, updated_by, created_at, updated_at
		FROM integrations
		WHERE tenant_id = $1
		ORDER BY name ASC
		LIMIT $2 OFFSET $3
	`

	rows, err := r.db.query(ctx, query, tenantID, opts.Limit(), opts.Offset())
	if err != nil {
		return nil, fmt.Errorf("failed to query integrations: %w", err)
	}
	defer rows.Close()

	integrations := make([]Integration, 0)
	for rows.Next() {
		var i Integration
		err := rows.Scan(
			&i.ID, &i.TenantID, &i.Name, &i.Description, &i.Definition, &i.Version,
			&i.Status, &i.Schedule, &i.Config, &i.CreatedBy, &i.UpdatedBy,
			&i.CreatedAt, &i.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan integration: %w", err)
		}
		i.Normalize()
		integrations = append(integrations, i)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating integrations: %w", err)
	}

	limit := opts.Limit()
	totalPages := int(total) / limit
	if int(total)%limit > 0 {
		totalPages++
	}

	return &ListResult[Integration]{
		Items:      integrations,
		Total:      total,
		Page:       opts.Page,
		PageSize:   limit,
		TotalPages: totalPages,
	}, nil
}

// GetByID returns an integration by ID
func (r *IntegrationRepository) GetByID(ctx context.Context, tenantID, integrationID string) (*Integration, error) {
	query := `
		SELECT id, tenant_id, name, description, definition, version, status,
		       schedule, config, created_by, updated_by, created_at, updated_at
		FROM integrations
		WHERE tenant_id = $1 AND id = $2
	`

	var i Integration
	err := r.db.queryRow(ctx, query, tenantID, integrationID).Scan(
		&i.ID, &i.TenantID, &i.Name, &i.Description, &i.Definition, &i.Version,
		&i.Status, &i.Schedule, &i.Config, &i.CreatedBy, &i.UpdatedBy,
		&i.CreatedAt, &i.UpdatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get integration: %w", err)
	}

	i.Normalize()
	return &i, nil
}

// Create creates a new integration
func (r *IntegrationRepository) Create(ctx context.Context, input CreateIntegrationInput) (*Integration, error) {
	definitionJSON, err := json.Marshal(input.Definition)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal definition: %w", err)
	}

	var scheduleJSON []byte
	if input.Schedule != nil {
		scheduleJSON, err = json.Marshal(input.Schedule)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal schedule: %w", err)
		}
	}

	configJSON := []byte("{}")
	if input.Config != nil {
		configJSON, err = json.Marshal(input.Config)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal config: %w", err)
		}
	}

	query := `
		INSERT INTO integrations (tenant_id, name, description, definition, schedule, config, created_by, updated_by)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
		RETURNING id, tenant_id, name, description, definition, version, status,
		          schedule, config, created_by, updated_by, created_at, updated_at
	`

	var createdBy interface{}
	if input.CreatedBy != "" {
		createdBy = input.CreatedBy
	}

	var i Integration
	err = r.db.queryRow(ctx, query,
		input.TenantID, input.Name, input.Description, definitionJSON,
		scheduleJSON, configJSON, createdBy,
	).Scan(
		&i.ID, &i.TenantID, &i.Name, &i.Description, &i.Definition, &i.Version,
		&i.Status, &i.Schedule, &i.Config, &i.CreatedBy, &i.UpdatedBy,
		&i.CreatedAt, &i.UpdatedAt,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create integration: %w", err)
	}

	i.Normalize()
	return &i, nil
}

// Update updates an existing integration
func (r *IntegrationRepository) Update(ctx context.Context, tenantID, integrationID string, input UpdateIntegrationInput) (*Integration, error) {
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
	if input.Definition != nil {
		definitionJSON, err := json.Marshal(input.Definition)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal definition: %w", err)
		}
		setClauses = append(setClauses, fmt.Sprintf("definition = $%d", argNum))
		args = append(args, definitionJSON)
		argNum++
		// Increment version when definition changes
		setClauses = append(setClauses, "version = version + 1")
	}
	if input.Schedule != nil {
		scheduleJSON, err := json.Marshal(input.Schedule)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal schedule: %w", err)
		}
		setClauses = append(setClauses, fmt.Sprintf("schedule = $%d", argNum))
		args = append(args, scheduleJSON)
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
	if input.Status != nil {
		setClauses = append(setClauses, fmt.Sprintf("status = $%d", argNum))
		args = append(args, *input.Status)
		argNum++
	}
	if input.UpdatedBy != "" {
		setClauses = append(setClauses, fmt.Sprintf("updated_by = $%d", argNum))
		args = append(args, input.UpdatedBy)
		argNum++
	}

	if len(setClauses) == 0 {
		return r.GetByID(ctx, tenantID, integrationID)
	}

	args = append(args, tenantID, integrationID)

	query := fmt.Sprintf(`
		UPDATE integrations
		SET %s, updated_at = NOW()
		WHERE tenant_id = $%d AND id = $%d
		RETURNING id, tenant_id, name, description, definition, version, status,
		          schedule, config, created_by, updated_by, created_at, updated_at
	`, joinStrings(setClauses, ", "), argNum, argNum+1)

	var i Integration
	err := r.db.queryRow(ctx, query, args...).Scan(
		&i.ID, &i.TenantID, &i.Name, &i.Description, &i.Definition, &i.Version,
		&i.Status, &i.Schedule, &i.Config, &i.CreatedBy, &i.UpdatedBy,
		&i.CreatedAt, &i.UpdatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to update integration: %w", err)
	}

	i.Normalize()
	return &i, nil
}

// Delete removes an integration by ID
func (r *IntegrationRepository) Delete(ctx context.Context, tenantID, integrationID string) error {
	query := `DELETE FROM integrations WHERE tenant_id = $1 AND id = $2`
	result, err := r.db.execStatement(ctx, query, tenantID, integrationID)
	if err != nil {
		return fmt.Errorf("failed to delete integration: %w", err)
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
