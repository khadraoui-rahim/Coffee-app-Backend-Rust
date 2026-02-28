# Business Rules System - Performance Optimization

This document describes the performance optimizations implemented in the Business Rules System.

## Overview

The Business Rules System is designed to meet the requirement of < 100ms evaluation time for rule processing. Multiple optimization strategies have been implemented to achieve this goal.

## Performance Monitoring (Task 12.1)

### Metrics Collection

A comprehensive metrics system tracks:
- **Cache Performance**: Hit rate, hits, misses
- **Operation Timing**: Average execution time for each operation type
- **Slow Operations**: Count of operations exceeding 100ms threshold

### Implementation

The `PerformanceMetrics` struct uses atomic counters for thread-safe, lock-free metrics collection:

```rust
pub struct PerformanceMetrics {
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    availability_checks: AtomicU64,
    pricing_calculations: AtomicU64,
    prep_time_estimates: AtomicU64,
    loyalty_calculations: AtomicU64,
    // Timing metrics in microseconds
    total_availability_time_us: AtomicU64,
    total_pricing_time_us: AtomicU64,
    // ... etc
}
```

### Automatic Timing

Each business rules operation is automatically timed using RAII-based timers:

```rust
pub async fn validate_order(&self, order_id: Uuid, items: &[OrderItem]) -> BRResult<OrderValidationResult> {
    let _timer = self.metrics.start_availability_check();
    // Operation is automatically timed when timer drops
    // ...
}
```

### Slow Operation Detection

Operations exceeding 100ms are automatically logged as warnings:

```rust
if duration.as_millis() as u64 > SLOW_OPERATION_THRESHOLD_MS {
    self.inner.slow_availability_checks.fetch_add(1, Ordering::Relaxed);
    tracing::warn!("Slow availability check: {}ms", duration.as_millis());
}
```

### Metrics API Endpoint

Performance metrics are exposed via `GET /api/business-rules/metrics`:

```json
{
  "cache": {
    "hit_rate": "95.2%",
    "hits": 1234,
    "misses": 62
  },
  "availability": {
    "checks": 856,
    "avg_time_ms": "12.34",
    "slow_operations": 2
  },
  "pricing": {
    "calculations": 856,
    "avg_time_ms": "45.67",
    "slow_operations": 5
  },
  "prep_time": {
    "estimates": 856,
    "avg_time_ms": "8.90",
    "slow_operations": 0
  },
  "loyalty": {
    "calculations": 234,
    "avg_time_ms": "15.23",
    "slow_operations": 1
  }
}
```

## Database Query Optimization (Task 12.2)

### Indexes

Database indexes were added in the migration `20260228000002_create_business_rules_tables.sql`:

```sql
-- Pricing rules index for active rule queries
CREATE INDEX idx_pricing_rules_active ON pricing_rules(is_active, valid_from, valid_until);

-- Audit log indexes for querying
CREATE INDEX idx_rule_audit_order ON rule_audit_log(order_id);
CREATE INDEX idx_rule_audit_created ON rule_audit_log(created_at);
```

### Prepared Statements

All database queries use sqlx's compile-time checked queries, which automatically use prepared statements:

```rust
sqlx::query_as!(
    CoffeeAvailability,
    r#"SELECT coffee_id, status, reason, available_from, available_until, updated_at
       FROM coffee_availability"#
)
.fetch_all(&self.pool)
.await?
```

### Batch Loading

Coffee data is batch-loaded for orders with multiple items:

```rust
// Fetch all coffee items in a single query
let coffees = self.coffee_repo.find_by_ids(&coffee_ids).await?;
```

## Cache Strategy Optimization (Task 12.3)

### Time-Based Cache with TTL

The configuration store implements a 60-second TTL cache:

```rust
const CACHE_TTL: Duration = Duration::from_secs(60);
```

This balances:
- **Performance**: Most requests hit the cache (no database query)
- **Freshness**: Configuration changes are reflected within 60 seconds
- **Consistency**: All requests within the TTL window see the same data

### Cache Structure

```rust
struct ConfigCache {
    availability_rules: HashMap<i32, CoffeeAvailability>,
    pricing_rules: Vec<PricingRule>,
    prep_time_config: HashMap<i32, CoffeeBaseTime>,
    loyalty_config: Option<LoyaltyConfig>,
    last_updated: HashMap<String, Instant>,
}
```

### Double-Checked Locking

Cache refresh uses double-checked locking to minimize contention:

```rust
async fn refresh_if_stale(&self, rule_type: &str) -> BRResult<()> {
    // Fast path: check with read lock
    {
        let cache = self.cache.read().await;
        if !cache.is_stale(rule_type, self.cache_ttl) {
            self.record_cache_hit();
            return Ok(());
        }
    }
    
    // Slow path: refresh with write lock
    self.record_cache_miss();
    let mut cache = self.cache.write().await;
    
    // Double-check (another thread might have refreshed)
    if !cache.is_stale(rule_type, self.cache_ttl) {
        return Ok(());
    }
    
    // Load fresh data
    // ...
}
```

### Cache Warming

Cache is pre-loaded on application startup to avoid cold-start latency:

```rust
// In main.rs
if let Err(e) = business_rules_engine.warm_cache().await {
    tracing::warn!("Failed to warm cache: {}. Continuing with cold cache.", e);
}
```

This ensures:
- First requests don't experience cache miss latency
- All configuration types are validated on startup
- Startup failures are detected early

### Cache Metrics

Cache effectiveness is tracked with hit/miss counters:

```rust
pub fn cache_hit_rate(&self) -> f64 {
    let hits = self.inner.cache_hits.load(Ordering::Relaxed);
    let misses = self.inner.cache_misses.load(Ordering::Relaxed);
    let total = hits + misses;
    
    if total == 0 {
        0.0
    } else {
        hits as f64 / total as f64
    }
}
```

Expected cache hit rate: > 95% in production (with 60s TTL and typical update frequency)

## Performance Targets

| Operation | Target | Typical |
|-----------|--------|---------|
| Availability Check | < 100ms | ~10-20ms (cached) |
| Pricing Calculation | < 100ms | ~30-50ms (cached) |
| Prep Time Estimate | < 100ms | ~5-10ms (cached) |
| Loyalty Calculation | < 100ms | ~10-20ms |
| Cache Hit Rate | > 90% | ~95-98% |

## Monitoring in Production

### Metrics Endpoint

Monitor performance via `GET /api/business-rules/metrics`

### Log Analysis

Slow operations are automatically logged:
```
WARN Slow pricing calculation: 125ms
```

Search logs for "Slow" to identify performance issues.

### Cache Effectiveness

Monitor cache hit rate. If < 90%:
- Consider increasing TTL (if staleness is acceptable)
- Check if configuration is being updated too frequently
- Verify cache warming is working on startup

## Future Optimizations

If performance targets are not met:

1. **Reduce Cache TTL Granularity**: Cache individual coffee availability instead of all at once
2. **Add Query Result Caching**: Cache pricing rule evaluation results for common item combinations
3. **Implement Read-Through Cache**: Load individual items on demand instead of all at once
4. **Add Connection Pooling Tuning**: Adjust sqlx pool size based on load
5. **Consider Redis**: For distributed deployments, use Redis for shared cache

## Testing Performance

Performance tests should verify:
- Rule evaluation completes in < 100ms for orders with 1, 5, 10 items
- Cache hit rate > 90% after warm-up period
- No memory leaks in metrics collection
- Concurrent requests don't cause lock contention

See task 12.4 for test implementation details.
