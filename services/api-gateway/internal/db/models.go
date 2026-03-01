package db

import (
	"database/sql"
	"encoding/json"
	"time"
)

// Agent represents an agent in the database
type Agent struct {
	ID              string          `json:"id"`
	TenantID        string          `json:"tenant_id"`
	Name            string          `json:"name"`
	Description     sql.NullString  `json:"-"`
	DescriptionStr  string          `json:"description,omitempty"`
	Status          string          `json:"status"`
	Version         sql.NullString  `json:"-"`
	VersionStr      string          `json:"version,omitempty"`
	Capabilities    json.RawMessage `json:"capabilities"`
	Labels          json.RawMessage `json:"labels"`
	LastHeartbeatAt sql.NullTime    `json:"-"`
	LastHeartbeat   *time.Time      `json:"last_heartbeat_at,omitempty"`
	LastConnectedAt sql.NullTime    `json:"-"`
	LastConnected   *time.Time      `json:"last_connected_at,omitempty"`
	Config          json.RawMessage `json:"config"`
	CreatedAt       time.Time       `json:"created_at"`
	UpdatedAt       time.Time       `json:"updated_at"`
}

// MarshalJSON customizes JSON serialization
func (a *Agent) Normalize() {
	if a.Description.Valid {
		a.DescriptionStr = a.Description.String
	}
	if a.Version.Valid {
		a.VersionStr = a.Version.String
	}
	if a.LastHeartbeatAt.Valid {
		a.LastHeartbeat = &a.LastHeartbeatAt.Time
	}
	if a.LastConnectedAt.Valid {
		a.LastConnected = &a.LastConnectedAt.Time
	}
}

// Connection represents a connection in the database
type Connection struct {
	ID                   string          `json:"id"`
	TenantID             string          `json:"tenant_id"`
	Name                 string          `json:"name"`
	Description          sql.NullString  `json:"-"`
	DescriptionStr       string          `json:"description,omitempty"`
	ConnectionType       string          `json:"connection_type"`
	Config               json.RawMessage `json:"config"`
	CredentialsEncrypted []byte          `json:"-"` // Never expose in JSON
	AgentID              sql.NullString  `json:"-"`
	AgentIDStr           string          `json:"agent_id,omitempty"`
	Status               string          `json:"status"`
	LastTestedAt         sql.NullTime    `json:"-"`
	LastTested           *time.Time      `json:"last_tested_at,omitempty"`
	LastTestStatus       sql.NullString  `json:"-"`
	LastTestStatusStr    string          `json:"last_test_status,omitempty"`
	CreatedAt            time.Time       `json:"created_at"`
	UpdatedAt            time.Time       `json:"updated_at"`
}

func (c *Connection) Normalize() {
	if c.Description.Valid {
		c.DescriptionStr = c.Description.String
	}
	if c.AgentID.Valid {
		c.AgentIDStr = c.AgentID.String
	}
	if c.LastTestedAt.Valid {
		c.LastTested = &c.LastTestedAt.Time
	}
	if c.LastTestStatus.Valid {
		c.LastTestStatusStr = c.LastTestStatus.String
	}
}

// Integration represents an integration in the database
type Integration struct {
	ID             string          `json:"id"`
	TenantID       string          `json:"tenant_id"`
	Name           string          `json:"name"`
	Description    sql.NullString  `json:"-"`
	DescriptionStr string          `json:"description,omitempty"`
	Definition     json.RawMessage `json:"definition"`
	Version        int             `json:"version"`
	Status         string          `json:"status"`
	Schedule       json.RawMessage `json:"schedule,omitempty"`
	Config         json.RawMessage `json:"config"`
	CreatedBy      sql.NullString  `json:"-"`
	CreatedByStr   string          `json:"created_by,omitempty"`
	UpdatedBy      sql.NullString  `json:"-"`
	UpdatedByStr   string          `json:"updated_by,omitempty"`
	CreatedAt      time.Time       `json:"created_at"`
	UpdatedAt      time.Time       `json:"updated_at"`
}

func (i *Integration) Normalize() {
	if i.Description.Valid {
		i.DescriptionStr = i.Description.String
	}
	if i.CreatedBy.Valid {
		i.CreatedByStr = i.CreatedBy.String
	}
	if i.UpdatedBy.Valid {
		i.UpdatedByStr = i.UpdatedBy.String
	}
}

// IntegrationRun represents an integration run in the database
type IntegrationRun struct {
	ID                 string          `json:"id"`
	TenantID           string          `json:"tenant_id"`
	IntegrationID      string          `json:"integration_id"`
	IntegrationVersion int             `json:"integration_version"`
	Status             string          `json:"status"`
	TriggerType        string          `json:"trigger_type"`
	TriggeredBy        sql.NullString  `json:"-"`
	TriggeredByStr     string          `json:"triggered_by,omitempty"`
	AgentID            sql.NullString  `json:"-"`
	AgentIDStr         string          `json:"agent_id,omitempty"`
	StartedAt          sql.NullTime    `json:"-"`
	Started            *time.Time      `json:"started_at,omitempty"`
	CompletedAt        sql.NullTime    `json:"-"`
	Completed          *time.Time      `json:"completed_at,omitempty"`
	ErrorMessage       sql.NullString  `json:"-"`
	ErrorMessageStr    string          `json:"error_message,omitempty"`
	ErrorDetails       json.RawMessage `json:"error_details,omitempty"`
	Metrics            json.RawMessage `json:"metrics"`
	CreatedAt          time.Time       `json:"created_at"`
}

func (r *IntegrationRun) Normalize() {
	if r.TriggeredBy.Valid {
		r.TriggeredByStr = r.TriggeredBy.String
	}
	if r.AgentID.Valid {
		r.AgentIDStr = r.AgentID.String
	}
	if r.StartedAt.Valid {
		r.Started = &r.StartedAt.Time
	}
	if r.CompletedAt.Valid {
		r.Completed = &r.CompletedAt.Time
	}
	if r.ErrorMessage.Valid {
		r.ErrorMessageStr = r.ErrorMessage.String
	}
}

// User represents a user in the database
type User struct {
	ID                   string         `json:"id"`
	TenantID             string         `json:"tenant_id"`
	Email                string         `json:"email"`
	Name                 sql.NullString `json:"-"`
	NameStr              string         `json:"name,omitempty"`
	PasswordHash         sql.NullString `json:"-"` // Never expose
	Roles                []string       `json:"roles"`
	Status               string         `json:"status"`
	AuthSource           string         `json:"auth_source"`
	IDPSubject           sql.NullString `json:"-"`
	IDPSubjectStr        string         `json:"idp_subject,omitempty"`
	SessionVersion       int            `json:"session_version"`
	BreakglassEligible   bool           `json:"breakglass_eligible"`
	LastLoginAt          sql.NullTime   `json:"-"`
	LastLogin            *time.Time     `json:"last_login_at,omitempty"`
	LastBreakglassLogin  sql.NullTime   `json:"-"`
	LastBreakglassLoginT *time.Time     `json:"last_breakglass_login_at,omitempty"`
	CreatedAt            time.Time      `json:"created_at"`
	UpdatedAt            time.Time      `json:"updated_at"`
}

func (u *User) Normalize() {
	if u.Name.Valid {
		u.NameStr = u.Name.String
	}
	if u.IDPSubject.Valid {
		u.IDPSubjectStr = u.IDPSubject.String
	}
	if u.LastLoginAt.Valid {
		u.LastLogin = &u.LastLoginAt.Time
	}
	if u.LastBreakglassLogin.Valid {
		u.LastBreakglassLoginT = &u.LastBreakglassLogin.Time
	}
}

// RefreshToken stores metadata for refresh-token rotation and revocation.
type RefreshToken struct {
	ID             string       `json:"id"`
	TenantID       string       `json:"tenant_id"`
	UserID         string       `json:"user_id"`
	TokenHash      string       `json:"-"`
	ReplacedByHash sql.NullString `json:"-"`
	ExpiresAt      time.Time    `json:"expires_at"`
	RevokedAt      sql.NullTime `json:"-"`
	UsedAt         sql.NullTime `json:"-"`
	CreatedAt      time.Time    `json:"created_at"`
}

// Task represents a task in the database
type Task struct {
	ID               string          `json:"id"`
	TenantID         string          `json:"tenant_id"`
	IntegrationRunID sql.NullString  `json:"-"`
	IntegrationRun   string          `json:"integration_run_id,omitempty"`
	AgentID          sql.NullString  `json:"-"`
	Agent            string          `json:"agent_id,omitempty"`
	TaskType         string          `json:"task_type"`
	Priority         int             `json:"priority"`
	Config           json.RawMessage `json:"config"`
	Status           string          `json:"status"`
	RetryCount       int             `json:"retry_count"`
	MaxRetries       int             `json:"max_retries"`
	TimeoutSeconds   int             `json:"timeout_seconds"`
	ScheduledAt      sql.NullTime    `json:"-"`
	Scheduled        *time.Time      `json:"scheduled_at,omitempty"`
	StartedAt        sql.NullTime    `json:"-"`
	Started          *time.Time      `json:"started_at,omitempty"`
	CompletedAt      sql.NullTime    `json:"-"`
	Completed        *time.Time      `json:"completed_at,omitempty"`
	Result           json.RawMessage `json:"result,omitempty"`
	ErrorMessage     sql.NullString  `json:"-"`
	Error            string          `json:"error_message,omitempty"`
	ErrorCode        sql.NullString  `json:"-"`
	ErrCode          string          `json:"error_code,omitempty"`
	CreatedAt        time.Time       `json:"created_at"`
}

func (t *Task) Normalize() {
	if t.IntegrationRunID.Valid {
		t.IntegrationRun = t.IntegrationRunID.String
	}
	if t.AgentID.Valid {
		t.Agent = t.AgentID.String
	}
	if t.ScheduledAt.Valid {
		t.Scheduled = &t.ScheduledAt.Time
	}
	if t.StartedAt.Valid {
		t.Started = &t.StartedAt.Time
	}
	if t.CompletedAt.Valid {
		t.Completed = &t.CompletedAt.Time
	}
	if t.ErrorMessage.Valid {
		t.Error = t.ErrorMessage.String
	}
	if t.ErrorCode.Valid {
		t.ErrCode = t.ErrorCode.String
	}
}

// ListResult holds paginated query results
type ListResult[T any] struct {
	Items      []T   `json:"items"`
	Total      int64 `json:"total"`
	Page       int   `json:"page"`
	PageSize   int   `json:"page_size"`
	TotalPages int   `json:"total_pages"`
}

// ListOptions holds pagination and filtering options
type ListOptions struct {
	Page     int
	PageSize int
	OrderBy  string
	Order    string // "asc" or "desc"
	Filters  map[string]interface{}
}

func (o *ListOptions) Offset() int {
	if o.Page <= 0 {
		o.Page = 1
	}
	return (o.Page - 1) * o.Limit()
}

func (o *ListOptions) Limit() int {
	if o.PageSize <= 0 {
		return 20
	}
	if o.PageSize > 100 {
		return 100
	}
	return o.PageSize
}
