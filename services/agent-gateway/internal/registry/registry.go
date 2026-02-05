package registry

import (
	"sync"
	"time"

	"go.uber.org/zap"
)

// Agent represents a connected agent
type Agent struct {
	ID            string
	TenantID      string
	Name          string
	Version       string
	Capabilities  AgentCapabilities
	Labels        map[string]string
	Status        AgentStatus
	ConnectedAt   time.Time
	LastHeartbeat time.Time
	RunningTasks  []string
}

// AgentCapabilities describes what the agent can do
type AgentCapabilities struct {
	SupportedAdapters  []string
	MaxConcurrentTasks int
	SupportsStreaming  bool
}

// AgentStatus represents the agent's current state
type AgentStatus string

const (
	AgentStatusConnected    AgentStatus = "connected"
	AgentStatusDisconnected AgentStatus = "disconnected"
	AgentStatusDegraded     AgentStatus = "degraded"
)

// Registry manages connected agents
type Registry struct {
	logger *zap.Logger
	mu     sync.RWMutex
	agents map[string]*Agent               // agentID -> Agent
	byTenant map[string]map[string]*Agent  // tenantID -> agentID -> Agent
}

// New creates a new agent registry
func New(logger *zap.Logger) *Registry {
	return &Registry{
		logger:   logger.Named("registry"),
		agents:   make(map[string]*Agent),
		byTenant: make(map[string]map[string]*Agent),
	}
}

// Register adds or updates an agent in the registry
func (r *Registry) Register(agent *Agent) {
	r.mu.Lock()
	defer r.mu.Unlock()

	agent.Status = AgentStatusConnected
	agent.ConnectedAt = time.Now()
	agent.LastHeartbeat = time.Now()

	r.agents[agent.ID] = agent

	// Add to tenant index
	if r.byTenant[agent.TenantID] == nil {
		r.byTenant[agent.TenantID] = make(map[string]*Agent)
	}
	r.byTenant[agent.TenantID][agent.ID] = agent

	r.logger.Info("Agent registered",
		zap.String("agent_id", agent.ID),
		zap.String("tenant_id", agent.TenantID),
		zap.String("name", agent.Name),
	)
}

// Unregister removes an agent from the registry
func (r *Registry) Unregister(agentID string) {
	r.mu.Lock()
	defer r.mu.Unlock()

	agent, ok := r.agents[agentID]
	if !ok {
		return
	}

	delete(r.agents, agentID)

	// Remove from tenant index
	if tenantAgents, ok := r.byTenant[agent.TenantID]; ok {
		delete(tenantAgents, agentID)
		if len(tenantAgents) == 0 {
			delete(r.byTenant, agent.TenantID)
		}
	}

	r.logger.Info("Agent unregistered",
		zap.String("agent_id", agentID),
		zap.String("tenant_id", agent.TenantID),
	)
}

// Get retrieves an agent by ID
func (r *Registry) Get(agentID string) (*Agent, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()

	agent, ok := r.agents[agentID]
	return agent, ok
}

// GetByTenant returns all agents for a tenant
func (r *Registry) GetByTenant(tenantID string) []*Agent {
	r.mu.RLock()
	defer r.mu.RUnlock()

	tenantAgents, ok := r.byTenant[tenantID]
	if !ok {
		return nil
	}

	agents := make([]*Agent, 0, len(tenantAgents))
	for _, agent := range tenantAgents {
		agents = append(agents, agent)
	}
	return agents
}

// UpdateHeartbeat updates the last heartbeat time for an agent
func (r *Registry) UpdateHeartbeat(agentID string, runningTasks []string) {
	r.mu.Lock()
	defer r.mu.Unlock()

	agent, ok := r.agents[agentID]
	if !ok {
		return
	}

	agent.LastHeartbeat = time.Now()
	agent.RunningTasks = runningTasks
}

// FindAvailableAgent finds an agent that can handle a task
func (r *Registry) FindAvailableAgent(tenantID string, requiredAdapter string) *Agent {
	r.mu.RLock()
	defer r.mu.RUnlock()

	tenantAgents, ok := r.byTenant[tenantID]
	if !ok {
		return nil
	}

	for _, agent := range tenantAgents {
		// Check if agent is healthy
		if agent.Status != AgentStatusConnected {
			continue
		}

		// Check if agent has capacity
		if len(agent.RunningTasks) >= agent.Capabilities.MaxConcurrentTasks {
			continue
		}

		// Check if agent supports the required adapter
		if requiredAdapter != "" {
			supported := false
			for _, adapter := range agent.Capabilities.SupportedAdapters {
				if adapter == requiredAdapter {
					supported = true
					break
				}
			}
			if !supported {
				continue
			}
		}

		return agent
	}

	return nil
}

// Stats returns registry statistics
func (r *Registry) Stats() RegistryStats {
	r.mu.RLock()
	defer r.mu.RUnlock()

	stats := RegistryStats{
		TotalAgents:   len(r.agents),
		TotalTenants:  len(r.byTenant),
		AgentsByTenant: make(map[string]int),
	}

	for tenantID, agents := range r.byTenant {
		stats.AgentsByTenant[tenantID] = len(agents)
	}

	return stats
}

// RegistryStats contains registry statistics
type RegistryStats struct {
	TotalAgents    int
	TotalTenants   int
	AgentsByTenant map[string]int
}
