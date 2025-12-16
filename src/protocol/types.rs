//! Common types used throughout the ACP protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Protocol version string.
pub const PROTOCOL_VERSION: &str = "2025.1";

/// Information about a client (editor/IDE).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Name of the client.
    pub name: String,
    /// Version of the client.
    pub version: String,
}

/// Information about an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Name of the agent.
    pub name: String,
    /// Version of the agent.
    pub version: String,
}

/// Capabilities that a client can provide.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Can read/write text files.
    #[serde(default)]
    pub text_files: bool,
    /// Can create and manage terminals.
    #[serde(default)]
    pub terminal: bool,
    /// Supports embedded context in prompts.
    #[serde(default)]
    pub embedded_context: bool,
    /// Supports audio content.
    #[serde(default)]
    pub audio: bool,
    /// Supports image content.
    #[serde(default)]
    pub image: bool,
    /// Experimental capabilities.
    #[serde(default)]
    pub experimental: HashMap<String, serde_json::Value>,
}

/// Capabilities that an agent can provide.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Supports streaming responses.
    #[serde(default)]
    pub streaming: bool,
    /// Supports audio content.
    #[serde(default)]
    pub audio: bool,
    /// Supports image content.
    #[serde(default)]
    pub image: bool,
    /// Supported operational modes.
    #[serde(default)]
    pub supported_modes: Vec<String>,
    /// Available tools.
    #[serde(default)]
    pub tools: Vec<ToolInfo>,
}

/// Information about a tool available to the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Name of the tool.
    pub name: String,
    /// Description of what the tool does.
    pub description: String,
    /// JSON schema for tool parameters.
    #[serde(default)]
    pub parameters: serde_json::Value,
}

/// MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    /// Name of the MCP server.
    pub name: String,
    /// URL or command to connect to the server.
    pub url: String,
    /// Credentials for authentication.
    #[serde(default)]
    pub credentials: HashMap<String, String>,
}

/// Content block in a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content.
    Text {
        /// The text content.
        text: String,
    },
    /// Image content.
    Image {
        /// Image format (png, jpeg, etc.).
        format: String,
        /// Base64-encoded image data.
        data: String,
    },
    /// Audio content.
    Audio {
        /// Audio format (wav, mp3, etc.).
        format: String,
        /// Base64-encoded audio data.
        data: String,
    },
    /// Resource content.
    Resource {
        /// URI of the resource.
        uri: String,
        /// MIME type.
        mime_type: String,
        /// Content of the resource.
        content: String,
    },
    /// Resource link (reference without content).
    ResourceLink {
        /// URI of the resource.
        uri: String,
        /// MIME type.
        mime_type: String,
    },
}

/// A tool call made by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call.
    pub id: String,
    /// Name of the tool being called.
    pub name: String,
    /// Arguments to the tool.
    pub arguments: serde_json::Value,
}

/// Update for a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallUpdate {
    /// ID of the tool call being updated.
    pub id: String,
    /// Status of the tool call.
    pub status: ToolCallStatus,
    /// Result of the tool call (if completed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error message (if failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Status of a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    /// Tool call is in progress.
    InProgress,
    /// Tool call completed successfully.
    Completed,
    /// Tool call failed.
    Failed,
}

/// A plan consisting of multiple steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Steps in the plan.
    pub steps: Vec<PlanStep>,
}

/// A step in a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    /// Unique identifier for this step.
    pub id: u32,
    /// Description of what this step does.
    pub description: String,
    /// Current status of the step.
    pub status: PlanStepStatus,
}

/// Status of a plan step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStepStatus {
    /// Step is pending.
    Pending,
    /// Step is in progress.
    InProgress,
    /// Step is completed.
    Completed,
    /// Step was skipped.
    Skipped,
    /// Step failed.
    Failed,
}

/// Session update sent from agent to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUpdate {
    /// Session ID.
    pub session_id: String,
    /// Type and data of the update.
    #[serde(flatten)]
    pub update_type: SessionUpdateType,
}

/// Types of session updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum SessionUpdateType {
    /// Chunk of agent message.
    AgentMessageChunk {
        /// Text chunk.
        text: String,
    },
    /// Chunk of agent thought/reasoning.
    AgentThoughtChunk {
        /// Thought text.
        text: String,
    },
    /// Agent is making a tool call.
    ToolCall(ToolCall),
    /// Update on a tool call.
    ToolCallUpdate(ToolCallUpdate),
    /// Agent's plan.
    Plan(Plan),
    /// Mode change.
    ModeChange {
        /// New mode.
        mode: String,
    },
    /// Agent is done with the response.
    Done,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, "2025.1");
    }

    #[test]
    fn test_client_info_serialization() {
        let info = ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-client"));
        assert!(json.contains("1.0.0"));

        let deserialized: ClientInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-client");
        assert_eq!(deserialized.version, "1.0.0");
    }

    #[test]
    fn test_agent_info_serialization() {
        let info = AgentInfo {
            name: "test-agent".to_string(),
            version: "2.0.0".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: AgentInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-agent");
        assert_eq!(deserialized.version, "2.0.0");
    }

    #[test]
    fn test_client_capabilities_default() {
        let caps = ClientCapabilities::default();
        assert!(!caps.text_files);
        assert!(!caps.terminal);
        assert!(!caps.audio);
        assert!(!caps.image);
        assert!(caps.experimental.is_empty());
    }

    #[test]
    fn test_client_capabilities_serialization() {
        let caps = ClientCapabilities {
            text_files: true,
            terminal: true,
            embedded_context: false,
            audio: false,
            image: true,
            experimental: HashMap::new(),
        };
        let json = serde_json::to_string(&caps).unwrap();
        let deserialized: ClientCapabilities = serde_json::from_str(&json).unwrap();
        assert!(deserialized.text_files);
        assert!(deserialized.terminal);
        assert!(deserialized.image);
        assert!(!deserialized.audio);
    }

    #[test]
    fn test_agent_capabilities_default() {
        let caps = AgentCapabilities::default();
        assert!(!caps.streaming);
        assert!(!caps.audio);
        assert!(!caps.image);
        assert!(caps.supported_modes.is_empty());
        assert!(caps.tools.is_empty());
    }

    #[test]
    fn test_content_block_text() {
        let block = ContentBlock::Text {
            text: "Hello, world!".to_string(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("Hello, world!"));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        if let ContentBlock::Text { text } = deserialized {
            assert_eq!(text, "Hello, world!");
        } else {
            panic!("Expected Text block");
        }
    }

    #[test]
    fn test_content_block_image() {
        let block = ContentBlock::Image {
            format: "png".to_string(),
            data: "base64data".to_string(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"image\""));

        let deserialized: ContentBlock = serde_json::from_str(&json).unwrap();
        if let ContentBlock::Image { format, data } = deserialized {
            assert_eq!(format, "png");
            assert_eq!(data, "base64data");
        } else {
            panic!("Expected Image block");
        }
    }

    #[test]
    fn test_content_block_resource() {
        let block = ContentBlock::Resource {
            uri: "file:///test.txt".to_string(),
            mime_type: "text/plain".to_string(),
            content: "file content".to_string(),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("\"type\":\"resource\""));
    }

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCall {
            id: "tool_1".to_string(),
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "/test.txt"}),
        };
        let json = serde_json::to_string(&tool_call).unwrap();
        let deserialized: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "tool_1");
        assert_eq!(deserialized.name, "read_file");
    }

    #[test]
    fn test_tool_call_status_serialization() {
        let status = ToolCallStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"completed\"");

        let status = ToolCallStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_progress\"");

        let status = ToolCallStatus::Failed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"failed\"");
    }

    #[test]
    fn test_tool_call_update_serialization() {
        let update = ToolCallUpdate {
            id: "tool_1".to_string(),
            status: ToolCallStatus::Completed,
            result: Some(serde_json::json!({"content": "test"})),
            error: None,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"status\":\"completed\""));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));

        let deserialized: ToolCallUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "tool_1");
        assert!(matches!(deserialized.status, ToolCallStatus::Completed));
    }

    #[test]
    fn test_plan_serialization() {
        let plan = Plan {
            steps: vec![
                PlanStep {
                    id: 1,
                    description: "Step 1".to_string(),
                    status: PlanStepStatus::Completed,
                },
                PlanStep {
                    id: 2,
                    description: "Step 2".to_string(),
                    status: PlanStepStatus::InProgress,
                },
                PlanStep {
                    id: 3,
                    description: "Step 3".to_string(),
                    status: PlanStepStatus::Pending,
                },
            ],
        };
        let json = serde_json::to_string(&plan).unwrap();
        let deserialized: Plan = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.steps.len(), 3);
        assert!(matches!(deserialized.steps[0].status, PlanStepStatus::Completed));
        assert!(matches!(deserialized.steps[1].status, PlanStepStatus::InProgress));
    }

    #[test]
    fn test_plan_step_status_serialization() {
        let statuses = vec![
            (PlanStepStatus::Pending, "\"pending\""),
            (PlanStepStatus::InProgress, "\"in_progress\""),
            (PlanStepStatus::Completed, "\"completed\""),
            (PlanStepStatus::Skipped, "\"skipped\""),
            (PlanStepStatus::Failed, "\"failed\""),
        ];

        for (status, expected) in statuses {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_session_update_agent_message_chunk() {
        let update = SessionUpdate {
            session_id: "session_1".to_string(),
            update_type: SessionUpdateType::AgentMessageChunk {
                text: "Hello".to_string(),
            },
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"session_id\":\"session_1\""));
        assert!(json.contains("\"type\":\"agent_message_chunk\""));
        assert!(json.contains("\"text\":\"Hello\""));
    }

    #[test]
    fn test_session_update_agent_thought_chunk() {
        let update = SessionUpdate {
            session_id: "session_1".to_string(),
            update_type: SessionUpdateType::AgentThoughtChunk {
                text: "Thinking...".to_string(),
            },
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"type\":\"agent_thought_chunk\""));
    }

    #[test]
    fn test_session_update_tool_call() {
        let update = SessionUpdate {
            session_id: "session_1".to_string(),
            update_type: SessionUpdateType::ToolCall(ToolCall {
                id: "tool_1".to_string(),
                name: "read_file".to_string(),
                arguments: serde_json::json!({}),
            }),
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"type\":\"tool_call\""));
    }

    #[test]
    fn test_session_update_done() {
        let update = SessionUpdate {
            session_id: "session_1".to_string(),
            update_type: SessionUpdateType::Done,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"type\":\"done\""));
    }

    #[test]
    fn test_mcp_server_serialization() {
        let server = McpServer {
            name: "test-mcp".to_string(),
            url: "stdio:///path/to/server".to_string(),
            credentials: HashMap::new(),
        };
        let json = serde_json::to_string(&server).unwrap();
        let deserialized: McpServer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-mcp");
        assert_eq!(deserialized.url, "stdio:///path/to/server");
    }

    #[test]
    fn test_tool_info_serialization() {
        let tool = ToolInfo {
            name: "read_file".to_string(),
            description: "Reads a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                }
            }),
        };
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: ToolInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "read_file");
        assert_eq!(deserialized.description, "Reads a file");
    }
}
