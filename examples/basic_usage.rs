//! Basic usage example for the XRPL client library.

#![allow(clippy::print_stdout)] // Allow prints in examples

use xrpl_rust_client::{XrplClient, XrplError};

#[tokio::main]
async fn main() -> Result<(), XrplError> {
    println!("🚀 XRPL Rust Client Example");

    // Create client (using testnet for safety)
    let client = XrplClient::new_testnet();

    // Example test data (these would be real values in production)
    let sender_secret = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
    let recipient_address = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";
    let issuer_address = "rUoCf4ixGkbmxkUEkF4jdB5ajm7Tqd8SfG";
    let currency_code = "USD";
    let amount = "100.50";

    println!("\n📤 Part 1: Sending token...");

    // TASK 1 & 2: Send and verify token
    match client
        .send_token(
            sender_secret,
            recipient_address,
            issuer_address,
            currency_code,
            amount,
        )
        .await
    {
        Ok(tx_hash) => {
            println!("✅ Token sent successfully!");
            println!("   Transaction hash: {tx_hash}");

            // Verify the transaction
            println!("\n🔍 Verifying transaction...");

            let sender_addr = xrpl_rust_client::crypto::XrplCrypto::derive_address(sender_secret)?;
            let verified = client
                .verify_token_transfer(
                    &tx_hash,
                    &sender_addr,
                    recipient_address,
                    currency_code,
                    amount,
                )
                .await?;

            if verified {
                println!("✅ Transaction verified successfully!");
            } else {
                println!("❌ Transaction verification failed!");
            }
        }
        Err(e) => {
            println!("❌ Error sending token: {e}");
            println!("   (This is expected if using example test data)");
        }
    }

    println!("\n🔐 Part 2: Offline signing workflow...");

    // TASK 3: Sign transaction offline
    match client
        .sign_transaction_offline(
            sender_secret,
            recipient_address,
            issuer_address,
            currency_code,
            "50.25", // Different amount for second transaction
        )
        .await
    {
        Ok(signed_blob) => {
            println!("✅ Transaction signed offline!");
            println!("   Signed blob length: {} characters", signed_blob.len());

            // TASK 4: Submit the pre-signed transaction
            println!("\n📤 Submitting pre-signed transaction...");
            match client.submit_signed_transaction(&signed_blob).await {
                Ok(tx_hash) => {
                    println!("✅ Pre-signed transaction submitted!");
                    println!("   Transaction hash: {tx_hash}");
                }
                Err(e) => {
                    println!("❌ Error submitting transaction: {e}");
                    println!("   (This is expected with example test data)");
                }
            }
        }
        Err(e) => {
            println!("❌ Error signing transaction: {e}");
        }
    }

    println!("\n🎉 Example completed!");
    println!("   Note: Actual transactions require valid testnet accounts with funding.");

    Ok(())
}
