use anyhow::Result;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::Mutex;

use tinybridge_core::{error_codes, methods, JsonRpcRequest, JsonRpcResponse};
use tinybridge_dds::DdsManager;

use crate::manager::EnvironmentManager;
use crate::server;

pub async fn handle_connection(
    mut socket: UnixStream,
    manager: Arc<Mutex<EnvironmentManager>>,
    dds_manager: Arc<parking_lot::Mutex<DdsManager>>,
) -> Result<()> {
    let (reader, mut writer) = socket.split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(line) = lines.next_line().await? {
        let response = process_request(&line, &manager, &dds_manager).await;

        let response_json = serde_json::to_string(&response)?;
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    Ok(())
}

async fn process_request(
    line: &str,
    manager: &Arc<Mutex<EnvironmentManager>>,
    dds_manager: &Arc<parking_lot::Mutex<DdsManager>>,
) -> JsonRpcResponse {
    let request: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(req) => req,
        Err(_) => {
            return JsonRpcResponse::error(0, error_codes::PARSE_ERROR, "Invalid JSON-RPC request");
        }
    };

    // Try DDS methods first
    let dds_handler = crate::dds_rpc::DdsRpcHandler::new(Arc::clone(dds_manager));
    if let Some(response) = dds_handler.dispatch(&request.method, &request.params, request.id) {
        return response;
    }

    // Then try environment methods
    let result = match request.method.as_str() {
        methods::ENVIRONMENT_UP => {
            let name = request
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let env_yaml_path = request
                .params
                .get("env_yaml_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mut mgr = manager.lock().await;
            mgr.up(name, env_yaml_path).await
        }

        methods::ENVIRONMENT_DOWN => {
            let name = request
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let force = request
                .params
                .get("force")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let mut mgr = manager.lock().await;
            mgr.down(name, force).await
        }

        methods::ENVIRONMENT_STATUS => {
            let name = request
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mgr = manager.lock().await;
            mgr.status(name)
        }

        methods::ENVIRONMENT_LIST => {
            let mgr = manager.lock().await;
            mgr.list()
        }

        methods::ENVIRONMENT_SHELL => {
            let name = request
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mgr = manager.lock().await;
            mgr.shell(name).await
        }

        _ => Err(anyhow::anyhow!("Unknown method")),
    };

    match result {
        Ok(response) => JsonRpcResponse::success(request.id, response),
        Err(e) => JsonRpcResponse::error(request.id, error_codes::INTERNAL_ERROR, e.to_string()),
    }
}
