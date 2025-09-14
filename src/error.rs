//! Error types for the XRPL client library.

use thiserror::Error;

/// Main error type for XRPL operations
#[derive(Error, Debug)]
pub enum XrplError {
    /// Network-related errors (HTTP, connectivity)
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON parsing errors
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Cryptographic errors
    #[error("Cryptographic error: {0}")]
    Crypto(#[from] secp256k1::Error),

    /// Ed25519 errors
    #[error("Ed25519 error: {0}")]
    Ed25519(#[from] ed25519_dalek::SignatureError),

    /// Hex decoding errors
    #[error("Hex decoding error: {0}")]
    Hex(#[from] hex::FromHexError),

    /// XRPL protocol errors
    #[error("XRPL protocol error: {message}")]
    Protocol {
        /// Error message describing the protocol issue
        message: String,
    },

    /// Transaction validation errors
    #[error("Transaction validation error: {0}")]
    Validation(String),

    /// Address format errors
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    /// Generic errors with context
    #[error("XRPL error: {0}")]
    Generic(String),
}

impl XrplError {
    /// Create a new protocol error
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
        }
    }

    /// Create a new validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Create a new generic error
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic(message.into())
    }
}
