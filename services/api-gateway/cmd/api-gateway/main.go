package main

import (
	"context"
	"flag"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/go-chi/chi/v5"
	chimiddleware "github.com/go-chi/chi/v5/middleware"
	"github.com/sysilo/sysilo/services/api-gateway/internal/config"
	"github.com/sysilo/sysilo/services/api-gateway/internal/db"
	"github.com/sysilo/sysilo/services/api-gateway/internal/handlers"
	"github.com/sysilo/sysilo/services/api-gateway/internal/middleware"
	"go.uber.org/zap"
)

var (
	version   = "dev"
	commit    = "unknown"
	buildDate = "unknown"
)

const (
	apiV1RoutePrefix       = "/api/v1"
	connectionsRoutePrefix = "/connections"
)

func main() {
	// Parse command line flags
	configPath := flag.String("config", "", "Path to configuration file")
	showVersion := flag.Bool("version", false, "Show version information")
	flag.Parse()

	if *showVersion {
		fmt.Printf("Sysilo API Gateway %s (commit: %s, built: %s)\n", version, commit, buildDate)
		os.Exit(0)
	}

	// Initialize logger
	logger, err := zap.NewProduction()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize logger: %v\n", err)
		os.Exit(1)
	}
	defer logger.Sync()

	logger.Info("Starting Sysilo API Gateway",
		zap.String("version", version),
		zap.String("commit", commit),
	)

	// Load configuration
	cfg, err := config.Load(*configPath)
	if err != nil {
		logger.Fatal("Failed to load configuration", zap.Error(err))
	}

	// Initialize database
	database, err := db.New(cfg.Database, logger)
	if err != nil {
		logger.Fatal("Failed to connect to database", zap.Error(err))
	}
	defer database.Close()

	// Create handlers with dependencies
	h := handlers.New(database, logger)

	// Create router
	r := chi.NewRouter()

	// Global middleware
	r.Use(chimiddleware.RequestID)
	r.Use(chimiddleware.RealIP)
	r.Use(middleware.Logger(logger))
	r.Use(chimiddleware.Recoverer)
	r.Use(middleware.CORS(cfg.CORS))
	r.Use(chimiddleware.Timeout(60 * time.Second))

	// Health check (no auth required)
	r.Get("/health", h.Health)
	r.Get("/ready", h.Ready)

	// Plan loader: queries the DB for tenant plan info
	planLoader := func(ctx context.Context, tenantID string) (*middleware.PlanInfo, error) {
		tp, err := database.Plans.GetTenantPlan(ctx, tenantID)
		if err != nil || tp == nil || tp.Plan == nil {
			return nil, err
		}
		return &middleware.PlanInfo{
			Name:     tp.Plan.Name,
			Status:   tp.PlanStatus,
			Features: tp.Plan.Features,
			Limits:   tp.Plan.Limits,
		}, nil
	}

	// Stripe webhook (no auth — Stripe signs with its own secret)
	r.Post(apiV1RoutePrefix+"/billing/webhooks", h.HandleStripeWebhook)

	// Public plan listing (no auth needed for pricing page)
	r.Get(apiV1RoutePrefix+"/plans", h.ListPlans)

	// Public auth flows
	r.Route(apiV1RoutePrefix+"/auth", func(r chi.Router) {
		r.Get("/sso/start", h.StartSSO)
		r.Get("/sso/callback", h.HandleSSOCallback)
		r.Post("/breakglass/start", h.StartBreakglassLogin)
		r.Post("/breakglass/complete", h.CompleteBreakglassLogin)
		r.Post("/session/refresh", h.RefreshSession)
	})

	// SCIM provisioning routes (SCIM token + admin scope required)
	r.Route(apiV1RoutePrefix+"/scim", func(r chi.Router) {
		r.Use(middleware.RequireSCIMToken(logger, cfg.Auth))
		r.Use(middleware.RequireSCIMAdminScope())
		r.Route("/users", func(r chi.Router) {
			r.Post("/", h.SCIMCreateUser)
			r.Put("/{userID}", h.SCIMUpdateUser)
			r.Delete("/{userID}", h.Deactivate)
		})
	})

	// API routes (auth required)
	r.Route(apiV1RoutePrefix, func(r chi.Router) {
		// Auth middleware
		r.Use(middleware.Auth(logger, cfg.Auth))

		// Tenant context
		r.Use(middleware.TenantContext(logger))

		// Load tenant plan into context
		r.Use(middleware.LoadTenantPlan(logger, planLoader))

		// Plan gate (feature gating)
		r.Use(middleware.PlanGate(logger))

		// Rate limiting
		r.Use(middleware.RateLimit(cfg.RateLimit))

		// Plan & billing
		r.Get("/plan", h.GetCurrentPlan)
		r.Get("/plan/usage", h.GetPlanUsage)
		r.Route("/billing", func(r chi.Router) {
			r.Post("/checkout", h.CreateCheckoutSession)
			r.Post("/portal", h.CreatePortalSession)
			r.Get("/subscription", h.GetSubscription)
		})

		// Agents
		r.Route("/agents", func(r chi.Router) {
			r.Get("/", h.ListAgents)
			r.Get("/{agentID}", h.GetAgent)
			r.Delete("/{agentID}", h.DeleteAgent)
		})

		// Connections
		r.Route(connectionsRoutePrefix, func(r chi.Router) {
			r.Get("/", h.ListConnections)
			r.Post("/", h.CreateConnection)
			r.Get("/{connectionID}", h.GetConnection)
			r.Put("/{connectionID}", h.UpdateConnection)
			r.Delete("/{connectionID}", h.DeleteConnection)
			r.Post("/{connectionID}/test", h.TestConnection)
		})

		// Integrations
		r.Route("/integrations", func(r chi.Router) {
			r.Get("/", h.ListIntegrations)
			r.Post("/", h.CreateIntegration)
			r.Get("/{integrationID}", h.GetIntegration)
			r.Put("/{integrationID}", h.UpdateIntegration)
			r.Delete("/{integrationID}", h.DeleteIntegration)
			r.Post("/{integrationID}/run", h.RunIntegration)
			r.Get("/{integrationID}/runs", h.ListIntegrationRuns)
		})

		// Integration runs
		r.Route("/runs", func(r chi.Router) {
			r.Get("/{runID}", h.GetRun)
			r.Post("/{runID}/cancel", h.CancelRun)
			r.Get("/{runID}/logs", h.GetRunLogs)
		})

		// Users (admin only)
		r.Route("/users", func(r chi.Router) {
			r.Use(middleware.RequireRole("admin"))
			r.Get("/", h.ListUsers)
			r.Post("/", h.CreateUser)
			r.Get("/{userID}", h.GetUser)
			r.Put("/{userID}", h.UpdateUser)
			r.Delete("/{userID}", h.DeleteUser)
		})
	})

	// Create HTTP server
	srv := &http.Server{
		Addr:         cfg.Server.Address,
		Handler:      r,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 60 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	// Start server in goroutine
	go func() {
		logger.Info("API Gateway listening", zap.String("address", cfg.Server.Address))
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			logger.Fatal("Server error", zap.Error(err))
		}
	}()

	// Wait for shutdown signal
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)
	<-sigCh

	logger.Info("Shutting down server...")

	// Graceful shutdown with timeout
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		logger.Error("Server forced to shutdown", zap.Error(err))
	}

	logger.Info("API Gateway shutdown complete")
}
