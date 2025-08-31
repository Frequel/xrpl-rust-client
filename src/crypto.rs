//! Cryptographic utilities for XRPL transactions.

use crate::{Result, XrplError};
use ripemd::Ripemd160;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

/// XRPL cryptographic utilities
#[derive(Debug)]
pub struct XrplCrypto;

impl XrplCrypto {
    /// Decodes a base58 secret and extracts the 32-byte key.
    fn decode_secret(secret: &str) -> Result<[u8; 32]> {
        let decoded = bs58::decode(secret)
            .into_vec()
            .map_err(|_| XrplError::InvalidAddress("Invalid secret key format".to_string()))?;

        if decoded.len() < 33 {
            return Err(XrplError::InvalidAddress(
                "Secret key too short".to_string(),
            ));
        }

        decoded[1..33]
            .try_into()
            .map_err(|_| XrplError::InvalidAddress("Invalid key length".to_string()))
    }

    /// Derives an XRPL address from a secret key
    ///
    /// # Arguments
    /// * `secret` - The secret key in XRPL format (starts with 's')
    ///
    /// # Returns
    /// * `Result<String>` - The derived XRPL address (starts with 'r')
    pub fn derive_address(secret: &str) -> Result<String> {
        let key_bytes = Self::decode_secret(secret)?;

        // Create secp256k1 context and secret key - FIXED API
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_byte_array(key_bytes).map_err(XrplError::Crypto)?;

        // Generate public key
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_bytes = public_key.serialize();

        // Hash the public key: SHA256 then RIPEMD160
        let sha256_hash = Sha256::digest(public_key_bytes);
        let ripemd_hash = Ripemd160::digest(sha256_hash);

        // Add version byte (0x00 for mainnet addresses)
        let mut address_bytes = vec![0x00];
        address_bytes.extend_from_slice(&ripemd_hash);

        // Calculate checksum (double SHA256 of versioned hash)
        let checksum_hash = Sha256::digest(Sha256::digest(&address_bytes));
        address_bytes.extend_from_slice(&checksum_hash[0..4]);

        // Encode with base58
        Ok(bs58::encode(&address_bytes).into_string())
    }

    /// Derives the public key from a secret key
    pub fn derive_public_key(secret: &str) -> Result<String> {
        let key_bytes = Self::decode_secret(secret)?;
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_byte_array(key_bytes)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(hex::encode(public_key.serialize()))
    }

    /// Signs a transaction hash with the given secret key
    pub fn sign_transaction_hash(tx_hash: &[u8], secret: &str) -> Result<String> {
        let key_bytes = Self::decode_secret(secret)?;
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_byte_array(key_bytes)?;

        // Convert hash to proper format - FIXED API
        let hash_array: [u8; 32] = tx_hash
            .try_into()
            .map_err(|_| XrplError::Generic("Hash must be exactly 32 bytes".to_string()))?;

        let message = Message::from_digest(hash_array);

        // Sign with message by value, not reference - FIXED API
        let signature = secp.sign_ecdsa(message, &secret_key);

        Ok(hex::encode(signature.serialize_compact()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_address() {
        let secret = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let expected_address = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";
        let result = XrplCrypto::derive_address(secret);
        assert!(result.is_ok());

        let address = result.expect("Address derivation should succeed");
        assert_eq!(address, expected_address);
    }

    #[test]
    fn test_derive_public_key() {
        let secret = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let result = XrplCrypto::derive_public_key(secret);
        assert!(result.is_ok());

        let pub_key = result.expect("Public key derivation should succeed");
        let expected_pub_key = "02ed0c35553e6020e522451a235e42bc91208e604a462e941fc3e003e92661fa99";
        assert_eq!(pub_key.to_lowercase(), expected_pub_key);
    }

    #[test]
    fn test_sign_and_verify() -> Result<()> {
        let secret = "sEdTM1uX8pu2do5XvTnutH6HsouMaM2";
        let tx_hash = Sha256::digest(b"test transaction").to_vec();

        // 1. Sign the hash
        let signature_hex = XrplCrypto::sign_transaction_hash(&tx_hash, secret)?;

        // 2. Derive the public key
        let public_key_hex = XrplCrypto::derive_public_key(secret)?;

        // 3. Prepare for verification
        let secp = Secp256k1::new();
        let tx_hash_array: [u8; 32] = tx_hash.try_into().map_err(|_| {
            XrplError::Generic("Failed to convert tx_hash to 32-byte array".to_string())
        })?;
        let message = Message::from_digest(tx_hash_array);

        let signature_bytes = hex::decode(signature_hex).expect("Signature hex should be valid");
        let signature = secp256k1::ecdsa::Signature::from_compact(&signature_bytes)?;

        let public_key_bytes = hex::decode(public_key_hex).expect("Public key hex should be valid");
        let public_key = PublicKey::from_slice(&public_key_bytes)?;

        // 4. Verify
        assert!(secp.verify_ecdsa(message, &signature, &public_key).is_ok());
        Ok(())
    }
}
