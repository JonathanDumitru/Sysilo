package ratelimit

import (
	"context"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
)

// Config holds rate limiting thresholds.
type Config struct {
	RequestsPerMinute int
	RequestsPerHour   int
	BurstSize         int
}

// RateLimitResult contains the outcome of a rate limit check.
type RateLimitResult struct {
	Allowed    bool
	Remaining  int
	ResetAt    time.Time
	RetryAfter time.Duration
}

// RateLimiter implements sliding-window rate limiting backed by Redis sorted sets.
type RateLimiter struct {
	client *redis.Client
	config Config
}

// New creates a RateLimiter with the given Redis client and configuration.
func New(client *redis.Client, cfg Config) *RateLimiter {
	return &RateLimiter{
		client: client,
		config: cfg,
	}
}

// Check verifies whether the request identified by tenantID and endpoint is
// within the configured rate limits. It uses a sliding window algorithm backed
// by Redis sorted sets with pipelined MULTI/EXEC transactions.
//
// Key format: ratelimit:{tenant_id}:{endpoint}:{window}
// where window is "min" or "hour".
func (rl *RateLimiter) Check(ctx context.Context, tenantID, endpoint string) (bool, int, time.Time, error) {
	now := time.Now()
	nowUnix := float64(now.UnixNano())
	member := fmt.Sprintf("%d", now.UnixNano())

	// Check per-minute window
	if rl.config.RequestsPerMinute > 0 {
		result, err := rl.checkWindow(ctx, tenantID, endpoint, "min", time.Minute, rl.config.RequestsPerMinute, now, nowUnix, member)
		if err != nil {
			return false, 0, time.Time{}, err
		}
		if !result.Allowed {
			return result.Allowed, result.Remaining, result.ResetAt, nil
		}
	}

	// Check per-hour window
	if rl.config.RequestsPerHour > 0 {
		result, err := rl.checkWindow(ctx, tenantID, endpoint, "hour", time.Hour, rl.config.RequestsPerHour, now, nowUnix, member)
		if err != nil {
			return false, 0, time.Time{}, err
		}
		if !result.Allowed {
			return result.Allowed, result.Remaining, result.ResetAt, nil
		}
	}

	// Both windows passed; report remaining from the per-minute window
	remaining := rl.config.RequestsPerMinute - 1 // approximate after add
	resetAt := now.Add(time.Minute).Truncate(time.Second)
	if rl.config.RequestsPerMinute <= 0 && rl.config.RequestsPerHour > 0 {
		remaining = rl.config.RequestsPerHour - 1
		resetAt = now.Add(time.Hour).Truncate(time.Second)
	}

	return true, remaining, resetAt, nil
}

// checkWindow performs a sliding-window check for one time window using a
// Redis MULTI/EXEC pipeline:
//  1. ZREMRANGEBYSCORE removes entries outside the window.
//  2. ZCARD counts current entries (before adding).
//  3. ZADD adds the new entry.
//  4. EXPIRE sets a TTL equal to the window duration.
func (rl *RateLimiter) checkWindow(
	ctx context.Context,
	tenantID, endpoint, windowName string,
	windowDuration time.Duration,
	limit int,
	now time.Time,
	nowUnix float64,
	member string,
) (*RateLimitResult, error) {
	key := fmt.Sprintf("ratelimit:%s:%s:%s", tenantID, endpoint, windowName)
	windowStart := float64(now.Add(-windowDuration).UnixNano())

	pipe := rl.client.TxPipeline()

	// Remove expired entries
	pipe.ZRemRangeByScore(ctx, key, "-inf", fmt.Sprintf("%f", windowStart))

	// Count current entries in the window
	countCmd := pipe.ZCard(ctx, key)

	// Add the current request
	pipe.ZAdd(ctx, key, redis.Z{
		Score:  nowUnix,
		Member: member,
	})

	// Set key expiry to auto-clean
	pipe.Expire(ctx, key, windowDuration+time.Second)

	_, err := pipe.Exec(ctx)
	if err != nil {
		return nil, fmt.Errorf("rate limit redis pipeline error: %w", err)
	}

	count := countCmd.Val()
	resetAt := now.Add(windowDuration).Truncate(time.Second)

	if count >= int64(limit) {
		// Over limit: remove the entry we just added since the request is denied
		rl.client.ZRem(ctx, key, member)

		remaining := 0
		retryAfter := windowDuration / time.Duration(limit)
		if retryAfter < time.Second {
			retryAfter = time.Second
		}

		return &RateLimitResult{
			Allowed:    false,
			Remaining:  remaining,
			ResetAt:    resetAt,
			RetryAfter: retryAfter,
		}, nil
	}

	remaining := int(int64(limit) - count - 1)
	if remaining < 0 {
		remaining = 0
	}

	return &RateLimitResult{
		Allowed:   true,
		Remaining: remaining,
		ResetAt:   resetAt,
	}, nil
}
