package db

import (
	"context"
	"database/sql"
	"fmt"

	"github.com/lib/pq"
)

// UserRepository handles user database operations
type UserRepository struct {
	db *DB
}

// CreateUserInput holds the data needed to create a user
type CreateUserInput struct {
	TenantID     string
	Email        string
	Name         string
	PasswordHash string
	Roles        []string
}

// UpdateUserInput holds the data needed to update a user
type UpdateUserInput struct {
	Email        *string
	Name         *string
	PasswordHash *string
	Roles        []string
	Status       *string
}

// List returns users for a tenant with pagination
func (r *UserRepository) List(ctx context.Context, tenantID string, opts ListOptions) (*ListResult[User], error) {
	// Count total
	var total int64
	countQuery := `SELECT COUNT(*) FROM users WHERE tenant_id = $1`
	if err := r.db.queryRow(ctx, countQuery, tenantID).Scan(&total); err != nil {
		return nil, fmt.Errorf("failed to count users: %w", err)
	}

	// Query users
	query := `
		SELECT id, tenant_id, email, name, roles, status, last_login_at, created_at, updated_at
		FROM users
		WHERE tenant_id = $1
		ORDER BY email ASC
		LIMIT $2 OFFSET $3
	`

	rows, err := r.db.query(ctx, query, tenantID, opts.Limit(), opts.Offset())
	if err != nil {
		return nil, fmt.Errorf("failed to query users: %w", err)
	}
	defer rows.Close()

	users := make([]User, 0)
	for rows.Next() {
		var u User
		err := rows.Scan(
			&u.ID, &u.TenantID, &u.Email, &u.Name, pq.Array(&u.Roles),
			&u.Status, &u.LastLoginAt, &u.CreatedAt, &u.UpdatedAt,
		)
		if err != nil {
			return nil, fmt.Errorf("failed to scan user: %w", err)
		}
		u.Normalize()
		users = append(users, u)
	}

	if err := rows.Err(); err != nil {
		return nil, fmt.Errorf("error iterating users: %w", err)
	}

	limit := opts.Limit()
	totalPages := int(total) / limit
	if int(total)%limit > 0 {
		totalPages++
	}

	return &ListResult[User]{
		Items:      users,
		Total:      total,
		Page:       opts.Page,
		PageSize:   limit,
		TotalPages: totalPages,
	}, nil
}

// GetByID returns a user by ID
func (r *UserRepository) GetByID(ctx context.Context, tenantID, userID string) (*User, error) {
	query := `
		SELECT id, tenant_id, email, name, roles, status, last_login_at, created_at, updated_at
		FROM users
		WHERE tenant_id = $1 AND id = $2
	`

	var u User
	err := r.db.queryRow(ctx, query, tenantID, userID).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, pq.Array(&u.Roles),
		&u.Status, &u.LastLoginAt, &u.CreatedAt, &u.UpdatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get user: %w", err)
	}

	u.Normalize()
	return &u, nil
}

// GetByEmail returns a user by email (for auth)
func (r *UserRepository) GetByEmail(ctx context.Context, tenantID, email string) (*User, error) {
	query := `
		SELECT id, tenant_id, email, name, password_hash, roles, status, last_login_at, created_at, updated_at
		FROM users
		WHERE tenant_id = $1 AND email = $2
	`

	var u User
	err := r.db.queryRow(ctx, query, tenantID, email).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, &u.PasswordHash, pq.Array(&u.Roles),
		&u.Status, &u.LastLoginAt, &u.CreatedAt, &u.UpdatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to get user by email: %w", err)
	}

	u.Normalize()
	return &u, nil
}

// Create creates a new user
func (r *UserRepository) Create(ctx context.Context, input CreateUserInput) (*User, error) {
	if len(input.Roles) == 0 {
		input.Roles = []string{"viewer"}
	}

	query := `
		INSERT INTO users (tenant_id, email, name, password_hash, roles)
		VALUES ($1, $2, $3, $4, $5)
		RETURNING id, tenant_id, email, name, roles, status, last_login_at, created_at, updated_at
	`

	var passwordHash interface{}
	if input.PasswordHash != "" {
		passwordHash = input.PasswordHash
	}

	var u User
	err := r.db.queryRow(ctx, query,
		input.TenantID, input.Email, input.Name, passwordHash, pq.Array(input.Roles),
	).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, pq.Array(&u.Roles),
		&u.Status, &u.LastLoginAt, &u.CreatedAt, &u.UpdatedAt,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create user: %w", err)
	}

	u.Normalize()
	return &u, nil
}

// Update updates an existing user
func (r *UserRepository) Update(ctx context.Context, tenantID, userID string, input UpdateUserInput) (*User, error) {
	setClauses := []string{}
	args := []interface{}{}
	argNum := 1

	if input.Email != nil {
		setClauses = append(setClauses, fmt.Sprintf("email = $%d", argNum))
		args = append(args, *input.Email)
		argNum++
	}
	if input.Name != nil {
		setClauses = append(setClauses, fmt.Sprintf("name = $%d", argNum))
		args = append(args, *input.Name)
		argNum++
	}
	if input.PasswordHash != nil {
		setClauses = append(setClauses, fmt.Sprintf("password_hash = $%d", argNum))
		args = append(args, *input.PasswordHash)
		argNum++
	}
	if input.Roles != nil {
		setClauses = append(setClauses, fmt.Sprintf("roles = $%d", argNum))
		args = append(args, pq.Array(input.Roles))
		argNum++
	}
	if input.Status != nil {
		setClauses = append(setClauses, fmt.Sprintf("status = $%d", argNum))
		args = append(args, *input.Status)
		argNum++
	}

	if len(setClauses) == 0 {
		return r.GetByID(ctx, tenantID, userID)
	}

	args = append(args, tenantID, userID)

	query := fmt.Sprintf(`
		UPDATE users
		SET %s, updated_at = NOW()
		WHERE tenant_id = $%d AND id = $%d
		RETURNING id, tenant_id, email, name, roles, status, last_login_at, created_at, updated_at
	`, joinStrings(setClauses, ", "), argNum, argNum+1)

	var u User
	err := r.db.queryRow(ctx, query, args...).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, pq.Array(&u.Roles),
		&u.Status, &u.LastLoginAt, &u.CreatedAt, &u.UpdatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("failed to update user: %w", err)
	}

	u.Normalize()
	return &u, nil
}

// Delete removes a user by ID
func (r *UserRepository) Delete(ctx context.Context, tenantID, userID string) error {
	query := `DELETE FROM users WHERE tenant_id = $1 AND id = $2`
	result, err := r.db.execStatement(ctx, query, tenantID, userID)
	if err != nil {
		return fmt.Errorf("failed to delete user: %w", err)
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

// UpdateLastLogin updates the last login timestamp
func (r *UserRepository) UpdateLastLogin(ctx context.Context, tenantID, userID string) error {
	query := `UPDATE users SET last_login_at = NOW() WHERE tenant_id = $1 AND id = $2`
	_, err := r.db.execStatement(ctx, query, tenantID, userID)
	if err != nil {
		return fmt.Errorf("failed to update last login: %w", err)
	}
	return nil
}
