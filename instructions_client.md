# ACP Client Implementation Guide

This guide provides complete instructions for implementing an ACP (Agent Client Protocol) client in Rust using the HeroACP library.

## Overview

An ACP client is typically an editor or IDE that:
- Spawns AI agents as subprocesses
- Communicates via JSON-RPC 2.0 over stdio (stdin/stdout)
- Sends prompts and receives streaming responses
- Handles agent requests for file operations and terminal access

## Architecture

```
┌─────────────────┐                     ┌─────────────────┐
│     Client      │ ─── stdin ───────>  │     Server      │
│  (Editor/IDE)   │ <── stdout ──────── │    (Agent)      │
└─────────────────┘                     └─────────────────┘
        │
        ├── Spawn agent subprocess
        ├── Send initialize request
        ├── Create sessions
        ├── Send prompts
        ├── Handle streaming updates
        └── Handle agent requests (file/terminal)
```

## Quick Start

### 1. Add Dependency

```toml
[dependencies]
heroacp = { path = "../heroacp" }
tokio = { version = "1", features = ["full"] }
```

### 2. Create a Client Connection

```rust
use heroacp::client::{Client, ClientCapabilities, ClientInfo};
use heroacp::protocol::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Spawn agent process and create client
    let mut client = Client::spawn("./acp-server").await?;

    // Initialize the connection
    let init_result = client.initialize(InitializeParams {
        protocol_version: "2025.1".to_string(),
        client_info: ClientInfo {
            name: "my-editor".to_string(),
            version: "1.0.0".to_string(),
        },
        capabilities: ClientCapabilities {
            text_files: true,
            terminal: true,
            embedded_context: false,
            audio: false,
            image: true,
            experimental: Default::default(),
        },
        working_directory: std::env::current_dir()?.to_string_lossy().to_string(),
        mcp_servers: vec![],
    }).await?;

    println!("Connected to: {} v{}",
        init_result.agent_info.name,
        init_result.agent_info.version
    );

    // Create a new session
    let session = client.session_new(SessionNewParams {
        session_id: "session-1".to_string(),
        mode: Some("agent".to_string()),
    }).await?;

    // Send a prompt
    let result = client.session_prompt(SessionPromptParams {
        session_id: session.session_id.clone(),
        content: vec![ContentBlock::Text {
            text: "Hello, can you help me with my code?".to_string(),
        }],
    }).await?;

    Ok(())
}
```

### 3. Handle Streaming Updates

```rust
use heroacp::client::UpdateHandler;

struct MyUpdateHandler;

impl UpdateHandler for MyUpdateHandler {
    fn on_agent_message(&self, session_id: &str, text: &str) {
        print!("{}", text);
    }

    fn on_agent_thought(&self, session_id: &str, text: &str) {
        eprintln!("[Thinking] {}", text);
    }

    fn on_tool_call(&self, session_id: &str, tool: &ToolCall) {
        println!("[Tool] {} - {}", tool.name, tool.id);
    }

    fn on_tool_update(&self, session_id: &str, update: &ToolCallUpdate) {
        println!("[Tool Result] {} - {:?}", update.id, update.status);
    }

    fn on_plan(&self, session_id: &str, plan: &Plan) {
        println!("[Plan] {} steps", plan.steps.len());
    }
}

// Register the handler
client.set_update_handler(Box::new(MyUpdateHandler));
```

## Detailed Implementation

### Spawning the Agent Process

```rust
use tokio::process::{Command, Child};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

struct AgentProcess {
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
}

impl AgentProcess {
    async fn spawn(command: &str, args: &[&str]) -> Result<Self, Error> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());

        Ok(Self { child, stdin, stdout })
    }

    async fn send(&mut self, msg: &str) -> Result<(), Error> {
        self.stdin.write_all(msg.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>, Error> {
        let mut line = String::new();
        match self.stdout.read_line(&mut line).await? {
            0 => Ok(None),
            _ => Ok(Some(line)),
        }
    }
}
```

### Managing Requests and Responses

```rust
use std::collections::HashMap;
use tokio::sync::oneshot;

struct PendingRequests {
    requests: HashMap<Value, oneshot::Sender<JsonRpcResponse>>,
    next_id: u64,
}

impl PendingRequests {
    fn new() -> Self {
        Self {
            requests: HashMap::new(),
            next_id: 1,
        }
    }

    fn create(&mut self) -> (u64, oneshot::Receiver<JsonRpcResponse>) {
        let id = self.next_id;
        self.next_id += 1;

        let (tx, rx) = oneshot::channel();
        self.requests.insert(Value::Number(id.into()), tx);

        (id, rx)
    }

    fn resolve(&mut self, id: &Value, response: JsonRpcResponse) {
        if let Some(tx) = self.requests.remove(id) {
            let _ = tx.send(response);
        }
    }
}
```

### Message Loop

```rust
async fn run_message_loop(&mut self) {
    loop {
        match self.process.receive().await {
            Ok(Some(line)) => {
                self.handle_message(&line).await;
            }
            Ok(None) => {
                // Agent process ended
                break;
            }
            Err(e) => {
                eprintln!("Error reading from agent: {}", e);
                break;
            }
        }
    }
}

async fn handle_message(&mut self, line: &str) {
    let msg: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse message: {}", e);
            return;
        }
    };

    if msg.get("id").is_some() && msg.get("method").is_some() {
        // This is a request from the agent
        self.handle_agent_request(msg).await;
    } else if msg.get("id").is_some() {
        // This is a response to our request
        self.handle_response(msg);
    } else if msg.get("method").is_some() {
        // This is a notification
        self.handle_notification(msg);
    }
}
```

### Handling Agent Requests

The agent may request file and terminal operations:

```rust
async fn handle_agent_request(&mut self, msg: Value) {
    let method = msg["method"].as_str().unwrap_or("");
    let id = msg["id"].clone();

    let result = match method {
        "fs/read_text_file" => {
            let path = msg["params"]["path"].as_str().unwrap();
            self.read_file(path).await
        }
        "fs/write_text_file" => {
            let path = msg["params"]["path"].as_str().unwrap();
            let content = msg["params"]["content"].as_str().unwrap();
            self.write_file(path, content).await
        }
        "terminal/create" => {
            let cwd = msg["params"]["cwd"].as_str().unwrap();
            let command = msg["params"]["command"].as_str().unwrap();
            self.create_terminal(cwd, command).await
        }
        "terminal/output" => {
            let terminal_id = msg["params"]["terminal_id"].as_str().unwrap();
            self.get_terminal_output(terminal_id).await
        }
        "terminal/wait_for_exit" => {
            let terminal_id = msg["params"]["terminal_id"].as_str().unwrap();
            self.wait_for_terminal(terminal_id).await
        }
        "terminal/kill" => {
            let terminal_id = msg["params"]["terminal_id"].as_str().unwrap();
            self.kill_terminal(terminal_id).await
        }
        _ => Err(Error::MethodNotFound),
    };

    let response = match result {
        Ok(value) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": value
        }),
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": e.code(),
                "message": e.message()
            }
        }),
    };

    self.send_message(&response.to_string()).await;
}
```

### File Operations

```rust
async fn read_file(&self, path: &str) -> Result<Value, Error> {
    // Validate path is absolute
    if !path.starts_with('/') {
        return Err(Error::InvalidParams("Path must be absolute".into()));
    }

    let content = tokio::fs::read_to_string(path).await
        .map_err(|_| Error::ResourceNotFound)?;

    Ok(json!({ "content": content }))
}

async fn write_file(&self, path: &str, content: &str) -> Result<Value, Error> {
    // Validate path is absolute
    if !path.starts_with('/') {
        return Err(Error::InvalidParams("Path must be absolute".into()));
    }

    // Optional: Check permissions, prompt user
    tokio::fs::write(path, content).await
        .map_err(|_| Error::PermissionDenied)?;

    Ok(json!({ "success": true }))
}
```

### Terminal Management

```rust
use std::collections::HashMap;
use tokio::process::{Child, Command};

struct TerminalManager {
    terminals: HashMap<String, Child>,
    next_id: u64,
}

impl TerminalManager {
    fn new() -> Self {
        Self {
            terminals: HashMap::new(),
            next_id: 1,
        }
    }

    async fn create(&mut self, cwd: &str, command: &str) -> Result<String, Error> {
        let id = format!("term_{}", self.next_id);
        self.next_id += 1;

        let child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|_| Error::InternalError)?;

        self.terminals.insert(id.clone(), child);
        Ok(id)
    }

    async fn kill(&mut self, terminal_id: &str) -> Result<(), Error> {
        if let Some(mut child) = self.terminals.remove(terminal_id) {
            child.kill().await.ok();
            Ok(())
        } else {
            Err(Error::ResourceNotFound)
        }
    }
}
```

### Handling Notifications

```rust
fn handle_notification(&mut self, msg: Value) {
    let method = msg["method"].as_str().unwrap_or("");

    match method {
        "session/update" => {
            let params = &msg["params"];
            let session_id = params["session_id"].as_str().unwrap_or("");
            let update_type = params["type"].as_str().unwrap_or("");

            match update_type {
                "agent_message_chunk" => {
                    if let Some(text) = params["data"]["text"].as_str() {
                        self.handler.on_agent_message(session_id, text);
                    }
                }
                "agent_thought_chunk" => {
                    if let Some(text) = params["data"]["text"].as_str() {
                        self.handler.on_agent_thought(session_id, text);
                    }
                }
                "tool_call" => {
                    // Handle tool call notification
                }
                "tool_call_update" => {
                    // Handle tool result
                }
                "plan" => {
                    // Handle plan update
                }
                _ => {}
            }
        }
        _ => {}
    }
}
```

## Complete Client Example

```rust
use heroacp::client::{Client, ClientCapabilities, ClientInfo, UpdateHandler};
use heroacp::protocol::*;
use tokio::io::{self, AsyncBufReadExt, BufReader};

struct TerminalHandler;

impl UpdateHandler for TerminalHandler {
    fn on_agent_message(&self, _session_id: &str, text: &str) {
        print!("{}", text);
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }

    fn on_agent_thought(&self, _session_id: &str, text: &str) {
        eprintln!("\x1b[90m[Thinking] {}\x1b[0m", text);
    }

    fn on_tool_call(&self, _session_id: &str, tool: &ToolCall) {
        eprintln!("\x1b[33m[Tool] {}\x1b[0m", tool.name);
    }

    fn on_tool_update(&self, _session_id: &str, update: &ToolCallUpdate) {
        eprintln!("\x1b[32m[Tool Done] {}\x1b[0m", update.id);
    }

    fn on_plan(&self, _session_id: &str, plan: &Plan) {
        eprintln!("\x1b[36m[Plan]\x1b[0m");
        for step in &plan.steps {
            let status = match step.status.as_str() {
                "completed" => "✓",
                "in_progress" => "→",
                _ => "○",
            };
            eprintln!("  {} {}", status, step.description);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let agent_command = args.get(1).map(|s| s.as_str()).unwrap_or("./acp-server");

    println!("Connecting to agent: {}", agent_command);

    // Create client and connect to agent
    let mut client = Client::spawn(agent_command).await?;

    // Set up update handler
    client.set_update_handler(Box::new(TerminalHandler));

    // Initialize connection
    let cwd = std::env::current_dir()?.to_string_lossy().to_string();
    let init_result = client.initialize(InitializeParams {
        protocol_version: "2025.1".to_string(),
        client_info: ClientInfo {
            name: "acp-client".to_string(),
            version: "1.0.0".to_string(),
        },
        capabilities: ClientCapabilities {
            text_files: true,
            terminal: true,
            embedded_context: false,
            audio: false,
            image: true,
            experimental: Default::default(),
        },
        working_directory: cwd,
        mcp_servers: vec![],
    }).await?;

    println!("Connected to: {} v{}",
        init_result.agent_info.name,
        init_result.agent_info.version
    );

    if let Some(instructions) = &init_result.instructions {
        println!("Agent: {}", instructions);
    }

    // Create session
    let session = client.session_new(SessionNewParams {
        session_id: uuid::Uuid::new_v4().to_string(),
        mode: Some("agent".to_string()),
    }).await?;

    println!("\nSession started. Type your prompts (Ctrl+C to exit):\n");

    // Interactive REPL
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        if line.is_empty() {
            continue;
        }

        // Send prompt
        let result = client.session_prompt(SessionPromptParams {
            session_id: session.session_id.clone(),
            content: vec![ContentBlock::Text { text: line }],
        }).await;

        match result {
            Ok(_) => println!("\n"),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(())
}
```

## Connecting to Goose

To connect to the Goose AI agent:

```rust
// Goose uses the same ACP protocol
let mut client = Client::spawn("goose").await?;

// Or with specific arguments
let mut client = Client::spawn_with_args("goose", &["--mode", "acp"]).await?;
```

## Permission Model

Implement a permission system for sensitive operations:

```rust
enum Permission {
    Allow,
    Deny,
    Ask,
}

struct PermissionManager {
    file_write: Permission,
    terminal: Permission,
}

impl PermissionManager {
    async fn check_file_write(&self, path: &str) -> bool {
        match self.file_write {
            Permission::Allow => true,
            Permission::Deny => false,
            Permission::Ask => {
                // Prompt user
                self.prompt_user(&format!("Allow writing to {}?", path)).await
            }
        }
    }
}
```

## Error Handling

Handle various error scenarios:

```rust
#[derive(Debug)]
enum ClientError {
    SpawnFailed(std::io::Error),
    ConnectionClosed,
    ProtocolError(String),
    Timeout,
    AgentError { code: i32, message: String },
}

impl Client {
    async fn send_request<T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<T, ClientError> {
        let (id, rx) = self.pending.create();

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        self.send_message(&request.to_string()).await?;

        // Wait for response with timeout
        let response = tokio::time::timeout(
            Duration::from_secs(30),
            rx
        ).await
            .map_err(|_| ClientError::Timeout)?
            .map_err(|_| ClientError::ConnectionClosed)?;

        if let Some(error) = response.error {
            return Err(ClientError::AgentError {
                code: error.code,
                message: error.message,
            });
        }

        serde_json::from_value(response.result.unwrap_or(Value::Null))
            .map_err(|e| ClientError::ProtocolError(e.to_string()))
    }
}
```

## Best Practices

1. **Handle agent crashes gracefully** - detect when the agent process exits
2. **Implement timeouts** for all requests
3. **Validate paths** before file operations
4. **Stream output** to show real-time responses
5. **Support cancellation** with Ctrl+C or cancel button
6. **Log agent stderr** for debugging
7. **Clean up resources** when session ends

## Resources

- [ACP Specification](./specs.md)
- [Server Implementation Guide](./instructions_server.md)
- [Goose Documentation](https://block.github.io/goose/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
