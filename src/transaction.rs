//! Transaction building and manipulation utilities.

use crate::{
    types::{AmountType, Payment, TransactionResponse},
    Result, XrplError,
};
use serde_json::Value;

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
            flags: Some(2147483648), // tfFullyCanonicalSig
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
                let _parsed_amount = amount_str
                    .parse::<u64>()
                    .map_err(|_| XrplError::validation("Invalid XRP amount"))?;
            }
            AmountType::IssuedToken {
                value,
                currency,
                issuer,
            } => {
                let _parsed_value = value
                    .parse::<f64>()
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
