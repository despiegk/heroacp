# ACP Server Implementation Guide

This guide provides complete instructions for implementing an ACP (Agent Client Protocol) server (agent) in Rust using the HeroACP library.

## Overview

An ACP server is an AI coding agent that:
- Runs as a subprocess spawned by the client (editor/IDE)
- Communicates via JSON-RPC 2.0 over stdio (stdin/stdout)
- Processes prompts and streams responses back to the client
- Can request file operations and terminal access from the client

## Architecture

```
┌─────────────────┐                     ┌─────────────────┐
│     Client      │ ─── stdin ───────>  │     Server      │
│  (Editor/IDE)   │ <── stdout ──────── │    (Agent)      │
└─────────────────┘                     └─────────────────┘
```

## Quick Start

### 1. Add Dependency

```toml
[dependencies]
heroacp = { path = "../heroacp" }
tokio = { version = "1", features = ["full"] }
```

### 2. Implement the Agent Trait

```rust
use heroacp::server::{Agent, AgentInfo, AgentCapabilities, Server};
use heroacp::protocol::{
    ContentBlock, SessionUpdate, SessionUpdateType,
    InitializeParams, InitializeResult, SessionNewParams, SessionNewResult,
    SessionPromptParams, SessionPromptResult,
};
use async_trait::async_trait;

struct MyAgent;

#[async_trait]
impl Agent for MyAgent {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult, Error> {
        Ok(InitializeResult {
            agent_info: AgentInfo {
                name: "my-agent".to_string(),
                version: "1.0.0".to_string(),
            },
            capabilities: AgentCapabilities {
                streaming: true,
                audio: false,
                image: true,
                supported_modes: vec!["agent".to_string(), "ask".to_string()],
            },
            instructions: Some("I am an AI coding assistant.".to_string()),
        })
    }

    async fn session_new(&self, params: SessionNewParams) -> Result<SessionNewResult, Error> {
        // Create a new session
        Ok(SessionNewResult {
            session_id: params.session_id,
        })
    }

    async fn session_prompt(
        &self,
        params: SessionPromptParams,
        update_tx: mpsc::Sender<SessionUpdate>,
    ) -> Result<SessionPromptResult, Error> {
        // Process the prompt and stream responses

        // Send a thought
        update_tx.send(SessionUpdate {
            session_id: params.session_id.clone(),
            update_type: SessionUpdateType::AgentThoughtChunk {
                text: "Let me think about this...".to_string(),
            },
        }).await?;

        // Send response chunks
        update_tx.send(SessionUpdate {
            session_id: params.session_id.clone(),
            update_type: SessionUpdateType::AgentMessageChunk {
                text: "Here is my response.".to_string(),
            },
        }).await?;

        Ok(SessionPromptResult { status: "ok".to_string() })
    }
}
```

### 3. Run the Server

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = MyAgent;
    let server = Server::new(agent);
    server.run().await?;
    Ok(())
}
```

## Detailed Implementation

### Transport Layer

The server reads NDJSON from stdin and writes to stdout:

```rust
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt, BufReader};

async fn read_message() -> Option<String> {
    let mut reader = BufReader::new(stdin());
    let mut line = String::new();
    match reader.read_line(&mut line).await {
        Ok(0) => None,  // EOF
        Ok(_) => Some(line),
        Err(_) => None,
    }
}

async fn write_message(msg: &str) {
    let mut stdout = stdout();
    stdout.write_all(msg.as_bytes()).await.unwrap();
    stdout.write_all(b"\n").await.unwrap();
    stdout.flush().await.unwrap();
}
```

### Message Handling

Parse incoming JSON-RPC messages:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}
```

### Handling Requests

Route incoming requests to appropriate handlers:

```rust
async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => {
            let params: InitializeParams = serde_json::from_value(
                request.params.unwrap_or(Value::Null)
            ).unwrap();
            let result = self.agent.initialize(params).await;
            // ... build response
        }
        "session/new" => {
            // Handle new session
        }
        "session/prompt" => {
            // Handle prompt - spawn task for async processing
        }
        "session/cancel" => {
            // Handle cancellation
        }
        _ => {
            // Method not found error
        }
    }
}
```

### Streaming Updates

Send notifications for streaming updates:

```rust
async fn send_update(&self, update: SessionUpdate) {
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "session/update".to_string(),
        params: serde_json::to_value(&update).unwrap(),
    };
    let msg = serde_json::to_string(&notification).unwrap();
    write_message(&msg).await;
}
```

### Requesting Client Operations

The agent can request file and terminal operations from the client:

```rust
// Read a file
async fn read_file(&self, path: &str) -> Result<String, Error> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(Value::Number(self.next_id().into())),
        method: "fs/read_text_file".to_string(),
        params: Some(json!({ "path": path })),
    };

    let response = self.send_request(request).await?;
    // Parse and return content
}

// Write a file
async fn write_file(&self, path: &str, content: &str) -> Result<(), Error> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(Value::Number(self.next_id().into())),
        method: "fs/write_text_file".to_string(),
        params: Some(json!({ "path": path, "content": content })),
    };

    self.send_request(request).await?;
    Ok(())
}

// Create terminal
async fn create_terminal(&self, cwd: &str, command: &str) -> Result<String, Error> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(Value::Number(self.next_id().into())),
        method: "terminal/create".to_string(),
        params: Some(json!({ "cwd": cwd, "command": command })),
    };

    let response = self.send_request(request).await?;
    // Return terminal_id
}
```

## Complete Server Example

```rust
use heroacp::server::{Agent, Server};
use heroacp::protocol::*;
use async_trait::async_trait;
use tokio::sync::mpsc;

struct CodingAssistant {
    // Your agent state
}

impl CodingAssistant {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Agent for CodingAssistant {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult, Error> {
        eprintln!("Initializing with protocol version: {}", params.protocol_version);

        Ok(InitializeResult {
            agent_info: AgentInfo {
                name: "coding-assistant".to_string(),
                version: "1.0.0".to_string(),
            },
            capabilities: AgentCapabilities {
                streaming: true,
                audio: false,
                image: true,
                supported_modes: vec!["agent".to_string()],
            },
            instructions: Some("I help with coding tasks.".to_string()),
        })
    }

    async fn session_new(&self, params: SessionNewParams) -> Result<SessionNewResult, Error> {
        eprintln!("Creating session: {}", params.session_id);
        Ok(SessionNewResult {
            session_id: params.session_id,
        })
    }

    async fn session_prompt(
        &self,
        params: SessionPromptParams,
        update_tx: mpsc::Sender<SessionUpdate>,
    ) -> Result<SessionPromptResult, Error> {
        let session_id = params.session_id.clone();

        // Extract text from content blocks
        let prompt_text: String = params.content.iter()
            .filter_map(|block| {
                if let ContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        eprintln!("Received prompt: {}", prompt_text);

        // Send thinking update
        let _ = update_tx.send(SessionUpdate {
            session_id: session_id.clone(),
            update_type: SessionUpdateType::AgentThoughtChunk {
                text: "Analyzing your request...".to_string(),
            },
        }).await;

        // Simulate processing and stream response
        for chunk in ["I understand ", "your request. ", "Here's my response."] {
            let _ = update_tx.send(SessionUpdate {
                session_id: session_id.clone(),
                update_type: SessionUpdateType::AgentMessageChunk {
                    text: chunk.to_string(),
                },
            }).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(SessionPromptResult { status: "ok".to_string() })
    }

    async fn session_cancel(&self, params: SessionCancelParams) -> Result<(), Error> {
        eprintln!("Cancelling session: {}", params.session_id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Starting ACP server...");

    let agent = CodingAssistant::new();
    let server = Server::new(agent);

    server.run().await?;

    Ok(())
}
```

## Error Handling

Return proper JSON-RPC errors:

```rust
#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// Standard error codes
const PARSE_ERROR: i32 = -32700;
const INVALID_REQUEST: i32 = -32600;
const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;

// ACP-specific error codes
const RESOURCE_NOT_FOUND: i32 = -32001;
const PERMISSION_DENIED: i32 = -32002;
const INVALID_STATE: i32 = -32003;
const CAPABILITY_NOT_SUPPORTED: i32 = -32004;
```

## Logging

Use stderr for logging (stdout is reserved for protocol messages):

```rust
eprintln!("Debug: Processing request...");
```

## Testing

Test your server with the provided client:

```bash
# Build the server
cargo build --release --bin acp-server

# Run with the test client
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocol_version":"2025.1","client_info":{"name":"test","version":"1.0"},"capabilities":{},"working_directory":"/"}}' | ./target/release/acp-server
```

## Best Practices

1. **Always use absolute paths** for file operations
2. **Stream responses** incrementally for better UX
3. **Handle cancellation** gracefully
4. **Log to stderr**, keep stdout for protocol only
5. **Validate capabilities** before using features
6. **Return meaningful errors** with proper codes
7. **Handle EOF** to shut down cleanly

## Integration with AI Models

For integrating with actual AI models (OpenAI, Anthropic, etc.):

```rust
impl CodingAssistant {
    async fn call_llm(&self, prompt: &str) -> Result<String, Error> {
        // Call your preferred LLM API
        // Stream tokens back via update_tx
    }
}
```

## Resources

- [ACP Specification](./specs.md)
- [Rust SDK Documentation](https://docs.rs/agent-client-protocol)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
