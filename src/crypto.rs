//! Cryptographic utilities for XRPL transactions.

use crate::{Result, XrplError};
use bs58::Alphabet;
use ripemd::Ripemd160;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256, Sha512};

/// XRPL cryptographic utilities
#[derive(Debug)]
pub struct XrplCrypto;

impl XrplCrypto {
    /// Decodes a base58 secret and extracts the 32-byte key.
    /// Decodes an XRPL base58 seed (classic or family-seed) and returns a 32-byte
    /// secp256k1 private key derived per XRPL reference (SHA-512 → first 32 bytes).
    fn decode_secret(secret: &str) -> Result<[u8; 32]> {
        let decoded = bs58::decode(secret)
            .with_alphabet(Alphabet::RIPPLE)
            .into_vec()
            .map_err(|_| XrplError::InvalidAddress("Invalid secret key format".into()))?;

        // Accept both classic (22 bytes) and family-seed (21 bytes) layouts
        let (version_byte, entropy, _checksum_len) = match decoded.len() {
            21 => (decoded[0], &decoded[1..17], 4), // family-seed 1+16+4
            22 => (decoded[0], &decoded[1..18], 4), // classic      1+17+4
            _ => {
                return Err(XrplError::InvalidAddress(format!(
                    "Unexpected seed length: {} bytes",
                    decoded.len()
                )))
            }
        };

        // Optional: sanity-check version byte (0x21 testnet, 0x23 mainnet, 0x01 classic)
        let _ = version_byte;

        // Expand to 32-byte private key
        let hash = Sha512::digest(entropy);
        let mut key = [0u8; 32];
        key.copy_from_slice(&hash[..32]);
        Ok(key)
    }

    /// Derives an XRPL address from a secret key
    ///
    /// # Arguments
    /// * `secret` - The secret key in XRPL format (starts with 's')
    ///
    /// # Returns
    /// * `Result<String>` - The derived XRPL address (starts with 'r')
    ///
    /// # Example
    /// ```
    /// use xrpl_rust_client::crypto::XrplCrypto;
    ///
    /// let address = XrplCrypto::derive_address("sn3nxiW7v8KXzPzAqzyHXbSSKNuN9")?;
    /// println!("Address: {}", address);
    /// # Ok::<(), xrpl_rust_client::XrplError>(())
    /// ```
    pub fn derive_address(secret: &str) -> Result<String> {
        // 1. Base58-decode family‐seed
        let key_bytes = Self::decode_secret(secret)?;

        // Create secp256k1 context and secret key
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_byte_array(key_bytes).map_err(XrplError::Crypto)?;

        // Generate public key -
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
        Ok(bs58::encode(&address_bytes)
            .with_alphabet(Alphabet::RIPPLE)
            .into_string())
    }

    /// Derives the public key from a secret key
    ///
    /// # Arguments
    /// * `secret` - The secret key in XRPL format
    ///
    /// # Returns
    /// * `Result<String>` - The public key as a hex string
    pub fn derive_public_key(secret: &str) -> Result<String> {
        let key_bytes = Self::decode_secret(secret)?;
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_byte_array(key_bytes)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(hex::encode(public_key.serialize()))
    }

    /// Signs a transaction hash with the given secret key
    ///
    /// # Arguments
    /// * `tx_hash` - The transaction hash to sign
    /// * `secret` - The secret key for signing
    ///
    /// # Returns
    /// * `Result<String>` - The signature as a hex string
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
        // Using a properly formatted XRPL testnet secret
        let secret = "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9";
        let result = XrplCrypto::derive_address(secret);
        assert!(result.is_ok());

        let address = result.expect("Address derivation should succeed");
        assert!(address.len() >= 25 && address.len() <= 35);
        assert!(address.starts_with('r'));
    }

    #[test]
    fn test_derive_public_key() {
        let secret = "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9";
        let result = XrplCrypto::derive_public_key(secret);
        assert!(result.is_ok());

        let pub_key = result.expect("Public key derivation should succeed");
        assert_eq!(pub_key.len(), 66); // 33 bytes * 2 (hex encoding)
    }

    #[test]
    fn test_sign_and_verify() -> Result<()> {
        let secret = "sn3nxiW7v8KXzPzAqzyHXbSSKNuN9";
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
