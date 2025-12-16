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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(1.into())),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({"test": "value"})),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"method\":\"initialize\""));

        let deserialized: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.method, "initialize");
    }

    #[test]
    fn test_json_rpc_request_notification() {
        let notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "session/update".to_string(),
            params: None,
        };
        let json = serde_json::to_string(&notification).unwrap();
        assert!(!json.contains("\"id\""));
        assert!(!json.contains("\"params\""));
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::Number(1.into()),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::Number(1.into()),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32601"));
        assert!(!json.contains("\"result\""));
    }

    #[test]
    fn test_json_rpc_error_serialization() {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(serde_json::json!({"detail": "missing field"})),
        };
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: JsonRpcError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, -32600);
        assert_eq!(deserialized.message, "Invalid Request");
    }

    #[test]
    fn test_json_rpc_notification_serialization() {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "session/update".to_string(),
            params: Some(serde_json::json!({"session_id": "test"})),
        };
        let json = serde_json::to_string(&notification).unwrap();
        assert!(!json.contains("\"id\""));
        assert!(json.contains("\"method\":\"session/update\""));
    }

    #[test]
    fn test_initialize_params_serialization() {
        let params = InitializeParams {
            protocol_version: "2025.1".to_string(),
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
            capabilities: ClientCapabilities::default(),
            working_directory: "/home/user".to_string(),
            mcp_servers: vec![],
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: InitializeParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.protocol_version, "2025.1");
        assert_eq!(deserialized.client_info.name, "test-client");
        assert_eq!(deserialized.working_directory, "/home/user");
    }

    #[test]
    fn test_initialize_params_with_mcp_servers() {
        let params = InitializeParams {
            protocol_version: "2025.1".to_string(),
            client_info: ClientInfo {
                name: "test".to_string(),
                version: "1.0".to_string(),
            },
            capabilities: ClientCapabilities::default(),
            working_directory: "/".to_string(),
            mcp_servers: vec![McpServer {
                name: "filesystem".to_string(),
                url: "stdio:///path".to_string(),
                credentials: HashMap::new(),
            }],
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("filesystem"));
    }

    #[test]
    fn test_initialize_result_serialization() {
        let result = InitializeResult {
            agent_info: AgentInfo {
                name: "test-agent".to_string(),
                version: "1.0.0".to_string(),
            },
            capabilities: AgentCapabilities {
                streaming: true,
                audio: false,
                image: true,
                supported_modes: vec!["agent".to_string()],
                tools: vec![],
            },
            instructions: Some("Hello!".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: InitializeResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.agent_info.name, "test-agent");
        assert!(deserialized.capabilities.streaming);
        assert_eq!(deserialized.instructions, Some("Hello!".to_string()));
    }

    #[test]
    fn test_initialize_result_without_instructions() {
        let result = InitializeResult {
            agent_info: AgentInfo {
                name: "agent".to_string(),
                version: "1.0".to_string(),
            },
            capabilities: AgentCapabilities::default(),
            instructions: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("instructions"));
    }

    #[test]
    fn test_authenticate_params_serialization() {
        let params = AuthenticateParams {
            auth_type: "token".to_string(),
            token: Some("secret123".to_string()),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"type\":\"token\""));
        let deserialized: AuthenticateParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.auth_type, "token");
    }

    #[test]
    fn test_authenticate_result_serialization() {
        let result = AuthenticateResult { success: true };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: AuthenticateResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.success);
    }

    #[test]
    fn test_session_new_params_serialization() {
        let params = SessionNewParams {
            session_id: "session_123".to_string(),
            mode: Some("agent".to_string()),
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SessionNewParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, "session_123");
        assert_eq!(deserialized.mode, Some("agent".to_string()));
    }

    #[test]
    fn test_session_new_params_without_mode() {
        let params = SessionNewParams {
            session_id: "session_123".to_string(),
            mode: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(!json.contains("mode"));
    }

    #[test]
    fn test_session_new_result_serialization() {
        let result = SessionNewResult {
            session_id: "session_123".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SessionNewResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, "session_123");
    }

    #[test]
    fn test_session_load_params_serialization() {
        let params = SessionLoadParams {
            session_id: "existing_session".to_string(),
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SessionLoadParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, "existing_session");
    }

    #[test]
    fn test_session_load_result_serialization() {
        let result = SessionLoadResult {
            session_id: "session_123".to_string(),
            loaded: true,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SessionLoadResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.loaded);
    }

    #[test]
    fn test_session_prompt_params_serialization() {
        let params = SessionPromptParams {
            session_id: "session_123".to_string(),
            content: vec![ContentBlock::Text {
                text: "Hello, agent!".to_string(),
            }],
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SessionPromptParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, "session_123");
        assert_eq!(deserialized.content.len(), 1);
    }

    #[test]
    fn test_session_prompt_result_serialization() {
        let result = SessionPromptResult {
            status: "ok".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SessionPromptResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, "ok");
    }

    #[test]
    fn test_session_cancel_params_serialization() {
        let params = SessionCancelParams {
            session_id: "session_123".to_string(),
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SessionCancelParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.session_id, "session_123");
    }

    #[test]
    fn test_fs_read_text_file_params_serialization() {
        let params = FsReadTextFileParams {
            path: "/home/user/test.txt".to_string(),
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: FsReadTextFileParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.path, "/home/user/test.txt");
    }

    #[test]
    fn test_fs_read_text_file_result_serialization() {
        let result = FsReadTextFileResult {
            content: "file content here".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: FsReadTextFileResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content, "file content here");
    }

    #[test]
    fn test_fs_write_text_file_params_serialization() {
        let params = FsWriteTextFileParams {
            path: "/home/user/output.txt".to_string(),
            content: "new content".to_string(),
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: FsWriteTextFileParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.path, "/home/user/output.txt");
        assert_eq!(deserialized.content, "new content");
    }

    #[test]
    fn test_fs_write_text_file_result_serialization() {
        let result = FsWriteTextFileResult { success: true };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: FsWriteTextFileResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.success);
    }

    #[test]
    fn test_terminal_create_params_serialization() {
        let params = TerminalCreateParams {
            cwd: "/home/user".to_string(),
            command: "ls -la".to_string(),
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: TerminalCreateParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.cwd, "/home/user");
        assert_eq!(deserialized.command, "ls -la");
    }

    #[test]
    fn test_terminal_create_result_serialization() {
        let result = TerminalCreateResult {
            terminal_id: "term_1".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TerminalCreateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.terminal_id, "term_1");
    }

    #[test]
    fn test_terminal_output_result_serialization() {
        let result = TerminalOutputResult {
            output: "command output".to_string(),
            exited: true,
            exit_code: Some(0),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TerminalOutputResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.output, "command output");
        assert!(deserialized.exited);
        assert_eq!(deserialized.exit_code, Some(0));
    }

    #[test]
    fn test_terminal_output_result_not_exited() {
        let result = TerminalOutputResult {
            output: "partial output".to_string(),
            exited: false,
            exit_code: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("exit_code"));
    }

    #[test]
    fn test_terminal_wait_for_exit_result_serialization() {
        let result = TerminalWaitForExitResult {
            exit_code: 0,
            output: "final output".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TerminalWaitForExitResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.exit_code, 0);
    }

    #[test]
    fn test_terminal_kill_result_serialization() {
        let result = TerminalKillResult { success: true };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TerminalKillResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.success);
    }

    #[test]
    fn test_terminal_release_result_serialization() {
        let result = TerminalReleaseResult { success: true };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: TerminalReleaseResult = serde_json::from_str(&json).unwrap();
        assert!(deserialized.success);
    }
}
