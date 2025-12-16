//! Error types for ACP.

use thiserror::Error;

/// Standard JSON-RPC error codes.
pub mod codes {
    /// Invalid JSON was received.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist or is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameters.
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;

    // ACP-specific error codes
    /// Requested resource was not found.
    pub const RESOURCE_NOT_FOUND: i32 = -32001;
    /// Permission denied for the operation.
    pub const PERMISSION_DENIED: i32 = -32002;
    /// Invalid protocol state.
    pub const INVALID_STATE: i32 = -32003;
    /// Capability not supported.
    pub const CAPABILITY_NOT_SUPPORTED: i32 = -32004;
}

/// ACP protocol error.
#[derive(Debug, Error)]
pub enum AcpError {
    /// JSON parsing error.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Invalid JSON-RPC request.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Method not found.
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Invalid parameters.
    #[error("Invalid params: {0}")]
    InvalidParams(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid state.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Capability not supported.
    #[error("Capability not supported: {0}")]
    CapabilityNotSupported(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Channel send error.
    #[error("Channel error: {0}")]
    ChannelError(String),

    /// Connection closed.
    #[error("Connection closed")]
    ConnectionClosed,

    /// Request timeout.
    #[error("Request timeout")]
    Timeout,
}

impl AcpError {
    /// Get the JSON-RPC error code for this error.
    pub fn code(&self) -> i32 {
        match self {
            AcpError::ParseError(_) => codes::PARSE_ERROR,
            AcpError::InvalidRequest(_) => codes::INVALID_REQUEST,
            AcpError::MethodNotFound(_) => codes::METHOD_NOT_FOUND,
            AcpError::InvalidParams(_) => codes::INVALID_PARAMS,
            AcpError::InternalError(_) => codes::INTERNAL_ERROR,
            AcpError::ResourceNotFound(_) => codes::RESOURCE_NOT_FOUND,
            AcpError::PermissionDenied(_) => codes::PERMISSION_DENIED,
            AcpError::InvalidState(_) => codes::INVALID_STATE,
            AcpError::CapabilityNotSupported(_) => codes::CAPABILITY_NOT_SUPPORTED,
            AcpError::IoError(_) => codes::INTERNAL_ERROR,
            AcpError::JsonError(_) => codes::PARSE_ERROR,
            AcpError::ChannelError(_) => codes::INTERNAL_ERROR,
            AcpError::ConnectionClosed => codes::INTERNAL_ERROR,
            AcpError::Timeout => codes::INTERNAL_ERROR,
        }
    }

    /// Get the error message.
    pub fn message(&self) -> String {
        self.to_string()
    }
}

/// Result type for ACP operations.
pub type AcpResult<T> = Result<T, AcpError>;
