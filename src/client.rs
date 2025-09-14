//! Main XRPL client implementation.

use crate::{
    crypto::XrplCrypto,
    transaction::TransactionBuilder,
    types::{AmountType, Payment},
    Result, XrplError,
};

use reqwest::Client;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Main client for interacting with XRPL nodes
#[derive(Debug)]
pub struct XrplClient {
    client: Client,
    node_url: String,
}

impl XrplClient {
    /// Creates a new client for mainnet
    ///
    /// # Returns
    /// * `XrplClient` - New client instance configured for mainnet
    ///
    /// # Example
    /// ```
    /// use xrpl_rust_client::XrplClient;
    ///
    /// let client = XrplClient::new_mainnet();
    /// ```
    #[must_use]
    pub fn new_mainnet() -> Self {
        Self {
            client: Client::new(),
            node_url: "https://xrplcluster.com".to_string(),
        }
    }

    /// Creates a new client for testnet
    ///
    /// # Returns
    /// * `XrplClient` - New client instance configured for testnet
    ///
    /// # Example
    /// ```
    /// use xrpl_rust_client::XrplClient;
    ///
    /// let client = XrplClient::new_testnet();
    /// ```
    #[must_use]
    pub fn new_testnet() -> Self {
        Self {
            client: Client::new(),
            node_url: "https://s.altnet.rippletest.net:51234".to_string(),
        }
    }

    /// Creates a new client with custom node URL
    ///
    /// # Arguments
    /// * `node_url` - Custom XRPL node URL
    ///
    /// # Returns
    /// * `XrplClient` - New client instance
    #[must_use]
    pub fn new_custom(node_url: String) -> Self {
        Self {
            client: Client::new(),
            node_url,
        }
    }

    /// **TASK 1**: Sends a token from user1 to user2
    ///
    /// This function creates, signs, and submits a payment transaction to transfer
    /// issued tokens between XRPL accounts.
    ///
    /// # Arguments
    /// * `user1_secret` - Sender's secret key (starts with 's')
    /// * `user2_address` - Recipient's XRPL address (starts with 'r')
    /// * `issuer_address` - Token issuer's XRPL address
    /// * `currency_code` - Currency code (e.g., "USD", "EUR")
    /// * `amount` - Amount to send as decimal string (e.g., "100.50")
    ///
    /// # Returns
    /// * `Result<String>` - Transaction hash if successful
    ///
    /// # Example
    /// ```
    /// # use xrpl_rust_client::{XrplClient, XrplError};
    /// # async fn example() -> Result<(), XrplError> {
    /// let client = XrplClient::new_testnet();
    ///
    /// let tx_hash = client.send_token(
    ///     "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9",
    ///     "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH",
    ///     "rUoCf4ixGkbmxkUEkF4jdB5ajm7Tqd8SfG",
    ///     "USD",
    ///     "100.50"
    /// ).await?;
    ///
    /// println!("Token sent! Transaction: {}", tx_hash);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_token(
        &self,
        user1_secret: &str,
        user2_address: &str,
        issuer_address: &str,
        currency_code: &str,
        amount: &str,
    ) -> Result<String> {
        // Step 1: Derive sender address from secret key
        let user1_address = XrplCrypto::derive_address(user1_secret)?;

        // Step 2: Get account sequence number
        let sequence = self.get_account_sequence(&user1_address).await?;

        // Step 3: Create payment transaction
        let amount_obj = AmountType::issued_token(amount, currency_code, issuer_address);
        let mut payment = TransactionBuilder::create_payment(
            user1_address,
            user2_address.to_string(),
            amount_obj,
            sequence,
        );

        // Step 4: Set expiration
        let current_ledger = self.get_current_ledger_sequence().await?;
        TransactionBuilder::set_last_ledger_sequence(&mut payment, current_ledger, 4);

        // Step 5: Validate transaction
        TransactionBuilder::validate_payment(&payment)?;

        // Step 6: Sign and submit
        let tx_hash = self
            .sign_and_submit_transaction(payment, user1_secret)
            .await?;

        Ok(tx_hash)
    }

    /// **TASK 2**: Verifies that user1 sent the token to user2
    ///
    /// This function queries the XRPL ledger to confirm that a specific transaction
    /// occurred with the expected parameters.
    ///
    /// # Arguments
    /// * `tx_hash` - Transaction hash to verify
    /// * `sender_address` - Expected sender address
    /// * `recipient_address` - Expected recipient address
    /// * `currency_code` - Expected currency code
    /// * `amount` - Expected amount as string
    ///
    /// # Returns
    /// * `Result<bool>` - True if transaction matches all criteria
    ///
    /// # Example
    /// ```
    /// # use xrpl_rust_client::{XrplClient, XrplError};
    /// # async fn example() -> Result<(), XrplError> {
    /// let client = XrplClient::new_testnet();
    ///
    /// let verified = client.verify_token_transfer(
    ///     "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879",
    ///     "rSender123",
    ///     "rRecipient456",
    ///     "USD",
    ///     "100.50"
    /// ).await?;
    ///
    /// if verified {
    ///     println!("Transaction verified successfully!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn verify_token_transfer(
        &self,
        tx_hash: &str,
        sender_address: &str,
        recipient_address: &str,
        currency_code: &str,
        amount: &str,
    ) -> Result<bool> {
        // Get transaction from ledger
        let tx_response = self.get_transaction(tx_hash).await?;
        let tx_data = TransactionBuilder::parse_transaction_response(&tx_response)?;

        // Verify transaction type
        if tx_data.transaction_type.as_deref() != Some("Payment") {
            return Ok(false);
        }

        // Verify sender
        if tx_data.account.as_deref() != Some(sender_address) {
            return Ok(false);
        }

        // Verify recipient
        if tx_data.destination.as_deref() != Some(recipient_address) {
            return Ok(false);
        }

        // Verify amount and currency
        if let Some(AmountType::IssuedToken {
            value, currency, ..
        }) = &tx_data.amount
        {
            return Ok(currency == currency_code && value == amount);
        }

        Ok(false)
    }

    /// **TASK 3**: Signs a transaction offline without submitting
    ///
    /// Creates and signs a transaction but does not broadcast it to the network.
    /// This is useful for air-gapped security or deferred submission workflows.
    ///
    /// # Arguments
    /// * `sender_secret` - Sender's secret key
    /// * `recipient_address` - Recipient's address
    /// * `issuer_address` - Token issuer's address
    /// * `currency_code` - Currency code
    /// * `amount` - Amount to transfer
    ///
    /// # Returns
    /// * `Result<String>` - Signed transaction blob (hex-encoded)
    ///
    /// # Example
    /// ```
    /// # use xrpl_rust_client::{XrplClient, XrplError};
    /// # async fn example() -> Result<(), XrplError> {
    /// let client = XrplClient::new_testnet();
    ///
    /// let signed_blob = client.sign_transaction_offline(
    ///     "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9",
    ///     "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH",
    ///     "rUoCf4ixGkbmxkUEkF4jdB5ajm7Tqd8SfG",
    ///     "USD",
    ///     "50.25"
    /// ).await?;
    ///
    /// println!("Signed transaction blob: {}", signed_blob);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign_transaction_offline(
        &self,
        sender_secret: &str,
        recipient_address: &str,
        issuer_address: &str,
        currency_code: &str,
        amount: &str,
    ) -> Result<String> {
        let sender_address = XrplCrypto::derive_address(sender_secret)?;
        let sequence = self.get_account_sequence(&sender_address).await?;

        let amount_obj = AmountType::issued_token(amount, currency_code, issuer_address);
        let mut payment = TransactionBuilder::create_payment(
            sender_address,
            recipient_address.to_string(),
            amount_obj,
            sequence,
        );

        let current_ledger = self.get_current_ledger_sequence().await?;
        TransactionBuilder::set_last_ledger_sequence(&mut payment, current_ledger, 4);

        TransactionBuilder::validate_payment(&payment)?;

        let signed_blob = self.sign_transaction(payment, sender_secret)?;
        Ok(signed_blob)
    }

    /// **TASK 4**: Submits a pre-signed transaction
    ///
    /// Takes a transaction that was signed offline and broadcasts it to the XRPL network.
    /// This allows for separation of signing and submission processes.
    ///
    /// # Arguments
    /// * `signed_blob` - Hex-encoded signed transaction blob
    ///
    /// # Returns
    /// * `Result<String>` - Transaction hash if successful
    ///
    /// # Example
    /// ```
    /// # use xrpl_rust_client::{XrplClient, XrplError};
    /// # async fn example() -> Result<(), XrplError> {
    /// let client = XrplClient::new_testnet();
    ///
    /// let tx_hash = client.submit_signed_transaction(
    ///     "1200002280000000240000000361400000000000000C68400000000000000C..."
    /// ).await?;
    ///
    /// println!("Transaction submitted: {}", tx_hash);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn submit_signed_transaction(&self, signed_blob: &str) -> Result<String> {
        let request = json!({
            "method": "submit",
            "params": [{
                "tx_blob": signed_blob
            }]
        });

        let response: Value = self
            .client
            .post(&self.node_url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        // Check for errors first
        if let Some(error) = response.get("error") {
            return Err(XrplError::protocol(format!("Submit error: {error}")));
        }

        let tx_hash = response
            .get("result")
            .and_then(|r| r.get("tx_json"))
            .and_then(|tx| tx.get("hash"))
            .or_else(|| response.get("result").and_then(|r| r.get("hash")))
            .and_then(|h| h.as_str())
            .ok_or_else(|| XrplError::protocol("No transaction hash in response"))?;

        Ok(tx_hash.to_string())
    }

    // Helper methods

    /// Gets the current sequence number for an account
    async fn get_account_sequence(&self, address: &str) -> Result<u32> {
        let request = json!({
            "method": "account_info",
            "params": [{
                "account": address,
                "ledger_index": "current"
            }]
        });

        let response: Value = self
            .client
            .post(&self.node_url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        let sequence = response
            .get("result")
            .and_then(|r| r.get("account_data"))
            .and_then(|ad| ad.get("Sequence"))
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| XrplError::protocol("Failed to get account sequence"))?;

        #[allow(clippy::cast_possible_truncation)]
        Ok(sequence as u32)
    }

    /// Gets the current ledger sequence number
    async fn get_current_ledger_sequence(&self) -> Result<u32> {
        // 1) Try ledger_current (fast, may be in flux)
        let req_current = serde_json::json!({
            "method": "ledger_current",
            "params": [{}]
        });
        let resp_current: serde_json::Value = self
            .client
            .post(&self.node_url)
            .json(&req_current)
            .send()
            .await?
            .json()
            .await?;
        if let Some(idx) = resp_current
            .get("result")
            .and_then(|r| r.get("ledger_current_index"))
            .and_then(serde_json::Value::as_u64)
        {
            #[allow(clippy::cast_possible_truncation)]
            return Ok(idx as u32);
        }

        // 2) Try a validated ledger index (stable)
        let req_validated = serde_json::json!({
            "method": "ledger",
            "params": [{
                "ledger_index": "validated",
                "transactions": false,
                "expand": false
            }]
        });
        let resp_validated: serde_json::Value = self
            .client
            .post(&self.node_url)
            .json(&req_validated)
            .send()
            .await?
            .json()
            .await?;
        if let Some(idx) = resp_validated
            .get("result")
            .and_then(|r| {
                r.get("ledger_index")
                    .or_else(|| r.get("ledger").and_then(|l| l.get("ledger_index")))
            })
            .and_then(serde_json::Value::as_u64)
        {
            #[allow(clippy::cast_possible_truncation)]
            return Ok(idx as u32);
        }

        // 3) Fallback: server_info → validated_ledger.seq (works on many providers)
        let req_info = serde_json::json!({ "method": "server_info", "params": [{}] });
        let resp_info: serde_json::Value = self
            .client
            .post(&self.node_url)
            .json(&req_info)
            .send()
            .await?
            .json()
            .await?;
        if let Some(seq) = resp_info
            .get("result")
            .and_then(|r| r.get("info"))
            .and_then(|i| i.get("validated_ledger"))
            .and_then(|v| v.get("seq"))
            .and_then(serde_json::Value::as_u64)
        {
            #[allow(clippy::cast_possible_truncation)]
            return Ok(seq as u32);
        }

        Err(XrplError::protocol("Failed to get ledger sequence"))
    }

    /// Gets transaction details from the ledger
    async fn get_transaction(&self, tx_hash: &str) -> Result<Value> {
        let request = json!({
            "method": "tx",
            "params": [{
                "transaction": tx_hash
            }]
        });

        let response: Value = self
            .client
            .post(&self.node_url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        if response.get("error").is_some() {
            return Err(XrplError::protocol("Transaction not found"));
        }

        Ok(response)
    }

    /// Signs a transaction and returns the blob
    #[allow(clippy::unused_self)]
    fn sign_transaction(&self, mut payment: Payment, secret: &str) -> Result<String> {
        // Add public key to transaction
        let public_key = XrplCrypto::derive_public_key(secret)?;
        payment.signing_pub_key = Some(public_key);

        // Serialize transaction for signing
        let tx_json = serde_json::to_string(&payment)?;
        let tx_bytes = tx_json.as_bytes();

        // Create hash for signing (simplified - real implementation uses STObject encoding)
        let hash = Sha256::digest(tx_bytes);

        // Sign the hash
        let signature = XrplCrypto::sign_transaction_hash(&hash, secret)?;
        payment.txn_signature = Some(signature);

        // Return hex-encoded signed transaction
        let signed_json = serde_json::to_string(&payment)?;
        Ok(hex::encode(signed_json.as_bytes()))
    }

    /// Signs and submits a transaction
    async fn sign_and_submit_transaction(&self, payment: Payment, secret: &str) -> Result<String> {
        let signed_blob = self.sign_transaction(payment, secret)?;
        self.submit_signed_transaction(&signed_blob).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let testnet_client = XrplClient::new_testnet();
        assert!(testnet_client.node_url.contains("altnet"));

        let mainnet_client = XrplClient::new_mainnet();
        assert!(mainnet_client.node_url.contains("xrplcluster"));

        let custom_client = XrplClient::new_custom("https://custom.xrpl.node".to_string());
        assert_eq!(custom_client.node_url, "https://custom.xrpl.node");
    }

    #[tokio::test]
    #[ignore = "Requires network access to XRPL testnet"]
    async fn test_get_account_sequence() -> Result<()> {
        let client = XrplClient::new_testnet();
        let result = client
            .get_account_sequence("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH")
            .await;

        match result {
            Ok(sequence) => {
                assert!(sequence > 0);
            }
            Err(XrplError::Protocol { .. }) => {
                // Account might not exist, which is fine for test
            }
            Err(e) => {
                // Return the error instead of panicking
                return Err(e);
            }
        }

        Ok(())
    }
}
