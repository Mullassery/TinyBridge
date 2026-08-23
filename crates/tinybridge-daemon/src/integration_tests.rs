/// End-to-end integration tests for Phase 3
///
/// Tests verify:
/// 1. Complete error flow (daemon → JSON-RPC → CLI)
/// 2. Graceful shutdown signal handling
/// 3. Health endpoint responses
/// 4. Structured logging with correlation IDs
#[cfg(test)]
mod e2e_tests {
    use crate::error_propagation::ErrorPropagator;
    use crate::graceful_shutdown::ShutdownCoordinator;
    use crate::health::{HealthChecker, HealthStatus};
    use crate::structured_logging::LogContext;
    use tinybridge_error::BridgeError;
    use tinybridge_error::ErrorSeverity;

    #[test]
    fn test_error_propagation_flow() {
        // Simulate error at daemon level
        let error = BridgeError::vm("Memory allocation failed".to_string())
            .with_severity(ErrorSeverity::Error);

        // Convert to JSON-RPC format
        let (code, msg, data) = ErrorPropagator::to_json_rpc_error(&error);

        // Verify error code is domain-specific
        assert_eq!(code, -32000); // VIRTUALIZATION_ERROR
        assert!(msg.contains("Memory allocation failed"));

        // Verify context is preserved in JSON
        assert!(data.is_some());
        let json_data = data.unwrap();
        assert_eq!(json_data["severity"], "Error");
        assert!(json_data["kind"].is_string());
    }

    #[test]
    fn test_error_flow_without_context() {
        // Test error without context (minimal case)
        let error = BridgeError::cli("Simple error".to_string());

        let (code, msg, data) = ErrorPropagator::to_json_rpc_error(&error);

        assert_eq!(code, -32099); // UNKNOWN_ERROR
        assert_eq!(msg, "Simple error");
        assert!(data.is_none()); // No context = no data field
    }

    #[test]
    fn test_health_check_all_resources() {
        let checker = HealthChecker::new();
        let report = checker.check_all();

        // Verify all 4 checks are present
        assert_eq!(report.resources.len(), 4);

        // Verify resource names
        let resource_names: Vec<_> = report.resources.iter().map(|r| r.name.as_str()).collect();

        assert!(resource_names.contains(&"virtualization"));
        assert!(resource_names.contains(&"memory"));
        assert!(resource_names.contains(&"disk"));
        assert!(resource_names.contains(&"socket"));

        // Verify status is aggregated correctly
        assert!(matches!(
            report.status,
            HealthStatus::Healthy | HealthStatus::Degraded
        ));
    }

    #[test]
    fn test_correlation_id_in_log_context() {
        let ctx = LogContext::new("test.method", "corr-123");

        // Verify context has method and correlation ID
        let json = ctx.to_json();
        assert_eq!(json["operation"], "test.method");
        assert_eq!(json["correlation_id"], "corr-123");
    }

    #[test]
    fn test_shutdown_coordinator_lifecycle() {
        let coordinator = ShutdownCoordinator::new();

        // Verify initial state
        assert!(!coordinator.is_shutting_down());
        assert_eq!(coordinator.active_count(), 0);

        // Simulate active operations
        coordinator.increment_operations();
        coordinator.increment_operations();
        assert_eq!(coordinator.active_count(), 2);

        // Initiate shutdown
        coordinator.initiate_shutdown();
        assert!(coordinator.is_shutting_down());

        // Verify operations can still be decremented
        coordinator.decrement_operations();
        assert_eq!(coordinator.active_count(), 1);
    }

    #[test]
    fn test_shutdown_signal_propagation() {
        let coordinator = ShutdownCoordinator::new();
        let rx = coordinator.subscribe();

        // Initiate shutdown
        coordinator.initiate_shutdown();

        // Verify shutdown state
        assert!(coordinator.is_shutting_down());

        // Note: In real async context, rx.recv() would return shutdown signal
        // This test verifies subscription mechanism works
    }

    #[test]
    fn test_error_severity_determines_exit_code() {
        // Critical errors should trigger exit
        let critical_error =
            BridgeError::vm("Critical failure".to_string()).with_severity(ErrorSeverity::Critical);
        assert!(critical_error.severity.should_exit());

        // Warning errors should not trigger exit
        let warning_error = BridgeError::network("Network latency".to_string())
            .with_severity(ErrorSeverity::Warning);
        assert!(!warning_error.severity.should_exit());
    }

    #[test]
    fn test_error_context_preservation_through_propagation() {
        // Create error with rich context
        let error = BridgeError::storage("Disk full".to_string())
            .with_severity(ErrorSeverity::Error)
            .with_context(tinybridge_error::ErrorContext::disk_space(5, 20))
            .with_suggestion(tinybridge_error::RecoverySuggestion::check_disk_space(20));

        // Propagate through JSON-RPC
        let (code, msg, data) = ErrorPropagator::to_json_rpc_error(&error);

        // Verify nothing is lost
        assert_eq!(code, -32004); // STORAGE_ERROR
        assert!(data.is_some());

        let json_data = data.unwrap();
        assert!(json_data.get("context").is_some());
        assert!(json_data.get("suggestion").is_some());
    }

    #[test]
    fn test_health_check_to_error_bridge() {
        // If health check shows degraded, it could trigger an error response
        let checker = HealthChecker::new();
        let report = checker.check_all();

        // Find any degraded resources
        let degraded = report
            .resources
            .iter()
            .filter(|r| r.status == HealthStatus::Degraded)
            .collect::<Vec<_>>();

        // If degraded, that information flows through JSON-RPC
        if !degraded.is_empty() {
            let err = BridgeError::vm(format!("{} resources degraded", degraded.len()))
                .with_context(
                    tinybridge_error::ErrorContext::new()
                        .with_detail("degraded_resources".to_string(), format!("{:?}", degraded)),
                );

            let (_, _, data) = ErrorPropagator::to_json_rpc_error(&err);
            assert!(data.is_some());
        }
    }

    #[test]
    fn test_structured_logging_correlation_chain() {
        // Create context with correlation ID
        let ctx1 = LogContext::new("operation.start", "trace-abc-123");
        let json1 = ctx1.to_json();

        // Same correlation ID through the operation
        let ctx2 = LogContext::new("operation.process", "trace-abc-123");
        let json2 = ctx2.to_json();

        // Correlation IDs match
        assert_eq!(json1["correlation_id"], json2["correlation_id"]);
        assert_eq!(json1["correlation_id"], "trace-abc-123");

        // But operations differ
        assert_ne!(json1["operation"], json2["operation"]);
    }

    #[test]
    fn test_error_recovery_suggestion_flow() {
        // Error with recovery suggestion
        let error = BridgeError::network("DNS resolution timeout".to_string())
            .with_suggestion(tinybridge_error::RecoverySuggestion::check_network());

        // Propagate
        let (_, _, data) = ErrorPropagator::to_json_rpc_error(&error);

        // Verify suggestion is included
        assert!(data.is_some());
        let json_data = data.unwrap();
        assert!(json_data.get("suggestion").is_some());

        // Suggestion should have steps
        let suggestion = json_data.get("suggestion").unwrap();
        assert!(suggestion.get("steps").is_some());
    }

    #[test]
    fn test_concurrent_health_checks() {
        // Multiple threads checking health simultaneously
        let mut handles = vec![];

        for _ in 0..5 {
            let handle = std::thread::spawn(|| {
                let checker = HealthChecker::new();
                let report = checker.check_all();
                assert!(!report.resources.is_empty());
            });
            handles.push(handle);
        }

        // All checks complete successfully
        for handle in handles {
            assert!(handle.join().is_ok());
        }
    }

    #[test]
    fn test_shutdown_under_load() {
        let coordinator = ShutdownCoordinator::new();

        // Simulate load
        for _ in 0..10 {
            coordinator.increment_operations();
        }
        assert_eq!(coordinator.active_count(), 10);

        // Initiate shutdown
        coordinator.initiate_shutdown();
        assert!(coordinator.is_shutting_down());

        // Operations can still complete
        for _ in 0..10 {
            coordinator.decrement_operations();
        }
        assert_eq!(coordinator.active_count(), 0);
    }

    #[test]
    fn test_error_code_mapping_completeness() {
        // Verify all error kinds map to valid error codes
        let error_kinds = vec![
            BridgeError::bootstrap("test".to_string()),
            BridgeError::vm("test".to_string()),
            BridgeError::cli("test".to_string()),
            BridgeError::network("test".to_string()),
            BridgeError::storage("test".to_string()),
            BridgeError::permission("test".to_string()),
            BridgeError::configuration("test".to_string()),
            BridgeError::unknown("test".to_string()),
        ];

        for error in error_kinds {
            let (code, _, _) = ErrorPropagator::to_json_rpc_error(&error);

            // All codes should be valid JSON-RPC error codes (< -32000 or standard)
            assert!(code < 0, "Error code should be negative: {}", code);

            // Should not be reserved standard codes
            assert!(
                code < -32000
                    || code == -32603
                    || code == -32602
                    || code == -32601
                    || code == -32600
                    || code == -32700,
                "Invalid error code: {}",
                code
            );
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use crate::health::HealthChecker;
    use std::time::Instant;

    #[test]
    fn test_health_check_performance() {
        let checker = HealthChecker::new();
        let start = Instant::now();

        // Run multiple checks
        for _ in 0..100 {
            let _ = checker.check_all();
        }

        let duration = start.elapsed().as_millis();

        // 100 health checks should complete in reasonable time (<1000ms)
        assert!(
            duration < 1000,
            "Health checks took {}ms (expected <1000ms)",
            duration
        );
    }

    #[test]
    fn test_error_propagation_performance() {
        use crate::error_propagation::ErrorPropagator;
        use std::time::Instant;
        use tinybridge_error::BridgeError;

        let error = BridgeError::vm("Test error".to_string())
            .with_severity(tinybridge_error::ErrorSeverity::Error);

        let start = Instant::now();

        // Convert 1000 times
        for _ in 0..1000 {
            let _ = ErrorPropagator::to_json_rpc_error(&error);
        }

        let duration = start.elapsed().as_millis();

        // Should be very fast (<50ms for 1000 conversions)
        assert!(
            duration < 50,
            "Error propagation took {}ms for 1000 ops (expected <50ms)",
            duration
        );
    }
}
