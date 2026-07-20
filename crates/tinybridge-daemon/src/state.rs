use tinybridge_core::EnvironmentStatus;

#[allow(dead_code)]
pub struct StateTransition;

#[allow(dead_code)]
impl StateTransition {
    pub fn can_start(status: &EnvironmentStatus) -> bool {
        matches!(status, EnvironmentStatus::Stopped)
    }

    pub fn can_stop(status: &EnvironmentStatus) -> bool {
        matches!(
            status,
            EnvironmentStatus::Running { .. } | EnvironmentStatus::Starting { .. }
        )
    }

    pub fn start_started(status: &EnvironmentStatus) -> bool {
        matches!(status, EnvironmentStatus::Starting { .. })
    }

    pub fn start_running(status: &EnvironmentStatus) -> bool {
        matches!(status, EnvironmentStatus::Running { .. })
    }

    pub fn is_terminal(status: &EnvironmentStatus) -> bool {
        status.is_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_start_from_stopped() {
        let status = EnvironmentStatus::Stopped;
        assert!(StateTransition::can_start(&status));
    }

    #[test]
    fn test_cannot_start_from_running() {
        let status = EnvironmentStatus::Running { uptime_secs: 10 };
        assert!(!StateTransition::can_start(&status));
    }

    #[test]
    fn test_can_stop_from_running() {
        let status = EnvironmentStatus::Running { uptime_secs: 10 };
        assert!(StateTransition::can_stop(&status));
    }

    #[test]
    fn test_can_stop_from_starting() {
        let status = EnvironmentStatus::Starting { progress_pct: 50 };
        assert!(StateTransition::can_stop(&status));
    }
}
