//! Type definitions for XRPL transactions and responses.

use serde::{Deserialize, Serialize};

/// Represents different amount types in XRPL
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum AmountType {
    /// XRP amounts are represented as strings in drops (1 XRP = 1,000,000 drops)
    Xrp(String),
    /// Issued tokens are represented as objects
    IssuedToken {
        /// The amount as a decimal string
        value: String,
        /// Currency code (3-character for standard, 40-hex for custom)
        currency: String,
        /// The address of the token issuer
        issuer: String,
    },
}

impl AmountType {
    /// Create a new issued token amount
    #[must_use]
    pub fn issued_token(value: &str, currency: &str, issuer: &str) -> Self {
        Self::IssuedToken {
            value: value.to_string(),
            currency: currency.to_string(),
            issuer: issuer.to_string(),
        }
    }

    /// Create an XRP amount from XRP value (converts to drops)
    #[must_use]
    pub fn xrp(xrp_amount: f64) -> Self {
        // Ensure non-negative values and round properly
        let drops = if xrp_amount < 0.0 {
            0_u64
        } else {
            // Use explicit conversion to avoid cast_sign_loss warning
            let rounded = (xrp_amount * 1_000_000.0).round();
            #[allow(clippy::cast_sign_loss)]
            {
                rounded as u64
            }
        };
        Self::Xrp(drops.to_string())
    }
}

/// XRPL Payment transaction structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payment {
    /// Transaction type (always "Payment" for payment transactions)
    #[serde(rename = "TransactionType")]
    pub transaction_type: String,
    /// Account address of the sender
    #[serde(rename = "Account")]
    pub account: String,
    /// Account address of the recipient
    #[serde(rename = "Destination")]
    pub destination: String,
    /// Amount to be transferred (XRP or issued token)
    #[serde(rename = "Amount")]
    pub amount: AmountType,
    /// Transaction fee in drops
    #[serde(rename = "Fee")]
    pub fee: String,
    /// Transaction flags
    #[serde(rename = "Flags", skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,
    /// Account sequence number for transaction ordering
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// Optional ledger sequence number for transaction expiration
    #[serde(rename = "LastLedgerSequence", skip_serializing_if = "Option::is_none")]
    pub last_ledger_sequence: Option<u32>,
    /// Public key used for signing (hex-encoded)
    #[serde(rename = "SigningPubKey", skip_serializing_if = "Option::is_none")]
    pub signing_pub_key: Option<String>,
    /// Transaction signature (hex-encoded)
    #[serde(rename = "TxnSignature", skip_serializing_if = "Option::is_none")]
    pub txn_signature: Option<String>,
}

/// Account information response from XRPL
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountInfo {
    /// Account address
    #[serde(rename = "Account")]
    pub account: String,
    /// Account balance in drops
    #[serde(rename = "Balance")]
    pub balance: String,
    /// Current account sequence number
    #[serde(rename = "Sequence")]
    pub sequence: u32,
}

/// Transaction response from XRPL
#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionResponse {
    /// Transaction sender account address
    #[serde(rename = "Account")]
    pub account: Option<String>,
    /// Type of transaction (e.g., "Payment")
    #[serde(rename = "TransactionType")]
    pub transaction_type: Option<String>,
    /// Transaction destination account address
    #[serde(rename = "Destination")]
    pub destination: Option<String>,
    /// Transaction amount (XRP or issued token)
    #[serde(rename = "Amount")]
    pub amount: Option<AmountType>,
    /// Transaction hash identifier
    #[serde(rename = "hash")]
    pub hash: Option<String>,
}
