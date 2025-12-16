//! Client SDK for connecting to ACP agents.
//!
//! This module provides the infrastructure for building ACP-compliant clients
//! (editors/IDEs) that can communicate with AI coding agents.
//!
//! # Example
//!
//! ```rust,no_run
//! use heroacp::client::{Client, UpdateHandler};
//! use heroacp::protocol::*;
//!
//! struct MyHandler;
//!
//! impl UpdateHandler for MyHandler {
//!     fn on_agent_message(&self, session_id: &str, text: &str) {
//!         print!("{}", text);
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = Client::spawn("./agent").await.unwrap();
//!     client.set_update_handler(Box::new(MyHandler));
//!     // Use client...
//! }
//! ```

use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::time::{timeout, Duration};

use crate::protocol::*;

/// Handler for session updates from the agent.
pub trait UpdateHandler: Send + Sync {
    /// Called when the agent sends a message chunk.
    fn on_agent_message(&self, _session_id: &str, _text: &str) {}

    /// Called when the agent sends a thought chunk.
    fn on_agent_thought(&self, _session_id: &str, _text: &str) {}

    /// Called when the agent makes a tool call.
    fn on_tool_call(&self, _session_id: &str, _tool: &ToolCall) {}

    /// Called when a tool call is updated.
    fn on_tool_update(&self, _session_id: &str, _update: &ToolCallUpdate) {}

    /// Called when the agent updates its plan.
    fn on_plan(&self, _session_id: &str, _plan: &Plan) {}

    /// Called when the agent changes mode.
    fn on_mode_change(&self, _session_id: &str, _mode: &str) {}

    /// Called when the agent is done.
    fn on_done(&self, _session_id: &str) {}
}

/// Default no-op update handler.
struct NoOpHandler;
impl UpdateHandler for NoOpHandler {}

/// ACP client for connecting to agents.
pub struct Client {
    /// The child process running the agent.
    child: Child,
    /// Channel to send messages to the agent.
    message_tx: mpsc::Sender<String>,
    /// Pending requests waiting for responses.
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<JsonRpcResponse>>>>,
    /// Next request ID.
    next_id: Arc<Mutex<u64>>,
    /// Update handler.
    update_handler: Arc<RwLock<Box<dyn UpdateHandler>>>,
    /// Terminal manager (kept alive for async task).
    #[allow(dead_code)]
    terminals: Arc<Mutex<TerminalManager>>,
    /// Working directory.
    working_directory: String,
    /// Handle to the message loop task.
    _message_loop_handle: tokio::task::JoinHandle<()>,
}

struct TerminalManager {
    terminals: HashMap<String, Child>,
    outputs: HashMap<String, String>,
    next_id: u64,
}

impl TerminalManager {
    fn new() -> Self {
        Self {
            terminals: HashMap::new(),
            outputs: HashMap::new(),
            next_id: 1,
        }
    }

    async fn create(&mut self, cwd: &str, command: &str) -> AcpResult<String> {
        let id = format!("term_{}", self.next_id);
        self.next_id += 1;

        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(AcpError::IoError)?;

        self.terminals.insert(id.clone(), child);
        self.outputs.insert(id.clone(), String::new());
        Ok(id)
    }

    async fn get_output(&mut self, terminal_id: &str) -> AcpResult<(String, bool, Option<i32>)> {
        let child = self
            .terminals
            .get_mut(terminal_id)
            .ok_or_else(|| AcpError::ResourceNotFound(terminal_id.to_string()))?;

        // Check if process has exited
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = self.outputs.get(terminal_id).cloned().unwrap_or_default();
                Ok((output, true, status.code()))
            }
            Ok(None) => {
                let output = self.outputs.get(terminal_id).cloned().unwrap_or_default();
                Ok((output, false, None))
            }
            Err(e) => Err(AcpError::IoError(e)),
        }
    }

    async fn kill(&mut self, terminal_id: &str) -> AcpResult<()> {
        if let Some(mut child) = self.terminals.remove(terminal_id) {
            child.kill().await.ok();
            self.outputs.remove(terminal_id);
            Ok(())
        } else {
            Err(AcpError::ResourceNotFound(terminal_id.to_string()))
        }
    }

    async fn release(&mut self, terminal_id: &str) -> AcpResult<()> {
        self.terminals.remove(terminal_id);
        self.outputs.remove(terminal_id);
        Ok(())
    }
}

impl Client {
    /// Spawn a new agent process and create a client.
    pub async fn spawn(command: &str) -> AcpResult<Self> {
        Self::spawn_with_args(command, &[]).await
    }

    /// Spawn a new agent process with arguments.
    pub async fn spawn_with_args(command: &str, args: &[&str]) -> AcpResult<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(AcpError::IoError)?;

        let stdin = child.stdin.take().ok_or_else(|| {
            AcpError::InternalError("Failed to get stdin".to_string())
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            AcpError::InternalError("Failed to get stdout".to_string())
        })?;

        let (message_tx, mut message_rx) = mpsc::channel::<String>(100);
        let pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<JsonRpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let update_handler: Arc<RwLock<Box<dyn UpdateHandler>>> =
            Arc::new(RwLock::new(Box::new(NoOpHandler)));
        let terminals = Arc::new(Mutex::new(TerminalManager::new()));

        // Clone for the message loop
        let pending_clone = pending_requests.clone();
        let handler_clone = update_handler.clone();
        let terminals_clone = terminals.clone();
        let message_tx_clone = message_tx.clone();

        // Spawn writer task
        let stdin = Arc::new(Mutex::new(stdin));
        let stdin_clone = stdin.clone();
        tokio::spawn(async move {
            while let Some(msg) = message_rx.recv().await {
                let mut stdin = stdin_clone.lock().await;
                if stdin.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.write_all(b"\n").await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        // Spawn reader task
        let message_loop_handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if line.is_empty() {
                    continue;
                }

                let msg: Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Failed to parse message: {}", e);
                        continue;
                    }
                };

                // Check if it's a request from the agent
                if msg.get("method").is_some() && msg.get("id").is_some() {
                    // Handle agent request
                    let method = msg["method"].as_str().unwrap_or("");
                    let id = msg["id"].clone();
                    let params = msg.get("params").cloned().unwrap_or(Value::Null);

                    let result = Self::handle_agent_request(
                        method,
                        &params,
                        &terminals_clone,
                    )
                    .await;

                    let response = match result {
                        Ok(value) => serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": value
                        }),
                        Err(e) => serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": e.code(),
                                "message": e.message()
                            }
                        }),
                    };

                    let _ = message_tx_clone.send(response.to_string()).await;
                } else if msg.get("method").is_some() {
                    // Notification from agent
                    let method = msg["method"].as_str().unwrap_or("");
                    if method == "session/update" {
                        if let Some(params) = msg.get("params") {
                            let session_id = params["session_id"].as_str().unwrap_or("");
                            let update_type = params["type"].as_str().unwrap_or("");

                            let handler = handler_clone.read().await;
                            match update_type {
                                "agent_message_chunk" => {
                                    if let Some(text) = params["data"]["text"].as_str() {
                                        handler.on_agent_message(session_id, text);
                                    }
                                }
                                "agent_thought_chunk" => {
                                    if let Some(text) = params["data"]["text"].as_str() {
                                        handler.on_agent_thought(session_id, text);
                                    }
                                }
                                "tool_call" => {
                                    if let Ok(tool) =
                                        serde_json::from_value::<ToolCall>(params["data"].clone())
                                    {
                                        handler.on_tool_call(session_id, &tool);
                                    }
                                }
                                "tool_call_update" => {
                                    if let Ok(update) = serde_json::from_value::<ToolCallUpdate>(
                                        params["data"].clone(),
                                    ) {
                                        handler.on_tool_update(session_id, &update);
                                    }
                                }
                                "plan" => {
                                    if let Ok(plan) =
                                        serde_json::from_value::<Plan>(params["data"].clone())
                                    {
                                        handler.on_plan(session_id, &plan);
                                    }
                                }
                                "mode_change" => {
                                    if let Some(mode) = params["data"]["mode"].as_str() {
                                        handler.on_mode_change(session_id, mode);
                                    }
                                }
                                "done" => {
                                    handler.on_done(session_id);
                                }
                                _ => {}
                            }
                        }
                    }
                } else if msg.get("id").is_some() {
                    // Response to our request
                    let id_str = msg["id"].to_string();
                    let mut pending = pending_clone.lock().await;
                    if let Some(tx) = pending.remove(&id_str) {
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: msg["id"].clone(),
                            result: msg.get("result").cloned(),
                            error: msg
                                .get("error")
                                .and_then(|e| serde_json::from_value(e.clone()).ok()),
                        };
                        let _ = tx.send(response);
                    }
                }
            }
        });

        let working_directory = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/".to_string());

        Ok(Self {
            child,
            message_tx,
            pending_requests,
            next_id: Arc::new(Mutex::new(1)),
            update_handler,
            terminals,
            working_directory,
            _message_loop_handle: message_loop_handle,
        })
    }

    async fn handle_agent_request(
        method: &str,
        params: &Value,
        terminals: &Arc<Mutex<TerminalManager>>,
    ) -> AcpResult<Value> {
        match method {
            "fs/read_text_file" => {
                let path = params["path"]
                    .as_str()
                    .ok_or_else(|| AcpError::InvalidParams("Missing path".to_string()))?;

                // Validate absolute path
                if !path.starts_with('/') {
                    return Err(AcpError::InvalidParams(
                        "Path must be absolute".to_string(),
                    ));
                }

                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|_| AcpError::ResourceNotFound(path.to_string()))?;

                Ok(serde_json::json!({ "content": content }))
            }
            "fs/write_text_file" => {
                let path = params["path"]
                    .as_str()
                    .ok_or_else(|| AcpError::InvalidParams("Missing path".to_string()))?;
                let content = params["content"]
                    .as_str()
                    .ok_or_else(|| AcpError::InvalidParams("Missing content".to_string()))?;

                // Validate absolute path
                if !path.starts_with('/') {
                    return Err(AcpError::InvalidParams(
                        "Path must be absolute".to_string(),
                    ));
                }

                tokio::fs::write(path, content)
                    .await
                    .map_err(|_| AcpError::PermissionDenied(path.to_string()))?;

                Ok(serde_json::json!({ "success": true }))
            }
            "terminal/create" => {
                let cwd = params["cwd"]
                    .as_str()
                    .ok_or_else(|| AcpError::InvalidParams("Missing cwd".to_string()))?;
                let command = params["command"]
                    .as_str()
                    .ok_or_else(|| AcpError::InvalidParams("Missing command".to_string()))?;

                let mut term_mgr = terminals.lock().await;
                let terminal_id = term_mgr.create(cwd, command).await?;

                Ok(serde_json::json!({ "terminal_id": terminal_id }))
            }
            "terminal/output" => {
                let terminal_id = params["terminal_id"]
                    .as_str()
                    .ok_or_else(|| AcpError::InvalidParams("Missing terminal_id".to_string()))?;

                let mut term_mgr = terminals.lock().await;
                let (output, exited, exit_code) = term_mgr.get_output(terminal_id).await?;

                Ok(serde_json::json!({
                    "output": output,
                    "exited": exited,
                    "exit_code": exit_code
                }))
            }
            "terminal/wait_for_exit" => {
                let terminal_id = params["terminal_id"]
                    .as_str()
                    .ok_or_else(|| AcpError::InvalidParams("Missing terminal_id".to_string()))?;

                // Wait for terminal to exit (with timeout)
                let term_id = terminal_id.to_string();
                let terminals = terminals.clone();

                let result = timeout(Duration::from_secs(300), async {
                    loop {
                        let mut term_mgr = terminals.lock().await;
                        let (output, exited, exit_code) = term_mgr.get_output(&term_id).await?;
                        if exited {
                            return Ok::<_, AcpError>((output, exit_code.unwrap_or(-1)));
                        }
                        drop(term_mgr);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                })
                .await
                .map_err(|_| AcpError::Timeout)?;

                let (output, exit_code) = result?;
                Ok(serde_json::json!({
                    "output": output,
                    "exit_code": exit_code
                }))
            }
            "terminal/kill" => {
                let terminal_id = params["terminal_id"]
                    .as_str()
                    .ok_or_else(|| AcpError::InvalidParams("Missing terminal_id".to_string()))?;

                let mut term_mgr = terminals.lock().await;
                term_mgr.kill(terminal_id).await?;

                Ok(serde_json::json!({ "success": true }))
            }
            "terminal/release" => {
                let terminal_id = params["terminal_id"]
                    .as_str()
                    .ok_or_else(|| AcpError::InvalidParams("Missing terminal_id".to_string()))?;

                let mut term_mgr = terminals.lock().await;
                term_mgr.release(terminal_id).await?;

                Ok(serde_json::json!({ "success": true }))
            }
            _ => Err(AcpError::MethodNotFound(method.to_string())),
        }
    }

    /// Set the update handler for session updates.
    pub async fn set_update_handler(&self, handler: Box<dyn UpdateHandler>) {
        let mut h = self.update_handler.write().await;
        *h = handler;
    }

    /// Send a request and wait for a response.
    async fn send_request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Value,
    ) -> AcpResult<T> {
        let id = {
            let mut next_id = self.next_id.lock().await;
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
        self.message_tx
            .send(msg)
            .await
            .map_err(|e| AcpError::ChannelError(e.to_string()))?;

        let response = timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| AcpError::Timeout)?
            .map_err(|_| AcpError::ConnectionClosed)?;

        if let Some(error) = response.error {
            return Err(AcpError::InternalError(error.message));
        }

        let result = response.result.unwrap_or(Value::Null);
        serde_json::from_value(result).map_err(|e| AcpError::InvalidParams(e.to_string()))
    }

    /// Initialize the connection with the agent.
    pub async fn initialize(&self, params: InitializeParams) -> AcpResult<InitializeResult> {
        self.send_request("initialize", serde_json::to_value(params)?).await
    }

    /// Create a new session.
    pub async fn session_new(&self, params: SessionNewParams) -> AcpResult<SessionNewResult> {
        self.send_request("session/new", serde_json::to_value(params)?).await
    }

    /// Load an existing session.
    pub async fn session_load(&self, params: SessionLoadParams) -> AcpResult<SessionLoadResult> {
        self.send_request("session/load", serde_json::to_value(params)?).await
    }

    /// Send a prompt to the agent.
    pub async fn session_prompt(
        &self,
        params: SessionPromptParams,
    ) -> AcpResult<SessionPromptResult> {
        self.send_request("session/prompt", serde_json::to_value(params)?).await
    }

    /// Cancel the current session operation.
    pub async fn session_cancel(&self, params: SessionCancelParams) -> AcpResult<()> {
        let _: Value = self
            .send_request("session/cancel", serde_json::to_value(params)?)
            .await?;
        Ok(())
    }

    /// Get the working directory.
    pub fn working_directory(&self) -> &str {
        &self.working_directory
    }

    /// Check if the agent process is still running.
    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        }
    }

    /// Kill the agent process.
    pub async fn kill(&mut self) -> AcpResult<()> {
        self.child.kill().await.map_err(AcpError::IoError)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // Try to kill the child process when the client is dropped
        let _ = self.child.start_kill();
    }
}

/// Create client capabilities with common defaults.
pub fn default_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        text_files: true,
        terminal: true,
        embedded_context: false,
        audio: false,
        image: true,
        experimental: HashMap::new(),
    }
}
