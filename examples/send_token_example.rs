//! A simple example for sending a token.

// ############################################################################
//
//  This example demonstrates how to send a token.
//
//  **NOTE:** To run this example successfully, you need a funded testnet
//  account. You can obtain one from the XRPL Testnet Faucet:
//  https://xrpl.org/xrp-testnet-faucet.html
//
//  Once you have a testnet account, replace the placeholder values for
//  `sender_secret`, `recipient_address`, and `issuer_address` with your
//  actual testnet credentials.
//
// ############################################################################

// Examples are allowed to print to stdout for demonstration purposes.
#![allow(clippy::print_stdout)]

use xrpl_rust_client::{XrplClient, XrplError};

#[tokio::main]
async fn main() -> Result<(), XrplError> {
    println!("🚀 XRPL Rust Client - Send Token Example");

    // Create client (using testnet for safety)
    let client = XrplClient::new_testnet();

    // --------------------------------------------------------------------------
    //  **Replace these with your actual testnet credentials**
    // --------------------------------------------------------------------------
    let sender_secret = "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9"; // Your sender secret
    let recipient_address = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"; // Your recipient address
    let issuer_address = "rUoCf4ixGkbmxkUEkF4jdB5ajm7Tqd8SfG"; // Your issuer address
                                                               // --------------------------------------------------------------------------

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
                "   (This is expected if using placeholder test data on a live testnet without account setup)."
            );
        }
    }

    Ok(())
}
