//! Type definitions for XRPL transactions and responses.

use crate::{Result, XrplError};
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
    ///
    /// # Arguments
    /// * `xrp_amount` - A f64 representing the XRP amount (e.g., 1.5, 100.0).
    ///
    /// # Errors
    /// )]
    pub fn xrp(xrp_amount: f64) -> Result<Self> {
        // Note: Using f64 for currency can be imprecise. For production systems,
        // a fixed-point decimal library would be a better choice.
        // This implementation is a simplification for demonstration purposes.
        if xrp_amount.is_sign_negative() || !xrp_amount.is_finite() {
            return Err(XrplError::validation(
                "XRP amount must be non-negative and finite",
            ));
        }
        let drops = (xrp_amount * 1_000_000.0).round();
        if drops > u64::MAX as f64 {
            return Err(XrplError::validation("XRP amount out of range"));
        }
        Ok(Self::Xrp((drops as u64).to_string()))
    }
}

/// XRPL Payment transaction structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payment {
    /// The type of transaction, which is "Payment".
    #[serde(rename = "TransactionType")]
    pub transaction_type: String,
    /// The sender's XRPL address.
    #[serde(rename = "Account")]
    pub account: String,
    /// The recipient's XRPL address.
    #[serde(rename = "Destination")]
    pub destination: String,
    /// The amount to be sent.
    #[serde(rename = "Amount")]
    pub amount: AmountType,
    /// The transaction fee in drops.
    #[serde(rename = "Fee")]
    pub fee: String,
    /// The sequence number of the sender's account.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
    /// The latest ledger index this transaction can be included in.
    #[serde(rename = "LastLedgerSequence", skip_serializing_if = "Option::is_none")]
    pub last_ledger_sequence: Option<u32>,
    /// The public key used for signing, in hex format.
    #[serde(rename = "SigningPubKey", skip_serializing_if = "Option::is_none")]
    pub signing_pub_key: Option<String>,
    /// The transaction signature, in hex format.
    #[serde(rename = "TxnSignature", skip_serializing_if = "Option::is_none")]
    pub txn_signature: Option<String>,
}

/// Account information response from XRPL
#[derive(Serialize, Deserialize, Debug)]
pub struct AccountInfo {
    /// The XRPL address of the account.
    #[serde(rename = "Account")]
    pub account: String,
    /// The account's balance in drops of XRP.
    #[serde(rename = "Balance")]
    pub balance: String,
    /// The current sequence number of the account.
    #[serde(rename = "Sequence")]
    pub sequence: u32,
}

/// Transaction response from XRPL
#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionResponse {
    /// The sender's account.
    #[serde(rename = "Account")]
    pub account: Option<String>,
    /// The type of the transaction.
    #[serde(rename = "TransactionType")]
    pub transaction_type: Option<String>,
    /// The destination account.
    #[serde(rename = "Destination")]
    pub destination: Option<String>,
    /// The amount transferred.
    #[serde(rename = "Amount")]
    pub amount: Option<AmountType>,
    /// The hash of the transaction.
    #[serde(rename = "hash")]
    pub hash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_type_creation() {
        let token_amount = AmountType::issued_token("100.50", "USD", "rIssuer123");
        assert_eq!(
            token_amount,
            AmountType::IssuedToken {
                value: "100.50".to_string(),
                currency: "USD".to_string(),
                issuer: "rIssuer123".to_string(),
            }
        );

        let xrp_amount = AmountType::xrp(1.5).expect("Creating XRP amount from f64 should succeed");
        assert_eq!(xrp_amount, AmountType::Xrp("1500000".to_string()));
    }
}
