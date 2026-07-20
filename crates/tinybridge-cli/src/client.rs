use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use tinybridge_core::{
    methods, DownResponse, JsonRpcRequest, JsonRpcResponse, ListResponse, StatusResponse,
    UpResponse,
};

pub struct DaemonClient {
    socket_path: PathBuf,
    request_id: u64,
}

impl DaemonClient {
    pub fn new(socket_path: Option<PathBuf>) -> Result<Self> {
        let socket_path =
            socket_path.unwrap_or_else(tinybridge_core::TinyBridgeConfig::socket_path);

        Ok(DaemonClient {
            socket_path,
            request_id: 1,
        })
    }

    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let request = JsonRpcRequest::new(self.request_id, method, params);
        self.request_id += 1;

        let mut stream = UnixStream::connect(&self.socket_path).map_err(|e| {
            anyhow!(
                "Failed to connect to daemon at {:?}: {}. Is tinybridged running?",
                self.socket_path,
                e
            )
        })?;

        let request_json = serde_json::to_string(&request)?;
        stream.write_all(request_json.as_bytes())?;
        stream.write_all(b"\n")?;

        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
        let response_line = lines
            .next()
            .ok_or_else(|| anyhow!("No response from daemon"))??;

        let response: JsonRpcResponse = serde_json::from_str(&response_line)?;

        if let Some(error) = response.error {
            return Err(anyhow!("Daemon error ({}): {}", error.code, error.message));
        }

        response
            .result
            .ok_or_else(|| anyhow!("No result in response"))
    }

    pub async fn up(&mut self, name: Option<String>, env_yaml_path: Option<String>) -> Result<()> {
        let params = json!({
            "name": name,
            "env_yaml_path": env_yaml_path,
            "wait": true,
        });

        let result = self.call(methods::ENVIRONMENT_UP, params).await?;
        let _resp: UpResponse = serde_json::from_value(result)?;
        Ok(())
    }

    pub async fn down(&mut self, name: Option<String>, force: bool) -> Result<()> {
        let params = json!({
            "name": name,
            "force": force,
        });

        let result = self.call(methods::ENVIRONMENT_DOWN, params).await?;
        let _resp: DownResponse = serde_json::from_value(result)?;
        Ok(())
    }

    pub async fn status(&mut self, name: Option<String>) -> Result<StatusResponse> {
        let params = json!({
            "name": name,
        });

        let result = self.call(methods::ENVIRONMENT_STATUS, params).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn list(&mut self) -> Result<ListResponse> {
        let params = json!({});
        let result = self.call(methods::ENVIRONMENT_LIST, params).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn open_shell(&mut self, name: &str) -> Result<UnixStream> {
        let params = json!({
            "name": name,
            "interactive": true,
        });

        // Initiate shell session
        let result = self.call(methods::ENVIRONMENT_SHELL, params).await?;

        // Extract shell ID from response
        let shell_id = result
            .get("shell_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No shell_id in response"))?;

        // Connect to shell socket
        let shell_socket_path = tinybridge_core::TinyBridgeConfig::shell_socket_path(shell_id);
        let socket = UnixStream::connect(&shell_socket_path).map_err(|e| {
            anyhow!("Failed to connect to shell socket: {}", e)
        })?;

        Ok(socket)
    }

    pub async fn shell_command(&mut self, params: Value) -> Result<()> {
        let result = self.call(methods::ENVIRONMENT_SHELL, params).await?;

        // Get output and print
        if let Some(output) = result.get("output").and_then(|v| v.as_str()) {
            println!("{}", output);
        }

        if let Some(error) = result.get("error").and_then(|v| v.as_str()) {
            eprintln!("Error: {}", error);
        }

        Ok(())
    }
}
