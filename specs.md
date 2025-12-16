# Agent Client Protocol (ACP) Specification

## Overview

The Agent Client Protocol (ACP) is a JSON-RPC 2.0 based protocol that standardizes communication between code editors/IDEs (clients) and AI coding agents (servers). It enables bidirectional, streaming, stateful communication over standard I/O.

ACP is to AI coding agents what the Language Server Protocol (LSP) is to programming languages - it decouples AI assistance from specific editors, creating a universal interface.

## Protocol Architecture

### Layered Design

```
┌─────────────────────────────────────────┐
│           Application Layer             │  Custom agent/client logic
├─────────────────────────────────────────┤
│            Session Layer                │  Conversation context & history
├─────────────────────────────────────────┤
│           Connection Layer              │  Initialization & authentication
├─────────────────────────────────────────┤
│            Protocol Layer               │  JSON-RPC 2.0 messaging
├─────────────────────────────────────────┤
│           Transport Layer               │  NDJSON over stdio
└─────────────────────────────────────────┘
```

### Transport Layer

- Uses **Newline-Delimited JSON (NDJSON)** over standard input/output (stdio)
- Agents run as subprocesses spawned by the editor/client
- Each JSON message is terminated by a newline character (`\n`)

### Protocol Layer

All messages follow JSON-RPC 2.0 specification with three message types:

1. **Requests**: Expect a response
2. **Notifications**: One-way messages, no response expected
3. **Errors**: Error responses to requests

## Message Format

### Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "session/prompt",
  "params": {
    "session_id": "abc123",
    "content": [{"type": "text", "text": "Hello"}]
  }
}
```

### Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "status": "ok"
  }
}
```

### Notification

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "session_id": "abc123",
    "type": "agent_message_chunk",
    "data": {"text": "Hello!"}
  }
}
```

### Error Response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32001,
    "message": "Resource not found"
  }
}
```

## Error Codes

### Standard JSON-RPC Errors

| Code   | Message          | Description                        |
|--------|------------------|------------------------------------|
| -32700 | Parse error      | Invalid JSON                       |
| -32600 | Invalid Request  | Invalid JSON-RPC request           |
| -32601 | Method not found | Unknown method                     |
| -32602 | Invalid params   | Invalid method parameters          |
| -32603 | Internal error   | Internal JSON-RPC error            |

### ACP-Specific Errors

| Code   | Message                   | Description                    |
|--------|---------------------------|--------------------------------|
| -32001 | Resource not found        | Requested resource not found   |
| -32002 | Permission denied         | Operation not permitted        |
| -32003 | Invalid state             | Invalid protocol state         |
| -32004 | Capability not supported  | Feature not available          |

## Connection Lifecycle

### 1. Initialization

Client sends `initialize` request with capabilities:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocol_version": "2025.1",
    "client_info": {
      "name": "my-editor",
      "version": "1.0.0"
    },
    "capabilities": {
      "text_files": true,
      "terminal": true,
      "experimental": {}
    },
    "working_directory": "/home/user/project",
    "mcp_servers": []
  }
}
```

Agent responds with its capabilities:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "agent_info": {
      "name": "my-agent",
      "version": "1.0.0"
    },
    "capabilities": {
      "streaming": true,
      "audio": false,
      "image": true,
      "supported_modes": ["agent", "ask"]
    },
    "instructions": "I am an AI coding assistant."
  }
}
```

### 2. Optional Authentication

If authentication is required:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "authenticate",
  "params": {
    "type": "token",
    "token": "..."
  }
}
```

## Session Management

### Create New Session

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "session/new",
  "params": {
    "session_id": "abc123",
    "mode": "agent"
  }
}
```

### Load Existing Session

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "session/load",
  "params": {
    "session_id": "abc123"
  }
}
```

### Send Prompt

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "session/prompt",
  "params": {
    "session_id": "abc123",
    "content": [
      {"type": "text", "text": "Fix the bug in main.rs"}
    ]
  }
}
```

### Cancel Processing

```json
{
  "jsonrpc": "2.0",
  "method": "session/cancel",
  "params": {
    "session_id": "abc123"
  }
}
```

## Session Updates (Agent -> Client Notifications)

### Agent Message Chunk

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "session_id": "abc123",
    "type": "agent_message_chunk",
    "data": {
      "text": "I'll help you fix that bug."
    }
  }
}
```

### Agent Thought Chunk

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "session_id": "abc123",
    "type": "agent_thought_chunk",
    "data": {
      "text": "Analyzing the code structure..."
    }
  }
}
```

### Tool Call

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "session_id": "abc123",
    "type": "tool_call",
    "data": {
      "id": "tool_1",
      "name": "read_file",
      "arguments": {"path": "/home/user/project/main.rs"}
    }
  }
}
```

### Tool Call Update

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "session_id": "abc123",
    "type": "tool_call_update",
    "data": {
      "id": "tool_1",
      "status": "completed",
      "result": {"content": "file contents..."}
    }
  }
}
```

### Plan

```json
{
  "jsonrpc": "2.0",
  "method": "session/update",
  "params": {
    "session_id": "abc123",
    "type": "plan",
    "data": {
      "steps": [
        {"id": 1, "description": "Read the file", "status": "completed"},
        {"id": 2, "description": "Identify the bug", "status": "in_progress"},
        {"id": 3, "description": "Fix the bug", "status": "pending"}
      ]
    }
  }
}
```

## File System Operations (Agent -> Client Requests)

### Read Text File

Request:
```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "fs/read_text_file",
  "params": {
    "path": "/absolute/path/to/file.rs"
  }
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "result": {
    "content": "fn main() {\n    println!(\"Hello\");\n}"
  }
}
```

### Write Text File

Request:
```json
{
  "jsonrpc": "2.0",
  "id": 11,
  "method": "fs/write_text_file",
  "params": {
    "path": "/absolute/path/to/file.rs",
    "content": "fn main() {\n    println!(\"Hello, World!\");\n}"
  }
}
```

## Terminal Operations (Agent -> Client Requests)

### Create Terminal

```json
{
  "jsonrpc": "2.0",
  "id": 20,
  "method": "terminal/create",
  "params": {
    "cwd": "/home/user/project",
    "command": "cargo build"
  }
}
```

### Get Terminal Output

```json
{
  "jsonrpc": "2.0",
  "id": 21,
  "method": "terminal/output",
  "params": {
    "terminal_id": "term_1"
  }
}
```

### Wait for Exit

```json
{
  "jsonrpc": "2.0",
  "id": 22,
  "method": "terminal/wait_for_exit",
  "params": {
    "terminal_id": "term_1"
  }
}
```

### Kill Terminal

```json
{
  "jsonrpc": "2.0",
  "id": 23,
  "method": "terminal/kill",
  "params": {
    "terminal_id": "term_1"
  }
}
```

### Release Terminal

```json
{
  "jsonrpc": "2.0",
  "id": 24,
  "method": "terminal/release",
  "params": {
    "terminal_id": "term_1"
  }
}
```

## Content Blocks

Messages can contain various content types:

### Text Block

```json
{
  "type": "text",
  "text": "Hello, world!"
}
```

### Image Block

```json
{
  "type": "image",
  "format": "png",
  "data": "base64-encoded-data..."
}
```

### Audio Block

```json
{
  "type": "audio",
  "format": "wav",
  "data": "base64-encoded-data..."
}
```

### Resource Block

```json
{
  "type": "resource",
  "uri": "file:///path/to/file",
  "mime_type": "text/plain",
  "content": "file contents..."
}
```

### Resource Link Block

```json
{
  "type": "resource_link",
  "uri": "file:///path/to/file",
  "mime_type": "text/plain"
}
```

## Capabilities

### Client Capabilities

| Capability       | Description                              |
|------------------|------------------------------------------|
| `text_files`     | Read/write text files                    |
| `terminal`       | Create and manage terminal sessions      |
| `embedded_context` | Accept embedded context in prompts     |
| `audio`          | Support audio content                    |
| `image`          | Support image content                    |
| `experimental`   | Experimental features                    |

### Agent Capabilities

| Capability         | Description                              |
|--------------------|------------------------------------------|
| `streaming`        | Stream responses incrementally           |
| `audio`            | Generate audio responses                 |
| `image`            | Process image inputs                     |
| `supported_modes`  | List of supported modes (agent, ask)     |
| `tools`            | Available tools                          |

## MCP Integration

ACP integrates with the Model Context Protocol (MCP) for tool access:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "mcp_servers": [
      {
        "name": "filesystem",
        "url": "stdio:///path/to/mcp-server",
        "credentials": {}
      }
    ]
  }
}
```

## Implementation Requirements

### Server (Agent) Requirements

1. **Transport**: Accept NDJSON over stdin, write to stdout
2. **Protocol Version**: Support version negotiation
3. **Capabilities**: Advertise supported features
4. **Session Management**: Handle session creation, loading, prompts
5. **Streaming**: Send incremental updates via notifications
6. **Error Handling**: Return proper JSON-RPC error responses

### Client (Editor) Requirements

1. **Transport**: Spawn agent as subprocess, communicate via stdio
2. **Initialization**: Send initialize request with capabilities
3. **Session Handling**: Manage session lifecycle
4. **File Operations**: Handle agent file read/write requests
5. **Terminal Operations**: Handle agent terminal requests
6. **Notification Processing**: Process streaming updates

## Protocol States

```
┌─────────────┐    initialize    ┌──────────────┐
│   Created   │ ───────────────> │ Initialized  │
└─────────────┘                  └──────────────┘
                                        │
                                        │ session/new
                                        ▼
                                 ┌──────────────┐
                                 │   Session    │
                                 │   Active     │
                                 └──────────────┘
                                   │        ▲
                         prompt    │        │  complete
                                   ▼        │
                                 ┌──────────────┐
                                 │  Processing  │
                                 └──────────────┘
```

## Security Considerations

1. **Absolute Paths**: All file paths MUST be absolute
2. **Permission Model**: Clients control file/terminal access
3. **Sandboxing**: Agents run as subprocesses with limited access
4. **Authentication**: Optional token-based authentication

## Version History

- **2025.1**: Current stable version
- Protocol is designed for forward compatibility with capability negotiation
