/// Phase 4.0.3: Policy Audit & DDS Networking Tests
///
/// Integration tests for compliance logging and ROS 2 networking

#[cfg(test)]
mod audit_logging_tests {
    use crate::policy_audit_logger::{AuditLogEntry, PolicyAuditLogger};

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditLogEntry::new(
            "ml-training".to_string(),
            "camera-usb-001".to_string(),
            "camera".to_string(),
            true,
            "Platform policy allows".to_string(),
            "platform".to_string(),
        );

        assert_eq!(entry.environment, "ml-training");
        assert_eq!(entry.device_type, "camera");
        assert_eq!(entry.decision, "ALLOW");
    }

    #[test]
    fn test_audit_logger_with_project_and_user() {
        let entry = AuditLogEntry::new(
            "robot-env".to_string(),
            "serial-arduino".to_string(),
            "serial".to_string(),
            false,
            "Project policy denies serial".to_string(),
            "project".to_string(),
        )
        .with_project("robotics".to_string())
        .with_user("alice@company.com".to_string());

        assert_eq!(entry.project, Some("robotics".to_string()));
        assert_eq!(entry.user_id, Some("alice@company.com".to_string()));
        assert_eq!(entry.decision, "DENY");
    }

    #[test]
    fn test_audit_logger_logging_flow() {
        let logger = PolicyAuditLogger::new(100);

        // Log some access attempts
        for i in 0..5 {
            let entry = AuditLogEntry::new(
                format!("env{}", i),
                format!("device-{}", i),
                "usb".to_string(),
                i % 2 == 0, // Alternating allow/deny
                format!("Policy decision {}", i),
                "platform".to_string(),
            );
            logger.log(entry);
        }

        assert_eq!(logger.entry_count(), 5);
        assert_eq!(logger.get_allowed_attempts().len(), 3);
        assert_eq!(logger.get_denied_attempts().len(), 2);
    }

    #[test]
    fn test_audit_logger_filtering() {
        let logger = PolicyAuditLogger::new(100);

        // Log accesses by different environments
        for env_idx in 0..3 {
            for device_idx in 0..2 {
                let entry = AuditLogEntry::new(
                    format!("env{}", env_idx),
                    format!("device-{}", device_idx),
                    "usb".to_string(),
                    true,
                    "Test".to_string(),
                    "platform".to_string(),
                );
                logger.log(entry);
            }
        }

        // Filter by environment
        let env1_entries = logger.get_environment_entries("env1");
        assert_eq!(env1_entries.len(), 2);

        // Filter by device
        let dev0_entries = logger.get_device_entries("device-0");
        assert_eq!(dev0_entries.len(), 3);
    }

    #[test]
    fn test_compliance_report_generation() {
        let logger = PolicyAuditLogger::new(100);

        // Simulate realistic audit trail
        for i in 0..50 {
            let allowed = i % 5 != 0; // 80% allow rate
            let entry = AuditLogEntry::new(
                format!("env{}", i % 3),
                format!("device-{}", i % 10),
                ["usb", "serial", "camera", "audio"][i % 4].to_string(),
                allowed,
                if allowed { "Policy allows" } else { "Policy denies" }.to_string(),
                ["platform", "project", "environment"][(i % 3) as usize].to_string(),
            );
            logger.log(entry);
        }

        let report = logger.generate_compliance_report();

        assert_eq!(report.total_requests, 50);
        assert_eq!(report.allowed_requests, 40);
        assert_eq!(report.denied_requests, 10);
        assert_eq!(report.allow_percentage, 80.0);
        assert_eq!(report.unique_environments, 3);
        assert!(report.unique_devices >= 1);
    }

    #[test]
    fn test_audit_logger_capacity_management() {
        let logger = PolicyAuditLogger::new(10);

        // Log more entries than capacity
        for i in 0..20 {
            logger.log(AuditLogEntry::new(
                format!("env{}", i),
                format!("device-{}", i),
                "usb".to_string(),
                true,
                "Test".to_string(),
                "platform".to_string(),
            ));
        }

        // Should maintain size limit
        assert_eq!(logger.entry_count(), 10);
    }

    #[test]
    fn test_audit_entries_since_timestamp() {
        let logger = PolicyAuditLogger::new(100);

        let entry1 = AuditLogEntry::new(
            "env1".to_string(),
            "dev1".to_string(),
            "usb".to_string(),
            true,
            "First".to_string(),
            "platform".to_string(),
        );
        let timestamp = entry1.timestamp.clone();

        logger.log(entry1);

        std::thread::sleep(std::time::Duration::from_millis(10));

        for i in 0..5 {
            logger.log(AuditLogEntry::new(
                format!("env{}", i),
                format!("dev{}", i),
                "serial".to_string(),
                false,
                "Later".to_string(),
                "platform".to_string(),
            ));
        }

        let recent = logger.get_entries_since(&timestamp);
        assert!(recent.len() > 1);
    }

    #[test]
    fn test_audit_log_entry_with_path() {
        let mut entry = AuditLogEntry::new(
            "env".to_string(),
            "dev".to_string(),
            "camera".to_string(),
            true,
            "Allowed".to_string(),
            "platform".to_string(),
        );

        let path = vec![
            "Checked environment policy".to_string(),
            "No override, checking project".to_string(),
            "Checked platform policy".to_string(),
            "Platform allows camera".to_string(),
        ];

        entry = entry.with_path(path.clone());

        assert_eq!(entry.decision_path.len(), 4);
        assert_eq!(entry.decision_path[0], "Checked environment policy");
    }

    #[test]
    fn test_audit_log_entry_with_error() {
        let entry = AuditLogEntry::new(
            "env".to_string(),
            "dev".to_string(),
            "audio".to_string(),
            false,
            "Device not found".to_string(),
            "system".to_string(),
        )
        .with_error("Device ID invalid: dev".to_string());

        assert!(entry.error.is_some());
        assert!(entry.error.unwrap().contains("invalid"));
    }
}

#[cfg(test)]
mod dds_networking_tests {
    use crate::dds_network_config::{
        DDSNetworkManager, DDSNetworkStatus, DDSParticipantConfig, DDSTransport,
    };

    #[test]
    fn test_dds_participant_config_default() {
        let config = DDSParticipantConfig::default();

        assert_eq!(config.domain_id, 0);
        assert_eq!(config.transport, DDSTransport::Multicast);
        assert!(config.enable_discovery);
        assert!(config.enable_multicast);
    }

    #[test]
    fn test_dds_participant_config_multicast() {
        let config = DDSParticipantConfig::default();
        assert_eq!(config.transport, DDSTransport::Multicast);
        assert!(config.multicast_addresses.contains(&"239.255.0.1".to_string()));
    }

    #[test]
    fn test_dds_config_xml_generation() {
        let config = DDSParticipantConfig::default();
        let xml = config.to_fast_dds_xml();

        assert!(xml.contains("<?xml version"));
        assert!(xml.contains("domainId>0"));
        assert!(xml.contains("ROS2Participant"));
        assert!(xml.contains("239.255.0.1"));
    }

    #[test]
    fn test_dds_config_env_vars() {
        let config = DDSParticipantConfig::default();
        let env_vars = config.to_env_vars();

        assert!(env_vars.contains_key("ROS_DOMAIN_ID"));
        assert!(env_vars.contains_key("RMW_IMPLEMENTATION"));
        assert_eq!(env_vars.get("RMW_IMPLEMENTATION"), Some(&"rmw_fastrtps_cpp".to_string()));
    }

    #[test]
    fn test_dds_config_for_environment() {
        let config = DDSParticipantConfig::for_environment("robotics-sim");

        assert_eq!(config.domain_id, 0);
        assert!(config.qos_settings.get("participant").is_some());
    }

    #[test]
    fn test_dds_network_status_creation() {
        let status = DDSNetworkStatus::new("ros-env".to_string(), 0);

        assert_eq!(status.environment, "ros-env");
        assert_eq!(status.domain_id, 0);
        assert!(!status.operational);
        assert_eq!(status.connected_participants, 0);
    }

    #[test]
    fn test_dds_network_status_marking_operational() {
        let mut status = DDSNetworkStatus::new("test-env".to_string(), 0);

        status.mark_operational();
        assert!(status.operational);

        let heartbeat = chrono::Utc::now().to_rfc3339();
        assert!(status.last_heartbeat >= heartbeat || status.last_heartbeat.len() > 0);
    }

    #[test]
    fn test_dds_network_status_node_discovery() {
        let mut status = DDSNetworkStatus::new("ros-env".to_string(), 0);

        status.add_node("/robot_controller".to_string());
        status.add_node("/sensor_driver".to_string());

        assert_eq!(status.ros_nodes.len(), 2);
        assert!(status.ros_nodes.contains(&"/robot_controller".to_string()));

        // Adding duplicate should not increase count
        status.add_node("/robot_controller".to_string());
        assert_eq!(status.ros_nodes.len(), 2);
    }

    #[test]
    fn test_dds_network_status_topic_discovery() {
        let mut status = DDSNetworkStatus::new("ros-env".to_string(), 0);

        status.add_topic("/robot/state".to_string());
        status.add_topic("/robot/cmd".to_string());
        status.add_topic("/sensor/imu".to_string());

        assert_eq!(status.ros_topics.len(), 3);
        assert!(status.ros_topics.contains(&"/robot/state".to_string()));
    }

    #[test]
    fn test_dds_network_manager_environment_registration() {
        let mut manager = DDSNetworkManager::new();

        let status = manager.register_environment("robotics-env".to_string(), 0);

        assert_eq!(status.environment, "robotics-env");
        assert_eq!(status.domain_id, 0);
    }

    #[test]
    fn test_dds_network_manager_status_tracking() {
        let mut manager = DDSNetworkManager::new();

        manager.register_environment("env1".to_string(), 0);
        manager.register_environment("env2".to_string(), 1);

        assert!(manager.get_status("env1").is_some());
        assert!(manager.get_status("env2").is_some());
        assert!(manager.get_status("env3").is_none());
    }

    #[test]
    fn test_dds_network_manager_operational_status() {
        let mut manager = DDSNetworkManager::new();

        manager.register_environment("env1".to_string(), 0);
        let mut status = manager.register_environment("env2".to_string(), 1);
        status.mark_operational();

        manager.update_status("env2", status);

        assert!(!manager.is_operational("env1"));
        assert!(manager.is_operational("env2"));
    }

    #[test]
    fn test_dds_network_manager_operational_list() {
        let mut manager = DDSNetworkManager::new();

        manager.register_environment("env1".to_string(), 0);
        manager.register_environment("env2".to_string(), 1);
        manager.register_environment("env3".to_string(), 2);

        // Mark env2 and env3 as operational
        let mut status2 = manager.register_environment("env2".to_string(), 1);
        status2.mark_operational();
        manager.update_status("env2", status2);

        let mut status3 = manager.register_environment("env3".to_string(), 2);
        status3.mark_operational();
        manager.update_status("env3", status3);

        let operational = manager.get_operational_environments();

        assert_eq!(operational.len(), 2);
        assert!(operational.contains(&"env2".to_string()));
        assert!(operational.contains(&"env3".to_string()));
    }

    #[test]
    fn test_dds_network_manager_get_all_statuses() {
        let mut manager = DDSNetworkManager::new();

        for i in 0..5 {
            manager.register_environment(format!("env{}", i), (i % 3) as u8);
        }

        let all_statuses = manager.get_all_statuses();
        assert_eq!(all_statuses.len(), 5);
    }

    #[test]
    fn test_dds_network_status_with_nodes_and_topics() {
        let mut status = DDSNetworkStatus::new("robotics".to_string(), 0);

        status.mark_operational();
        status.add_node("/ur10_controller".to_string());
        status.add_node("/camera_driver".to_string());
        status.add_topic("/joint_states".to_string());
        status.add_topic("/camera/image".to_string());

        assert!(status.operational);
        assert_eq!(status.ros_nodes.len(), 2);
        assert_eq!(status.ros_topics.len(), 2);
    }
}
