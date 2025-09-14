# XRPL Rust Client Library

A comprehensive Rust library for interacting with the XRP Ledger (XRPL), providing secure and efficient tools for token transfers, transaction verification, and offline signing capabilities.

## 🎯 Overview

This library was developed as part of a technical assessment and implements core XRPL functionality with a focus on security, performance, and ease of use. It provides a complete solution for integrating XRPL operations into Rust applications.

### Key Features

- ✅ **Send Issued Tokens**: Transfer custom tokens between XRPL accounts
- ✅ **Transaction Verification**: Confirm transaction execution on the ledger
- ✅ **Offline Signing**: Sign transactions securely without network exposure
- ✅ **Pre-signed Submission**: Submit transactions signed elsewhere
- ✅ **Dual Cryptography**: Support for both secp256k1 and Ed25519 algorithms
- ✅ **Async/Await**: Full async support with Tokio
- ✅ **Comprehensive Testing**: Unit, integration, and documentation tests
- ✅ **Type Safety**: Strong typing with custom error handling

## 🚀 Quick Start

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
xrpl_rust_client = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use xrpl_rust_client::{XrplClient, XrplError};

#[tokio::main]
async fn main() -> Result<(), XrplError> {
    // Create testnet client
    let client = XrplClient::new_testnet();
    
    // Send 100.50 USD tokens
    let tx_hash = client.send_token(
        "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9",     // sender secret
        "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH", // recipient
        "rUoCf4ixGkbmxkUEkF4jdB5ajm7Tqd8SfG", // token issuer
        "USD",                                 // currency code
        "100.50"                              // amount
    ).await?;
    
    println!("Transaction hash: {}", tx_hash);
    Ok(())
}
```

## 📚 Architecture Overview

The library is organized into several focused modules:

### Core Modules

- **`client`**: Main XRPL client implementation with network operations
- **`crypto`**: Cryptographic utilities for signing and address derivation
- **`transaction`**: Transaction building and validation logic
- **`types`**: Type definitions for XRPL data structures
- **`error`**: Comprehensive error handling types

### Module Details

#### Client Module (`client.rs`)
The heart of the library, implementing the four core functions required by the assessment:

```rust
impl XrplClient {
    // TASK 1: Send token between accounts
    pub async fn send_token(&self, ...) -> Result<String>
    
    // TASK 2: Verify transaction execution
    pub async fn verify_token_transfer(&self, ...) -> Result<bool>
    
    // TASK 3: Sign transaction offline
    pub async fn sign_transaction_offline(&self, ...) -> Result<String>
    
    // TASK 4: Submit pre-signed transaction
    pub async fn submit_signed_transaction(&self, ...) -> Result<String>
}
```

#### Crypto Module (`crypto.rs`)
Handles all cryptographic operations with support for both XRPL signing algorithms:

- **secp256k1**: Traditional ECDSA signatures (family seeds starting with `sn`)
- **Ed25519**: Modern EdDSA signatures (seeds starting with `sEd`)

Key functions:
- `derive_address()`: Generate XRPL addresses from secrets
- `derive_public_key()`: Extract public keys for transactions
- `sign_transaction_hash()`: Create cryptographic signatures

#### Transaction Module (`transaction.rs`)
Provides utilities for building and validating XRPL transactions:

- Transaction construction with proper field validation
- Sequence number and ledger expiration handling
- Response parsing for verification workflows

#### Types Module (`types.rs`)
Defines all XRPL data structures with proper serialization:

```rust
pub enum AmountType {
    Xrp(String),                    // XRP in drops
    IssuedToken {                   // Custom tokens
        value: String,
        currency: String,
        issuer: String,
    },
}

pub struct Payment {
    // All required XRPL payment fields...
}
```

## 🔧 API Reference

### XrplClient

#### Constructors

```rust
// Create mainnet client
let client = XrplClient::new_mainnet();

// Create testnet client (recommended for development)
let client = XrplClient::new_testnet();

// Create custom client
let client = XrplClient::new_custom("https://custom.node.url".to_string());
```

#### Core Methods

##### `send_token`
Send issued tokens between XRPL accounts.

```rust
pub async fn send_token(
    &self,
    user1_secret: &str,      // Sender's secret key
    user2_address: &str,     // Recipient address
    issuer_address: &str,    // Token issuer address
    currency_code: &str,     // Currency code (e.g., "USD")
    amount: &str,            // Amount as decimal string
) -> Result<String>          // Returns transaction hash
```

**Example:**
```rust
let tx_hash = client.send_token(
    "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9",
    "rRecipientAddress123",
    "rTokenIssuerAddress456",
    "USD",
    "150.75"
).await?;
```

##### `verify_token_transfer`
Verify that a specific transaction occurred with expected parameters.

```rust
pub async fn verify_token_transfer(
    &self,
    tx_hash: &str,           // Transaction hash to verify
    sender_address: &str,    // Expected sender
    recipient_address: &str, // Expected recipient
    currency_code: &str,     // Expected currency
    amount: &str,            // Expected amount
) -> Result<bool>            // Returns true if verified
```

**Example:**
```rust
let verified = client.verify_token_transfer(
    "E3FE6EA3D48F0C2B639448020EA4F03D4F4F8FFDB243A852A0F59177921B4879",
    "rSenderAddress123",
    "rRecipientAddress456",
    "USD",
    "150.75"
).await?;

if verified {
    println!("Transaction confirmed!");
}
```

##### `sign_transaction_offline`
Create a signed transaction without submitting to the network.

```rust
pub async fn sign_transaction_offline(
    &self,
    sender_secret: &str,     // Sender's secret key
    recipient_address: &str, // Recipient address
    issuer_address: &str,    // Token issuer address
    currency_code: &str,     // Currency code
    amount: &str,            // Amount to transfer
) -> Result<String>          // Returns hex-encoded signed blob
```

**Example:**
```rust
let signed_blob = client.sign_transaction_offline(
    "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9",
    "rRecipientAddress123",
    "rTokenIssuerAddress456",
    "USD",
    "75.25"
).await?;

// Store signed_blob securely for later submission
```

##### `submit_signed_transaction`
Submit a pre-signed transaction to the network.

```rust
pub async fn submit_signed_transaction(
    &self,
    signed_blob: &str,       // Hex-encoded signed transaction
) -> Result<String>          // Returns transaction hash
```

**Example:**
```rust
let tx_hash = client.submit_signed_transaction(&signed_blob).await?;
println!("Transaction submitted: {}", tx_hash);
```

### Cryptographic Utilities

#### XrplCrypto

```rust
// Derive XRPL address from secret
let address = XrplCrypto::derive_address("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9")?;

// Get public key for transaction signing
let public_key = XrplCrypto::derive_public_key("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9")?;

// Sign transaction hash
let signature = XrplCrypto::sign_transaction_hash(&hash, "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9")?;
```

### Error Handling

The library uses a comprehensive error system:

```rust
pub enum XrplError {
    Network(reqwest::Error),                // HTTP/network errors
    Json(serde_json::Error),                // JSON parsing errors
    Crypto(secp256k1::Error),               // Cryptographic errors
    Ed25519(ed25519_dalek::SignatureError), // Ed25519 errors
    Hex(hex::FromHexError),                 // Hex decoding errors
    Protocol { message: String },           // XRPL protocol errors
    Validation(String),                     // Transaction validation errors
    InvalidAddress(String),                 // Address format errors
    Generic(String),                        // Generic errors
}
```

## 🧪 Testing

The library includes comprehensive testing at multiple levels:

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test crypto::tests

# Run integration tests
cargo test --test integration_tests
```

### Test Categories

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test component interactions
3. **Documentation Tests**: Verify code examples in documentation
4. **Network Tests**: Test real XRPL interactions (marked as `#[ignore]`)

### Using the Test Script

A convenience script is provided for comprehensive testing:

```bash
chmod +x testALL.sh
./testALL.sh
```

This script runs:
- Code formatting (`cargo fmt`)
- Compilation (`cargo build`)
- Linting (`cargo clippy`)
- All tests (`cargo test`)
- Example execution

## 📖 Examples

### Complete Workflow Example

```rust
use xrpl_rust_client::{XrplClient, XrplError, crypto::XrplCrypto};

#[tokio::main]
async fn main() -> Result<(), XrplError> {
    let client = XrplClient::new_testnet();
    
    // Credentials (replace with actual testnet accounts)
    let sender_secret = "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9";
    let recipient_address = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";
    let issuer_address = "rUoCf4ixGkbmxkUEkF4jdB5ajm7Tqd8SfG";
    
    // 1. Send token
    println!("Sending token...");
    match client.send_token(
        sender_secret,
        recipient_address,
        issuer_address,
        "USD",
        "100.50"
    ).await {
        Ok(tx_hash) => {
            println!("✅ Token sent! Hash: {}", tx_hash);
            
            // 2. Verify the transaction
            let sender_addr = XrplCrypto::derive_address(sender_secret)?;
            let verified = client.verify_token_transfer(
                &tx_hash,
                &sender_addr,
                recipient_address,
                "USD",
                "100.50"
            ).await?;
            
            if verified {
                println!("✅ Transaction verified!");
            } else {
                println!("❌ Verification failed!");
            }
        }
        Err(e) => println!("❌ Send failed: {}", e),
    }
    
    // 3. Demonstrate offline signing workflow
    println!("\nDemonstrating offline signing...");
    let signed_blob = client.sign_transaction_offline(
        sender_secret,
        recipient_address,
        issuer_address,
        "USD",
        "50.25"
    ).await?;
    
    println!("✅ Transaction signed offline");
    println!("Signed blob length: {} characters", signed_blob.len());
    
    // 4. Submit pre-signed transaction
    match client.submit_signed_transaction(&signed_blob).await {
        Ok(tx_hash) => println!("✅ Pre-signed transaction submitted: {}", tx_hash),
        Err(e) => println!("❌ Submission failed: {}", e),
    }
    
    Ok(())
}
```

### Ed25519 Support Example

```rust
// The library automatically detects and handles Ed25519 seeds
let ed25519_secret = "sEd7XBzNyArjGSXYxwf3BpfTMBtR43r";
let address = XrplCrypto::derive_address(ed25519_secret)?;
println!("Ed25519 address: {}", address);

// All client methods work seamlessly with Ed25519
let tx_hash = client.send_token(
    ed25519_secret,
    recipient_address,
    issuer_address,
    "EUR",
    "25.0"
).await?;
```

## 🔒 Security Considerations

### Best Practices

1. **Secret Key Management**
   - Never hardcode secrets in production code
   - Use environment variables or secure key management systems
   - Consider using the offline signing workflow for enhanced security

2. **Network Security**
   - Use HTTPS endpoints only (library enforces this)
   - Validate all transaction parameters before signing
   - Implement proper error handling for network failures

3. **Transaction Validation**
   - Always verify transactions after submission
   - Use appropriate fee values (library uses standard 12 drops)
   - Set reasonable ledger expiration limits

### Offline Signing Workflow

For maximum security, use the two-step signing process:

```rust
// Step 1: On air-gapped machine
let signed_blob = client.sign_transaction_offline(/* params */).await?;

// Step 2: On connected machine (different client instance)
let tx_hash = client.submit_signed_transaction(&signed_blob).await?;
```

## 🌐 Network Configuration

### Supported Networks

- **Mainnet**: `XrplClient::new_mainnet()`
- **Testnet**: `XrplClient::new_testnet()` (recommended for development)
- **Custom**: `XrplClient::new_custom(url)` for private networks

### Testnet Setup

To use testnet functionality:

1. Visit [XRPL Testnet Faucet](https://xrpl.org/xrp-testnet-faucet.html)
2. Generate funded testnet accounts
3. Replace example credentials in code with your testnet accounts

## 🛠️ Development

### Building from Source

```bash
git clone <repository-url>
cd xrpl-rust-client
cargo build --release
```

### Development Dependencies

- Rust 1.70+ (2021 edition)
- Tokio for async runtime
- All other dependencies are automatically managed

### Code Quality

The project uses strict linting rules defined in `Cargo.toml`:

```toml
[lints.rust]
warnings = "deny"
unsafe_code = "forbid"
missing_docs = "deny"

[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
```

## 📈 Performance Characteristics

- **Async I/O**: Non-blocking network operations
- **Memory Efficient**: Minimal allocations with string interning
- **CPU Efficient**: Optimized cryptographic operations
- **Network Efficient**: Concurrent request handling

## 🤝 Contributing

This library was created as part of a technical assessment. For production use, consider:

1. Implementing proper STObject serialization instead of JSON
2. Adding more comprehensive transaction types
3. Implementing connection pooling and retry logic
4. Adding more extensive integration tests with funded testnet accounts

## 📄 License

Licensed under either of:
- Apache License, Version 2.0
- MIT License

at your option.

## 🙏 Acknowledgments

- Built for XRPL integration assessment
- Implements Ripple's XRPL protocol specifications
- Uses industry-standard cryptographic libraries (secp256k1, Ed25519)

***

**Note**: This library demonstrates core XRPL functionality and serves as a foundation for more comprehensive XRPL integrations. For production use, additional features like connection pooling, advanced error recovery, and comprehensive logging should be considered.
