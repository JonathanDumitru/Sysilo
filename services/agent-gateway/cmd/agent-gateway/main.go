package main

import (
	"context"
	"flag"
	"fmt"
	"net"
	"os"
	"os/signal"
	"strings"
	"syscall"

	"github.com/sysilo/sysilo/services/agent-gateway/internal/config"
	"github.com/sysilo/sysilo/services/agent-gateway/internal/kafka"
	"github.com/sysilo/sysilo/services/agent-gateway/internal/registry"
	"github.com/sysilo/sysilo/services/agent-gateway/internal/tunnel"
	"go.uber.org/zap"
	"google.golang.org/grpc"
)

var (
	version   = "dev"
	commit    = "unknown"
	buildDate = "unknown"
)

func main() {
	// Parse command line flags
	configPath := flag.String("config", "", "Path to configuration file")
	showVersion := flag.Bool("version", false, "Show version information")
	flag.Parse()

	if *showVersion {
		fmt.Printf("Sysilo Agent Gateway %s (commit: %s, built: %s)\n", version, commit, buildDate)
		os.Exit(0)
	}

	// Initialize logger
	logger, err := zap.NewProduction()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize logger: %v\n", err)
		os.Exit(1)
	}
	defer logger.Sync()

	logger.Info("Starting Sysilo Agent Gateway",
		zap.String("version", version),
		zap.String("commit", commit),
	)

	// Load configuration
	cfg, err := config.Load(*configPath)
	if err != nil {
		logger.Fatal("Failed to load configuration", zap.Error(err))
	}

	// Create context with cancellation
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Initialize agent registry
	agentRegistry := registry.New(logger)

	// Initialize Kafka producer if enabled
	var kafkaProducer *kafka.Producer
	if cfg.Kafka.Enabled {
		brokers := strings.Split(cfg.Kafka.Brokers, ",")
		producerCfg := kafka.ProducerConfig{
			Brokers:     brokers,
			ResultTopic: cfg.Kafka.TaskResultTopic,
			LogsTopic:   cfg.Kafka.LogsTopic,
		}
		var err error
		kafkaProducer, err = kafka.NewProducer(logger, producerCfg)
		if err != nil {
			logger.Fatal("Failed to create Kafka producer", zap.Error(err))
		}
		defer kafkaProducer.Close()
		logger.Info("Kafka producer initialized", zap.Strings("brokers", brokers))
	} else {
		logger.Info("Kafka integration disabled")
	}

	// Initialize tunnel server
	tunnelServer := tunnel.NewServer(logger, cfg, agentRegistry, kafkaProducer)

	// Create gRPC server
	var opts []grpc.ServerOption
	// TODO: Add TLS credentials for production

	grpcServer := grpc.NewServer(opts...)
	tunnelServer.Register(grpcServer)

	// Start listening
	listener, err := net.Listen("tcp", cfg.Server.Address)
	if err != nil {
		logger.Fatal("Failed to listen", zap.Error(err), zap.String("address", cfg.Server.Address))
	}

	// Start server in goroutine
	go func() {
		logger.Info("Agent Gateway listening", zap.String("address", cfg.Server.Address))
		if err := grpcServer.Serve(listener); err != nil {
			logger.Error("Server error", zap.Error(err))
			cancel()
		}
	}()

	// Wait for shutdown signal
	sigCh := make(chan os.Signal, 1)
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	select {
	case sig := <-sigCh:
		logger.Info("Received shutdown signal", zap.String("signal", sig.String()))
	case <-ctx.Done():
		logger.Info("Context cancelled")
	}

	// Graceful shutdown
	logger.Info("Shutting down server...")
	grpcServer.GracefulStop()

	logger.Info("Agent Gateway shutdown complete")
}
