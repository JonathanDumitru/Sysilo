package orchestration

import (
	"encoding/json"
	"fmt"
	"sort"
	"sync"
	"time"

	"go.uber.org/zap"
)

// AgentType represents the category of an enterprise agent.
type AgentType string

const (
	DataAgent     AgentType = "DataAgent"
	SalesAgent    AgentType = "SalesAgent"
	SupportAgent  AgentType = "SupportAgent"
	SecurityAgent AgentType = "SecurityAgent"
	CustomAgent   AgentType = "CustomAgent"
)

// EnterpriseAgent extends the concept beyond just Sysilo agents, representing
// a governed, metrics-tracked agent within a tenant's portfolio.
type EnterpriseAgent struct {
	ID                   string            `json:"id"`
	TenantID             string            `json:"tenant_id"`
	Name                 string            `json:"name"`
	Type                 AgentType         `json:"type"`
	Owner                string            `json:"owner"`
	Description          string            `json:"description"`
	DeclaredCapabilities []string          `json:"declared_capabilities"`
	Permissions          []Permission      `json:"permissions"`
	GovernancePolicy     string            `json:"governance_policy"`
	Status               string            `json:"status"`
	RegisteredAt         time.Time         `json:"registered_at"`
	LastActiveAt         time.Time         `json:"last_active_at"`
	ResourceQuota        ResourceQuota     `json:"resource_quota"`
	MetricsWindow        AgentMetrics      `json:"metrics_window"`
}

// Permission describes what an agent is allowed to do on a given resource.
type Permission struct {
	Resource   string            `json:"resource"`
	Actions    []string          `json:"actions"`
	Conditions map[string]string `json:"conditions,omitempty"`
}

// ResourceQuota defines the resource limits for an agent.
type ResourceQuota struct {
	MaxConcurrentOps     int `json:"max_concurrent_ops"`
	MaxMemoryMB          int `json:"max_memory_mb"`
	MaxCPUMillis         int `json:"max_cpu_millis"`
	MaxAPICallsPerMinute int `json:"max_api_calls_per_minute"`
}

// AgentMetrics holds rolling-window performance metrics for an agent.
type AgentMetrics struct {
	TotalInvocations int64   `json:"total_invocations"`
	SuccessCount     int64   `json:"success_count"`
	FailureCount     int64   `json:"failure_count"`
	AvgLatencyMs     float64 `json:"avg_latency_ms"`
	P99LatencyMs     float64 `json:"p99_latency_ms"`
	LastHourCalls    int64   `json:"last_hour_calls"`
	DecisionsMade    int64   `json:"decisions_made"`
}

// A2AMessage represents an agent-to-agent communication message.
type A2AMessage struct {
	ID            string          `json:"id"`
	FromAgentID   string          `json:"from_agent_id"`
	ToAgentID     string          `json:"to_agent_id"`
	MessageType   string          `json:"message_type"` // request, response, broadcast
	Payload       json.RawMessage `json:"payload"`
	CorrelationID string          `json:"correlation_id"`
	Priority      int             `json:"priority"`
	CreatedAt     time.Time       `json:"created_at"`
	ExpiresAt     time.Time       `json:"expires_at"`
	Status        string          `json:"status"` // pending, delivered, processed, failed
}

// EscalationRule defines when an action should be escalated to a human.
type EscalationRule struct {
	AgentType       string   `json:"agent_type"`
	ActionType      string   `json:"action_type"`
	RiskLevel       string   `json:"risk_level"` // low, medium, high, critical
	RequiresApproval bool    `json:"requires_approval"`
	ApproverRoles   []string `json:"approver_roles"`
	TimeoutMinutes  int      `json:"timeout_minutes"`
}

// PortfolioView provides an enterprise-level overview of all agents for a tenant.
type PortfolioView struct {
	TenantID            string            `json:"tenant_id"`
	TotalAgents         int               `json:"total_agents"`
	ActiveAgents        int               `json:"active_agents"`
	AgentsByType        map[string]int    `json:"agents_by_type"`
	TotalInvocations    int64             `json:"total_invocations"`
	OverallSuccessRate  float64           `json:"overall_success_rate"`
	TopPerformers       []AgentSummary    `json:"top_performers"`
	Underperformers     []AgentSummary    `json:"underperformers"`
	ResourceUtilization float64           `json:"resource_utilization"`
}

// AgentSummary provides a compact performance summary for an agent.
type AgentSummary struct {
	AgentID         string  `json:"agent_id"`
	Name            string  `json:"name"`
	Type            string  `json:"type"`
	SuccessRate     float64 `json:"success_rate"`
	AvgLatencyMs    float64 `json:"avg_latency_ms"`
	InvocationCount int64   `json:"invocation_count"`
}

// OrchestrationHub is the central coordinator for enterprise agent management,
// built as an orchestration layer on top of the base agent registry.
type OrchestrationHub struct {
	logger          *zap.Logger
	mu              sync.RWMutex
	agents          map[string]*EnterpriseAgent
	byTenant        map[string]map[string]*EnterpriseAgent
	messageQueue    chan *A2AMessage
	escalationRules map[string]*EscalationRule
}

// New creates a new OrchestrationHub.
func New(logger *zap.Logger) *OrchestrationHub {
	return &OrchestrationHub{
		logger:          logger.Named("orchestration-hub"),
		agents:          make(map[string]*EnterpriseAgent),
		byTenant:        make(map[string]map[string]*EnterpriseAgent),
		messageQueue:    make(chan *A2AMessage, 1000),
		escalationRules: make(map[string]*EscalationRule),
	}
}

// RegisterAgent adds an enterprise agent to the hub.
func (h *OrchestrationHub) RegisterAgent(agent *EnterpriseAgent) {
	h.mu.Lock()
	defer h.mu.Unlock()

	agent.RegisteredAt = time.Now()
	agent.LastActiveAt = time.Now()
	if agent.Status == "" {
		agent.Status = "active"
	}

	h.agents[agent.ID] = agent

	if h.byTenant[agent.TenantID] == nil {
		h.byTenant[agent.TenantID] = make(map[string]*EnterpriseAgent)
	}
	h.byTenant[agent.TenantID][agent.ID] = agent

	h.logger.Info("Enterprise agent registered",
		zap.String("agent_id", agent.ID),
		zap.String("tenant_id", agent.TenantID),
		zap.String("name", agent.Name),
		zap.String("type", string(agent.Type)),
	)
}

// UnregisterAgent removes an enterprise agent from the hub.
func (h *OrchestrationHub) UnregisterAgent(agentID string) {
	h.mu.Lock()
	defer h.mu.Unlock()

	agent, ok := h.agents[agentID]
	if !ok {
		return
	}

	delete(h.agents, agentID)

	if tenantAgents, ok := h.byTenant[agent.TenantID]; ok {
		delete(tenantAgents, agentID)
		if len(tenantAgents) == 0 {
			delete(h.byTenant, agent.TenantID)
		}
	}

	h.logger.Info("Enterprise agent unregistered",
		zap.String("agent_id", agentID),
		zap.String("tenant_id", agent.TenantID),
	)
}

// GetAgent retrieves an enterprise agent by ID.
func (h *OrchestrationHub) GetAgent(agentID string) (*EnterpriseAgent, bool) {
	h.mu.RLock()
	defer h.mu.RUnlock()

	agent, ok := h.agents[agentID]
	return agent, ok
}

// ListAgentsByTenant returns all enterprise agents belonging to a tenant.
func (h *OrchestrationHub) ListAgentsByTenant(tenantID string) []*EnterpriseAgent {
	h.mu.RLock()
	defer h.mu.RUnlock()

	tenantAgents, ok := h.byTenant[tenantID]
	if !ok {
		return nil
	}

	agents := make([]*EnterpriseAgent, 0, len(tenantAgents))
	for _, agent := range tenantAgents {
		agents = append(agents, agent)
	}
	return agents
}

// ListAgentsByType returns all agents of a specific type within a tenant.
func (h *OrchestrationHub) ListAgentsByType(tenantID string, agentType string) []*EnterpriseAgent {
	h.mu.RLock()
	defer h.mu.RUnlock()

	tenantAgents, ok := h.byTenant[tenantID]
	if !ok {
		return nil
	}

	var result []*EnterpriseAgent
	for _, agent := range tenantAgents {
		if string(agent.Type) == agentType {
			result = append(result, agent)
		}
	}
	return result
}

// SendMessage enqueues an agent-to-agent message for delivery.
func (h *OrchestrationHub) SendMessage(msg *A2AMessage) error {
	if msg.ID == "" {
		return fmt.Errorf("message ID is required")
	}
	if msg.FromAgentID == "" {
		return fmt.Errorf("from_agent_id is required")
	}
	if msg.ToAgentID == "" && msg.MessageType != "broadcast" {
		return fmt.Errorf("to_agent_id is required for non-broadcast messages")
	}

	h.mu.RLock()
	_, fromExists := h.agents[msg.FromAgentID]
	h.mu.RUnlock()

	if !fromExists {
		return fmt.Errorf("sender agent %s not found", msg.FromAgentID)
	}

	if msg.CreatedAt.IsZero() {
		msg.CreatedAt = time.Now()
	}
	if msg.Status == "" {
		msg.Status = "pending"
	}

	select {
	case h.messageQueue <- msg:
		h.logger.Debug("A2A message enqueued",
			zap.String("message_id", msg.ID),
			zap.String("from", msg.FromAgentID),
			zap.String("to", msg.ToAgentID),
			zap.String("type", msg.MessageType),
		)
		return nil
	default:
		return fmt.Errorf("message queue is full")
	}
}

// DiscoverAgents finds agents within a tenant that declare a required capability.
func (h *OrchestrationHub) DiscoverAgents(tenantID string, requiredCapability string) []*EnterpriseAgent {
	h.mu.RLock()
	defer h.mu.RUnlock()

	tenantAgents, ok := h.byTenant[tenantID]
	if !ok {
		return nil
	}

	var result []*EnterpriseAgent
	for _, agent := range tenantAgents {
		if agent.Status != "active" {
			continue
		}
		for _, cap := range agent.DeclaredCapabilities {
			if cap == requiredCapability {
				result = append(result, agent)
				break
			}
		}
	}
	return result
}

// CheckPermission verifies whether an agent is permitted to perform an action
// on a resource. Returns true if at least one matching permission is found.
func (h *OrchestrationHub) CheckPermission(agentID string, resource string, action string) bool {
	h.mu.RLock()
	defer h.mu.RUnlock()

	agent, ok := h.agents[agentID]
	if !ok {
		return false
	}

	for _, perm := range agent.Permissions {
		if perm.Resource != resource {
			continue
		}
		for _, a := range perm.Actions {
			if a == action {
				return true
			}
		}
	}
	return false
}

// RecordMetrics updates an agent's rolling performance metrics with a new
// invocation result.
func (h *OrchestrationHub) RecordMetrics(agentID string, latencyMs float64, success bool) {
	h.mu.Lock()
	defer h.mu.Unlock()

	agent, ok := h.agents[agentID]
	if !ok {
		return
	}

	m := &agent.MetricsWindow
	m.TotalInvocations++
	m.LastHourCalls++

	if success {
		m.SuccessCount++
	} else {
		m.FailureCount++
	}

	// Update rolling average latency
	if m.TotalInvocations == 1 {
		m.AvgLatencyMs = latencyMs
	} else {
		m.AvgLatencyMs = m.AvgLatencyMs + (latencyMs-m.AvgLatencyMs)/float64(m.TotalInvocations)
	}

	// Update P99 approximation (track the max seen as a simple upper-bound)
	if latencyMs > m.P99LatencyMs {
		m.P99LatencyMs = latencyMs
	}

	agent.LastActiveAt = time.Now()
}

// GetPortfolioView builds an enterprise-level overview of all agents for a
// given tenant, including performance rankings.
func (h *OrchestrationHub) GetPortfolioView(tenantID string) *PortfolioView {
	h.mu.RLock()
	defer h.mu.RUnlock()

	tenantAgents, ok := h.byTenant[tenantID]
	if !ok {
		return &PortfolioView{
			TenantID:     tenantID,
			AgentsByType: make(map[string]int),
		}
	}

	view := &PortfolioView{
		TenantID:     tenantID,
		TotalAgents:  len(tenantAgents),
		AgentsByType: make(map[string]int),
	}

	var totalSuccess, totalInvocations int64
	summaries := make([]AgentSummary, 0, len(tenantAgents))

	for _, agent := range tenantAgents {
		view.AgentsByType[string(agent.Type)]++

		if agent.Status == "active" {
			view.ActiveAgents++
		}

		m := agent.MetricsWindow
		view.TotalInvocations += m.TotalInvocations
		totalSuccess += m.SuccessCount
		totalInvocations += m.TotalInvocations

		var successRate float64
		if m.TotalInvocations > 0 {
			successRate = float64(m.SuccessCount) / float64(m.TotalInvocations)
		}

		summaries = append(summaries, AgentSummary{
			AgentID:         agent.ID,
			Name:            agent.Name,
			Type:            string(agent.Type),
			SuccessRate:     successRate,
			AvgLatencyMs:    m.AvgLatencyMs,
			InvocationCount: m.TotalInvocations,
		})
	}

	if totalInvocations > 0 {
		view.OverallSuccessRate = float64(totalSuccess) / float64(totalInvocations)
	}

	// Sort by success rate descending, then by invocation count descending
	sort.Slice(summaries, func(i, j int) bool {
		if summaries[i].SuccessRate != summaries[j].SuccessRate {
			return summaries[i].SuccessRate > summaries[j].SuccessRate
		}
		return summaries[i].InvocationCount > summaries[j].InvocationCount
	})

	// Top performers: up to 5 agents with the highest success rate
	topN := 5
	if len(summaries) < topN {
		topN = len(summaries)
	}
	view.TopPerformers = summaries[:topN]

	// Underperformers: up to 5 agents with the lowest success rate (from the
	// bottom of the sorted list), only those that have at least one invocation.
	var underperformers []AgentSummary
	for i := len(summaries) - 1; i >= 0 && len(underperformers) < 5; i-- {
		if summaries[i].InvocationCount > 0 {
			underperformers = append(underperformers, summaries[i])
		}
	}
	view.Underperformers = underperformers

	// Resource utilization: ratio of active agents to total agents
	if view.TotalAgents > 0 {
		view.ResourceUtilization = float64(view.ActiveAgents) / float64(view.TotalAgents)
	}

	return view
}

// AddEscalationRule registers an escalation rule keyed by agent type, action
// type, and risk level.
func (h *OrchestrationHub) AddEscalationRule(rule *EscalationRule) {
	h.mu.Lock()
	defer h.mu.Unlock()

	key := escalationKey(rule.AgentType, rule.ActionType, rule.RiskLevel)
	h.escalationRules[key] = rule

	h.logger.Info("Escalation rule added",
		zap.String("agent_type", rule.AgentType),
		zap.String("action_type", rule.ActionType),
		zap.String("risk_level", rule.RiskLevel),
		zap.Bool("requires_approval", rule.RequiresApproval),
	)
}

// ShouldEscalate checks whether an action matches an escalation rule that
// requires human approval.
func (h *OrchestrationHub) ShouldEscalate(agentType string, actionType string, riskLevel string) (*EscalationRule, bool) {
	h.mu.RLock()
	defer h.mu.RUnlock()

	key := escalationKey(agentType, actionType, riskLevel)
	rule, ok := h.escalationRules[key]
	if !ok {
		return nil, false
	}
	return rule, rule.RequiresApproval
}

// escalationKey builds a composite map key for escalation rule lookup.
func escalationKey(agentType, actionType, riskLevel string) string {
	return agentType + ":" + actionType + ":" + riskLevel
}
