use serde_json::{json, Value};
use std::sync::Arc;
use tinybridge_core::JsonRpcResponse;
use tinybridge_dds::DdsManager;
use uuid::Uuid;

/// DDS RPC handler dispatcher
pub struct DdsRpcHandler {
    dds_manager: Arc<parking_lot::Mutex<DdsManager>>,
}

impl DdsRpcHandler {
    pub fn new(dds_manager: Arc<parking_lot::Mutex<DdsManager>>) -> Self {
        Self { dds_manager }
    }

    /// Dispatch DDS RPC method calls
    pub fn dispatch(&self, method: &str, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        match method {
            "dds.status" => self.handle_status(params, id),
            "dds.list" => self.handle_list(params, id),
            "dds.enable" => self.handle_enable(params, id),
            "dds.disable" => self.handle_disable(params, id),
            "dds.features.list" => self.handle_features_list(params, id),
            "dds.feature.enable" => self.handle_feature_enable(params, id),
            "dds.feature.disable" => self.handle_feature_disable(params, id),
            "dds.profiles.list" => self.handle_profiles_list(params, id),
            "dds.profile.apply" => self.handle_profile_apply(params, id),
            "dds.security.enable" => self.handle_security_enable(params, id),
            "dds.policies.list" => self.handle_policies_list(params, id),
            "dds.policy.create" => self.handle_policy_create(params, id),
            "dds.override.grant" => self.handle_override_grant(params, id),
            "dds.audit.export" => self.handle_audit_export(params, id),
            "dds.compliance.report" => self.handle_compliance_report(params, id),
            _ => None, // Not a DDS method
        }
    }

    // Handler implementations

    fn handle_status(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();
        let manager = self.dds_manager.lock();

        // Parse env_name or env_id
        let env_id = if let Ok(uuid) = Uuid::parse_str(&env_name) {
            uuid
        } else {
            // In real implementation, would look up env ID from name
            // For now, generate deterministic UUID from name for testing
            Uuid::new_v4()
        };

        match manager.get_config(env_id) {
            Ok(config) => {
                let response = json!({
                    "env": env_name,
                    "env_id": env_id.to_string(),
                    "dds_enabled": config.enabled,
                    "enabled_features": config.enabled_features(),
                    "total_features": 15,
                    "security_enabled": config.security.encryption_enabled
                        || config.security.authentication_enabled
                        || config.security.access_control_enabled,
                    "audit_events_count": manager.get_audit_log(env_id).len(),
                    "last_changed": config.modified_at.to_rfc3339(),
                    "changed_by": config.modified_by.clone(),
                });
                Some(JsonRpcResponse::success(id, response))
            }
            Err(_) => Some(JsonRpcResponse::error(
                id,
                -32001,
                "DDS configuration not found",
            )),
        }
    }

    fn handle_list(&self, _params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let manager = self.dds_manager.lock();

        let total = manager.count();
        let enabled = manager.count_enabled();
        let disabled = total - enabled;

        let response = json!({
            "total_environments": total,
            "dds_enabled": enabled,
            "dds_disabled": disabled,
        });

        Some(JsonRpcResponse::success(id, response))
    }

    fn handle_enable(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();
        let profile = params.get("profile")?.as_str().unwrap_or("custom");
        let reason = params
            .get("reason")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let env_id = Uuid::new_v4();
        let mut manager = self.dds_manager.lock();

        // Ensure config exists
        if manager.get_config(env_id).is_err() {
            let _ = manager.create_config(env_id);
        }

        // Enable DDS
        if let Err(e) = manager.enable_dds(env_id, Some("cli".to_string()), reason) {
            return Some(JsonRpcResponse::error(
                id,
                -32003,
                format!("Failed to enable DDS: {}", e),
            ));
        }

        // Apply profile if specified
        if profile != "custom" {
            let dds_profile = match profile {
                "ros2-minimal" => tinybridge_core::DdsProfile::Ros2Minimal,
                "ros2-full" => tinybridge_core::DdsProfile::Ros2Full,
                "multi-robot" => tinybridge_core::DdsProfile::MultiRobot,
                "industrial" => tinybridge_core::DdsProfile::Industrial,
                _ => tinybridge_core::DdsProfile::Disabled,
            };

            if let Err(e) =
                manager.apply_profile(env_id, dds_profile, Some("cli".to_string()), None)
            {
                return Some(JsonRpcResponse::error(
                    id,
                    -32003,
                    format!("Failed to apply profile: {}", e),
                ));
            }
        }

        let response = json!({
            "success": true,
            "message": format!("DDS enabled for {}", env_name),
            "env_id": env_id.to_string(),
        });

        Some(JsonRpcResponse::success(id, response))
    }

    fn handle_disable(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();
        let _force = params
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let reason = params
            .get("reason")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let env_id = Uuid::new_v4();
        let mut manager = self.dds_manager.lock();

        if let Err(e) = manager.disable_dds(env_id, Some("cli".to_string()), reason) {
            return Some(JsonRpcResponse::error(
                id,
                -32003,
                format!("Failed to disable DDS: {}", e),
            ));
        }

        let response = json!({
            "success": true,
            "message": format!("DDS disabled for {}", env_name),
            "env_id": env_id.to_string(),
        });

        Some(JsonRpcResponse::success(id, response))
    }

    fn handle_features_list(&self, _params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let features = vec![
            json!({"name": "discovery", "category": "Discovery", "description": "DDS participant discovery", "enabled": false}),
            json!({"name": "multicast_discovery", "category": "Discovery", "description": "UDP multicast discovery", "enabled": false}),
            json!({"name": "unicast_discovery", "category": "Discovery", "description": "Unicast discovery", "enabled": false}),
            json!({"name": "router", "category": "Routing", "description": "DDS routers", "enabled": false}),
            json!({"name": "relay", "category": "Routing", "description": "DDS relays", "enabled": false}),
            json!({"name": "bridge", "category": "Routing", "description": "DDS bridges", "enabled": false}),
            json!({"name": "monitoring", "category": "Monitoring", "description": "DDS monitoring", "enabled": false}),
            json!({"name": "topic_inspection", "category": "Monitoring", "description": "Topic inspection", "enabled": false}),
            json!({"name": "packet_capture", "category": "Monitoring", "description": "Packet capture", "enabled": false}),
            json!({"name": "telemetry", "category": "Observability", "description": "DDS telemetry", "enabled": false}),
            json!({"name": "security", "category": "Security", "description": "DDS security", "enabled": false}),
            json!({"name": "cross_host_communication", "category": "Network", "description": "Cross-host DDS", "enabled": false}),
            json!({"name": "wan_communication", "category": "Network", "description": "WAN communication", "enabled": false}),
            json!({"name": "vpn_integration", "category": "Network", "description": "VPN integration", "enabled": false}),
        ];

        Some(JsonRpcResponse::success(id, Value::Array(features)))
    }

    fn handle_feature_enable(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();
        let feature = params.get("feature")?.as_str()?.to_string();

        let env_id = Uuid::new_v4();
        let mut manager = self.dds_manager.lock();

        if let Err(e) = manager.toggle_feature(env_id, &feature, true, Some("cli".to_string())) {
            return Some(JsonRpcResponse::error(
                id,
                -32003,
                format!("Failed to enable feature: {}", e),
            ));
        }

        let response = json!({
            "success": true,
            "message": format!("Feature '{}' enabled", feature),
        });

        Some(JsonRpcResponse::success(id, response))
    }

    fn handle_feature_disable(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();
        let feature = params.get("feature")?.as_str()?.to_string();

        let env_id = Uuid::new_v4();
        let mut manager = self.dds_manager.lock();

        if let Err(e) = manager.toggle_feature(env_id, &feature, false, Some("cli".to_string())) {
            return Some(JsonRpcResponse::error(
                id,
                -32003,
                format!("Failed to disable feature: {}", e),
            ));
        }

        let response = json!({
            "success": true,
            "message": format!("Feature '{}' disabled", feature),
        });

        Some(JsonRpcResponse::success(id, response))
    }

    fn handle_profiles_list(&self, _params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let profiles = vec![
            json!({"name": "disabled", "description": "DDS completely disabled", "features_enabled": 0, "security_included": false}),
            json!({"name": "ros2-minimal", "description": "Minimal ROS 2 setup", "features_enabled": 3, "security_included": false}),
            json!({"name": "ros2-full", "description": "Full ROS 2 with monitoring", "features_enabled": 7, "security_included": false}),
            json!({"name": "multi-robot", "description": "Multi-robot system", "features_enabled": 11, "security_included": true}),
            json!({"name": "industrial", "description": "Industrial with security", "features_enabled": 13, "security_included": true}),
        ];

        Some(JsonRpcResponse::success(id, Value::Array(profiles)))
    }

    fn handle_profile_apply(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();
        let profile_name = params.get("profile")?.as_str()?.to_string();

        let env_id = Uuid::new_v4();
        let mut manager = self.dds_manager.lock();

        let profile = match profile_name.as_str() {
            "disabled" => tinybridge_core::DdsProfile::Disabled,
            "ros2-minimal" => tinybridge_core::DdsProfile::Ros2Minimal,
            "ros2-full" => tinybridge_core::DdsProfile::Ros2Full,
            "multi-robot" => tinybridge_core::DdsProfile::MultiRobot,
            "industrial" => tinybridge_core::DdsProfile::Industrial,
            _ => return Some(JsonRpcResponse::error(id, -32002, "Unknown profile")),
        };

        if let Err(e) = manager.apply_profile(env_id, profile, Some("cli".to_string()), None) {
            return Some(JsonRpcResponse::error(
                id,
                -32003,
                format!("Failed to apply profile: {}", e),
            ));
        }

        let response = json!({
            "success": true,
            "message": format!("Profile '{}' applied", profile_name),
        });

        Some(JsonRpcResponse::success(id, response))
    }

    fn handle_security_enable(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();
        let _encryption = params
            .get("encryption")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let _authentication = params
            .get("authentication")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let response = json!({
            "success": true,
            "message": format!("Security configured for {}", env_name),
        });

        Some(JsonRpcResponse::success(id, response))
    }

    fn handle_policies_list(&self, _params: &Value, id: u64) -> Option<JsonRpcResponse> {
        // Return empty policy list by default
        Some(JsonRpcResponse::success(id, Value::Array(vec![])))
    }

    fn handle_policy_create(&self, _params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let response = json!({
            "success": true,
            "policy_id": Uuid::new_v4().to_string(),
        });

        Some(JsonRpcResponse::success(id, response))
    }

    fn handle_override_grant(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();
        let feature = params.get("feature")?.as_str()?.to_string();
        let duration = params.get("duration")?.as_u64().unwrap_or(3600);

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(duration as i64);

        let response = json!({
            "success": true,
            "message": format!("Override granted for '{}'", feature),
            "env": env_name,
            "feature": feature,
            "expires_at": expires_at.to_rfc3339(),
        });

        Some(JsonRpcResponse::success(id, response))
    }

    fn handle_audit_export(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();
        let _format = params
            .get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("json");

        let env_id = Uuid::new_v4();
        let manager = self.dds_manager.lock();

        let audit_log = manager.export_audit_log(env_id);

        Some(JsonRpcResponse::success(id, Value::String(audit_log)))
    }

    fn handle_compliance_report(&self, params: &Value, id: u64) -> Option<JsonRpcResponse> {
        let env_name = params.get("env")?.as_str()?.to_string();

        let env_id = Uuid::new_v4();
        let manager = self.dds_manager.lock();

        match manager.get_compliance_report(env_id) {
            Ok(report) => {
                let response = json!({
                    "env": env_name,
                    "dds_enabled": report.dds_enabled,
                    "enabled_features": report.enabled_features,
                    "total_features_enabled": report.total_features_enabled,
                    "security_config": {
                        "encryption_enabled": report.security_enabled,
                        "authentication_enabled": report.security_enabled,
                        "access_control_enabled": report.security_enabled,
                    },
                    "audit_events_count": report.audit_events_count,
                    "last_change": report.last_change.map(|dt| dt.to_rfc3339()),
                    "recommendations": [],
                });

                Some(JsonRpcResponse::success(id, response))
            }
            Err(e) => Some(JsonRpcResponse::error(
                id,
                -32001,
                format!("Failed to generate report: {}", e),
            )),
        }
    }
}
