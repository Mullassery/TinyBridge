use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Graceful shutdown coordinator
pub struct ShutdownCoordinator {
    shutdown_signal: broadcast::Sender<()>,
    is_shutting_down: Arc<AtomicBool>,
    active_operations: Arc<std::sync::atomic::AtomicUsize>,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1);
        ShutdownCoordinator {
            shutdown_signal: tx,
            is_shutting_down: Arc::new(AtomicBool::new(false)),
            active_operations: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Get a receiver for shutdown signals
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_signal.subscribe()
    }

    /// Increment active operation counter
    pub fn increment_operations(&self) {
        self.active_operations
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Decrement active operation counter
    pub fn decrement_operations(&self) {
        self.active_operations
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// Get current count of active operations
    pub fn active_count(&self) -> usize {
        self.active_operations
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Check if shutdown has been initiated
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::SeqCst)
    }

    /// Initiate graceful shutdown
    pub fn initiate_shutdown(&self) {
        info!("Initiating graceful shutdown");
        self.is_shutting_down.store(true, Ordering::SeqCst);

        // Send shutdown signal to all subscribers
        let _ = self.shutdown_signal.send(());
    }

    /// Wait for all operations to complete (with timeout)
    pub async fn wait_for_operations(&self, timeout_secs: u64) {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            let count = self.active_count();
            if count == 0 {
                info!("All operations completed");
                break;
            }

            if start.elapsed() > timeout {
                warn!(
                    "Timeout waiting for operations to complete ({} still active)",
                    count
                );
                break;
            }

            // Wait a bit before checking again
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    /// Perform cleanup operations
    pub async fn cleanup(&self) {
        info!("Performing cleanup operations");
        // Future: close database connections, flush caches, etc.
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ShutdownCoordinator {
    fn clone(&self) -> Self {
        ShutdownCoordinator {
            shutdown_signal: self.shutdown_signal.clone(),
            is_shutting_down: Arc::clone(&self.is_shutting_down),
            active_operations: Arc::clone(&self.active_operations),
        }
    }
}

/// Scope guard that tracks operation lifetime
pub struct OperationGuard {
    coordinator: ShutdownCoordinator,
}

impl OperationGuard {
    /// Create a new operation guard (auto-increments counter)
    pub fn new(coordinator: &ShutdownCoordinator) -> Self {
        coordinator.increment_operations();
        OperationGuard {
            coordinator: coordinator.clone(),
        }
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        self.coordinator.decrement_operations();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_coordinator_creation() {
        let coord = ShutdownCoordinator::new();
        assert!(!coord.is_shutting_down());
        assert_eq!(coord.active_count(), 0);
    }

    #[test]
    fn test_operation_counting() {
        let coord = ShutdownCoordinator::new();
        coord.increment_operations();
        assert_eq!(coord.active_count(), 1);

        coord.increment_operations();
        assert_eq!(coord.active_count(), 2);

        coord.decrement_operations();
        assert_eq!(coord.active_count(), 1);
    }

    #[test]
    fn test_shutdown_initiation() {
        let coord = ShutdownCoordinator::new();
        assert!(!coord.is_shutting_down());

        coord.initiate_shutdown();
        assert!(coord.is_shutting_down());
    }

    #[test]
    fn test_operation_guard() {
        let coord = ShutdownCoordinator::new();
        assert_eq!(coord.active_count(), 0);

        {
            let _guard = OperationGuard::new(&coord);
            assert_eq!(coord.active_count(), 1);

            {
                let _guard2 = OperationGuard::new(&coord);
                assert_eq!(coord.active_count(), 2);
            }

            assert_eq!(coord.active_count(), 1);
        }

        assert_eq!(coord.active_count(), 0);
    }

    #[test]
    fn test_shutdown_coordinator_clone() {
        let coord = ShutdownCoordinator::new();
        let coord2 = coord.clone();

        coord.increment_operations();
        assert_eq!(coord2.active_count(), 1);

        coord2.initiate_shutdown();
        assert!(coord.is_shutting_down());
    }

    #[tokio::test]
    async fn test_wait_for_operations() {
        let coord = ShutdownCoordinator::new();
        coord.increment_operations();

        // Spawn a task to decrement after a delay
        let coord_clone = coord.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            coord_clone.decrement_operations();
        });

        // Wait for operations with a 2-second timeout
        coord.wait_for_operations(2).await;
        assert_eq!(coord.active_count(), 0);
    }

    #[tokio::test]
    async fn test_wait_for_operations_timeout() {
        let coord = ShutdownCoordinator::new();
        coord.increment_operations();

        // Don't decrement, just wait with short timeout
        coord.wait_for_operations(0).await;
        // Should timeout and log warning
        assert_eq!(coord.active_count(), 1);
    }

    #[test]
    fn test_shutdown_subscribe() {
        let coord = ShutdownCoordinator::new();
        let rx = coord.subscribe();

        // Send shutdown signal
        coord.initiate_shutdown();

        // Subscriber should receive signal (in real async context)
        assert!(coord.is_shutting_down());
    }
}
