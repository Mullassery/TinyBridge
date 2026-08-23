/// Lazy Loading System for Deferred Initialization
/// Phase 4.0.4: Boot Optimization
///
/// Load non-critical components on demand or after boot tiers complete
use crate::boot_stages::BootTier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Loadable component state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadState {
    /// Not yet loaded
    Unloaded,
    /// Currently loading
    Loading,
    /// Loaded successfully
    Loaded,
    /// Failed to load
    Failed,
    /// Deferred (waiting for tier advancement)
    Deferred,
}

impl std::fmt::Display for LoadState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadState::Unloaded => write!(f, "unloaded"),
            LoadState::Loading => write!(f, "loading"),
            LoadState::Loaded => write!(f, "loaded"),
            LoadState::Failed => write!(f, "failed"),
            LoadState::Deferred => write!(f, "deferred"),
        }
    }
}

/// Loadable component descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loadable {
    /// Component name
    pub name: String,
    /// Component description
    pub description: String,
    /// Current load state
    pub state: LoadState,
    /// Load timing priority (0 = highest)
    pub priority: u32,
    /// Load tier (when to load this)
    pub load_tier: BootTier,
    /// Required components (dependencies)
    pub dependencies: Vec<String>,
    /// Module size estimate (bytes)
    pub size_bytes: u64,
    /// Load time estimate (ms)
    pub load_time_ms: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
}

impl Loadable {
    /// Create new loadable component
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        load_tier: BootTier,
    ) -> Self {
        Loadable {
            name: name.into(),
            description: description.into(),
            state: LoadState::Unloaded,
            priority: 100,
            load_tier,
            dependencies: Vec::new(),
            size_bytes: 0,
            load_time_ms: None,
            error: None,
        }
    }

    /// Set priority (lower = earlier)
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add dependency
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Set size estimate
    pub fn with_size(mut self, bytes: u64) -> Self {
        self.size_bytes = bytes;
        self
    }

    /// Mark as loading
    pub fn mark_loading(mut self) -> Self {
        self.state = LoadState::Loading;
        self
    }

    /// Mark as loaded
    pub fn mark_loaded(mut self, time_ms: u64) -> Self {
        self.state = LoadState::Loaded;
        self.load_time_ms = Some(time_ms);
        self.error = None;
        self
    }

    /// Mark as failed
    pub fn mark_failed(mut self, error: impl Into<String>) -> Self {
        self.state = LoadState::Failed;
        self.error = Some(error.into());
        self
    }

    /// Defer loading
    pub fn defer(mut self) -> Self {
        self.state = LoadState::Deferred;
        self
    }

    /// Check if ready to load (all dependencies loaded)
    pub fn can_load(&self, loaded: &HashMap<String, Loadable>) -> bool {
        self.state == LoadState::Unloaded
            || (self.state == LoadState::Deferred)
                && self.dependencies.iter().all(|dep| {
                    loaded
                        .get(dep)
                        .map(|c| c.state == LoadState::Loaded)
                        .unwrap_or(false)
                })
    }
}

/// Lazy load scheduler
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LazyLoadScheduler {
    /// Registered loadables
    pub components: HashMap<String, Loadable>,
    /// Current tier
    pub current_tier: Option<BootTier>,
}

impl LazyLoadScheduler {
    /// Create new scheduler
    pub fn new() -> Self {
        LazyLoadScheduler {
            components: HashMap::new(),
            current_tier: None,
        }
    }

    /// Register a loadable component
    pub fn register(&mut self, component: Loadable) {
        self.components.insert(component.name.clone(), component);
    }

    /// Register multiple components
    pub fn register_batch(&mut self, components: Vec<Loadable>) {
        for comp in components {
            self.register(comp);
        }
    }

    /// Advance to next tier
    pub fn advance_tier(&mut self, tier: BootTier) {
        self.current_tier = Some(tier);
    }

    /// Get components ready to load for current tier
    pub fn ready_to_load(&self) -> Vec<String> {
        let Some(tier) = self.current_tier else {
            return Vec::new();
        };

        self.components
            .iter()
            .filter(|(_, comp)| {
                (comp.load_tier <= tier)
                    && comp.can_load(&self.components)
                    && comp.state == LoadState::Deferred
            })
            .map(|(name, comp)| (name.clone(), comp.priority))
            .collect::<Vec<_>>()
            .sort_by_key(|(_, priority)| *priority);

        self.components
            .iter()
            .filter(|(_, comp)| {
                (comp.load_tier <= tier)
                    && comp.can_load(&self.components)
                    && comp.state == LoadState::Unloaded
            })
            .map(|(name, comp)| (name.clone(), comp.priority))
            .collect::<Vec<_>>()
            .sort_by_key(|(_, priority)| *priority);

        self.components
            .iter()
            .filter(|(_, comp)| (comp.load_tier <= tier) && comp.can_load(&self.components))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get deferred components
    pub fn deferred(&self) -> Vec<String> {
        self.components
            .iter()
            .filter(|(_, comp)| comp.state == LoadState::Deferred)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Mark component as loaded
    pub fn mark_loaded(&mut self, name: &str, time_ms: u64) -> Result<(), String> {
        let comp = self
            .components
            .get_mut(name)
            .ok_or_else(|| format!("Component not found: {}", name))?;
        comp.state = LoadState::Loaded;
        comp.load_time_ms = Some(time_ms);
        comp.error = None;
        Ok(())
    }

    /// Mark component as failed
    pub fn mark_failed(&mut self, name: &str, error: impl Into<String>) -> Result<(), String> {
        let comp = self
            .components
            .get_mut(name)
            .ok_or_else(|| format!("Component not found: {}", name))?;
        comp.state = LoadState::Failed;
        comp.error = Some(error.into());
        Ok(())
    }

    /// Get load summary
    pub fn summary(&self) -> LoaderSummary {
        let total = self.components.len();
        let loaded = self
            .components
            .values()
            .filter(|c| c.state == LoadState::Loaded)
            .count();
        let failed = self
            .components
            .values()
            .filter(|c| c.state == LoadState::Failed)
            .count();
        let deferred = self
            .components
            .values()
            .filter(|c| c.state == LoadState::Deferred)
            .count();

        let total_size = self.components.values().map(|c| c.size_bytes).sum();
        let load_time: u64 = self
            .components
            .values()
            .filter_map(|c| c.load_time_ms)
            .sum();

        LoaderSummary {
            total_components: total,
            loaded_components: loaded,
            failed_components: failed,
            deferred_components: deferred,
            total_size_bytes: total_size,
            total_load_time_ms: load_time,
            current_tier: self.current_tier,
        }
    }

    /// Get readiness percentage
    pub fn readiness_percent(&self) -> u32 {
        let total = self.components.len();
        if total == 0 {
            return 0;
        }
        let loaded = self
            .components
            .values()
            .filter(|c| c.state == LoadState::Loaded)
            .count();
        ((loaded as f64 / total as f64) * 100.0) as u32
    }
}

/// Loader summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderSummary {
    /// Total components registered
    pub total_components: usize,
    /// Components successfully loaded
    pub loaded_components: usize,
    /// Components that failed
    pub failed_components: usize,
    /// Components deferred for later
    pub deferred_components: usize,
    /// Total size of loaded components
    pub total_size_bytes: u64,
    /// Total time spent loading
    pub total_load_time_ms: u64,
    /// Current boot tier
    pub current_tier: Option<BootTier>,
}

impl LoaderSummary {
    /// All critical components loaded
    pub fn all_critical_loaded(&self) -> bool {
        self.failed_components == 0 && self.deferred_components == 0
    }

    /// Get memory usage in MB
    pub fn total_size_mb(&self) -> f64 {
        self.total_size_bytes as f64 / (1024.0 * 1024.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loadable_creation() {
        let loadable = Loadable::new("test_module", "Test module", BootTier::Api);
        assert_eq!(loadable.name, "test_module");
        assert_eq!(loadable.state, LoadState::Unloaded);
        assert_eq!(loadable.load_tier, BootTier::Api);
    }

    #[test]
    fn test_loadable_with_priority() {
        let loadable = Loadable::new("module", "desc", BootTier::Usable).with_priority(10);
        assert_eq!(loadable.priority, 10);
    }

    #[test]
    fn test_loadable_with_dependency() {
        let loadable = Loadable::new("module", "desc", BootTier::Api)
            .with_dependency("dep1")
            .with_dependency("dep2");
        assert_eq!(loadable.dependencies.len(), 2);
    }

    #[test]
    fn test_loadable_mark_loaded() {
        let loadable = Loadable::new("module", "desc", BootTier::Usable).mark_loaded(150);
        assert_eq!(loadable.state, LoadState::Loaded);
        assert_eq!(loadable.load_time_ms, Some(150));
    }

    #[test]
    fn test_loadable_mark_failed() {
        let loadable =
            Loadable::new("module", "desc", BootTier::Api).mark_failed("connection error");
        assert_eq!(loadable.state, LoadState::Failed);
        assert_eq!(loadable.error, Some("connection error".to_string()));
    }

    #[test]
    fn test_lazy_load_scheduler_register() {
        let mut scheduler = LazyLoadScheduler::new();
        let loadable = Loadable::new("module", "desc", BootTier::Api);
        scheduler.register(loadable);
        assert_eq!(scheduler.components.len(), 1);
    }

    #[test]
    fn test_lazy_load_scheduler_register_batch() {
        let mut scheduler = LazyLoadScheduler::new();
        let components = vec![
            Loadable::new("mod1", "desc", BootTier::Api),
            Loadable::new("mod2", "desc", BootTier::Full),
        ];
        scheduler.register_batch(components);
        assert_eq!(scheduler.components.len(), 2);
    }

    #[test]
    fn test_lazy_load_scheduler_advance_tier() {
        let mut scheduler = LazyLoadScheduler::new();
        scheduler.advance_tier(BootTier::Usable);
        assert_eq!(scheduler.current_tier, Some(BootTier::Usable));
    }

    #[test]
    fn test_lazy_load_scheduler_deferred() {
        let mut scheduler = LazyLoadScheduler::new();
        let mut comp1 = Loadable::new("mod1", "desc", BootTier::Api);
        comp1.state = LoadState::Deferred;
        scheduler.register(comp1);
        assert_eq!(scheduler.deferred().len(), 1);
    }

    #[test]
    fn test_lazy_load_scheduler_mark_loaded() {
        let mut scheduler = LazyLoadScheduler::new();
        scheduler.register(Loadable::new("module", "desc", BootTier::Api));
        assert!(scheduler.mark_loaded("module", 100).is_ok());
        assert_eq!(scheduler.components["module"].state, LoadState::Loaded);
    }

    #[test]
    fn test_lazy_load_scheduler_mark_failed() {
        let mut scheduler = LazyLoadScheduler::new();
        scheduler.register(Loadable::new("module", "desc", BootTier::Api));
        assert!(scheduler.mark_failed("module", "error").is_ok());
        assert_eq!(scheduler.components["module"].state, LoadState::Failed);
    }

    #[test]
    fn test_loader_summary_all_critical_loaded() {
        let summary = LoaderSummary {
            total_components: 5,
            loaded_components: 5,
            failed_components: 0,
            deferred_components: 0,
            total_size_bytes: 1024 * 1024,
            total_load_time_ms: 500,
            current_tier: Some(BootTier::Full),
        };
        assert!(summary.all_critical_loaded());
    }

    #[test]
    fn test_loader_summary_size_mb() {
        let summary = LoaderSummary {
            total_components: 1,
            loaded_components: 1,
            failed_components: 0,
            deferred_components: 0,
            total_size_bytes: 2 * 1024 * 1024,
            total_load_time_ms: 100,
            current_tier: Some(BootTier::Full),
        };
        assert_eq!(summary.total_size_mb(), 2.0);
    }

    #[test]
    fn test_lazy_load_scheduler_readiness() {
        let mut scheduler = LazyLoadScheduler::new();
        let mut comp = Loadable::new("module", "desc", BootTier::Api);
        comp.state = LoadState::Loaded;
        scheduler.register(comp);
        assert_eq!(scheduler.readiness_percent(), 100);
    }

    #[test]
    fn test_lazy_load_scheduler_readiness_partial() {
        let mut scheduler = LazyLoadScheduler::new();
        let mut comp1 = Loadable::new("module1", "desc", BootTier::Api);
        comp1.state = LoadState::Loaded;
        scheduler.register(comp1);
        scheduler.register(Loadable::new("module2", "desc", BootTier::Full));
        assert_eq!(scheduler.readiness_percent(), 50);
    }
}
