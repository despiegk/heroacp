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
