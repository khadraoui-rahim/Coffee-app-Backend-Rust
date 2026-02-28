// Performance Metrics for Business Rules System
//
// Tracks execution times, cache hit rates, and slow operations
// to help identify performance bottlenecks and optimization opportunities.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance threshold for slow operations (100ms)
const SLOW_OPERATION_THRESHOLD_MS: u64 = 100;

/// Performance metrics for the business rules system
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    inner: Arc<MetricsInner>,
}

#[derive(Debug)]
struct MetricsInner {
    // Cache metrics
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    
    // Operation counts
    availability_checks: AtomicU64,
    pricing_calculations: AtomicU64,
    prep_time_estimates: AtomicU64,
    loyalty_calculations: AtomicU64,
    
    // Timing metrics (in microseconds)
    total_availability_time_us: AtomicU64,
    total_pricing_time_us: AtomicU64,
    total_prep_time_us: AtomicU64,
    total_loyalty_time_us: AtomicU64,
    
    // Slow operation counts
    slow_availability_checks: AtomicU64,
    slow_pricing_calculations: AtomicU64,
    slow_prep_time_estimates: AtomicU64,
    slow_loyalty_calculations: AtomicU64,
}

impl PerformanceMetrics {
    /// Create a new PerformanceMetrics instance
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                cache_hits: AtomicU64::new(0),
                cache_misses: AtomicU64::new(0),
                availability_checks: AtomicU64::new(0),
                pricing_calculations: AtomicU64::new(0),
                prep_time_estimates: AtomicU64::new(0),
                loyalty_calculations: AtomicU64::new(0),
                total_availability_time_us: AtomicU64::new(0),
                total_pricing_time_us: AtomicU64::new(0),
                total_prep_time_us: AtomicU64::new(0),
                total_loyalty_time_us: AtomicU64::new(0),
                slow_availability_checks: AtomicU64::new(0),
                slow_pricing_calculations: AtomicU64::new(0),
                slow_prep_time_estimates: AtomicU64::new(0),
                slow_loyalty_calculations: AtomicU64::new(0),
            }),
        }
    }
    
    /// Record a cache hit
    pub fn record_cache_hit(&self) {
        self.inner.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Record a cache miss
    pub fn record_cache_miss(&self) {
        self.inner.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get cache hit rate (0.0 to 1.0)
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
    
    /// Start timing an availability check
    pub fn start_availability_check(&self) -> OperationTimer {
        OperationTimer::new(OperationType::Availability, self.clone())
    }
    
    /// Start timing a pricing calculation
    pub fn start_pricing_calculation(&self) -> OperationTimer {
        OperationTimer::new(OperationType::Pricing, self.clone())
    }
    
    /// Start timing a prep time estimate
    pub fn start_prep_time_estimate(&self) -> OperationTimer {
        OperationTimer::new(OperationType::PrepTime, self.clone())
    }
    
    /// Start timing a loyalty calculation
    pub fn start_loyalty_calculation(&self) -> OperationTimer {
        OperationTimer::new(OperationType::Loyalty, self.clone())
    }
    
    /// Record an availability check completion
    fn record_availability_check(&self, duration: Duration) {
        self.inner.availability_checks.fetch_add(1, Ordering::Relaxed);
        self.inner.total_availability_time_us.fetch_add(
            duration.as_micros() as u64,
            Ordering::Relaxed,
        );
        
        if duration.as_millis() as u64 > SLOW_OPERATION_THRESHOLD_MS {
            self.inner.slow_availability_checks.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                "Slow availability check: {}ms",
                duration.as_millis()
            );
        }
    }
    
    /// Record a pricing calculation completion
    fn record_pricing_calculation(&self, duration: Duration) {
        self.inner.pricing_calculations.fetch_add(1, Ordering::Relaxed);
        self.inner.total_pricing_time_us.fetch_add(
            duration.as_micros() as u64,
            Ordering::Relaxed,
        );
        
        if duration.as_millis() as u64 > SLOW_OPERATION_THRESHOLD_MS {
            self.inner.slow_pricing_calculations.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                "Slow pricing calculation: {}ms",
                duration.as_millis()
            );
        }
    }
    
    /// Record a prep time estimate completion
    fn record_prep_time_estimate(&self, duration: Duration) {
        self.inner.prep_time_estimates.fetch_add(1, Ordering::Relaxed);
        self.inner.total_prep_time_us.fetch_add(
            duration.as_micros() as u64,
            Ordering::Relaxed,
        );
        
        if duration.as_millis() as u64 > SLOW_OPERATION_THRESHOLD_MS {
            self.inner.slow_prep_time_estimates.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                "Slow prep time estimate: {}ms",
                duration.as_millis()
            );
        }
    }
    
    /// Record a loyalty calculation completion
    fn record_loyalty_calculation(&self, duration: Duration) {
        self.inner.loyalty_calculations.fetch_add(1, Ordering::Relaxed);
        self.inner.total_loyalty_time_us.fetch_add(
            duration.as_micros() as u64,
            Ordering::Relaxed,
        );
        
        if duration.as_millis() as u64 > SLOW_OPERATION_THRESHOLD_MS {
            self.inner.slow_loyalty_calculations.fetch_add(1, Ordering::Relaxed);
            tracing::warn!(
                "Slow loyalty calculation: {}ms",
                duration.as_millis()
            );
        }
    }
    
    /// Get average availability check time in milliseconds
    pub fn avg_availability_time_ms(&self) -> f64 {
        let count = self.inner.availability_checks.load(Ordering::Relaxed);
        let total_us = self.inner.total_availability_time_us.load(Ordering::Relaxed);
        
        if count == 0 {
            0.0
        } else {
            (total_us as f64 / count as f64) / 1000.0
        }
    }
    
    /// Get average pricing calculation time in milliseconds
    pub fn avg_pricing_time_ms(&self) -> f64 {
        let count = self.inner.pricing_calculations.load(Ordering::Relaxed);
        let total_us = self.inner.total_pricing_time_us.load(Ordering::Relaxed);
        
        if count == 0 {
            0.0
        } else {
            (total_us as f64 / count as f64) / 1000.0
        }
    }
    
    /// Get average prep time estimate time in milliseconds
    pub fn avg_prep_time_ms(&self) -> f64 {
        let count = self.inner.prep_time_estimates.load(Ordering::Relaxed);
        let total_us = self.inner.total_prep_time_us.load(Ordering::Relaxed);
        
        if count == 0 {
            0.0
        } else {
            (total_us as f64 / count as f64) / 1000.0
        }
    }
    
    /// Get average loyalty calculation time in milliseconds
    pub fn avg_loyalty_time_ms(&self) -> f64 {
        let count = self.inner.loyalty_calculations.load(Ordering::Relaxed);
        let total_us = self.inner.total_loyalty_time_us.load(Ordering::Relaxed);
        
        if count == 0 {
            0.0
        } else {
            (total_us as f64 / count as f64) / 1000.0
        }
    }
    
    /// Get metrics summary
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            cache_hit_rate: self.cache_hit_rate(),
            cache_hits: self.inner.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.inner.cache_misses.load(Ordering::Relaxed),
            availability_checks: self.inner.availability_checks.load(Ordering::Relaxed),
            avg_availability_time_ms: self.avg_availability_time_ms(),
            slow_availability_checks: self.inner.slow_availability_checks.load(Ordering::Relaxed),
            pricing_calculations: self.inner.pricing_calculations.load(Ordering::Relaxed),
            avg_pricing_time_ms: self.avg_pricing_time_ms(),
            slow_pricing_calculations: self.inner.slow_pricing_calculations.load(Ordering::Relaxed),
            prep_time_estimates: self.inner.prep_time_estimates.load(Ordering::Relaxed),
            avg_prep_time_ms: self.avg_prep_time_ms(),
            slow_prep_time_estimates: self.inner.slow_prep_time_estimates.load(Ordering::Relaxed),
            loyalty_calculations: self.inner.loyalty_calculations.load(Ordering::Relaxed),
            avg_loyalty_time_ms: self.avg_loyalty_time_ms(),
            slow_loyalty_calculations: self.inner.slow_loyalty_calculations.load(Ordering::Relaxed),
        }
    }
    
    /// Log metrics summary
    pub fn log_summary(&self) {
        let summary = self.summary();
        tracing::info!(
            "Business Rules Performance Metrics:\n\
             Cache: {:.1}% hit rate ({} hits, {} misses)\n\
             Availability: {} checks, avg {:.2}ms, {} slow\n\
             Pricing: {} calculations, avg {:.2}ms, {} slow\n\
             Prep Time: {} estimates, avg {:.2}ms, {} slow\n\
             Loyalty: {} calculations, avg {:.2}ms, {} slow",
            summary.cache_hit_rate * 100.0,
            summary.cache_hits,
            summary.cache_misses,
            summary.availability_checks,
            summary.avg_availability_time_ms,
            summary.slow_availability_checks,
            summary.pricing_calculations,
            summary.avg_pricing_time_ms,
            summary.slow_pricing_calculations,
            summary.prep_time_estimates,
            summary.avg_prep_time_ms,
            summary.slow_prep_time_estimates,
            summary.loyalty_calculations,
            summary.avg_loyalty_time_ms,
            summary.slow_loyalty_calculations,
        );
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of operation being timed
#[derive(Debug, Clone, Copy)]
enum OperationType {
    Availability,
    Pricing,
    PrepTime,
    Loyalty,
}

/// Timer for tracking operation duration
pub struct OperationTimer {
    start: Instant,
    operation_type: OperationType,
    metrics: PerformanceMetrics,
}

impl OperationTimer {
    fn new(operation_type: OperationType, metrics: PerformanceMetrics) -> Self {
        Self {
            start: Instant::now(),
            operation_type,
            metrics,
        }
    }
    
    /// Complete the timer and record the duration
    pub fn complete(self) {
        let duration = self.start.elapsed();
        
        match self.operation_type {
            OperationType::Availability => self.metrics.record_availability_check(duration),
            OperationType::Pricing => self.metrics.record_pricing_calculation(duration),
            OperationType::PrepTime => self.metrics.record_prep_time_estimate(duration),
            OperationType::Loyalty => self.metrics.record_loyalty_calculation(duration),
        }
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        // Auto-complete if not explicitly completed
        let duration = self.start.elapsed();
        
        match self.operation_type {
            OperationType::Availability => self.metrics.record_availability_check(duration),
            OperationType::Pricing => self.metrics.record_pricing_calculation(duration),
            OperationType::PrepTime => self.metrics.record_prep_time_estimate(duration),
            OperationType::Loyalty => self.metrics.record_loyalty_calculation(duration),
        }
    }
}

/// Summary of performance metrics
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub cache_hit_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub availability_checks: u64,
    pub avg_availability_time_ms: f64,
    pub slow_availability_checks: u64,
    pub pricing_calculations: u64,
    pub avg_pricing_time_ms: f64,
    pub slow_pricing_calculations: u64,
    pub prep_time_estimates: u64,
    pub avg_prep_time_ms: f64,
    pub slow_prep_time_estimates: u64,
    pub loyalty_calculations: u64,
    pub avg_loyalty_time_ms: f64,
    pub slow_loyalty_calculations: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_metrics_creation() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.cache_hit_rate(), 0.0);
        assert_eq!(metrics.avg_availability_time_ms(), 0.0);
    }
    
    #[test]
    fn test_cache_metrics() {
        let metrics = PerformanceMetrics::new();
        
        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();
        
        assert_eq!(metrics.cache_hit_rate(), 2.0 / 3.0);
    }
    
    #[test]
    fn test_operation_timer() {
        let metrics = PerformanceMetrics::new();
        
        {
            let _timer = metrics.start_availability_check();
            thread::sleep(Duration::from_millis(10));
        }
        
        let summary = metrics.summary();
        assert_eq!(summary.availability_checks, 1);
        assert!(summary.avg_availability_time_ms >= 10.0);
    }
    
    #[test]
    fn test_slow_operation_detection() {
        let metrics = PerformanceMetrics::new();
        
        {
            let _timer = metrics.start_pricing_calculation();
            thread::sleep(Duration::from_millis(150));
        }
        
        let summary = metrics.summary();
        assert_eq!(summary.slow_pricing_calculations, 1);
    }
}
