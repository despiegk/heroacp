//! Example ACP client that connects to agents.
//!
//! This client can connect to:
//! - The built-in bogus agent (acp-server)
//! - Goose AI agent
//! - Any other ACP-compatible agent
//!
//! Run with: cargo run --bin acp-client [agent-command]
//!
//! Examples:
//!   cargo run --bin acp-client ./target/release/acp-server
//!   cargo run --bin acp-client goose

use heroacp::client::{default_capabilities, Client, UpdateHandler};
use heroacp::protocol::*;
use std::io::Write;
use tokio::io::{self, AsyncBufReadExt, BufReader};

/// Terminal-based update handler that prints responses to stdout.
struct TerminalHandler {
    show_thoughts: bool,
    show_tools: bool,
}

impl TerminalHandler {
    fn new() -> Self {
        Self {
            show_thoughts: true,
            show_tools: true,
        }
    }
}

impl UpdateHandler for TerminalHandler {
    fn on_agent_message(&self, _session_id: &str, text: &str) {
        print!("{}", text);
        std::io::stdout().flush().ok();
    }

    fn on_agent_thought(&self, _session_id: &str, text: &str) {
        if self.show_thoughts {
            eprintln!("\x1b[90m[Thinking] {}\x1b[0m", text);
        }
    }

    fn on_tool_call(&self, _session_id: &str, tool: &ToolCall) {
        if self.show_tools {
            eprintln!(
                "\x1b[33m[Tool Call] {} ({})\x1b[0m",
                tool.name, tool.id
            );
            if !tool.arguments.is_null() {
                eprintln!(
                    "\x1b[33m  Args: {}\x1b[0m",
                    serde_json::to_string_pretty(&tool.arguments).unwrap_or_default()
                );
            }
        }
    }

    fn on_tool_update(&self, _session_id: &str, update: &ToolCallUpdate) {
        if self.show_tools {
            let status = match update.status {
                ToolCallStatus::InProgress => "\x1b[34m[In Progress]\x1b[0m",
                ToolCallStatus::Completed => "\x1b[32m[Completed]\x1b[0m",
                ToolCallStatus::Failed => "\x1b[31m[Failed]\x1b[0m",
            };
            eprintln!("[Tool Update] {} {}", update.id, status);

            if let Some(ref result) = update.result {
                eprintln!(
                    "  Result: {}",
                    serde_json::to_string_pretty(result).unwrap_or_default()
                );
            }
            if let Some(ref error) = update.error {
                eprintln!("\x1b[31m  Error: {}\x1b[0m", error);
            }
        }
    }

    fn on_plan(&self, _session_id: &str, plan: &Plan) {
        eprintln!("\x1b[36m[Plan]\x1b[0m");
        for step in &plan.steps {
            let status = match step.status {
                PlanStepStatus::Completed => "\x1b[32m✓\x1b[0m",
                PlanStepStatus::InProgress => "\x1b[34m→\x1b[0m",
                PlanStepStatus::Pending => "○",
                PlanStepStatus::Skipped => "\x1b[90m-\x1b[0m",
                PlanStepStatus::Failed => "\x1b[31m✗\x1b[0m",
            };
            eprintln!("  {} {}", status, step.description);
        }
    }

    fn on_mode_change(&self, _session_id: &str, mode: &str) {
        eprintln!("\x1b[35m[Mode Change] {}\x1b[0m", mode);
    }

    fn on_done(&self, _session_id: &str) {
        // Print newline after done
        println!();
    }
}

fn print_help() {
    println!("HeroACP Client - Agent Client Protocol CLI");
    println!();
    println!("Commands:");
    println!("  /help     - Show this help message");
    println!("  /info     - Show agent information");
    println!("  /quit     - Exit the client");
    println!("  /new      - Start a new session");
    println!();
    println!("Just type your message and press Enter to send it to the agent.");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Determine agent command
    let agent_command = if args.len() > 1 {
        args[1].as_str()
    } else {
        // Try to find the built-in server
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()));

        if let Some(dir) = exe_dir {
            let server_path = dir.join("acp-server");
            if server_path.exists() {
                println!("Using built-in acp-server...");
                // We need to handle this differently since we can't return a reference to a local
                "./target/release/acp-server"
            } else {
                "./target/debug/acp-server"
            }
        } else {
            "./target/debug/acp-server"
        }
    };

    println!("╔════════════════════════════════════════════╗");
    println!("║         HeroACP Client v0.1.0              ║");
    println!("╚════════════════════════════════════════════╝");
    println!();
    println!("Connecting to agent: {}", agent_command);

    // Spawn client
    let client = match Client::spawn(agent_command).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to spawn agent: {}", e);
            eprintln!();
            eprintln!("Make sure the agent is built:");
            eprintln!("  cargo build --release");
            eprintln!();
            eprintln!("Or specify a different agent:");
            eprintln!("  cargo run --bin acp-client -- goose");
            return Ok(());
        }
    };

    // Set up update handler
    client.set_update_handler(Box::new(TerminalHandler::new())).await;

    // Get working directory
    let cwd = std::env::current_dir()?.to_string_lossy().to_string();

    // Initialize connection
    println!("Initializing connection...");
    let init_result = client
        .initialize(InitializeParams {
            protocol_version: PROTOCOL_VERSION.to_string(),
            client_info: ClientInfo {
                name: "heroacp-client".to_string(),
                version: "0.1.0".to_string(),
            },
            capabilities: default_capabilities(),
            working_directory: cwd,
            mcp_servers: vec![],
        })
        .await?;

    println!();
    println!("Connected to: {} v{}",
        init_result.agent_info.name,
        init_result.agent_info.version
    );

    if let Some(instructions) = &init_result.instructions {
        println!("Agent: {}", instructions);
    }

    // Show capabilities
    println!();
    println!("Capabilities:");
    println!("  Streaming: {}", init_result.capabilities.streaming);
    println!("  Audio: {}", init_result.capabilities.audio);
    println!("  Image: {}", init_result.capabilities.image);
    if !init_result.capabilities.supported_modes.is_empty() {
        println!("  Modes: {}", init_result.capabilities.supported_modes.join(", "));
    }
    if !init_result.capabilities.tools.is_empty() {
        println!("  Tools: {}",
            init_result.capabilities.tools.iter()
                .map(|t| t.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Create initial session
    let session_id = uuid::Uuid::new_v4().to_string();
    let session = client
        .session_new(SessionNewParams {
            session_id: session_id.clone(),
            mode: Some("agent".to_string()),
        })
        .await?;

    println!();
    println!("Session started: {}", session.session_id);
    println!();
    println!("Type /help for commands, or just type your message.");
    println!("─────────────────────────────────────────────");
    println!();

    // Interactive REPL
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();
    let mut current_session = session.session_id;

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        let line = match lines.next_line().await? {
            Some(l) => l,
            None => break, // EOF
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Handle commands
        if line.starts_with('/') {
            match line {
                "/help" => {
                    print_help();
                    continue;
                }
                "/quit" | "/exit" | "/q" => {
                    println!("Goodbye!");
                    break;
                }
                "/info" => {
                    println!("Agent: {} v{}",
                        init_result.agent_info.name,
                        init_result.agent_info.version
                    );
                    println!("Session: {}", current_session);
                    continue;
                }
                "/new" => {
                    let new_session_id = uuid::Uuid::new_v4().to_string();
                    match client.session_new(SessionNewParams {
                        session_id: new_session_id.clone(),
                        mode: Some("agent".to_string()),
                    }).await {
                        Ok(s) => {
                            current_session = s.session_id.clone();
                            println!("New session: {}", s.session_id);
                        }
                        Err(e) => {
                            eprintln!("Failed to create session: {}", e);
                        }
                    }
                    continue;
                }
                _ => {
                    println!("Unknown command: {}", line);
                    println!("Type /help for available commands.");
                    continue;
                }
            }
        }

        // Send prompt
        match client
            .session_prompt(SessionPromptParams {
                session_id: current_session.clone(),
                content: vec![ContentBlock::Text {
                    text: line.to_string(),
                }],
            })
            .await
        {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}
