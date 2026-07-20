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

    async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
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
}
