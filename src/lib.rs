//! # HeroACP - Agent Client Protocol Implementation
//!
//! HeroACP is a Rust implementation of the Agent Client Protocol (ACP),
//! a JSON-RPC 2.0 based protocol for communication between code editors
//! and AI coding agents.
//!
//! ## Features
//!
//! - **Server SDK**: Build ACP-compliant AI agents
//! - **Client SDK**: Build ACP-compliant editors/clients
//! - **Protocol Types**: Complete message type definitions
//! - **Async/Await**: Built on Tokio for async operations
//!
//! ## Quick Start - Server
//!
//! ```rust,ignore
//! use heroacp::server::{Agent, Server};
//! use heroacp::protocol::*;
//! use async_trait::async_trait;
//! use tokio::sync::mpsc;
//!
//! struct MyAgent;
//!
//! #[async_trait]
//! impl Agent for MyAgent {
//!     async fn initialize(&self, params: InitializeParams) -> AcpResult<InitializeResult> {
//!         Ok(InitializeResult {
//!             agent_info: AgentInfo { name: "my-agent".into(), version: "1.0".into() },
//!             capabilities: AgentCapabilities::default(),
//!             instructions: None,
//!         })
//!     }
//!     async fn session_new(&self, params: SessionNewParams) -> AcpResult<SessionNewResult> {
//!         Ok(SessionNewResult { session_id: params.session_id })
//!     }
//!     async fn session_prompt(&self, params: SessionPromptParams, tx: mpsc::Sender<SessionUpdate>) -> AcpResult<SessionPromptResult> {
//!         Ok(SessionPromptResult { status: "ok".into() })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let server = Server::new(MyAgent);
//!     server.run().await.unwrap();
//! }
//! ```
//!
//! ## Quick Start - Client
//!
//! ```rust,no_run
//! use heroacp::client::Client;
//! use heroacp::protocol::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = Client::spawn("./acp-server").await.unwrap();
//!     // Use client...
//! }
//! ```

pub mod protocol;
pub mod server;
pub mod client;

pub use protocol::*;
