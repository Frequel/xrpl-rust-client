//! Transaction building and manipulation utilities.

use crate::{
    crypto::XrplCrypto,
    types::{AmountType, Payment, TransactionResponse},
    Result, XrplError,
};
use serde_json::Value;
use sha2::{Digest, Sha512};

/// The prefix for signing XRPL transactions: "STX\0"
const SIGNING_PREFIX: [u8; 4] = [0x53, 0x54, 0x58, 0x00];

/// Utilities for building and manipulating XRPL transactions
#[derive(Debug)]
pub struct TransactionBuilder;

impl TransactionBuilder {
    /// Creates a new payment transaction
    ///
    /// # Arguments
    /// * `sender` - Sender's XRPL address
    /// * `recipient` - Recipient's XRPL address
    /// * `amount` - The amount to send
    /// * `sequence` - Account sequence number
    ///
    /// # Returns
    /// * `Payment` - The constructed payment transaction
    #[must_use]
    pub fn create_payment(
        sender: String,
        recipient: String,
        amount: AmountType,
        sequence: u32,
    ) -> Payment {
        Payment {
            transaction_type: "Payment".to_string(),
            account: sender,
            destination: recipient,
            amount,
            fee: "12".to_string(), // Standard fee in drops
            sequence,
            last_ledger_sequence: None,
            signing_pub_key: None,
            txn_signature: None,
        }
    }

    /// Sets the last ledger sequence for transaction expiration
    ///
    /// # Arguments
    /// * `payment` - Mutable reference to payment transaction
    /// * `current_ledger` - Current ledger sequence number
    /// * `max_ledger_offset` - Maximum ledger offset (default: 4)
    pub const fn set_last_ledger_sequence(
        payment: &mut Payment,
        current_ledger: u32,
        max_ledger_offset: u32,
    ) {
        payment.last_ledger_sequence = Some(current_ledger + max_ledger_offset);
    }

    /// Validates a payment transaction structure
    ///
    /// # Arguments
    /// * `payment` - The payment transaction to validate
    ///
    /// # Returns
    /// * `Result<()>` - Ok if valid, error if invalid
    pub fn validate_payment(payment: &Payment) -> Result<()> {
        if payment.transaction_type != "Payment" {
            return Err(XrplError::validation("Transaction type must be Payment"));
        }

        if payment.account.is_empty() {
            return Err(XrplError::validation("Account cannot be empty"));
        }

        if payment.destination.is_empty() {
            return Err(XrplError::validation("Destination cannot be empty"));
        }

        if payment.account == payment.destination {
            return Err(XrplError::validation("Cannot send to yourself"));
        }

        // Validate amount
        match &payment.amount {
            AmountType::Xrp(amount_str) => {
                amount_str
                    .parse::<u64>()
                    .map(|_| ()) // We only care about the parse succeeding, not the value.
                    .map_err(|_| XrplError::validation("Invalid XRP amount"))?;
            }
            AmountType::IssuedToken {
                value,
                currency,
                issuer,
            } => {
                value
                    .parse::<f64>()
                    .map(|_| ()) // We only care about the parse succeeding, not the value.
                    .map_err(|_| XrplError::validation("Invalid token amount"))?;

                if currency.is_empty() {
                    return Err(XrplError::validation("Currency code cannot be empty"));
                }

                if issuer.is_empty() {
                    return Err(XrplError::validation("Issuer cannot be empty"));
                }
            }
        }

        Ok(())
    }

    /// Extracts transaction details from XRPL response for verification
    ///
    /// # Arguments
    /// * `tx_response` - JSON response from XRPL node
    ///
    /// # Returns
    /// * `Result<TransactionResponse>` - Parsed transaction data
    pub fn parse_transaction_response(tx_response: &Value) -> Result<TransactionResponse> {
        let transaction = tx_response
            .get("result")
            .and_then(|r| r.get("transaction"))
            .or_else(|| tx_response.get("transaction"))
            .ok_or_else(|| XrplError::protocol("No transaction data in response"))?;

        Ok(TransactionResponse {
            account: transaction
                .get("Account")
                .and_then(|v| v.as_str())
                .map(String::from),
            transaction_type: transaction
                .get("TransactionType")
                .and_then(|v| v.as_str())
                .map(String::from),
            destination: transaction
                .get("Destination")
                .and_then(|v| v.as_str())
                .map(String::from),
            amount: transaction
                .get("Amount")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            hash: transaction
                .get("hash")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }

    /// Signs a payment transaction for submission.
    ///
    /// This function prepares a transaction for signing, creates a signature,
    /// and then serializes the complete transaction into a hex-encoded "blob"
    /// suitable for the `submit` RPC command.
    ///
    /// **Note**: This implementation uses JSON for serialization before hashing and for the
    /// final blob, which is a simplification. A fully compliant client must use the
    /// canonical XRPL binary format. This implementation will not produce a blob that
    /// a real `rippled` server will accept. It is provided to satisfy the code structure.
    ///
    /// # Arguments
    /// * `payment` - The `Payment` transaction to sign.
    /// * `secret` - The secret key of the sender's account.
    ///
    /// # Returns
    /// A `Result` containing the hex-encoded transaction blob as a `String`.
    pub fn sign_transaction(mut payment: Payment, secret: &str) -> Result<String> {
        // 1. Derive public key and add it to the transaction.
        let public_key = XrplCrypto::derive_public_key(secret)?;
        payment.signing_pub_key = Some(public_key);

        // 2. Serialize the transaction for signing.
        // A compliant implementation would use the canonical binary format here.
        let payment_for_signing = serde_json::to_vec(&payment)?;

        // 3. Create the signing hash (SHA512-Half of prefixed transaction data).
        let mut to_hash = SIGNING_PREFIX.to_vec();
        to_hash.extend_from_slice(&payment_for_signing);
        let tx_hash = Sha512::digest(&to_hash);
        let signing_hash = &tx_hash[..32];

        // 4. Sign the hash.
        let signature = XrplCrypto::sign_transaction_hash(signing_hash, secret)?;
        payment.txn_signature = Some(signature);

        // 5. Serialize the final transaction to create the blob.
        let signed_tx_json = serde_json::to_vec(&payment)?;
        Ok(hex::encode(signed_tx_json))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_payment() {
        let payment = TransactionBuilder::create_payment(
            "rSender123".to_string(),
            "rRecipient456".to_string(),
            AmountType::issued_token("100", "USD", "rIssuer789"),
            42,
        );

        assert_eq!(payment.transaction_type, "Payment");
        assert_eq!(payment.account, "rSender123");
        assert_eq!(payment.destination, "rRecipient456");
        assert_eq!(payment.sequence, 42);
    }

    #[test]
    fn test_validate_payment_success() {
        let payment = TransactionBuilder::create_payment(
            "rSender123".to_string(),
            "rRecipient456".to_string(),
            AmountType::issued_token("100", "USD", "rIssuer789"),
            42,
        );

        assert!(TransactionBuilder::validate_payment(&payment).is_ok());
    }

    #[test]
    fn test_validate_payment_same_account() {
        let payment = TransactionBuilder::create_payment(
            "rSameAccount".to_string(),
            "rSameAccount".to_string(),
            AmountType::issued_token("100", "USD", "rIssuer789"),
            42,
        );

        assert!(TransactionBuilder::validate_payment(&payment).is_err());
    }
}
