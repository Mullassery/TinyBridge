use crate::ExecutionTier;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Performance metrics for a routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    /// Command or binary name
    pub name: String,

    /// Tier it was routed to
    pub tier: ExecutionTier,

    /// Time spent in routing decision (microseconds)
    pub routing_decision_us: u64,

    /// Execution time (milliseconds)
    pub execution_time_ms: u64,

    /// Memory used (MB)
    pub memory_used_mb: Option<f64>,

    /// CPU usage percentage
    pub cpu_usage_percent: Option<f32>,

    /// Success/failure status
    pub success: bool,

    /// Additional notes
    pub notes: Option<String>,
}

/// Profiler for tracking routing and execution performance
pub struct RoutingProfiler {
    metrics: Vec<RoutingMetrics>,
    max_metrics: usize,
}

impl RoutingProfiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            max_metrics: 10000,
        }
    }

    /// Record a routing decision
    pub fn record(&mut self, metric: RoutingMetrics) {
        self.metrics.push(metric);

        // Keep only recent metrics
        if self.metrics.len() > self.max_metrics {
            self.metrics.remove(0);
        }
    }

    /// Get average routing decision time
    pub fn avg_routing_decision_us(&self) -> u64 {
        if self.metrics.is_empty() {
            return 0;
        }

        let sum: u64 = self.metrics.iter().map(|m| m.routing_decision_us).sum();
        sum / self.metrics.len() as u64
    }

    /// Get average execution time for a tier
    pub fn avg_execution_time_for_tier(&self, tier: ExecutionTier) -> u64 {
        let relevant: Vec<_> = self.metrics.iter().filter(|m| m.tier == tier).collect();

        if relevant.is_empty() {
            return 0;
        }

        let sum: u64 = relevant.iter().map(|m| m.execution_time_ms).sum();
        sum / relevant.len() as u64
    }

    /// Get success rate
    pub fn success_rate(&self) -> f32 {
        if self.metrics.is_empty() {
            return 0.0;
        }

        let successes = self.metrics.iter().filter(|m| m.success).count();
        (successes as f32) / (self.metrics.len() as f32)
    }

    /// Get metrics summary
    pub fn summary(&self) -> Summary {
        Summary {
            total_routed: self.metrics.len(),
            avg_routing_decision_us: self.avg_routing_decision_us(),
            tier_stats: self.tier_stats(),
            success_rate: self.success_rate(),
        }
    }

    fn tier_stats(&self) -> Vec<TierStat> {
        let mut stats = Vec::new();

        for tier in [
            ExecutionTier::Native,
            ExecutionTier::Linux,
            ExecutionTier::Remote,
        ] {
            let relevant: Vec<_> = self.metrics.iter().filter(|m| m.tier == tier).collect();

            if !relevant.is_empty() {
                let avg_time: u64 = relevant.iter().map(|m| m.execution_time_ms).sum::<u64>()
                    / relevant.len() as u64;

                stats.push(TierStat {
                    tier,
                    count: relevant.len(),
                    avg_time_ms: avg_time,
                });
            }
        }

        stats
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for RoutingProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub total_routed: usize,
    pub avg_routing_decision_us: u64,
    pub tier_stats: Vec<TierStat>,
    pub success_rate: f32,
}

/// Per-tier statistics
#[derive(Debug, Clone, Serialize)]
pub struct TierStat {
    pub tier: ExecutionTier,
    pub count: usize,
    pub avg_time_ms: u64,
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in microseconds
    pub fn elapsed_us(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Get elapsed time as Duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = RoutingProfiler::new();
        assert_eq!(profiler.metrics.len(), 0);
    }

    #[test]
    fn test_record_metric() {
        let mut profiler = RoutingProfiler::new();
        let metric = RoutingMetrics {
            name: "test".to_string(),
            tier: ExecutionTier::Linux,
            routing_decision_us: 100,
            execution_time_ms: 50,
            memory_used_mb: Some(256.0),
            cpu_usage_percent: Some(50.0),
            success: true,
            notes: None,
        };

        profiler.record(metric);
        assert_eq!(profiler.metrics.len(), 1);
    }

    #[test]
    fn test_success_rate() {
        let mut profiler = RoutingProfiler::new();

        for i in 0..10 {
            profiler.record(RoutingMetrics {
                name: format!("test-{}", i),
                tier: ExecutionTier::Linux,
                routing_decision_us: 100,
                execution_time_ms: 50,
                memory_used_mb: None,
                cpu_usage_percent: None,
                success: i < 8,
                notes: None,
            });
        }

        assert_eq!(profiler.success_rate(), 0.8);
    }

    #[test]
    fn test_timer() {
        let timer = Timer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed_ms = timer.elapsed_ms();
        assert!(elapsed_ms >= 10);
    }
}
