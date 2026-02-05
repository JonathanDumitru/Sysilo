package executor

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/sysilo/sysilo/agent/internal/config"
	"go.uber.org/zap"
)

// TaskHandler processes a specific type of task
type TaskHandler interface {
	// Type returns the task type this handler processes
	Type() string

	// Execute runs the task and returns the result
	Execute(ctx context.Context, task *Task) (*TaskResult, error)
}

// Task represents a unit of work to be executed
type Task struct {
	ID            string
	IntegrationID string
	TenantID      string
	Type          string
	Config        map[string]interface{}
	Priority      int
	Timeout       time.Duration
	RetryCount    int
}

// TaskResult represents the outcome of task execution
type TaskResult struct {
	TaskID      string
	Status      TaskStatus
	StartedAt   time.Time
	CompletedAt time.Time
	Output      interface{}
	Error       *TaskError
	Metrics     TaskMetrics
}

// TaskStatus represents the execution status
type TaskStatus string

const (
	TaskStatusPending   TaskStatus = "pending"
	TaskStatusRunning   TaskStatus = "running"
	TaskStatusCompleted TaskStatus = "completed"
	TaskStatusFailed    TaskStatus = "failed"
	TaskStatusCancelled TaskStatus = "cancelled"
	TaskStatusTimeout   TaskStatus = "timeout"
)

// TaskError contains error details
type TaskError struct {
	Code      string
	Message   string
	Details   string
	Retryable bool
}

// TaskMetrics contains execution metrics
type TaskMetrics struct {
	RecordsRead    int64
	RecordsWritten int64
	BytesProcessed int64
	DurationMs     int64
}

// QueryResultOutput represents structured query result output
type QueryResultOutput struct {
	Columns      []ColumnInfo
	Rows         [][]interface{}
	RowsAffected int64
	HasMore      bool
	Cursor       string
}

// ColumnInfo describes a column in a query result
type ColumnInfo struct {
	Name     string
	DataType string
	Nullable bool
}

// RunningTaskInfo tracks information about a currently executing task
type RunningTaskInfo struct {
	TaskID        string
	IntegrationID string
	StartedAt     time.Time
	Cancel        context.CancelFunc
}

// Executor manages task execution with configurable concurrency
type Executor struct {
	logger       *zap.Logger
	config       *config.Config
	handlers     map[string]TaskHandler
	runningTasks map[string]*RunningTaskInfo
	mu           sync.RWMutex
	semaphore    chan struct{}
	wg           sync.WaitGroup
}

// New creates a new Executor instance
func New(logger *zap.Logger, cfg *config.Config) (*Executor, error) {
	e := &Executor{
		logger:       logger.Named("executor"),
		config:       cfg,
		handlers:     make(map[string]TaskHandler),
		runningTasks: make(map[string]*RunningTaskInfo),
		semaphore:    make(chan struct{}, cfg.Agent.MaxConcurrentTasks),
	}

	// Register built-in handlers
	e.registerBuiltinHandlers()

	return e, nil
}

// registerBuiltinHandlers registers the default task handlers
func (e *Executor) registerBuiltinHandlers() {
	// Handlers are registered from main.go after executor creation
	// to avoid circular imports. See cmd/agent/main.go
}

// RegisterHandler registers a task handler for a specific task type
func (e *Executor) RegisterHandler(handler TaskHandler) {
	e.mu.Lock()
	defer e.mu.Unlock()
	e.handlers[handler.Type()] = handler
	e.logger.Info("Registered task handler", zap.String("type", handler.Type()))
}

// Execute runs a task asynchronously and returns immediately
func (e *Executor) Execute(ctx context.Context, task *Task) <-chan *TaskResult {
	resultCh := make(chan *TaskResult, 1)

	go func() {
		defer close(resultCh)

		// Acquire semaphore slot
		select {
		case e.semaphore <- struct{}{}:
			defer func() { <-e.semaphore }()
		case <-ctx.Done():
			resultCh <- &TaskResult{
				TaskID:      task.ID,
				Status:      TaskStatusCancelled,
				CompletedAt: time.Now(),
				Error: &TaskError{
					Code:    "cancelled",
					Message: "Task cancelled before execution started",
				},
			}
			return
		}

		// Execute the task
		result := e.executeTask(ctx, task)
		resultCh <- result
	}()

	return resultCh
}

// ExecuteSync runs a task synchronously and waits for completion
func (e *Executor) ExecuteSync(ctx context.Context, task *Task) *TaskResult {
	resultCh := e.Execute(ctx, task)
	return <-resultCh
}

// executeTask performs the actual task execution
func (e *Executor) executeTask(ctx context.Context, task *Task) *TaskResult {
	startedAt := time.Now()

	e.logger.Info("Starting task execution",
		zap.String("task_id", task.ID),
		zap.String("type", task.Type),
		zap.String("integration_id", task.IntegrationID),
	)

	// Create task-specific context with timeout
	taskCtx, cancel := context.WithTimeout(ctx, task.Timeout)
	defer cancel()

	// Track running task with start time
	e.mu.Lock()
	e.runningTasks[task.ID] = &RunningTaskInfo{
		TaskID:        task.ID,
		IntegrationID: task.IntegrationID,
		StartedAt:     startedAt,
		Cancel:        cancel,
	}
	e.mu.Unlock()

	defer func() {
		e.mu.Lock()
		delete(e.runningTasks, task.ID)
		e.mu.Unlock()
	}()

	// Find handler for task type
	e.mu.RLock()
	handler, ok := e.handlers[task.Type]
	e.mu.RUnlock()

	if !ok {
		return &TaskResult{
			TaskID:      task.ID,
			Status:      TaskStatusFailed,
			StartedAt:   startedAt,
			CompletedAt: time.Now(),
			Error: &TaskError{
				Code:      "unknown_task_type",
				Message:   fmt.Sprintf("No handler registered for task type: %s", task.Type),
				Retryable: false,
			},
		}
	}

	// Execute the task
	result, err := handler.Execute(taskCtx, task)
	completedAt := time.Now()

	if err != nil {
		// Check if it was a timeout
		if taskCtx.Err() == context.DeadlineExceeded {
			e.logger.Warn("Task timed out",
				zap.String("task_id", task.ID),
				zap.Duration("timeout", task.Timeout),
			)
			return &TaskResult{
				TaskID:      task.ID,
				Status:      TaskStatusTimeout,
				StartedAt:   startedAt,
				CompletedAt: completedAt,
				Error: &TaskError{
					Code:      "timeout",
					Message:   fmt.Sprintf("Task exceeded timeout of %v", task.Timeout),
					Retryable: true,
				},
				Metrics: TaskMetrics{
					DurationMs: completedAt.Sub(startedAt).Milliseconds(),
				},
			}
		}

		// Check if cancelled
		if taskCtx.Err() == context.Canceled {
			return &TaskResult{
				TaskID:      task.ID,
				Status:      TaskStatusCancelled,
				StartedAt:   startedAt,
				CompletedAt: completedAt,
				Error: &TaskError{
					Code:    "cancelled",
					Message: "Task was cancelled",
				},
			}
		}

		e.logger.Error("Task execution failed",
			zap.String("task_id", task.ID),
			zap.Error(err),
		)
		return &TaskResult{
			TaskID:      task.ID,
			Status:      TaskStatusFailed,
			StartedAt:   startedAt,
			CompletedAt: completedAt,
			Error: &TaskError{
				Code:      "execution_error",
				Message:   err.Error(),
				Retryable: true,
			},
			Metrics: TaskMetrics{
				DurationMs: completedAt.Sub(startedAt).Milliseconds(),
			},
		}
	}

	// Set timestamps on successful result
	result.TaskID = task.ID
	result.Status = TaskStatusCompleted
	result.StartedAt = startedAt
	result.CompletedAt = completedAt
	result.Metrics.DurationMs = completedAt.Sub(startedAt).Milliseconds()

	e.logger.Info("Task completed successfully",
		zap.String("task_id", task.ID),
		zap.Int64("duration_ms", result.Metrics.DurationMs),
	)

	return result
}

// CancelTask cancels a running task
func (e *Executor) CancelTask(taskID string) bool {
	e.mu.RLock()
	info, ok := e.runningTasks[taskID]
	e.mu.RUnlock()

	if ok {
		info.Cancel()
		e.logger.Info("Task cancelled", zap.String("task_id", taskID))
		return true
	}

	return false
}

// RunningTasks returns task IDs of currently executing tasks
func (e *Executor) RunningTasks() []string {
	e.mu.RLock()
	defer e.mu.RUnlock()

	tasks := make([]string, 0, len(e.runningTasks))
	for taskID := range e.runningTasks {
		tasks = append(tasks, taskID)
	}
	return tasks
}

// RunningTasksInfo returns detailed information about currently executing tasks
func (e *Executor) RunningTasksInfo() []*RunningTaskInfo {
	e.mu.RLock()
	defer e.mu.RUnlock()

	tasks := make([]*RunningTaskInfo, 0, len(e.runningTasks))
	for _, info := range e.runningTasks {
		tasks = append(tasks, info)
	}
	return tasks
}

// Shutdown gracefully shuts down the executor
func (e *Executor) Shutdown(ctx context.Context) error {
	e.logger.Info("Shutting down executor")

	// Cancel all running tasks
	e.mu.Lock()
	for taskID, info := range e.runningTasks {
		e.logger.Info("Cancelling task due to shutdown", zap.String("task_id", taskID))
		info.Cancel()
	}
	e.mu.Unlock()

	// Wait for all tasks to complete (with timeout from context)
	done := make(chan struct{})
	go func() {
		e.wg.Wait()
		close(done)
	}()

	select {
	case <-done:
		e.logger.Info("All tasks completed")
		return nil
	case <-ctx.Done():
		e.logger.Warn("Shutdown timeout reached, some tasks may not have completed")
		return ctx.Err()
	}
}
