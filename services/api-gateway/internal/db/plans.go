package db

import (
	"context"
	"database/sql"
	"encoding/json"
	"time"
)

// Plan represents a billing plan
type Plan struct {
	ID              string          `json:"id"`
	Name            string          `json:"name"`
	DisplayName     string          `json:"display_name"`
	Description     sql.NullString  `json:"-"`
	DescriptionStr  string          `json:"description,omitempty"`
	PriceCents      int             `json:"price_cents"`
	BillingInterval string          `json:"billing_interval"`
	StripePriceID   sql.NullString  `json:"-"`
	StripePriceStr  string          `json:"stripe_price_id,omitempty"`
	IsActive        bool            `json:"is_active"`
	Limits          json.RawMessage `json:"limits"`
	Features        json.RawMessage `json:"features"`
	CreatedAt       time.Time       `json:"created_at"`
}

func (p *Plan) Normalize() {
	if p.Description.Valid {
		p.DescriptionStr = p.Description.String
	}
	if p.StripePriceID.Valid {
		p.StripePriceStr = p.StripePriceID.String
	}
}

// PlanLimits represents the parsed limits from a plan
type PlanLimits struct {
	MaxUsers          int `json:"max_users"`
	MaxIntegrations   int `json:"max_integrations"`
	MaxConnections    int `json:"max_connections"`
	MaxPlaybooks      int `json:"max_playbooks"`
	MaxRunsPerMonth   int `json:"max_runs_per_month"`
	MaxAgents         int `json:"max_agents"`
	AuditRetentionDays int `json:"audit_retention_days"`
}

// PlanFeatures represents the parsed features from a plan
type PlanFeatures struct {
	GovernanceEnabled  bool   `json:"governance_enabled"`
	GovernanceLevel    string `json:"governance_level,omitempty"`
	ComplianceEnabled  bool   `json:"compliance_enabled"`
	RationalizationEnabled bool `json:"rationalization_enabled"`
	AIEnabled          bool   `json:"ai_enabled"`
	AILevel            string `json:"ai_level,omitempty"`
	AdvancedOpsEnabled bool   `json:"advanced_ops_enabled"`
	OpsLevel           string `json:"ops_level,omitempty"`
}

// TenantPlan is the joined result of a tenant's plan info
type TenantPlan struct {
	TenantID             string          `json:"tenant_id"`
	PlanID               sql.NullString  `json:"-"`
	PlanIDStr            string          `json:"plan_id,omitempty"`
	PlanStatus           string          `json:"plan_status"`
	TrialEndsAt          sql.NullTime    `json:"-"`
	TrialEnds            *time.Time      `json:"trial_ends_at,omitempty"`
	BillingEmail         sql.NullString  `json:"-"`
	BillingEmailStr      string          `json:"billing_email,omitempty"`
	StripeCustomerID     sql.NullString  `json:"-"`
	StripeCustomerStr    string          `json:"stripe_customer_id,omitempty"`
	StripeSubscriptionID sql.NullString  `json:"-"`
	StripeSubscriptionStr string         `json:"stripe_subscription_id,omitempty"`
	Plan                 *Plan           `json:"plan,omitempty"`
}

func (tp *TenantPlan) Normalize() {
	if tp.PlanID.Valid {
		tp.PlanIDStr = tp.PlanID.String
	}
	if tp.TrialEndsAt.Valid {
		tp.TrialEnds = &tp.TrialEndsAt.Time
	}
	if tp.BillingEmail.Valid {
		tp.BillingEmailStr = tp.BillingEmail.String
	}
	if tp.StripeCustomerID.Valid {
		tp.StripeCustomerStr = tp.StripeCustomerID.String
	}
	if tp.StripeSubscriptionID.Valid {
		tp.StripeSubscriptionStr = tp.StripeSubscriptionID.String
	}
}

// UsageCounters represents a tenant's usage for a billing period
type UsageCounters struct {
	ID                  string     `json:"id"`
	TenantID            string     `json:"tenant_id"`
	PeriodStart         time.Time  `json:"period_start"`
	PeriodEnd           time.Time  `json:"period_end"`
	IntegrationRuns     int        `json:"integration_runs"`
	ActiveUsers         int        `json:"active_users"`
	DataBytesProcessed  int64      `json:"data_bytes_processed"`
	CreatedAt           time.Time  `json:"created_at"`
	ResetAt             sql.NullTime `json:"-"`
	Reset               *time.Time `json:"reset_at,omitempty"`
}

func (uc *UsageCounters) Normalize() {
	if uc.ResetAt.Valid {
		uc.Reset = &uc.ResetAt.Time
	}
}

// PlanRepository handles plan database operations
type PlanRepository struct {
	db *DB
}

func (r *PlanRepository) GetByID(ctx context.Context, planID string) (*Plan, error) {
	query := `SELECT id, name, display_name, description, price_cents, billing_interval,
		stripe_price_id, is_active, limits, features, created_at
		FROM plans WHERE id = $1`

	var p Plan
	err := r.db.queryRow(ctx, query, planID).Scan(
		&p.ID, &p.Name, &p.DisplayName, &p.Description, &p.PriceCents,
		&p.BillingInterval, &p.StripePriceID, &p.IsActive, &p.Limits,
		&p.Features, &p.CreatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	p.Normalize()
	return &p, nil
}

func (r *PlanRepository) GetByName(ctx context.Context, name string) (*Plan, error) {
	query := `SELECT id, name, display_name, description, price_cents, billing_interval,
		stripe_price_id, is_active, limits, features, created_at
		FROM plans WHERE name = $1`

	var p Plan
	err := r.db.queryRow(ctx, query, name).Scan(
		&p.ID, &p.Name, &p.DisplayName, &p.Description, &p.PriceCents,
		&p.BillingInterval, &p.StripePriceID, &p.IsActive, &p.Limits,
		&p.Features, &p.CreatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}
	p.Normalize()
	return &p, nil
}

func (r *PlanRepository) ListActive(ctx context.Context) ([]Plan, error) {
	query := `SELECT id, name, display_name, description, price_cents, billing_interval,
		stripe_price_id, is_active, limits, features, created_at
		FROM plans WHERE is_active = true AND name != 'trial'
		ORDER BY price_cents ASC`

	rows, err := r.db.query(ctx, query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var plans []Plan
	for rows.Next() {
		var p Plan
		err := rows.Scan(
			&p.ID, &p.Name, &p.DisplayName, &p.Description, &p.PriceCents,
			&p.BillingInterval, &p.StripePriceID, &p.IsActive, &p.Limits,
			&p.Features, &p.CreatedAt,
		)
		if err != nil {
			return nil, err
		}
		p.Normalize()
		plans = append(plans, p)
	}
	return plans, rows.Err()
}

// GetTenantPlan returns the current plan for a tenant, joined with plan details
func (r *PlanRepository) GetTenantPlan(ctx context.Context, tenantID string) (*TenantPlan, error) {
	query := `SELECT t.id, t.plan_id, t.plan_status, t.trial_ends_at, t.billing_email,
		t.stripe_customer_id, t.stripe_subscription_id,
		p.id, p.name, p.display_name, p.description, p.price_cents, p.billing_interval,
		p.stripe_price_id, p.is_active, p.limits, p.features, p.created_at
		FROM tenants t
		LEFT JOIN plans p ON t.plan_id = p.id
		WHERE t.id = $1`

	var tp TenantPlan
	var plan Plan
	var planID sql.NullString

	err := r.db.queryRow(ctx, query, tenantID).Scan(
		&tp.TenantID, &tp.PlanID, &tp.PlanStatus, &tp.TrialEndsAt, &tp.BillingEmail,
		&tp.StripeCustomerID, &tp.StripeSubscriptionID,
		&planID, &plan.Name, &plan.DisplayName, &plan.Description, &plan.PriceCents,
		&plan.BillingInterval, &plan.StripePriceID, &plan.IsActive, &plan.Limits,
		&plan.Features, &plan.CreatedAt,
	)
	if err == sql.ErrNoRows {
		return nil, nil
	}
	if err != nil {
		return nil, err
	}

	tp.Normalize()
	if planID.Valid {
		plan.ID = planID.String
		plan.Normalize()
		tp.Plan = &plan
	}
	return &tp, nil
}

// UpdateTenantPlan updates a tenant's plan and status
func (r *PlanRepository) UpdateTenantPlan(ctx context.Context, tenantID, planID, planStatus string) error {
	query := `UPDATE tenants SET plan_id = $2, plan_status = $3, updated_at = NOW() WHERE id = $1`
	_, err := r.db.execStatement(ctx, query, tenantID, planID, planStatus)
	return err
}

// SetStripeCustomer updates the Stripe customer ID for a tenant
func (r *PlanRepository) SetStripeCustomer(ctx context.Context, tenantID, customerID string) error {
	query := `UPDATE tenants SET stripe_customer_id = $2, updated_at = NOW() WHERE id = $1`
	_, err := r.db.execStatement(ctx, query, tenantID, customerID)
	return err
}

// SetStripeSubscription updates the Stripe subscription ID for a tenant
func (r *PlanRepository) SetStripeSubscription(ctx context.Context, tenantID, subscriptionID string) error {
	query := `UPDATE tenants SET stripe_subscription_id = $2, updated_at = NOW() WHERE id = $1`
	_, err := r.db.execStatement(ctx, query, tenantID, subscriptionID)
	return err
}

// SetPlanStatus updates only the plan_status for a tenant
func (r *PlanRepository) SetPlanStatus(ctx context.Context, tenantID, status string) error {
	query := `UPDATE tenants SET plan_status = $2, updated_at = NOW() WHERE id = $1`
	_, err := r.db.execStatement(ctx, query, tenantID, status)
	return err
}

// GetExpiredTrials returns tenants whose trial has expired
func (r *PlanRepository) GetExpiredTrials(ctx context.Context) ([]string, error) {
	query := `SELECT id FROM tenants WHERE plan_status = 'trial' AND trial_ends_at < NOW()`

	rows, err := r.db.query(ctx, query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var ids []string
	for rows.Next() {
		var id string
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		ids = append(ids, id)
	}
	return ids, rows.Err()
}

// UsageRepository handles usage counter operations
type UsageRepository struct {
	db *DB
}

// GetOrCreateCurrentPeriod returns the current billing period usage, creating one if needed
func (r *UsageRepository) GetOrCreateCurrentPeriod(ctx context.Context, tenantID string) (*UsageCounters, error) {
	// Try to get current period
	query := `SELECT id, tenant_id, period_start, period_end, integration_runs,
		active_users, data_bytes_processed, created_at, reset_at
		FROM usage_counters
		WHERE tenant_id = $1 AND period_start <= NOW() AND period_end > NOW()
		ORDER BY period_start DESC LIMIT 1`

	var uc UsageCounters
	err := r.db.queryRow(ctx, query, tenantID).Scan(
		&uc.ID, &uc.TenantID, &uc.PeriodStart, &uc.PeriodEnd,
		&uc.IntegrationRuns, &uc.ActiveUsers, &uc.DataBytesProcessed,
		&uc.CreatedAt, &uc.ResetAt,
	)
	if err == nil {
		uc.Normalize()
		return &uc, nil
	}
	if err != sql.ErrNoRows {
		return nil, err
	}

	// Create new period (first of current month to first of next month)
	insertQuery := `INSERT INTO usage_counters (tenant_id, period_start, period_end)
		VALUES ($1, date_trunc('month', NOW()), date_trunc('month', NOW()) + INTERVAL '1 month')
		RETURNING id, tenant_id, period_start, period_end, integration_runs,
		active_users, data_bytes_processed, created_at, reset_at`

	err = r.db.queryRow(ctx, insertQuery, tenantID).Scan(
		&uc.ID, &uc.TenantID, &uc.PeriodStart, &uc.PeriodEnd,
		&uc.IntegrationRuns, &uc.ActiveUsers, &uc.DataBytesProcessed,
		&uc.CreatedAt, &uc.ResetAt,
	)
	if err != nil {
		return nil, err
	}
	uc.Normalize()
	return &uc, nil
}

// IncrementRunCount increments the integration_runs counter for the current period
func (r *UsageRepository) IncrementRunCount(ctx context.Context, tenantID string) error {
	query := `UPDATE usage_counters SET integration_runs = integration_runs + 1
		WHERE tenant_id = $1 AND period_start <= NOW() AND period_end > NOW()`
	_, err := r.db.execStatement(ctx, query, tenantID)
	return err
}

// CountTenantResources returns counts of various resources for a tenant
func (r *UsageRepository) CountTenantResources(ctx context.Context, tenantID string) (map[string]int, error) {
	counts := make(map[string]int)

	queries := map[string]string{
		"integrations": `SELECT COUNT(*) FROM integrations WHERE tenant_id = $1`,
		"connections":  `SELECT COUNT(*) FROM connections WHERE tenant_id = $1`,
		"agents":       `SELECT COUNT(*) FROM agents WHERE tenant_id = $1 AND status != 'disconnected'`,
		"users":        `SELECT COUNT(*) FROM users WHERE tenant_id = $1 AND status = 'active'`,
	}

	for resource, query := range queries {
		var count int
		if err := r.db.queryRow(ctx, query, tenantID).Scan(&count); err != nil {
			return nil, err
		}
		counts[resource] = count
	}

	// Playbooks are in the integration service DB, count via the same postgres
	var playbookCount int
	err := r.db.queryRow(ctx,
		`SELECT COUNT(*) FROM playbooks WHERE tenant_id = $1`, tenantID).Scan(&playbookCount)
	if err != nil {
		// Table may not exist in api-gateway's view, default to 0
		playbookCount = 0
	}
	counts["playbooks"] = playbookCount

	// Monthly runs from usage_counters
	var runCount int
	err = r.db.queryRow(ctx,
		`SELECT COALESCE(integration_runs, 0) FROM usage_counters
		WHERE tenant_id = $1 AND period_start <= NOW() AND period_end > NOW()
		LIMIT 1`, tenantID).Scan(&runCount)
	if err != nil {
		runCount = 0
	}
	counts["runs_this_month"] = runCount

	return counts, nil
}
