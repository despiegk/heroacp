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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(codes::PARSE_ERROR, -32700);
        assert_eq!(codes::INVALID_REQUEST, -32600);
        assert_eq!(codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(codes::INVALID_PARAMS, -32602);
        assert_eq!(codes::INTERNAL_ERROR, -32603);
        assert_eq!(codes::RESOURCE_NOT_FOUND, -32001);
        assert_eq!(codes::PERMISSION_DENIED, -32002);
        assert_eq!(codes::INVALID_STATE, -32003);
        assert_eq!(codes::CAPABILITY_NOT_SUPPORTED, -32004);
    }

    #[test]
    fn test_parse_error_code() {
        let error = AcpError::ParseError("invalid json".to_string());
        assert_eq!(error.code(), codes::PARSE_ERROR);
    }

    #[test]
    fn test_invalid_request_code() {
        let error = AcpError::InvalidRequest("missing id".to_string());
        assert_eq!(error.code(), codes::INVALID_REQUEST);
    }

    #[test]
    fn test_method_not_found_code() {
        let error = AcpError::MethodNotFound("unknown/method".to_string());
        assert_eq!(error.code(), codes::METHOD_NOT_FOUND);
    }

    #[test]
    fn test_invalid_params_code() {
        let error = AcpError::InvalidParams("missing field".to_string());
        assert_eq!(error.code(), codes::INVALID_PARAMS);
    }

    #[test]
    fn test_internal_error_code() {
        let error = AcpError::InternalError("unexpected error".to_string());
        assert_eq!(error.code(), codes::INTERNAL_ERROR);
    }

    #[test]
    fn test_resource_not_found_code() {
        let error = AcpError::ResourceNotFound("/path/to/file".to_string());
        assert_eq!(error.code(), codes::RESOURCE_NOT_FOUND);
    }

    #[test]
    fn test_permission_denied_code() {
        let error = AcpError::PermissionDenied("write access".to_string());
        assert_eq!(error.code(), codes::PERMISSION_DENIED);
    }

    #[test]
    fn test_invalid_state_code() {
        let error = AcpError::InvalidState("not initialized".to_string());
        assert_eq!(error.code(), codes::INVALID_STATE);
    }

    #[test]
    fn test_capability_not_supported_code() {
        let error = AcpError::CapabilityNotSupported("audio".to_string());
        assert_eq!(error.code(), codes::CAPABILITY_NOT_SUPPORTED);
    }

    #[test]
    fn test_channel_error_code() {
        let error = AcpError::ChannelError("channel closed".to_string());
        assert_eq!(error.code(), codes::INTERNAL_ERROR);
    }

    #[test]
    fn test_connection_closed_code() {
        let error = AcpError::ConnectionClosed;
        assert_eq!(error.code(), codes::INTERNAL_ERROR);
    }

    #[test]
    fn test_timeout_code() {
        let error = AcpError::Timeout;
        assert_eq!(error.code(), codes::INTERNAL_ERROR);
    }

    #[test]
    fn test_error_message() {
        let error = AcpError::ParseError("invalid json".to_string());
        assert_eq!(error.message(), "Parse error: invalid json");

        let error = AcpError::MethodNotFound("foo".to_string());
        assert_eq!(error.message(), "Method not found: foo");

        let error = AcpError::ConnectionClosed;
        assert_eq!(error.message(), "Connection closed");

        let error = AcpError::Timeout;
        assert_eq!(error.message(), "Request timeout");
    }

    #[test]
    fn test_error_display() {
        let error = AcpError::ResourceNotFound("/test.txt".to_string());
        let display = format!("{}", error);
        assert_eq!(display, "Resource not found: /test.txt");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let acp_error: AcpError = io_error.into();
        assert_eq!(acp_error.code(), codes::INTERNAL_ERROR);
        assert!(acp_error.message().contains("I/O error"));
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "invalid json {";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json_str);
        let json_error = result.unwrap_err();
        let acp_error: AcpError = json_error.into();
        assert_eq!(acp_error.code(), codes::PARSE_ERROR);
    }

    #[test]
    fn test_acp_result_ok() {
        let result: AcpResult<i32> = Ok(42);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_acp_result_err() {
        let result: AcpResult<i32> = Err(AcpError::Timeout);
        assert!(result.is_err());
    }
}
