package main

import (
	"context"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"syscall"

	"github.com/sysilo/sysilo/agent/internal/adapters/postgresql"
	"github.com/sysilo/sysilo/agent/internal/config"
	"github.com/sysilo/sysilo/agent/internal/executor"
	"github.com/sysilo/sysilo/agent/internal/tunnel"
	"github.com/sysilo/sysilo/agent/pkg/logging"
	"github.com/sysilo/sysilo/agent/pkg/version"
	"go.uber.org/zap"
)

func main() {
	// Parse command line flags
	configPath := flag.String("config", "", "Path to configuration file")
	showVersion := flag.Bool("version", false, "Show version information")
	flag.Parse()

	if *showVersion {
		fmt.Printf("Sysilo Agent %s\n", version.Info())
		os.Exit(0)
	}

	// Initialize logger
	logger, err := logging.NewLogger("info")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize logger: %v\n", err)
		os.Exit(1)
	}
	defer logger.Sync()

	logger.Info("Starting Sysilo Agent",
		zap.String("version", version.Version),
		zap.String("commit", version.Commit),
	)

	// Load configuration
	cfg, err := config.Load(*configPath)
	if err != nil {
		logger.Fatal("Failed to load configuration", zap.Error(err))
	}

	// Create context with cancellation
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Initialize task executor
	exec, err := executor.New(logger, cfg)
	if err != nil {
		logger.Fatal("Failed to initialize executor", zap.Error(err))
	}

	// Register adapters
	exec.RegisterHandler(postgresql.NewAdapter(logger))

	// Initialize tunnel client
	tunnelClient, err := tunnel.NewClient(logger, cfg, exec)
	if err != nil {
		logger.Fatal("Failed to initialize tunnel client", zap.Error(err))
	}

	// Start the tunnel connection
	go func() {
		if err := tunnelClient.Connect(ctx); err != nil {
			logger.Error("Tunnel connection failed", zap.Error(err))
			cancel()
		}
	}()

	// Wait for shutdown signal or restart request
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	exitCode := 0
	select {
	case sig := <-sigCh:
		logger.Info("Received shutdown signal", zap.String("signal", sig.String()))
	case <-ctx.Done():
		logger.Info("Context cancelled")
	case restart := <-tunnelClient.RestartCh():
		logger.Info("Received restart request from gateway",
			zap.String("reason", restart.Reason),
		)
		// Exit with code 75 to signal supervisor to restart
		exitCode = 75
	}

	// Graceful shutdown
	logger.Info("Shutting down agent...")
	cancel()

	if err := tunnelClient.Close(); err != nil {
		logger.Error("Error closing tunnel", zap.Error(err))
	}

	if err := exec.Shutdown(context.Background()); err != nil {
		logger.Error("Error shutting down executor", zap.Error(err))
	}

	logger.Info("Agent shutdown complete")
	os.Exit(exitCode)
}
