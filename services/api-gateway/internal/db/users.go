package db

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	"time"

	"github.com/lib/pq"
)

// UserRepository handles user database operations
type UserRepository struct {
	db *DB
}

// CreateUserInput holds the data needed to create a user
type CreateUserInput struct {
	TenantID           string
	Email              string
	Name               string
	PasswordHash       string
	Roles              []string
	AuthSource         string
	IDPSubject         string
	BreakglassEligible bool
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
		SELECT id, tenant_id, email, name, roles, status, auth_source, idp_subject,
		       session_version, breakglass_eligible, last_login_at, last_breakglass_login_at,
		       created_at, updated_at
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
			&u.Status, &u.AuthSource, &u.IDPSubject, &u.SessionVersion, &u.BreakglassEligible,
			&u.LastLoginAt, &u.LastBreakglassLogin, &u.CreatedAt, &u.UpdatedAt,
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
		SELECT id, tenant_id, email, name, roles, status, auth_source, idp_subject,
		       session_version, breakglass_eligible, last_login_at, last_breakglass_login_at,
		       created_at, updated_at
		FROM users
		WHERE tenant_id = $1 AND id = $2
	`

	var u User
	err := r.db.queryRow(ctx, query, tenantID, userID).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, pq.Array(&u.Roles),
		&u.Status, &u.AuthSource, &u.IDPSubject, &u.SessionVersion, &u.BreakglassEligible,
		&u.LastLoginAt, &u.LastBreakglassLogin, &u.CreatedAt, &u.UpdatedAt,
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
		SELECT id, tenant_id, email, name, password_hash, roles, status, auth_source, idp_subject,
		       session_version, breakglass_eligible, last_login_at, last_breakglass_login_at,
		       created_at, updated_at
		FROM users
		WHERE tenant_id = $1 AND email = $2
	`

	var u User
	err := r.db.queryRow(ctx, query, tenantID, email).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, &u.PasswordHash, pq.Array(&u.Roles),
		&u.Status, &u.AuthSource, &u.IDPSubject, &u.SessionVersion, &u.BreakglassEligible,
		&u.LastLoginAt, &u.LastBreakglassLogin, &u.CreatedAt, &u.UpdatedAt,
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
	if input.AuthSource == "" {
		input.AuthSource = "local"
	}

	query := `
		INSERT INTO users (tenant_id, email, name, password_hash, roles, auth_source, idp_subject, breakglass_eligible)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
		RETURNING id, tenant_id, email, name, roles, status, auth_source, idp_subject,
		          session_version, breakglass_eligible, last_login_at, last_breakglass_login_at,
		          created_at, updated_at
	`

	var passwordHash interface{}
	if input.PasswordHash != "" {
		passwordHash = input.PasswordHash
	}

	var idpSubject interface{}
	if input.IDPSubject != "" {
		idpSubject = input.IDPSubject
	}

	var u User
	err := r.db.queryRow(ctx, query,
		input.TenantID, input.Email, input.Name, passwordHash, pq.Array(input.Roles),
		input.AuthSource, idpSubject, input.BreakglassEligible,
	).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, pq.Array(&u.Roles),
		&u.Status, &u.AuthSource, &u.IDPSubject, &u.SessionVersion, &u.BreakglassEligible,
		&u.LastLoginAt, &u.LastBreakglassLogin, &u.CreatedAt, &u.UpdatedAt,
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
		RETURNING id, tenant_id, email, name, roles, status, auth_source, idp_subject,
		          session_version, breakglass_eligible, last_login_at, last_breakglass_login_at,
		          created_at, updated_at
	`, joinStrings(setClauses, ", "), argNum, argNum+1)

	var u User
	err := r.db.queryRow(ctx, query, args...).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, pq.Array(&u.Roles),
		&u.Status, &u.AuthSource, &u.IDPSubject, &u.SessionVersion, &u.BreakglassEligible,
		&u.LastLoginAt, &u.LastBreakglassLogin, &u.CreatedAt, &u.UpdatedAt,
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

// UpsertJITBySubject creates or updates a user from OIDC claims.
func (r *UserRepository) UpsertJITBySubject(ctx context.Context, tenantID, subject, email, name string) (*User, error) {
	query := `
		INSERT INTO users (tenant_id, email, name, auth_source, idp_subject, roles, status)
		VALUES ($1, $2, $3, 'sso', $4, $5, 'active')
		ON CONFLICT (tenant_id, email)
		DO UPDATE SET
			name = EXCLUDED.name,
			auth_source = 'sso',
			idp_subject = EXCLUDED.idp_subject,
			status = 'active',
			updated_at = NOW()
		RETURNING id, tenant_id, email, name, roles, status, auth_source, idp_subject,
		          session_version, breakglass_eligible, last_login_at, last_breakglass_login_at,
		          created_at, updated_at
	`

	var u User
	err := r.db.queryRow(ctx, query, tenantID, email, name, subject, pq.Array([]string{"viewer"})).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, pq.Array(&u.Roles),
		&u.Status, &u.AuthSource, &u.IDPSubject, &u.SessionVersion, &u.BreakglassEligible,
		&u.LastLoginAt, &u.LastBreakglassLogin, &u.CreatedAt, &u.UpdatedAt,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to upsert jit user: %w", err)
	}
	u.Normalize()
	return &u, nil
}

// SetStatusAndRotateSession marks a user active/inactive and rotates session version when changed.
func (r *UserRepository) SetStatusAndRotateSession(ctx context.Context, tenantID, userID, status string) error {
	query := `
		UPDATE users
		SET status = $3,
		    session_version = session_version + 1,
		    updated_at = NOW()
		WHERE tenant_id = $1 AND id = $2
	`
	_, err := r.db.execStatement(ctx, query, tenantID, userID, status)
	if err != nil {
		return fmt.Errorf("failed to set user status: %w", err)
	}
	return nil
}

type AuthProfile struct {
	Status         string
	SessionVersion int
}

// GetAuthProfile fetches request-time user status and session version.
func (r *UserRepository) GetAuthProfile(ctx context.Context, tenantID, userID string) (*AuthProfile, error) {
	query := `SELECT status, session_version FROM users WHERE tenant_id = $1 AND id = $2`
	var profile AuthProfile
	if err := r.db.queryRow(ctx, query, tenantID, userID).Scan(&profile.Status, &profile.SessionVersion); err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, fmt.Errorf("failed to get auth profile: %w", err)
	}
	return &profile, nil
}

// RecordBreakglassLogin stamps break-glass auth usage and writes an audit event.
func (r *UserRepository) RecordBreakglassLogin(ctx context.Context, tenantID, userID, reason string) error {
	_, err := r.db.execStatement(ctx,
		`UPDATE users SET last_breakglass_login_at = NOW(), last_login_at = NOW() WHERE tenant_id = $1 AND id = $2`,
		tenantID, userID,
	)
	if err != nil {
		return fmt.Errorf("failed to update breakglass timestamp: %w", err)
	}

	_, err = r.db.execStatement(ctx, `
		INSERT INTO audit_logs (tenant_id, user_id, action, resource_type, changes)
		VALUES ($1, $2, 'auth.breakglass.login', 'user', $3::jsonb)
	`, tenantID, userID, fmt.Sprintf(`{"result":"success","reason":%q}`, reason))
	if err != nil {
		return fmt.Errorf("failed to write breakglass audit event: %w", err)
	}
	return nil
}

type refreshTokenRow struct {
	UserID    string
	ExpiresAt time.Time
	RevokedAt sql.NullTime
	UsedAt    sql.NullTime
}

// StoreRefreshToken persists a refresh token hash for later rotation/revocation checks.
func (r *UserRepository) StoreRefreshToken(ctx context.Context, tenantID, userID, tokenHash string, expiresAt time.Time) error {
	_, err := r.db.execStatement(ctx, `
		INSERT INTO refresh_tokens (tenant_id, user_id, token_hash, expires_at)
		VALUES ($1, $2, $3, $4)
	`, tenantID, userID, tokenHash, expiresAt)
	if err != nil {
		return fmt.Errorf("failed to store refresh token: %w", err)
	}
	return nil
}

// RotateRefreshToken atomically consumes one refresh token and stores the rotated token.
func (r *UserRepository) RotateRefreshToken(ctx context.Context, tenantID, oldTokenHash, newTokenHash string, newExpiresAt time.Time) (*User, error) {
	tx, err := r.db.conn.BeginTx(ctx, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to start refresh rotation transaction: %w", err)
	}
	defer tx.Rollback()

	var rt refreshTokenRow
	err = tx.QueryRowContext(ctx, `
		SELECT user_id, expires_at, revoked_at, used_at
		FROM refresh_tokens
		WHERE tenant_id = $1 AND token_hash = $2
		FOR UPDATE
	`, tenantID, oldTokenHash).Scan(&rt.UserID, &rt.ExpiresAt, &rt.RevokedAt, &rt.UsedAt)
	if err == sql.ErrNoRows {
		return nil, errors.New("refresh token not found")
	}
	if err != nil {
		return nil, fmt.Errorf("failed to read refresh token: %w", err)
	}
	if rt.RevokedAt.Valid || rt.UsedAt.Valid || time.Now().After(rt.ExpiresAt) {
		return nil, errors.New("refresh token is not active")
	}

	var profile AuthProfile
	err = tx.QueryRowContext(ctx, `
		SELECT status, session_version
		FROM users
		WHERE tenant_id = $1 AND id = $2
		FOR UPDATE
	`, tenantID, rt.UserID).Scan(&profile.Status, &profile.SessionVersion)
	if err == sql.ErrNoRows {
		return nil, errors.New("user not found")
	}
	if err != nil {
		return nil, fmt.Errorf("failed to read refresh token user: %w", err)
	}
	if profile.Status != "active" {
		return nil, errors.New("user is inactive")
	}

	_, err = tx.ExecContext(ctx, `
		UPDATE refresh_tokens
		SET used_at = NOW(), revoked_at = NOW(), replaced_by_hash = $3
		WHERE tenant_id = $1 AND token_hash = $2
	`, tenantID, oldTokenHash, newTokenHash)
	if err != nil {
		return nil, fmt.Errorf("failed to consume refresh token: %w", err)
	}

	_, err = tx.ExecContext(ctx, `
		INSERT INTO refresh_tokens (tenant_id, user_id, token_hash, expires_at)
		VALUES ($1, $2, $3, $4)
	`, tenantID, rt.UserID, newTokenHash, newExpiresAt)
	if err != nil {
		return nil, fmt.Errorf("failed to store rotated refresh token: %w", err)
	}

	if err := tx.Commit(); err != nil {
		return nil, fmt.Errorf("failed to commit refresh token rotation: %w", err)
	}

	user, err := r.GetByID(ctx, tenantID, rt.UserID)
	if err != nil {
		return nil, err
	}
	if user == nil {
		return nil, errors.New("user not found")
	}
	return user, nil
}

// UpsertSCIMBySubject creates or updates user identity from SCIM payloads.
func (r *UserRepository) UpsertSCIMBySubject(ctx context.Context, tenantID, subject, email, name string, active bool) (*User, error) {
	status := "inactive"
	if active {
		status = "active"
	}

	query := `
		INSERT INTO users (tenant_id, email, name, auth_source, idp_subject, roles, status)
		VALUES ($1, $2, $3, 'sso', $4, $5, $6)
		ON CONFLICT (tenant_id, email)
		DO UPDATE SET
			name = EXCLUDED.name,
			auth_source = 'sso',
			idp_subject = EXCLUDED.idp_subject,
			status = EXCLUDED.status,
			session_version = CASE WHEN users.status IS DISTINCT FROM EXCLUDED.status
				THEN users.session_version + 1
				ELSE users.session_version
			END,
			updated_at = NOW()
		RETURNING id, tenant_id, email, name, roles, status, auth_source, idp_subject,
		          session_version, breakglass_eligible, last_login_at, last_breakglass_login_at,
		          created_at, updated_at
	`

	var u User
	err := r.db.queryRow(ctx, query, tenantID, email, name, subject, pq.Array([]string{"viewer"}), status).Scan(
		&u.ID, &u.TenantID, &u.Email, &u.Name, pq.Array(&u.Roles),
		&u.Status, &u.AuthSource, &u.IDPSubject, &u.SessionVersion, &u.BreakglassEligible,
		&u.LastLoginAt, &u.LastBreakglassLogin, &u.CreatedAt, &u.UpdatedAt,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to upsert SCIM user: %w", err)
	}
	u.Normalize()
	return &u, nil
}

// DeactivateBySubject marks an SCIM subject inactive and rotates session version.
func (r *UserRepository) DeactivateBySubject(ctx context.Context, tenantID, subject string) error {
	result, err := r.db.execStatement(ctx, `
		UPDATE users
		SET status = 'inactive',
		    session_version = session_version + 1,
		    updated_at = NOW()
		WHERE tenant_id = $1 AND idp_subject = $2
	`, tenantID, subject)
	if err != nil {
		return fmt.Errorf("failed to deactivate user by subject: %w", err)
	}
	rows, err := result.RowsAffected()
	if err != nil {
		return fmt.Errorf("failed to inspect deactivate result: %w", err)
	}
	if rows == 0 {
		return sql.ErrNoRows
	}
	return nil
}
