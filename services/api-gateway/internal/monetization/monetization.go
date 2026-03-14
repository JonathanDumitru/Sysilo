package monetization

import (
	"encoding/json"
	"sync"
	"time"

	"go.uber.org/zap"
)

// PricingModel defines how an API is priced
type PricingModel string

const (
	PricingFree         PricingModel = "free"
	PricingPayPerCall   PricingModel = "pay_per_call"
	PricingSubscription PricingModel = "subscription"
	PricingTiered       PricingModel = "tiered"
	PricingFreemium     PricingModel = "freemium"
)

// APIProduct represents a monetizable API product
type APIProduct struct {
	ID              string            `json:"id"`
	TenantID        string            `json:"tenant_id"`
	Name            string            `json:"name"`
	Description     string            `json:"description"`
	Version         string            `json:"version"`
	BasePath        string            `json:"base_path"`
	Endpoints       []APIEndpoint     `json:"endpoints"`
	PricingModel    PricingModel      `json:"pricing_model"`
	PricePerCall    float64           `json:"price_per_call,omitempty"`
	MonthlyPrice    float64           `json:"monthly_price,omitempty"`
	FreeTierLimit   int64             `json:"free_tier_limit,omitempty"`
	RateLimit       int               `json:"rate_limit_per_minute"`
	Tags            []string          `json:"tags,omitempty"`
	Documentation   string            `json:"documentation_url,omitempty"`
	Status          string            `json:"status"` // draft, active, deprecated
	PublishedAt     *time.Time        `json:"published_at,omitempty"`
	CreatedAt       time.Time         `json:"created_at"`
	UpdatedAt       time.Time         `json:"updated_at"`
}

// APIEndpoint represents a single endpoint within an API product
type APIEndpoint struct {
	Method      string   `json:"method"`
	Path        string   `json:"path"`
	Description string   `json:"description"`
	Scopes      []string `json:"scopes,omitempty"`
}

// APISubscription represents a consumer's subscription to an API product
type APISubscription struct {
	ID              string    `json:"id"`
	ProductID       string    `json:"product_id"`
	ConsumerID      string    `json:"consumer_id"`
	ConsumerTenant  string    `json:"consumer_tenant"`
	APIKey          string    `json:"api_key"`
	Status          string    `json:"status"` // active, suspended, cancelled
	CurrentPeriodStart time.Time `json:"current_period_start"`
	CurrentPeriodEnd   time.Time `json:"current_period_end"`
	UsageCount      int64     `json:"usage_count"`
	UsageLimit      int64     `json:"usage_limit,omitempty"`
	CreatedAt       time.Time `json:"created_at"`
}

// UsageRecord tracks API usage for billing
type UsageRecord struct {
	ID             string    `json:"id"`
	SubscriptionID string    `json:"subscription_id"`
	ProductID      string    `json:"product_id"`
	Endpoint       string    `json:"endpoint"`
	Method         string    `json:"method"`
	StatusCode     int       `json:"status_code"`
	LatencyMs      int64     `json:"latency_ms"`
	RequestSize    int64     `json:"request_size_bytes"`
	ResponseSize   int64     `json:"response_size_bytes"`
	BilledAmount   float64   `json:"billed_amount"`
	Timestamp      time.Time `json:"timestamp"`
}

// RevenueReport provides revenue analytics for API products
type RevenueReport struct {
	TenantID        string           `json:"tenant_id"`
	PeriodStart     time.Time        `json:"period_start"`
	PeriodEnd       time.Time        `json:"period_end"`
	TotalRevenue    float64          `json:"total_revenue"`
	TotalAPICalls   int64            `json:"total_api_calls"`
	UniqueConsumers int              `json:"unique_consumers"`
	RevenueByProduct map[string]float64 `json:"revenue_by_product"`
	TopConsumers    []ConsumerUsage  `json:"top_consumers"`
}

// ConsumerUsage summarizes a consumer's API usage
type ConsumerUsage struct {
	ConsumerID   string  `json:"consumer_id"`
	TotalCalls   int64   `json:"total_calls"`
	TotalRevenue float64 `json:"total_revenue"`
	AvgLatencyMs float64 `json:"avg_latency_ms"`
}

// APIAnalytics provides real-time analytics for an API product
type APIAnalytics struct {
	ProductID       string           `json:"product_id"`
	TotalCalls      int64            `json:"total_calls"`
	ErrorRate       float64          `json:"error_rate"`
	AvgLatencyMs    float64          `json:"avg_latency_ms"`
	P99LatencyMs    float64          `json:"p99_latency_ms"`
	CallsByEndpoint map[string]int64 `json:"calls_by_endpoint"`
	ActiveConsumers int              `json:"active_consumers"`
	Revenue24h      float64          `json:"revenue_24h"`
}

// MonetizationEngine manages API products, subscriptions, and billing
type MonetizationEngine struct {
	logger        *zap.Logger
	mu            sync.RWMutex
	products      map[string]*APIProduct      // productID -> product
	byTenant      map[string]map[string]*APIProduct // tenantID -> productID -> product
	subscriptions map[string]*APISubscription // subscriptionID -> subscription
	byAPIKey      map[string]*APISubscription // apiKey -> subscription
	usageRecords  []*UsageRecord
}

// New creates a new MonetizationEngine
func New(logger *zap.Logger) *MonetizationEngine {
	return &MonetizationEngine{
		logger:        logger.Named("monetization"),
		products:      make(map[string]*APIProduct),
		byTenant:      make(map[string]map[string]*APIProduct),
		subscriptions: make(map[string]*APISubscription),
		byAPIKey:      make(map[string]*APISubscription),
		usageRecords:  make([]*UsageRecord, 0),
	}
}

// RegisterProduct adds a new API product
func (e *MonetizationEngine) RegisterProduct(product *APIProduct) {
	e.mu.Lock()
	defer e.mu.Unlock()

	product.CreatedAt = time.Now()
	product.UpdatedAt = time.Now()
	product.Status = "draft"

	e.products[product.ID] = product

	if e.byTenant[product.TenantID] == nil {
		e.byTenant[product.TenantID] = make(map[string]*APIProduct)
	}
	e.byTenant[product.TenantID][product.ID] = product

	e.logger.Info("API product registered",
		zap.String("product_id", product.ID),
		zap.String("name", product.Name),
		zap.String("pricing", string(product.PricingModel)),
	)
}

// PublishProduct makes an API product available for subscription
func (e *MonetizationEngine) PublishProduct(productID string) (*APIProduct, bool) {
	e.mu.Lock()
	defer e.mu.Unlock()

	product, ok := e.products[productID]
	if !ok {
		return nil, false
	}

	now := time.Now()
	product.Status = "active"
	product.PublishedAt = &now
	product.UpdatedAt = now

	return product, true
}

// GetProduct retrieves an API product
func (e *MonetizationEngine) GetProduct(productID string) (*APIProduct, bool) {
	e.mu.RLock()
	defer e.mu.RUnlock()
	product, ok := e.products[productID]
	return product, ok
}

// ListProducts returns all products for a tenant
func (e *MonetizationEngine) ListProducts(tenantID string) []*APIProduct {
	e.mu.RLock()
	defer e.mu.RUnlock()

	tenantProducts, ok := e.byTenant[tenantID]
	if !ok {
		return nil
	}

	products := make([]*APIProduct, 0, len(tenantProducts))
	for _, p := range tenantProducts {
		products = append(products, p)
	}
	return products
}

// CreateSubscription creates a new API subscription
func (e *MonetizationEngine) CreateSubscription(sub *APISubscription) error {
	e.mu.Lock()
	defer e.mu.Unlock()

	// Verify product exists and is active
	product, ok := e.products[sub.ProductID]
	if !ok {
		return &monetizationError{"product not found"}
	}
	if product.Status != "active" {
		return &monetizationError{"product is not active"}
	}

	sub.CreatedAt = time.Now()
	sub.Status = "active"
	sub.CurrentPeriodStart = time.Now()
	sub.CurrentPeriodEnd = time.Now().AddDate(0, 1, 0) // 1 month

	if product.FreeTierLimit > 0 {
		sub.UsageLimit = product.FreeTierLimit
	}

	e.subscriptions[sub.ID] = sub
	e.byAPIKey[sub.APIKey] = sub

	e.logger.Info("API subscription created",
		zap.String("subscription_id", sub.ID),
		zap.String("product_id", sub.ProductID),
		zap.String("consumer_id", sub.ConsumerID),
	)

	return nil
}

// GetSubscriptionByAPIKey looks up a subscription by API key
func (e *MonetizationEngine) GetSubscriptionByAPIKey(apiKey string) (*APISubscription, bool) {
	e.mu.RLock()
	defer e.mu.RUnlock()
	sub, ok := e.byAPIKey[apiKey]
	return sub, ok
}

// RecordUsage records an API call for billing
func (e *MonetizationEngine) RecordUsage(record *UsageRecord) error {
	e.mu.Lock()
	defer e.mu.Unlock()

	sub, ok := e.subscriptions[record.SubscriptionID]
	if !ok {
		return &monetizationError{"subscription not found"}
	}

	// Check usage limits
	if sub.UsageLimit > 0 && sub.UsageCount >= sub.UsageLimit {
		return &monetizationError{"usage limit exceeded"}
	}

	// Calculate billing amount
	product, ok := e.products[sub.ProductID]
	if ok {
		switch product.PricingModel {
		case PricingPayPerCall:
			record.BilledAmount = product.PricePerCall
		case PricingFreemium:
			if sub.UsageCount >= product.FreeTierLimit {
				record.BilledAmount = product.PricePerCall
			}
		}
	}

	record.Timestamp = time.Now()
	sub.UsageCount++
	e.usageRecords = append(e.usageRecords, record)

	return nil
}

// GetRevenueReport generates a revenue report for a tenant
func (e *MonetizationEngine) GetRevenueReport(tenantID string, start, end time.Time) *RevenueReport {
	e.mu.RLock()
	defer e.mu.RUnlock()

	report := &RevenueReport{
		TenantID:         tenantID,
		PeriodStart:      start,
		PeriodEnd:        end,
		RevenueByProduct: make(map[string]float64),
	}

	consumerUsage := make(map[string]*ConsumerUsage)
	uniqueConsumers := make(map[string]bool)

	// Find products belonging to this tenant
	tenantProducts := make(map[string]bool)
	if prods, ok := e.byTenant[tenantID]; ok {
		for id := range prods {
			tenantProducts[id] = true
		}
	}

	for _, record := range e.usageRecords {
		if !tenantProducts[record.ProductID] {
			continue
		}
		if record.Timestamp.Before(start) || record.Timestamp.After(end) {
			continue
		}

		report.TotalRevenue += record.BilledAmount
		report.TotalAPICalls++
		report.RevenueByProduct[record.ProductID] += record.BilledAmount

		sub, ok := e.subscriptions[record.SubscriptionID]
		if ok {
			uniqueConsumers[sub.ConsumerID] = true

			if cu, exists := consumerUsage[sub.ConsumerID]; exists {
				cu.TotalCalls++
				cu.TotalRevenue += record.BilledAmount
			} else {
				consumerUsage[sub.ConsumerID] = &ConsumerUsage{
					ConsumerID:   sub.ConsumerID,
					TotalCalls:   1,
					TotalRevenue: record.BilledAmount,
				}
			}
		}
	}

	report.UniqueConsumers = len(uniqueConsumers)

	for _, cu := range consumerUsage {
		report.TopConsumers = append(report.TopConsumers, *cu)
	}

	return report
}

// GetProductAnalytics returns analytics for a specific API product
func (e *MonetizationEngine) GetProductAnalytics(productID string) *APIAnalytics {
	e.mu.RLock()
	defer e.mu.RUnlock()

	analytics := &APIAnalytics{
		ProductID:       productID,
		CallsByEndpoint: make(map[string]int64),
	}

	consumers := make(map[string]bool)
	var totalLatency int64
	var errorCount int64
	cutoff := time.Now().Add(-24 * time.Hour)

	for _, record := range e.usageRecords {
		if record.ProductID != productID {
			continue
		}

		analytics.TotalCalls++
		totalLatency += record.LatencyMs
		analytics.CallsByEndpoint[record.Endpoint]++

		if record.StatusCode >= 400 {
			errorCount++
		}

		if record.Timestamp.After(cutoff) {
			analytics.Revenue24h += record.BilledAmount
		}

		sub, ok := e.subscriptions[record.SubscriptionID]
		if ok {
			consumers[sub.ConsumerID] = true
		}
	}

	analytics.ActiveConsumers = len(consumers)

	if analytics.TotalCalls > 0 {
		analytics.AvgLatencyMs = float64(totalLatency) / float64(analytics.TotalCalls)
		analytics.ErrorRate = float64(errorCount) / float64(analytics.TotalCalls) * 100.0
	}

	return analytics
}

// ListSubscriptions returns all subscriptions for a product
func (e *MonetizationEngine) ListSubscriptions(productID string) []*APISubscription {
	e.mu.RLock()
	defer e.mu.RUnlock()

	subs := make([]*APISubscription, 0)
	for _, sub := range e.subscriptions {
		if sub.ProductID == productID {
			subs = append(subs, sub)
		}
	}
	return subs
}

type monetizationError struct {
	msg string
}

func (e *monetizationError) Error() string {
	return e.msg
}
