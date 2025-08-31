//! A simple example for sending a token.

// Examples are allowed to print to stdout for demonstration purposes.
#![allow(clippy::print_stdout)]

use xrpl_rust_client::{XrplClient, XrplError};

#[tokio::main]
async fn main() -> Result<(), XrplError> {
    println!("🚀 XRPL Rust Client - Send Token Example");

    // Create client (using testnet for safety)
    let client = XrplClient::new_testnet();

    // Example test data (these would be real values in production)
    let sender_secret = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
    let recipient_address = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";
    let issuer_address = "rUoCf4ixGkbmxkUEkF4jdB5ajm7Tqd8SfG";
    let currency_code = "USD";
    let amount = "100.50";

    println!("\n📤 Attempting to send {amount} {currency_code}...");

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
        }
        Err(e) => {
            println!("❌ Error sending token: {e}");
            println!(
                "   (This is expected if using example test data on a live testnet without account setup)."
            );
        }
    }

    Ok(())
}
