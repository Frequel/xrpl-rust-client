//! Integration tests for XRPL client functionality.

use xrpl_rust_client::{crypto::XrplCrypto, types::AmountType, XrplClient};

#[tokio::test]
async fn test_address_derivation() {
    let secret = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
    let address = XrplCrypto::derive_address(secret).expect("Invalid secret");

    assert!(address.len() >= 25);
    assert!(address.starts_with('r') || address.starts_with('X'));
}

#[tokio::test]
async fn test_amount_type_creation() {
    let token_amount = AmountType::issued_token("100.50", "USD", "rIssuer123");

    #[allow(clippy::panic)]
    match token_amount {
        AmountType::IssuedToken {
            value,
            currency,
            issuer,
        } => {
            assert_eq!(value, "100.50");
            assert_eq!(currency, "USD");
            assert_eq!(issuer, "rIssuer123");
        }
        AmountType::Xrp(_) => panic!("Expected IssuedToken variant"),
    }

    let xrp_amount = AmountType::xrp(1.5);

    #[allow(clippy::panic)]
    match xrp_amount {
        AmountType::Xrp(drops) => {
            assert_eq!(drops, "1500000");
        }
        AmountType::IssuedToken { .. } => panic!("Expected Xrp variant"),
    }
}

#[tokio::test]
async fn test_client_instantiation() {
    let testnet_client = XrplClient::new_testnet();
    let mainnet_client = XrplClient::new_mainnet();
    let custom_client = XrplClient::new_custom("https://test.example.com".to_string());

    // These should not panic
    drop(testnet_client);
    drop(mainnet_client);
    drop(custom_client);
}

#[tokio::test]
async fn test_error_handling() {
    let client = XrplClient::new_custom("http://nonexistent.server".to_string());

    let result = client
        .send_token(
            "sInvalidSecret",
            "rInvalidAddress",
            "rInvalidIssuer",
            "USD",
            "100",
        )
        .await;

    assert!(result.is_err());
}
