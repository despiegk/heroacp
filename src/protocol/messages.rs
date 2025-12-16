//! JSON-RPC message types for ACP.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::*;

/// JSON-RPC 2.0 request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (always "2.0").
    pub jsonrpc: String,
    /// Request ID (omitted for notifications).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// Method name.
    pub method: String,
    /// Method parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0").
    pub jsonrpc: String,
    /// Request ID this responds to.
    pub id: Value,
    /// Result (on success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (on failure).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
    /// Additional error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 notification (request without id).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC version (always "2.0").
    pub jsonrpc: String,
    /// Method name.
    pub method: String,
    /// Method parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

// ============================================================================
// Initialize
// ============================================================================

/// Parameters for the initialize request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version the client supports.
    pub protocol_version: String,
    /// Information about the client.
    pub client_info: ClientInfo,
    /// Client capabilities.
    pub capabilities: ClientCapabilities,
    /// Working directory for the session.
    pub working_directory: String,
    /// MCP servers available to the agent.
    #[serde(default)]
    pub mcp_servers: Vec<McpServer>,
}

/// Result of the initialize request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Information about the agent.
    pub agent_info: AgentInfo,
    /// Agent capabilities.
    pub capabilities: AgentCapabilities,
    /// Optional instructions/description from the agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

// ============================================================================
// Authentication
// ============================================================================

/// Parameters for the authenticate request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticateParams {
    /// Authentication type.
    #[serde(rename = "type")]
    pub auth_type: String,
    /// Authentication token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// Result of the authenticate request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticateResult {
    /// Whether authentication was successful.
    pub success: bool,
}

// ============================================================================
// Session Management
// ============================================================================

/// Parameters for creating a new session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionNewParams {
    /// Unique session ID.
    pub session_id: String,
    /// Operational mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

/// Result of creating a new session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionNewResult {
    /// The session ID.
    pub session_id: String,
}

/// Parameters for loading an existing session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLoadParams {
    /// Session ID to load.
    pub session_id: String,
}

/// Result of loading a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLoadResult {
    /// The session ID.
    pub session_id: String,
    /// Whether the session was found and loaded.
    pub loaded: bool,
}

/// Parameters for sending a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPromptParams {
    /// Session ID.
    pub session_id: String,
    /// Content blocks in the prompt.
    pub content: Vec<ContentBlock>,
}

/// Result of sending a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPromptResult {
    /// Status of the prompt processing.
    pub status: String,
}

/// Parameters for cancelling a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCancelParams {
    /// Session ID to cancel.
    pub session_id: String,
}

// ============================================================================
// File System Operations
// ============================================================================

/// Parameters for reading a text file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsReadTextFileParams {
    /// Absolute path to the file.
    pub path: String,
}

/// Result of reading a text file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsReadTextFileResult {
    /// Content of the file.
    pub content: String,
}

/// Parameters for writing a text file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsWriteTextFileParams {
    /// Absolute path to the file.
    pub path: String,
    /// Content to write.
    pub content: String,
}

/// Result of writing a text file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsWriteTextFileResult {
    /// Whether the write was successful.
    pub success: bool,
}

// ============================================================================
// Terminal Operations
// ============================================================================

/// Parameters for creating a terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCreateParams {
    /// Working directory.
    pub cwd: String,
    /// Command to execute.
    pub command: String,
}

/// Result of creating a terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCreateResult {
    /// Terminal ID.
    pub terminal_id: String,
}

/// Parameters for getting terminal output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputParams {
    /// Terminal ID.
    pub terminal_id: String,
}

/// Result of getting terminal output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalOutputResult {
    /// Output text.
    pub output: String,
    /// Whether the terminal has exited.
    pub exited: bool,
    /// Exit code (if exited).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

/// Parameters for waiting for terminal exit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalWaitForExitParams {
    /// Terminal ID.
    pub terminal_id: String,
}

/// Result of waiting for terminal exit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalWaitForExitResult {
    /// Exit code.
    pub exit_code: i32,
    /// Final output.
    pub output: String,
}

/// Parameters for killing a terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalKillParams {
    /// Terminal ID.
    pub terminal_id: String,
}

/// Result of killing a terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalKillResult {
    /// Whether the kill was successful.
    pub success: bool,
}

/// Parameters for releasing a terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalReleaseParams {
    /// Terminal ID.
    pub terminal_id: String,
}

/// Result of releasing a terminal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalReleaseResult {
    /// Whether the release was successful.
    pub success: bool,
}
