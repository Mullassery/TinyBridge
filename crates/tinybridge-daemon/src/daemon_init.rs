use crate::graceful_shutdown::ShutdownCoordinator;
use crate::health::HealthChecker;
use crate::structured_logging;
use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

/// Daemon initialization state
pub struct DaemonState {
    pub shutdown_coordinator: ShutdownCoordinator,
    pub health_checker: HealthChecker,
}

impl DaemonState {
    /// Initialize daemon state and signal handlers
    pub async fn initialize() -> Result<Self> {
        info!("Initializing daemon state");

        // Initialize structured logging
        structured_logging::init_structured_logging();

        // Create shutdown coordinator
        let shutdown_coordinator = ShutdownCoordinator::new();
        let shutdown_clone = shutdown_coordinator.clone();

        // Create health checker
        let health_checker = HealthChecker::new();

        // Install signal handlers (UNIX only)
        #[cfg(unix)]
        {
            install_signal_handlers(shutdown_clone).await?;
        }

        info!("Daemon initialization complete");

        Ok(DaemonState {
            shutdown_coordinator,
            health_checker,
        })
    }

    /// Check if shutdown is in progress
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown_coordinator.is_shutting_down()
    }

    /// Initiate graceful shutdown
    pub fn initiate_shutdown(&self) {
        self.shutdown_coordinator.initiate_shutdown();
    }

    /// Wait for active operations to complete
    pub async fn wait_for_operations(&self, timeout_secs: u64) {
        self.shutdown_coordinator
            .wait_for_operations(timeout_secs)
            .await;
    }

    /// Perform final cleanup
    pub async fn cleanup(&self) {
        self.shutdown_coordinator.cleanup().await;
    }
}

/// Install signal handlers for graceful shutdown (UNIX)
#[cfg(unix)]
async fn install_signal_handlers(shutdown: ShutdownCoordinator) -> Result<()> {
    use tokio::signal::unix::{signal, SignalKind};

    // Handle SIGTERM
    let mut sigterm = signal(SignalKind::terminate())?;
    let shutdown_sigterm = shutdown.clone();
    tokio::spawn(async move {
        sigterm.recv().await;
        info!("Received SIGTERM, initiating graceful shutdown");
        shutdown_sigterm.initiate_shutdown();
    });

    // Handle SIGINT (Ctrl-C)
    let mut sigint = signal(SignalKind::interrupt())?;
    let shutdown_sigint = shutdown.clone();
    tokio::spawn(async move {
        sigint.recv().await;
        info!("Received SIGINT, initiating graceful shutdown");
        shutdown_sigint.initiate_shutdown();
    });

    // Handle SIGHUP (reload config)
    let mut sighup = signal(SignalKind::hangup())?;
    tokio::spawn(async move {
        sighup.recv().await;
        info!("Received SIGHUP, reloading configuration");
        // Future: implement config reload
    });

    info!("Signal handlers installed");
    Ok(())
}

/// Stub for non-UNIX platforms
#[cfg(not(unix))]
async fn install_signal_handlers(_shutdown: ShutdownCoordinator) -> Result<()> {
    info!("Signal handlers not available on this platform");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_daemon_state_creation() {
        let state = DaemonState::initialize().await.unwrap();
        assert!(!state.is_shutting_down());
    }

    #[tokio::test]
    async fn test_daemon_shutdown_initiation() {
        let state = DaemonState::initialize().await.unwrap();
        assert!(!state.is_shutting_down());

        state.initiate_shutdown();
        assert!(state.is_shutting_down());
    }

    #[test]
    fn test_shutdown_coordinator_operations() {
        let shutdown = ShutdownCoordinator::new();
        shutdown.increment_operations();
        assert_eq!(shutdown.active_count(), 1);

        shutdown.decrement_operations();
        assert_eq!(shutdown.active_count(), 0);
    }
}
