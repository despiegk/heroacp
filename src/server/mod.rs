//! Server SDK for building ACP agents.
//!
//! This module provides the infrastructure for building ACP-compliant AI coding agents
//! that can be integrated with any ACP-compatible editor or IDE.
//!
//! # Example
//!
//! ```rust,no_run
//! use heroacp::server::{Agent, Server};
//! use heroacp::protocol::*;
//! use async_trait::async_trait;
//! use tokio::sync::mpsc;
//!
//! struct MyAgent;
//!
//! #[async_trait]
//! impl Agent for MyAgent {
//!     async fn initialize(
//!         &self,
//!         params: InitializeParams,
//!     ) -> AcpResult<InitializeResult> {
//!         Ok(InitializeResult {
//!             agent_info: AgentInfo {
//!                 name: "my-agent".to_string(),
//!                 version: "1.0.0".to_string(),
//!             },
//!             capabilities: AgentCapabilities::default(),
//!             instructions: Some("Hello!".to_string()),
//!         })
//!     }
//!
//!     async fn session_new(
//!         &self,
//!         params: SessionNewParams,
//!     ) -> AcpResult<SessionNewResult> {
//!         Ok(SessionNewResult {
//!             session_id: params.session_id,
//!         })
//!     }
//!
//!     async fn session_prompt(
//!         &self,
//!         params: SessionPromptParams,
//!         update_tx: mpsc::Sender<SessionUpdate>,
//!     ) -> AcpResult<SessionPromptResult> {
//!         Ok(SessionPromptResult {
//!             status: "ok".to_string(),
//!         })
//!     }
//! }
//! ```

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::protocol::*;

/// Trait for implementing an ACP agent.
///
/// Implement this trait to create your own AI coding agent that can
/// communicate with ACP-compatible editors.
#[async_trait]
pub trait Agent: Send + Sync + 'static {
    /// Handle the initialize request.
    ///
    /// This is called when the client first connects. Return your agent's
    /// capabilities and information.
    async fn initialize(&self, params: InitializeParams) -> AcpResult<InitializeResult>;

    /// Handle optional authentication.
    ///
    /// Override this if your agent requires authentication.
    async fn authenticate(&self, _params: AuthenticateParams) -> AcpResult<AuthenticateResult> {
        Ok(AuthenticateResult { success: true })
    }

    /// Handle creating a new session.
    async fn session_new(&self, params: SessionNewParams) -> AcpResult<SessionNewResult>;

    /// Handle loading an existing session.
    ///
    /// Override this to support session persistence.
    async fn session_load(&self, params: SessionLoadParams) -> AcpResult<SessionLoadResult> {
        Ok(SessionLoadResult {
            session_id: params.session_id,
            loaded: false,
        })
    }

    /// Handle a prompt from the user.
    ///
    /// Use the `update_tx` channel to send streaming updates back to the client.
    async fn session_prompt(
        &self,
        params: SessionPromptParams,
        update_tx: mpsc::Sender<SessionUpdate>,
    ) -> AcpResult<SessionPromptResult>;

    /// Handle cancellation of the current operation.
    async fn session_cancel(&self, _params: SessionCancelParams) -> AcpResult<()> {
        Ok(())
    }
}

/// ACP server that runs an agent.
pub struct Server<A: Agent> {
    agent: Arc<A>,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<JsonRpcResponse>>>>,
    next_request_id: Arc<Mutex<u64>>,
}

impl<A: Agent> Server<A> {
    /// Create a new server with the given agent.
    pub fn new(agent: A) -> Self {
        Self {
            agent: Arc::new(agent),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            next_request_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Run the server, reading from stdin and writing to stdout.
    pub async fn run(&self) -> AcpResult<()> {
        let stdin = io::stdin();
        let stdout = io::stdout();

        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();

        let (update_tx, mut update_rx) = mpsc::channel::<SessionUpdate>(100);
        let (response_tx, mut response_rx) = mpsc::channel::<String>(100);

        // Spawn task to write responses
        let stdout = Arc::new(Mutex::new(stdout));
        let stdout_clone = stdout.clone();
        tokio::spawn(async move {
            while let Some(msg) = response_rx.recv().await {
                let mut stdout = stdout_clone.lock().await;
                if let Err(e) = stdout.write_all(msg.as_bytes()).await {
                    eprintln!("Failed to write response: {}", e);
                    break;
                }
                if let Err(e) = stdout.write_all(b"\n").await {
                    eprintln!("Failed to write newline: {}", e);
                    break;
                }
                if let Err(e) = stdout.flush().await {
                    eprintln!("Failed to flush stdout: {}", e);
                    break;
                }
            }
        });

        // Spawn task to send updates as notifications
        let response_tx_clone = response_tx.clone();
        tokio::spawn(async move {
            while let Some(update) = update_rx.recv().await {
                let notification = JsonRpcNotification {
                    jsonrpc: "2.0".to_string(),
                    method: "session/update".to_string(),
                    params: Some(serde_json::to_value(&update).unwrap()),
                };
                let msg = serde_json::to_string(&notification).unwrap();
                if response_tx_clone.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Main message loop
        while let Ok(Some(line)) = lines.next_line().await {
            if line.is_empty() {
                continue;
            }

            let response = self
                .handle_message(&line, update_tx.clone())
                .await;

            if let Some(resp) = response {
                let msg = serde_json::to_string(&resp)?;
                if response_tx.send(msg).await.is_err() {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_message(
        &self,
        line: &str,
        update_tx: mpsc::Sender<SessionUpdate>,
    ) -> Option<JsonRpcResponse> {
        // Try to parse as a request
        let msg: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: codes::PARSE_ERROR,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                });
            }
        };

        // Check if it's a request (has id and method) or response (has id but no method)
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(|m| m.as_str());

        // If it has method, it's a request
        if let Some(method) = method {
            let params = msg.get("params").cloned().unwrap_or(Value::Null);

            // If it has id, it expects a response
            if let Some(id) = id {
                let result = self.handle_request(method, params, update_tx).await;
                return Some(match result {
                    Ok(value) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(value),
                        error: None,
                    },
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: e.code(),
                            message: e.message(),
                            data: None,
                        }),
                    },
                });
            } else {
                // Notification - no response needed
                let _ = self.handle_request(method, params, update_tx).await;
                return None;
            }
        } else if let Some(id) = id {
            // This is a response to our request
            let id_str = id.to_string();
            let mut pending = self.pending_requests.lock().await;
            if let Some(tx) = pending.remove(&id_str) {
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: msg.get("result").cloned(),
                    error: msg.get("error").and_then(|e| serde_json::from_value(e.clone()).ok()),
                };
                let _ = tx.send(response);
            }
        }

        None
    }

    async fn handle_request(
        &self,
        method: &str,
        params: Value,
        update_tx: mpsc::Sender<SessionUpdate>,
    ) -> AcpResult<Value> {
        match method {
            "initialize" => {
                let params: InitializeParams = serde_json::from_value(params)
                    .map_err(|e| AcpError::InvalidParams(e.to_string()))?;
                let result = self.agent.initialize(params).await?;
                Ok(serde_json::to_value(result)?)
            }
            "authenticate" => {
                let params: AuthenticateParams = serde_json::from_value(params)
                    .map_err(|e| AcpError::InvalidParams(e.to_string()))?;
                let result = self.agent.authenticate(params).await?;
                Ok(serde_json::to_value(result)?)
            }
            "session/new" => {
                let params: SessionNewParams = serde_json::from_value(params)
                    .map_err(|e| AcpError::InvalidParams(e.to_string()))?;
                let result = self.agent.session_new(params).await?;
                Ok(serde_json::to_value(result)?)
            }
            "session/load" => {
                let params: SessionLoadParams = serde_json::from_value(params)
                    .map_err(|e| AcpError::InvalidParams(e.to_string()))?;
                let result = self.agent.session_load(params).await?;
                Ok(serde_json::to_value(result)?)
            }
            "session/prompt" => {
                let params: SessionPromptParams = serde_json::from_value(params)
                    .map_err(|e| AcpError::InvalidParams(e.to_string()))?;
                let result = self.agent.session_prompt(params, update_tx).await?;
                Ok(serde_json::to_value(result)?)
            }
            "session/cancel" => {
                let params: SessionCancelParams = serde_json::from_value(params)
                    .map_err(|e| AcpError::InvalidParams(e.to_string()))?;
                self.agent.session_cancel(params).await?;
                Ok(Value::Null)
            }
            _ => Err(AcpError::MethodNotFound(method.to_string())),
        }
    }

    /// Send a request to the client and wait for a response.
    ///
    /// Use this to request file operations or terminal access from the client.
    pub async fn send_request(
        &self,
        method: &str,
        params: Value,
        response_tx: &mpsc::Sender<String>,
    ) -> AcpResult<Value> {
        let id = {
            let mut next_id = self.next_request_id.lock().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let id_value = Value::Number(id.into());
        let id_str = id_value.to_string();

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id_str, tx);
        }

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(id_value),
            method: method.to_string(),
            params: Some(params),
        };

        let msg = serde_json::to_string(&request)?;
        response_tx
            .send(msg)
            .await
            .map_err(|e| AcpError::ChannelError(e.to_string()))?;

        let response = rx.await.map_err(|_| AcpError::ConnectionClosed)?;

        if let Some(error) = response.error {
            return Err(AcpError::InternalError(error.message));
        }

        Ok(response.result.unwrap_or(Value::Null))
    }
}

/// Helper functions for agents to request client operations.
pub mod client_requests {
    use super::*;

    /// Read a text file from the client.
    pub async fn read_file(
        server: &Server<impl Agent>,
        path: &str,
        response_tx: &mpsc::Sender<String>,
    ) -> AcpResult<String> {
        let params = serde_json::json!({ "path": path });
        let result = server.send_request("fs/read_text_file", params, response_tx).await?;
        let content = result["content"]
            .as_str()
            .ok_or_else(|| AcpError::InvalidParams("Missing content".to_string()))?;
        Ok(content.to_string())
    }

    /// Write a text file via the client.
    pub async fn write_file(
        server: &Server<impl Agent>,
        path: &str,
        content: &str,
        response_tx: &mpsc::Sender<String>,
    ) -> AcpResult<()> {
        let params = serde_json::json!({ "path": path, "content": content });
        server.send_request("fs/write_text_file", params, response_tx).await?;
        Ok(())
    }

    /// Create a terminal session via the client.
    pub async fn create_terminal(
        server: &Server<impl Agent>,
        cwd: &str,
        command: &str,
        response_tx: &mpsc::Sender<String>,
    ) -> AcpResult<String> {
        let params = serde_json::json!({ "cwd": cwd, "command": command });
        let result = server.send_request("terminal/create", params, response_tx).await?;
        let terminal_id = result["terminal_id"]
            .as_str()
            .ok_or_else(|| AcpError::InvalidParams("Missing terminal_id".to_string()))?;
        Ok(terminal_id.to_string())
    }

    /// Get terminal output.
    pub async fn get_terminal_output(
        server: &Server<impl Agent>,
        terminal_id: &str,
        response_tx: &mpsc::Sender<String>,
    ) -> AcpResult<(String, bool, Option<i32>)> {
        let params = serde_json::json!({ "terminal_id": terminal_id });
        let result = server.send_request("terminal/output", params, response_tx).await?;
        let output = result["output"].as_str().unwrap_or("").to_string();
        let exited = result["exited"].as_bool().unwrap_or(false);
        let exit_code = result["exit_code"].as_i64().map(|c| c as i32);
        Ok((output, exited, exit_code))
    }

    /// Kill a terminal.
    pub async fn kill_terminal(
        server: &Server<impl Agent>,
        terminal_id: &str,
        response_tx: &mpsc::Sender<String>,
    ) -> AcpResult<()> {
        let params = serde_json::json!({ "terminal_id": terminal_id });
        server.send_request("terminal/kill", params, response_tx).await?;
        Ok(())
    }
}
