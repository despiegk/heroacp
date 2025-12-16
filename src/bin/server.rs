//! Example ACP server (bogus agent) that responds to requests.
//!
//! This is a demo agent that:
//! - Responds to prompts with mock AI-like responses
//! - Simulates thinking with thought chunks
//! - Shows example tool calls
//! - Demonstrates the ACP protocol
//!
//! Run with: cargo run --bin acp-server

use async_trait::async_trait;
use heroacp::protocol::*;
use heroacp::server::{Agent, Server};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

/// A bogus AI agent that provides mock responses.
struct BogusAgent {
    name: String,
    version: String,
}

impl BogusAgent {
    fn new() -> Self {
        Self {
            name: "HeroACP Bogus Agent".to_string(),
            version: "0.1.0".to_string(),
        }
    }

    /// Generate a mock response based on the input.
    fn generate_response(&self, input: &str) -> Vec<String> {
        let input_lower = input.to_lowercase();

        if input_lower.contains("hello") || input_lower.contains("hi") {
            vec![
                "Hello! ".to_string(),
                "I'm the HeroACP Bogus Agent. ".to_string(),
                "I'm here to demonstrate the Agent Client Protocol. ".to_string(),
                "How can I help you today?".to_string(),
            ]
        } else if input_lower.contains("help") {
            vec![
                "I can help you with:\n".to_string(),
                "- Answering questions about the ACP protocol\n".to_string(),
                "- Demonstrating streaming responses\n".to_string(),
                "- Showing tool call examples\n".to_string(),
                "- Testing your ACP client implementation\n".to_string(),
                "\nJust ask me anything!".to_string(),
            ]
        } else if input_lower.contains("tool") || input_lower.contains("read") || input_lower.contains("file") {
            vec![
                "I'll demonstrate a tool call. ".to_string(),
                "In a real agent, I would read files or execute commands. ".to_string(),
                "The tool call notification shows the protocol in action.".to_string(),
            ]
        } else if input_lower.contains("plan") {
            vec![
                "I'll create a plan for you:\n\n".to_string(),
                "1. Analyze the request\n".to_string(),
                "2. Search for relevant files\n".to_string(),
                "3. Implement the solution\n".to_string(),
                "4. Test the changes\n".to_string(),
                "5. Provide summary\n".to_string(),
            ]
        } else {
            vec![
                "I received your message: \"".to_string(),
                input.chars().take(50).collect::<String>(),
                if input.len() > 50 { "..." } else { "" }.to_string(),
                "\"\n\n".to_string(),
                "As a bogus agent, I provide mock responses. ".to_string(),
                "In a real agent, I would process your request using an AI model. ".to_string(),
                "This demonstrates the ACP protocol working correctly!".to_string(),
            ]
        }
    }

    /// Generate a thinking message based on the input.
    fn generate_thought(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();

        if input_lower.contains("file") || input_lower.contains("read") {
            "Analyzing file request... I should use the fs/read_text_file method.".to_string()
        } else if input_lower.contains("code") || input_lower.contains("fix") {
            "Let me think about how to approach this coding task...".to_string()
        } else if input_lower.contains("plan") {
            "Creating a structured plan for this task...".to_string()
        } else {
            "Processing your request...".to_string()
        }
    }
}

#[async_trait]
impl Agent for BogusAgent {
    async fn initialize(&self, params: InitializeParams) -> AcpResult<InitializeResult> {
        eprintln!(
            "[BogusAgent] Initializing with protocol version: {}",
            params.protocol_version
        );
        eprintln!(
            "[BogusAgent] Client: {} v{}",
            params.client_info.name, params.client_info.version
        );
        eprintln!(
            "[BogusAgent] Working directory: {}",
            params.working_directory
        );

        Ok(InitializeResult {
            agent_info: AgentInfo {
                name: self.name.clone(),
                version: self.version.clone(),
            },
            capabilities: AgentCapabilities {
                streaming: true,
                audio: false,
                image: true,
                supported_modes: vec!["agent".to_string(), "ask".to_string()],
                tools: vec![
                    ToolInfo {
                        name: "read_file".to_string(),
                        description: "Read a file from the filesystem".to_string(),
                        parameters: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "path": {
                                    "type": "string",
                                    "description": "Absolute path to the file"
                                }
                            },
                            "required": ["path"]
                        }),
                    },
                    ToolInfo {
                        name: "run_command".to_string(),
                        description: "Run a shell command".to_string(),
                        parameters: serde_json::json!({
                            "type": "object",
                            "properties": {
                                "command": {
                                    "type": "string",
                                    "description": "Command to execute"
                                }
                            },
                            "required": ["command"]
                        }),
                    },
                ],
            },
            instructions: Some(
                "I am the HeroACP Bogus Agent, a demonstration agent for the Agent Client Protocol. \
                I provide mock responses to test ACP client implementations.".to_string(),
            ),
        })
    }

    async fn session_new(&self, params: SessionNewParams) -> AcpResult<SessionNewResult> {
        eprintln!(
            "[BogusAgent] Creating new session: {} (mode: {:?})",
            params.session_id,
            params.mode
        );

        Ok(SessionNewResult {
            session_id: params.session_id,
        })
    }

    async fn session_load(&self, params: SessionLoadParams) -> AcpResult<SessionLoadResult> {
        eprintln!("[BogusAgent] Loading session: {}", params.session_id);

        // Bogus agent doesn't persist sessions
        Ok(SessionLoadResult {
            session_id: params.session_id,
            loaded: false,
        })
    }

    async fn session_prompt(
        &self,
        params: SessionPromptParams,
        update_tx: mpsc::Sender<SessionUpdate>,
    ) -> AcpResult<SessionPromptResult> {
        let session_id = params.session_id.clone();

        // Extract text from content blocks
        let prompt_text: String = params
            .content
            .iter()
            .filter_map(|block| {
                if let ContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        eprintln!(
            "[BogusAgent] Received prompt in session {}: {}",
            session_id,
            prompt_text.chars().take(100).collect::<String>()
        );

        // Send thinking update
        let thought = self.generate_thought(&prompt_text);
        let _ = update_tx
            .send(SessionUpdate {
                session_id: session_id.clone(),
                update_type: SessionUpdateType::AgentThoughtChunk { text: thought },
            })
            .await;

        sleep(Duration::from_millis(200)).await;

        // Show a plan if the user asks for one
        if prompt_text.to_lowercase().contains("plan") {
            let _ = update_tx
                .send(SessionUpdate {
                    session_id: session_id.clone(),
                    update_type: SessionUpdateType::Plan(Plan {
                        steps: vec![
                            PlanStep {
                                id: 1,
                                description: "Analyze the request".to_string(),
                                status: PlanStepStatus::Completed,
                            },
                            PlanStep {
                                id: 2,
                                description: "Search for relevant files".to_string(),
                                status: PlanStepStatus::InProgress,
                            },
                            PlanStep {
                                id: 3,
                                description: "Implement the solution".to_string(),
                                status: PlanStepStatus::Pending,
                            },
                            PlanStep {
                                id: 4,
                                description: "Test the changes".to_string(),
                                status: PlanStepStatus::Pending,
                            },
                        ],
                    }),
                })
                .await;

            sleep(Duration::from_millis(200)).await;
        }

        // Show a tool call if the user asks about tools/files
        if prompt_text.to_lowercase().contains("tool")
            || prompt_text.to_lowercase().contains("file")
            || prompt_text.to_lowercase().contains("read")
        {
            let tool_id = format!("tool_{}", uuid::Uuid::new_v4());

            // Send tool call
            let _ = update_tx
                .send(SessionUpdate {
                    session_id: session_id.clone(),
                    update_type: SessionUpdateType::ToolCall(ToolCall {
                        id: tool_id.clone(),
                        name: "read_file".to_string(),
                        arguments: serde_json::json!({
                            "path": "/example/file.txt"
                        }),
                    }),
                })
                .await;

            sleep(Duration::from_millis(300)).await;

            // Send tool result
            let _ = update_tx
                .send(SessionUpdate {
                    session_id: session_id.clone(),
                    update_type: SessionUpdateType::ToolCallUpdate(ToolCallUpdate {
                        id: tool_id,
                        status: ToolCallStatus::Completed,
                        result: Some(serde_json::json!({
                            "content": "Example file content from bogus agent"
                        })),
                        error: None,
                    }),
                })
                .await;

            sleep(Duration::from_millis(200)).await;
        }

        // Stream response chunks
        let response_chunks = self.generate_response(&prompt_text);
        for chunk in response_chunks {
            let _ = update_tx
                .send(SessionUpdate {
                    session_id: session_id.clone(),
                    update_type: SessionUpdateType::AgentMessageChunk { text: chunk },
                })
                .await;

            // Simulate typing delay
            sleep(Duration::from_millis(50)).await;
        }

        // Send done notification
        let _ = update_tx
            .send(SessionUpdate {
                session_id: session_id.clone(),
                update_type: SessionUpdateType::Done,
            })
            .await;

        Ok(SessionPromptResult {
            status: "ok".to_string(),
        })
    }

    async fn session_cancel(&self, params: SessionCancelParams) -> AcpResult<()> {
        eprintln!("[BogusAgent] Cancelling session: {}", params.session_id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[BogusAgent] Starting HeroACP Bogus Agent...");
    eprintln!("[BogusAgent] Waiting for client connection on stdio...");

    let agent = BogusAgent::new();
    let server = Server::new(agent);

    server.run().await?;

    eprintln!("[BogusAgent] Agent shutting down.");
    Ok(())
}
