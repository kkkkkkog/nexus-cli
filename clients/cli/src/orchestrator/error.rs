//! Error handling for the orchestrator module

use http::StatusCode;
use prost::DecodeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Failed to decode a Protobuf message from the server
    #[error("Decoding error: {0}")]
    Decode(#[from] DecodeError),

    /// Reqwest error, typically related to network issues or request failures.
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// An error occurred while processing the request.
    #[error("HTTP error with status {status}: {message}")]
    Http { status: u16, message: String },
}

impl OrchestratorError {
    pub async fn from_response(response: reqwest::Response) -> OrchestratorError {
        let status = response.status().as_u16();
        // let message = response
        //     .text()
        //     .await
        //     .unwrap_or_else(|_| "Failed to read response text".to_string());

        let message = describe_status(status).to_string();

        OrchestratorError::Http { status, message }
    }
}

/// Return a short, human-readable description for an HTTP status code.
///
/// Falls back to a tiny custom table when the `http` crate has no
/// canonical reason phrase (e.g. 418 “I’m a teapot”).
pub fn describe_status(code: u16) -> String {
    let default_reason = format!("Unknown status code {}", code);
    match StatusCode::from_u16(code) {
        // Valid RFC-defined status
        Ok(status) => status.canonical_reason().unwrap_or(&default_reason),
        // Any non-RFC code outside 100-599
        Err(_) => default_reason.as_str(),
    }
    .to_string()
}
