# HeroACP

A complete Rust implementation of the Agent Client Protocol (ACP) - the standard for AI coding agent and editor integration.

## Overview

HeroACP provides:

- **Protocol Types**: Complete JSON-RPC 2.0 message definitions
- **Server SDK**: Build ACP-compliant AI coding agents
- **Client SDK**: Build ACP-compliant editors/IDEs
- **Example Server**: A "bogus" demo agent for testing
- **Example Client**: A terminal-based ACP client

## What is ACP?

The Agent Client Protocol (ACP) is a JSON-RPC 2.0 based protocol that standardizes communication between code editors (clients) and AI coding agents (servers). It's designed to be:

- **Bidirectional**: Both clients and agents can send requests
- **Streaming**: Real-time response streaming support
- **Stateful**: Session-based conversation management
- **Standard I/O**: Uses stdin/stdout for subprocess communication

ACP is to AI coding agents what LSP (Language Server Protocol) is to language tooling.

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/heroacp/heroacp.git
cd heroacp

# Build the project
./install.sh

# Or manually with cargo
cargo build --release
```

### Run the Demo

```bash
# Run the client with the built-in bogus agent
./run.sh

# Or connect to Goose
./run_goose.sh

# Or specify any agent
./target/release/acp-client <agent-command>
```

## Project Structure

```
heroacp/
├── src/
│   ├── lib.rs              # Library root
│   ├── protocol/           # Protocol types
│   │   ├── mod.rs
│   │   ├── messages.rs     # JSON-RPC messages
│   │   ├── types.rs        # Common types
│   │   └── errors.rs       # Error definitions
│   ├── server/             # Server SDK
│   │   └── mod.rs
│   ├── client/             # Client SDK
│   │   └── mod.rs
│   └── bin/
│       ├── server.rs       # Example bogus agent
│       └── client.rs       # Example client
├── specs.md                # ACP Specification
├── instructions_server.md  # Server implementation guide
├── instructions_client.md  # Client implementation guide
├── install.sh              # Installation script
├── run.sh                  # Run demo script
└── run_goose.sh           # Run with Goose
```

## Using the SDK

### Building an Agent (Server)

```rust
use heroacp::server::{Agent, Server};
use heroacp::protocol::*;
use async_trait::async_trait;
use tokio::sync::mpsc;

struct MyAgent;

#[async_trait]
impl Agent for MyAgent {
    async fn initialize(&self, params: InitializeParams) -> AcpResult<InitializeResult> {
        Ok(InitializeResult {
            agent_info: AgentInfo {
                name: "my-agent".to_string(),
                version: "1.0.0".to_string(),
            },
            capabilities: AgentCapabilities {
                streaming: true,
                ..Default::default()
            },
            instructions: Some("I'm a helpful AI assistant.".to_string()),
        })
    }

    async fn session_new(&self, params: SessionNewParams) -> AcpResult<SessionNewResult> {
        Ok(SessionNewResult { session_id: params.session_id })
    }

    async fn session_prompt(
        &self,
        params: SessionPromptParams,
        update_tx: mpsc::Sender<SessionUpdate>,
    ) -> AcpResult<SessionPromptResult> {
        // Stream response chunks
        update_tx.send(SessionUpdate {
            session_id: params.session_id.clone(),
            update_type: SessionUpdateType::AgentMessageChunk {
                text: "Hello! ".to_string(),
            },
        }).await.ok();

        Ok(SessionPromptResult { status: "ok".to_string() })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new(MyAgent);
    server.run().await?;
    Ok(())
}
```

### Building a Client

```rust
use heroacp::client::{Client, UpdateHandler};
use heroacp::protocol::*;

struct MyHandler;

impl UpdateHandler for MyHandler {
    fn on_agent_message(&self, _session_id: &str, text: &str) {
        print!("{}", text);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::spawn("./my-agent").await?;
    client.set_update_handler(Box::new(MyHandler)).await;

    // Initialize
    let init = client.initialize(InitializeParams {
        protocol_version: "2025.1".to_string(),
        client_info: ClientInfo {
            name: "my-client".to_string(),
            version: "1.0.0".to_string(),
        },
        capabilities: ClientCapabilities::default(),
        working_directory: "/home/user".to_string(),
        mcp_servers: vec![],
    }).await?;

    // Create session
    let session = client.session_new(SessionNewParams {
        session_id: "session-1".to_string(),
        mode: Some("agent".to_string()),
    }).await?;

    // Send prompt
    client.session_prompt(SessionPromptParams {
        session_id: session.session_id,
        content: vec![ContentBlock::Text { text: "Hello!".to_string() }],
    }).await?;

    Ok(())
}
```

## Protocol Messages

### Session Flow

1. **initialize**: Capability negotiation
2. **session/new**: Create conversation context
3. **session/prompt**: Send user messages
4. **session/update**: Receive agent responses (notifications)
5. **session/cancel**: Interrupt processing

### Agent Requests (to Client)

- `fs/read_text_file`: Read a file
- `fs/write_text_file`: Write a file
- `terminal/create`: Create terminal session
- `terminal/output`: Get terminal output
- `terminal/kill`: Kill terminal

## Compatible Agents

- [Goose](https://block.github.io/goose/) - Block's AI coding agent
- [Claude Code](https://github.com/anthropics/claude-code) - Anthropic's CLI agent
- [Codex CLI](https://github.com/openai/codex) - OpenAI's coding agent
- [Gemini CLI](https://github.com/google/gemini-cli) - Google's AI agent

## Compatible Editors

- [Zed](https://zed.dev) - High-performance code editor
- [Neovim](https://neovim.io) - With ACP plugin
- [JetBrains IDEs](https://www.jetbrains.com) - Via AI Assistant

## Documentation

- [ACP Specification](./specs.md) - Full protocol specification
- [Server Guide](./instructions_server.md) - How to build an agent
- [Client Guide](./instructions_client.md) - How to build a client

## Resources

- [Official ACP Website](https://agentclientprotocol.com)
- [Goose Blog: Intro to ACP](https://block.github.io/goose/blog/2025/10/24/intro-to-agent-client-protocol-acp/)
- [agent-client-protocol crate](https://docs.rs/agent-client-protocol)

## License

Apache-2.0
