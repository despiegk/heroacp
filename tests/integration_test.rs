//! Integration tests for HeroACP.
//!
//! These tests verify the end-to-end functionality of the ACP server and client.

use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

/// Helper to send a JSON-RPC request and receive a response.
async fn send_receive(
    stdin: &mut tokio::process::ChildStdin,
    stdout: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    request: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    stdin.write_all(request.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    let response = timeout(Duration::from_secs(5), stdout.next_line())
        .await??
        .ok_or("No response")?;

    Ok(serde_json::from_str(&response)?)
}

/// Helper to receive a notification (no request sent).
#[allow(dead_code)]
async fn receive_notification(
    stdout: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let response = timeout(Duration::from_secs(5), stdout.next_line())
        .await??
        .ok_or("No response")?;

    Ok(serde_json::from_str(&response)?)
}

#[tokio::test]
async fn test_server_initialize() {
    let mut child = Command::new("./target/release/acp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start acp-server");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocol_version": "2025.1",
            "client_info": {"name": "test", "version": "1.0"},
            "capabilities": {},
            "working_directory": "/"
        }
    });

    let response = send_receive(&mut stdin, &mut lines, &request.to_string())
        .await
        .expect("Failed to get response");

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["agent_info"]["name"].is_string());
    assert!(response["result"]["agent_info"]["version"].is_string());
    assert!(response["result"]["capabilities"].is_object());

    // Clean up
    child.kill().await.ok();
}

#[tokio::test]
async fn test_server_session_new() {
    let mut child = Command::new("./target/release/acp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start acp-server");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    // Initialize first
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocol_version": "2025.1",
            "client_info": {"name": "test", "version": "1.0"},
            "capabilities": {},
            "working_directory": "/"
        }
    });
    let _ = send_receive(&mut stdin, &mut lines, &init_request.to_string()).await;

    // Create session
    let session_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {
            "session_id": "test-session-123",
            "mode": "agent"
        }
    });

    let response = send_receive(&mut stdin, &mut lines, &session_request.to_string())
        .await
        .expect("Failed to create session");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    assert_eq!(response["result"]["session_id"], "test-session-123");

    child.kill().await.ok();
}

#[tokio::test]
async fn test_server_session_prompt_streaming() {
    let mut child = Command::new("./target/release/acp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start acp-server");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    // Initialize
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocol_version": "2025.1",
            "client_info": {"name": "test", "version": "1.0"},
            "capabilities": {},
            "working_directory": "/"
        }
    });
    let _ = send_receive(&mut stdin, &mut lines, &init_request.to_string()).await;

    // Create session
    let session_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {
            "session_id": "test-session",
            "mode": "agent"
        }
    });
    let _ = send_receive(&mut stdin, &mut lines, &session_request.to_string()).await;

    // Send prompt
    let prompt_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/prompt",
        "params": {
            "session_id": "test-session",
            "content": [{"type": "text", "text": "Hello!"}]
        }
    });
    stdin
        .write_all(prompt_request.to_string().as_bytes())
        .await
        .unwrap();
    stdin.write_all(b"\n").await.unwrap();
    stdin.flush().await.unwrap();

    // Collect notifications and final response
    let mut notifications = Vec::new();
    let mut got_response = false;

    for _ in 0..20 {
        // Max iterations to prevent infinite loop
        if let Ok(Some(line)) = timeout(Duration::from_secs(2), lines.next_line()).await.unwrap() {
            let msg: serde_json::Value = serde_json::from_str(&line).unwrap();

            if msg.get("id").is_some() && msg.get("result").is_some() {
                // This is the final response
                assert_eq!(msg["id"], 3);
                assert_eq!(msg["result"]["status"], "ok");
                got_response = true;
                break;
            } else if msg.get("method").is_some() {
                // This is a notification
                notifications.push(msg);
            }
        } else {
            break;
        }
    }

    assert!(got_response, "Did not receive prompt response");
    assert!(!notifications.is_empty(), "Did not receive any notifications");

    // Check that we got at least one message chunk
    let has_message_chunk = notifications
        .iter()
        .any(|n| n["params"]["type"] == "agent_message_chunk");
    assert!(has_message_chunk, "No agent_message_chunk notifications");

    child.kill().await.ok();
}

#[tokio::test]
async fn test_server_method_not_found() {
    let mut child = Command::new("./target/release/acp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start acp-server");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "unknown/method",
        "params": {}
    });

    let response = send_receive(&mut stdin, &mut lines, &request.to_string())
        .await
        .expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601); // METHOD_NOT_FOUND

    child.kill().await.ok();
}

#[tokio::test]
async fn test_server_invalid_params() {
    let mut child = Command::new("./target/release/acp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start acp-server");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    // Send initialize with missing required fields
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocol_version": "2025.1"
            // Missing client_info, capabilities, working_directory
        }
    });

    let response = send_receive(&mut stdin, &mut lines, &request.to_string())
        .await
        .expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32602); // INVALID_PARAMS

    child.kill().await.ok();
}

#[tokio::test]
async fn test_server_parse_error() {
    let mut child = Command::new("./target/release/acp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start acp-server");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    // Send invalid JSON
    let response = send_receive(&mut stdin, &mut lines, "invalid json {")
        .await
        .expect("Failed to get response");

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32700); // PARSE_ERROR

    child.kill().await.ok();
}

#[tokio::test]
async fn test_server_capabilities() {
    let mut child = Command::new("./target/release/acp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start acp-server");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocol_version": "2025.1",
            "client_info": {"name": "test", "version": "1.0"},
            "capabilities": {
                "text_files": true,
                "terminal": true
            },
            "working_directory": "/tmp"
        }
    });

    let response = send_receive(&mut stdin, &mut lines, &request.to_string())
        .await
        .expect("Failed to get response");

    // Verify agent capabilities
    let caps = &response["result"]["capabilities"];
    assert!(caps["streaming"].as_bool().unwrap());
    assert!(caps["supported_modes"].is_array());

    // Check tools are advertised
    let tools = caps["tools"].as_array().unwrap();
    assert!(!tools.is_empty());

    child.kill().await.ok();
}

#[tokio::test]
async fn test_server_session_cancel() {
    let mut child = Command::new("./target/release/acp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start acp-server");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    // Initialize
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocol_version": "2025.1",
            "client_info": {"name": "test", "version": "1.0"},
            "capabilities": {},
            "working_directory": "/"
        }
    });
    let _ = send_receive(&mut stdin, &mut lines, &init_request.to_string()).await;

    // Create session
    let session_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {"session_id": "test-session"}
    });
    let _ = send_receive(&mut stdin, &mut lines, &session_request.to_string()).await;

    // Cancel session
    let cancel_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/cancel",
        "params": {"session_id": "test-session"}
    });

    let response = send_receive(&mut stdin, &mut lines, &cancel_request.to_string())
        .await
        .expect("Failed to cancel session");

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 3);
    // Session cancel returns null result
    assert!(response.get("error").is_none());

    child.kill().await.ok();
}

#[tokio::test]
async fn test_multiple_sessions() {
    let mut child = Command::new("./target/release/acp-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start acp-server");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    // Initialize
    let init_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocol_version": "2025.1",
            "client_info": {"name": "test", "version": "1.0"},
            "capabilities": {},
            "working_directory": "/"
        }
    });
    let _ = send_receive(&mut stdin, &mut lines, &init_request.to_string()).await;

    // Create first session
    let session1_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "session/new",
        "params": {"session_id": "session-1"}
    });
    let response1 = send_receive(&mut stdin, &mut lines, &session1_request.to_string())
        .await
        .unwrap();
    assert_eq!(response1["result"]["session_id"], "session-1");

    // Create second session
    let session2_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "session/new",
        "params": {"session_id": "session-2"}
    });
    let response2 = send_receive(&mut stdin, &mut lines, &session2_request.to_string())
        .await
        .unwrap();
    assert_eq!(response2["result"]["session_id"], "session-2");

    child.kill().await.ok();
}
