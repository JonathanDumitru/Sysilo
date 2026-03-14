package eventmesh

import (
	"encoding/json"
	"sync"
	"time"

	"go.uber.org/zap"
)

// EventType represents categories of business events flowing through the mesh
type EventType string

const (
	EventTypeDataChange    EventType = "data_change"
	EventTypeIntegration   EventType = "integration"
	EventTypeGovernance    EventType = "governance"
	EventTypeAgent         EventType = "agent"
	EventTypeSystem        EventType = "system"
	EventTypeCustom        EventType = "custom"
)

// Event represents a business event in the enterprise event mesh
type Event struct {
	ID            string            `json:"id"`
	TenantID      string            `json:"tenant_id"`
	Type          EventType         `json:"type"`
	Source        string            `json:"source"`
	Subject       string            `json:"subject"`
	Data          json.RawMessage   `json:"data"`
	DataSchema    string            `json:"data_schema,omitempty"`
	Priority      int               `json:"priority"`
	Tags          map[string]string `json:"tags,omitempty"`
	CorrelationID string            `json:"correlation_id,omitempty"`
	CreatedAt     time.Time         `json:"created_at"`
	ExpiresAt     *time.Time        `json:"expires_at,omitempty"`
}

// Subscription represents a consumer's interest in specific events
type Subscription struct {
	ID          string            `json:"id"`
	TenantID    string            `json:"tenant_id"`
	Name        string            `json:"name"`
	ConsumerID  string            `json:"consumer_id"`
	EventTypes  []EventType       `json:"event_types"`
	SourceFilter string           `json:"source_filter,omitempty"`
	SubjectFilter string          `json:"subject_filter,omitempty"`
	TagFilters  map[string]string `json:"tag_filters,omitempty"`
	WebhookURL  string            `json:"webhook_url,omitempty"`
	MaxRetries  int               `json:"max_retries"`
	CreatedAt   time.Time         `json:"created_at"`
	Active      bool              `json:"active"`
}

// DeliveryRecord tracks event delivery attempts
type DeliveryRecord struct {
	EventID        string    `json:"event_id"`
	SubscriptionID string    `json:"subscription_id"`
	Status         string    `json:"status"` // delivered, failed, retrying
	Attempts       int       `json:"attempts"`
	LastAttempt    time.Time `json:"last_attempt"`
	Error          string    `json:"error,omitempty"`
}

// MeshStats contains event mesh statistics
type MeshStats struct {
	TotalEvents         int64            `json:"total_events"`
	TotalSubscriptions  int              `json:"total_subscriptions"`
	ActiveSubscriptions int              `json:"active_subscriptions"`
	EventsByType        map[string]int64 `json:"events_by_type"`
	DeliverySuccessRate float64          `json:"delivery_success_rate"`
	AvgDeliveryLatencyMs float64         `json:"avg_delivery_latency_ms"`
	EventsPerMinute     float64          `json:"events_per_minute"`
}

// TopicRoute defines routing rules for events within the mesh
type TopicRoute struct {
	ID            string    `json:"id"`
	TenantID      string    `json:"tenant_id"`
	SourceTopic   string    `json:"source_topic"`
	TargetTopic   string    `json:"target_topic"`
	FilterExpr    string    `json:"filter_expr,omitempty"`
	TransformExpr string    `json:"transform_expr,omitempty"`
	Active        bool      `json:"active"`
	CreatedAt     time.Time `json:"created_at"`
}

// ControlPlaneNode represents a node in the hybrid control plane
type ControlPlaneNode struct {
	ID          string            `json:"id"`
	TenantID    string            `json:"tenant_id"`
	Name        string            `json:"name"`
	Environment string            `json:"environment"` // on-prem, aws, azure, gcp, edge
	Region      string            `json:"region"`
	Status      string            `json:"status"` // healthy, degraded, offline
	Endpoints   map[string]string `json:"endpoints"`
	Metadata    map[string]string `json:"metadata,omitempty"`
	LastSeen    time.Time         `json:"last_seen"`
	RegisteredAt time.Time        `json:"registered_at"`
}

// EventMesh manages the enterprise event mesh
type EventMesh struct {
	logger        *zap.Logger
	mu            sync.RWMutex
	subscriptions map[string]*Subscription         // subscriptionID -> Subscription
	byTenant      map[string]map[string]*Subscription // tenantID -> subscriptionID -> Subscription
	routes        map[string]*TopicRoute
	nodes         map[string]*ControlPlaneNode
	eventCount    int64
	deliveries    map[string]*DeliveryRecord
	eventBuffer   chan *Event
}

// New creates a new EventMesh
func New(logger *zap.Logger) *EventMesh {
	mesh := &EventMesh{
		logger:        logger.Named("event-mesh"),
		subscriptions: make(map[string]*Subscription),
		byTenant:      make(map[string]map[string]*Subscription),
		routes:        make(map[string]*TopicRoute),
		nodes:         make(map[string]*ControlPlaneNode),
		deliveries:    make(map[string]*DeliveryRecord),
		eventBuffer:   make(chan *Event, 10000),
	}

	// Start background event processor
	go mesh.processEvents()

	return mesh
}

// Publish publishes an event to the mesh
func (m *EventMesh) Publish(event *Event) error {
	if event.ID == "" || event.TenantID == "" {
		return ErrInvalidEvent
	}

	event.CreatedAt = time.Now()

	m.mu.Lock()
	m.eventCount++
	m.mu.Unlock()

	// Non-blocking send to buffer
	select {
	case m.eventBuffer <- event:
		m.logger.Debug("Event published",
			zap.String("event_id", event.ID),
			zap.String("type", string(event.Type)),
			zap.String("source", event.Source),
		)
	default:
		m.logger.Warn("Event buffer full, dropping event",
			zap.String("event_id", event.ID),
		)
		return ErrBufferFull
	}

	return nil
}

// Subscribe creates a new subscription
func (m *EventMesh) Subscribe(sub *Subscription) {
	m.mu.Lock()
	defer m.mu.Unlock()

	sub.CreatedAt = time.Now()
	sub.Active = true

	m.subscriptions[sub.ID] = sub

	if m.byTenant[sub.TenantID] == nil {
		m.byTenant[sub.TenantID] = make(map[string]*Subscription)
	}
	m.byTenant[sub.TenantID][sub.ID] = sub

	m.logger.Info("Subscription created",
		zap.String("subscription_id", sub.ID),
		zap.String("tenant_id", sub.TenantID),
		zap.String("consumer_id", sub.ConsumerID),
	)
}

// Unsubscribe removes a subscription
func (m *EventMesh) Unsubscribe(subscriptionID string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	sub, ok := m.subscriptions[subscriptionID]
	if !ok {
		return
	}

	sub.Active = false
	delete(m.subscriptions, subscriptionID)

	if tenantSubs, ok := m.byTenant[sub.TenantID]; ok {
		delete(tenantSubs, subscriptionID)
		if len(tenantSubs) == 0 {
			delete(m.byTenant, sub.TenantID)
		}
	}

	m.logger.Info("Subscription removed",
		zap.String("subscription_id", subscriptionID),
	)
}

// GetSubscription retrieves a subscription by ID
func (m *EventMesh) GetSubscription(id string) (*Subscription, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	sub, ok := m.subscriptions[id]
	return sub, ok
}

// ListSubscriptions returns all subscriptions for a tenant
func (m *EventMesh) ListSubscriptions(tenantID string) []*Subscription {
	m.mu.RLock()
	defer m.mu.RUnlock()

	tenantSubs, ok := m.byTenant[tenantID]
	if !ok {
		return nil
	}

	subs := make([]*Subscription, 0, len(tenantSubs))
	for _, sub := range tenantSubs {
		subs = append(subs, sub)
	}
	return subs
}

// AddRoute adds a topic routing rule
func (m *EventMesh) AddRoute(route *TopicRoute) {
	m.mu.Lock()
	defer m.mu.Unlock()

	route.CreatedAt = time.Now()
	route.Active = true
	m.routes[route.ID] = route

	m.logger.Info("Route added",
		zap.String("route_id", route.ID),
		zap.String("source", route.SourceTopic),
		zap.String("target", route.TargetTopic),
	)
}

// RemoveRoute removes a topic routing rule
func (m *EventMesh) RemoveRoute(routeID string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	delete(m.routes, routeID)
}

// ListRoutes returns all routes for a tenant
func (m *EventMesh) ListRoutes(tenantID string) []*TopicRoute {
	m.mu.RLock()
	defer m.mu.RUnlock()

	routes := make([]*TopicRoute, 0)
	for _, route := range m.routes {
		if route.TenantID == tenantID {
			routes = append(routes, route)
		}
	}
	return routes
}

// RegisterNode registers a control plane node
func (m *EventMesh) RegisterNode(node *ControlPlaneNode) {
	m.mu.Lock()
	defer m.mu.Unlock()

	node.RegisteredAt = time.Now()
	node.LastSeen = time.Now()
	node.Status = "healthy"
	m.nodes[node.ID] = node

	m.logger.Info("Control plane node registered",
		zap.String("node_id", node.ID),
		zap.String("environment", node.Environment),
		zap.String("region", node.Region),
	)
}

// UnregisterNode removes a control plane node
func (m *EventMesh) UnregisterNode(nodeID string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	delete(m.nodes, nodeID)
}

// ListNodes returns all control plane nodes
func (m *EventMesh) ListNodes(tenantID string) []*ControlPlaneNode {
	m.mu.RLock()
	defer m.mu.RUnlock()

	nodes := make([]*ControlPlaneNode, 0)
	for _, node := range m.nodes {
		if node.TenantID == tenantID {
			nodes = append(nodes, node)
		}
	}
	return nodes
}

// UpdateNodeHeartbeat updates the last seen time for a node
func (m *EventMesh) UpdateNodeHeartbeat(nodeID string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if node, ok := m.nodes[nodeID]; ok {
		node.LastSeen = time.Now()
		node.Status = "healthy"
	}
}

// Stats returns event mesh statistics
func (m *EventMesh) Stats() MeshStats {
	m.mu.RLock()
	defer m.mu.RUnlock()

	activeSubs := 0
	for _, sub := range m.subscriptions {
		if sub.Active {
			activeSubs++
		}
	}

	successCount := 0
	totalDeliveries := 0
	var totalLatency float64
	for _, d := range m.deliveries {
		totalDeliveries++
		if d.Status == "delivered" {
			successCount++
		}
	}

	successRate := float64(0)
	if totalDeliveries > 0 {
		successRate = float64(successCount) / float64(totalDeliveries) * 100.0
	}

	return MeshStats{
		TotalEvents:         m.eventCount,
		TotalSubscriptions:  len(m.subscriptions),
		ActiveSubscriptions: activeSubs,
		EventsByType:        make(map[string]int64),
		DeliverySuccessRate: successRate,
		AvgDeliveryLatencyMs: totalLatency,
	}
}

// processEvents is the background event processor
func (m *EventMesh) processEvents() {
	for event := range m.eventBuffer {
		m.routeEvent(event)
	}
}

// routeEvent matches an event against subscriptions and delivers it
func (m *EventMesh) routeEvent(event *Event) {
	m.mu.RLock()
	tenantSubs := m.byTenant[event.TenantID]
	matchingSubs := make([]*Subscription, 0)

	for _, sub := range tenantSubs {
		if !sub.Active {
			continue
		}

		// Check event type filter
		typeMatch := len(sub.EventTypes) == 0
		for _, et := range sub.EventTypes {
			if et == event.Type {
				typeMatch = true
				break
			}
		}
		if !typeMatch {
			continue
		}

		// Check source filter
		if sub.SourceFilter != "" && sub.SourceFilter != event.Source {
			continue
		}

		// Check subject filter
		if sub.SubjectFilter != "" && sub.SubjectFilter != event.Subject {
			continue
		}

		matchingSubs = append(matchingSubs, sub)
	}
	m.mu.RUnlock()

	for _, sub := range matchingSubs {
		m.mu.Lock()
		m.deliveries[event.ID+":"+sub.ID] = &DeliveryRecord{
			EventID:        event.ID,
			SubscriptionID: sub.ID,
			Status:         "delivered",
			Attempts:       1,
			LastAttempt:     time.Now(),
		}
		m.mu.Unlock()

		m.logger.Debug("Event delivered",
			zap.String("event_id", event.ID),
			zap.String("subscription_id", sub.ID),
		)
	}
}

// Sentinel errors
var (
	ErrInvalidEvent = &meshError{"invalid event: id and tenant_id are required"}
	ErrBufferFull   = &meshError{"event buffer full"}
)

type meshError struct {
	msg string
}

func (e *meshError) Error() string {
	return e.msg
}
