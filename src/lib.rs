//! # XRPL Rust Client Library
//!
//! A comprehensive Rust library for interacting with the XRP Ledger (XRPL).
//! This library provides functionality to send tokens, verify transactions,
//! sign transactions offline, and submit pre-signed transactions.
//!
//! ## Features
//!
//! - Send issued tokens between XRPL accounts
//! - Verify transaction execution on the ledger
//! - Offline transaction signing for enhanced security
//! - Submit pre-signed transactions
//! - Full async/await support with Tokio
//! - Comprehensive error handling
//!
//! ## Example
//!
//! ```
//! use xrpl_rust_client::{XrplClient, XrplError};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), XrplError> {
//!     let client = XrplClient::new_testnet();
//!     
//!     let tx_hash = client.send_token(
//!         "sEdTM1uX8pu2do5XvTnutH6HsouMaM2",  // sender secret
//!         "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH", // recipient
//!         "rUoCf4ixGkbmxkUEkF4jdB5ajm7Tqd8SfG", // token issuer
//!         "USD",                                  // currency code
//!         "100.50"                               // amount
//!     ).await?;
//!     
//!     println!("Transaction hash: {}", tx_hash);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod crypto;
pub mod error;
pub mod transaction;
pub mod types;

pub use client::XrplClient;
pub use error::XrplError;
pub use types::*;

// Re-export commonly used types for convenience
/// A convenient type alias for `Result<T, XrplError>`.
pub type Result<T> = std::result::Result<T, XrplError>;
