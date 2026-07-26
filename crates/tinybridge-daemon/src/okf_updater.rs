use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::VecDeque;
use std::path::PathBuf;

/// Production metrics collected from OTel
#[derive(Debug, Clone)]
pub struct ProductionMetrics {
    pub timestamp: DateTime<Utc>,
    pub env_name: String,

    // Boot metrics
    pub boot_time_ms: u64,
    pub boot_tier: u8,

    // Resource metrics
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
    pub disk_usage_percent: f32,

    // Quality metrics
    pub error_rate: f32,
    pub availability_percent: f32,
}

/// Historical window of metrics (last N measurements)
#[derive(Debug, Clone)]
pub struct MetricsWindow {
    pub samples: VecDeque<ProductionMetrics>,
    pub window_size: usize,
}

impl MetricsWindow {
    pub fn new(window_size: usize) -> Self {
        MetricsWindow {
            samples: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    pub fn add_sample(&mut self, metric: ProductionMetrics) {
        if self.samples.len() >= self.window_size {
            self.samples.pop_front();
        }
        self.samples.push_back(metric);
    }

    pub fn average_boot_time(&self) -> Option<u64> {
        if self.samples.is_empty() {
            return None;
        }
        let sum: u64 = self.samples.iter().map(|m| m.boot_time_ms).sum();
        Some(sum / self.samples.len() as u64)
    }

    pub fn average_cpu_usage(&self) -> Option<f32> {
        if self.samples.is_empty() {
            return None;
        }
        let sum: f32 = self.samples.iter().map(|m| m.cpu_usage_percent).sum();
        Some(sum / self.samples.len() as f32)
    }

    pub fn trend(&self, getter: impl Fn(&ProductionMetrics) -> f32) -> Option<String> {
        if self.samples.len() < 2 {
            return None;
        }

        let first = getter(&self.samples[0]);
        let last = getter(&self.samples[self.samples.len() - 1]);
        let delta = last - first;

        if delta.abs() < 0.01 {
            Some("stable".to_string())
        } else if delta > 0.0 {
            Some(format!("degrading (+{:.1}%)", delta))
        } else {
            Some(format!("improving ({:.1}%)", delta))
        }
    }
}

/// OKF file representation (production data snapshot)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OkfSnapshot {
    pub phase: String,
    pub updated_at: DateTime<Utc>,
    pub metrics: ProductionMetrics,
    pub history: Vec<ProductionMetrics>,
    pub status: OkfStatus,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum OkfStatus {
    Healthy,
    Degraded,
    AtRisk,
    Failed,
}

impl OkfSnapshot {
    #[allow(dead_code)]
    pub fn new(phase: String, metrics: ProductionMetrics) -> Self {
        OkfSnapshot {
            phase,
            updated_at: Utc::now(),
            metrics: metrics.clone(),
            history: vec![metrics],
            status: OkfStatus::Healthy,
        }
    }

    #[allow(dead_code)]
    pub fn update(&mut self, metrics: ProductionMetrics) {
        self.updated_at = Utc::now();
        self.metrics = metrics.clone();
        self.history.push(metrics);

        // Keep last 100 samples
        if self.history.len() > 100 {
            self.history.remove(0);
        }

        // Update status based on metrics
        self.status = self.calculate_status();
    }

    #[allow(dead_code)]
    fn calculate_status(&self) -> OkfStatus {
        // Boot time SLO check (4.9s target)
        if self.metrics.boot_time_ms > 5000 {
            return OkfStatus::Failed;
        }

        if self.metrics.boot_time_ms > 4900 {
            return OkfStatus::AtRisk;
        }

        // Resource usage check
        if self.metrics.cpu_usage_percent > 95.0 || self.metrics.memory_usage_percent > 95.0 {
            return OkfStatus::Failed;
        }

        if self.metrics.cpu_usage_percent > 85.0 || self.metrics.memory_usage_percent > 85.0 {
            return OkfStatus::Degraded;
        }

        // Availability check (99.9% target)
        if self.metrics.availability_percent < 99.0 {
            return OkfStatus::Failed;
        }

        if self.metrics.availability_percent < 99.5 {
            return OkfStatus::Degraded;
        }

        OkfStatus::Healthy
    }

    #[allow(dead_code)]
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "phase": self.phase,
            "status": format!("{:?}", self.status),
            "updated_at": self.updated_at.to_rfc3339(),
            "metrics": {
                "boot_time_ms": self.metrics.boot_time_ms,
                "boot_tier": self.metrics.boot_tier,
                "cpu_usage_percent": self.metrics.cpu_usage_percent,
                "memory_usage_percent": self.metrics.memory_usage_percent,
                "disk_usage_percent": self.metrics.disk_usage_percent,
                "error_rate": self.metrics.error_rate,
                "availability_percent": self.metrics.availability_percent,
            },
            "history_samples": self.history.len(),
            "average_boot_time_ms": self.history.iter().map(|m| m.boot_time_ms).sum::<u64>() / self.history.len().max(1) as u64,
        })
    }
}

/// OKF Updater - maintains live production state
#[allow(dead_code)]
pub struct OkfUpdater {
    okf_dir: PathBuf,
    snapshots: std::collections::HashMap<String, OkfSnapshot>,
}

impl OkfUpdater {
    #[allow(dead_code)]
    pub fn new(okf_dir: PathBuf) -> Self {
        OkfUpdater {
            okf_dir,
            snapshots: std::collections::HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn update_from_metrics(&mut self, metrics: ProductionMetrics) -> Result<()> {
        let phase = "okr_phase_1_week_4".to_string();

        let snapshot = self
            .snapshots
            .entry(metrics.env_name.clone())
            .or_insert_with(|| OkfSnapshot::new(phase.clone(), metrics.clone()));

        snapshot.update(metrics);

        tracing::info!(
            env_name = snapshot.metrics.env_name,
            boot_time_ms = snapshot.metrics.boot_time_ms,
            status = ?snapshot.status,
            "OKF snapshot updated"
        );

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_snapshot(&self, env_name: &str) -> Option<&OkfSnapshot> {
        self.snapshots.get(env_name)
    }

    #[allow(dead_code)]
    pub fn all_snapshots(&self) -> Vec<&OkfSnapshot> {
        self.snapshots.values().collect()
    }

    #[allow(dead_code)]
    pub fn export_json(&self) -> serde_json::Value {
        let snapshots: Vec<_> = self.snapshots.values().map(|s| s.to_json()).collect();

        json!({
            "updated_at": Utc::now().to_rfc3339(),
            "snapshots": snapshots,
            "total_environments": self.snapshots.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metrics() -> ProductionMetrics {
        ProductionMetrics {
            timestamp: Utc::now(),
            env_name: "test-env".to_string(),
            boot_time_ms: 4700,
            boot_tier: 1,
            cpu_usage_percent: 45.0,
            memory_usage_percent: 65.0,
            disk_usage_percent: 70.0,
            error_rate: 0.05,
            availability_percent: 99.95,
        }
    }

    #[test]
    fn test_metrics_window() {
        let mut window = MetricsWindow::new(5);
        let metric = create_test_metrics();

        window.add_sample(metric.clone());
        assert_eq!(window.samples.len(), 1);
        assert_eq!(window.average_boot_time(), Some(4700));
    }

    #[test]
    fn test_okf_snapshot_status_healthy() {
        let metrics = create_test_metrics();
        let snapshot = OkfSnapshot::new("phase1".to_string(), metrics);
        assert_eq!(snapshot.status, OkfStatus::Healthy);
    }

    #[test]
    fn test_okf_snapshot_status_at_risk() {
        let mut metrics = create_test_metrics();
        metrics.boot_time_ms = 4950; // Above 4.9s target
        let snapshot = OkfSnapshot::new("phase1".to_string(), metrics);
        assert_eq!(snapshot.status, OkfStatus::AtRisk);
    }

    #[test]
    fn test_okf_snapshot_status_failed() {
        let mut metrics = create_test_metrics();
        metrics.boot_time_ms = 5100; // Over 5s limit
        let snapshot = OkfSnapshot::new("phase1".to_string(), metrics);
        assert_eq!(snapshot.status, OkfStatus::Failed);
    }

    #[test]
    fn test_okf_updater() {
        let mut updater = OkfUpdater::new(PathBuf::from("/tmp/okf"));
        let metrics = create_test_metrics();

        assert!(updater.update_from_metrics(metrics.clone()).is_ok());
        assert!(updater.get_snapshot("test-env").is_some());

        let snapshot = updater.get_snapshot("test-env").unwrap();
        assert_eq!(snapshot.metrics.boot_time_ms, 4700);
    }

    #[test]
    fn test_okf_export_json() {
        let mut updater = OkfUpdater::new(PathBuf::from("/tmp/okf"));
        let metrics = create_test_metrics();

        updater.update_from_metrics(metrics).ok();
        let json = updater.export_json();

        assert!(json["snapshots"].is_array());
        assert!(json["updated_at"].is_string());
    }
}
