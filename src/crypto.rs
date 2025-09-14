//! Cryptographic utilities for XRPL transactions.

use crate::{Result, XrplError};

use bs58::Alphabet;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256, Sha512};

// secp256k1 (ECDSA)
use secp256k1::{
    ecdsa::Signature as SecpSignature, Message, PublicKey as SecpPublicKey, Secp256k1,
    SecretKey as SecpSecretKey,
};

// Ed25519 (EdDSA)
#[allow(unused_imports)]
use ed25519_dalek::{
    Signature as Ed25519Signature, SigningKey as Ed25519SigningKey,
    VerifyingKey as Ed25519VerifyingKey,
};
#[allow(unused_imports)]
use ed25519_dalek::{Signer, Verifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyAlgorithm {
    Secp256k1,
    Ed25519,
}

/// XRPL cryptographic utilities
#[derive(Debug)]
pub struct XrplCrypto;

impl XrplCrypto {
    /// Decode an XRPL base58 seed and return (algorithm, 32-byte private key seed).
    /// - secp256k1 family-seed (sn…): 1+16+4 = 21 bytes total
    /// - Ed25519 seed (sEd…): 3+16+4 = 23 bytes total (prefix 0x01 0xE1 0x4B)
    fn decode_seed(secret: &str) -> Result<(KeyAlgorithm, [u8; 32])> {
        let decoded = bs58::decode(secret)
            .with_alphabet(Alphabet::RIPPLE)
            .into_vec()
            .map_err(|_| XrplError::InvalidAddress("Invalid secret key format".into()))?;

        match decoded.len() {
            // family-seed (secp256k1): version(1) + entropy(16) + checksum(4)
            21 => {
                // Validate version byte for secp256k1 family seed
                if decoded[0] != 0x21 {
                    return Err(XrplError::InvalidAddress(
                        "Invalid secp256k1 seed version byte".into(),
                    ));
                }

                // Validate checksum
                let payload = &decoded[0..17];
                let expected_checksum = &Sha256::digest(Sha256::digest(payload))[0..4];
                if &decoded[17..21] != expected_checksum {
                    return Err(XrplError::InvalidAddress("Invalid checksum".into()));
                }

                let entropy = &decoded[1..17];
                let hash = Sha512::digest(entropy);
                let mut key = [0u8; 32];
                key.copy_from_slice(&hash[..32]);
                Ok((KeyAlgorithm::Secp256k1, key))
            }

            // Ed25519 seed (sEd…): version(3) + entropy(16) + checksum(4)
            23 => {
                let version = &decoded[0..3];
                if version != [0x01, 0xE1, 0x4B] {
                    return Err(XrplError::InvalidAddress(
                        "Invalid Ed25519 seed version bytes".into(),
                    ));
                }

                // Validate checksum
                let payload = &decoded[0..19];
                let expected_checksum = &Sha256::digest(Sha256::digest(payload))[0..4];
                if &decoded[19..23] != expected_checksum {
                    return Err(XrplError::InvalidAddress("Invalid checksum".into()));
                }

                let entropy = &decoded[3..19];
                let hash = Sha512::digest(entropy);
                let mut key = [0u8; 32];
                key.copy_from_slice(&hash[..32]);
                Ok((KeyAlgorithm::Ed25519, key))
            }

            _ => Err(XrplError::InvalidAddress(format!(
                "Unexpected seed length: {} bytes (expected 21 for secp256k1 or 23 for Ed25519)",
                decoded.len()
            ))),
        }
    }

    /// Compute a classic address from a 33-byte signing public key (compressed secp or 0xED||ed25519)
    fn classic_address_from_pubkey33(pubkey33: &[u8]) -> std::string::String {
        // AccountID = RIPEMD160(SHA256(pubkey33)), then base58-check with version 0x00
        let sha = Sha256::digest(pubkey33);
        let account_id = Ripemd160::digest(sha);

        let mut addr_bytes = Vec::with_capacity(1 + 20 + 4);
        addr_bytes.push(0x00); // version for classic addresses
        addr_bytes.extend_from_slice(&account_id);

        let checksum = Sha256::digest(Sha256::digest(&addr_bytes));
        addr_bytes.extend_from_slice(&checksum[..4]);

        bs58::encode(&addr_bytes)
            .with_alphabet(Alphabet::RIPPLE)
            .into_string()
    }

    /// Derives an XRPL classic address from a secret (supports sn… and sEd…)
    pub fn derive_address(secret: &str) -> Result<String> {
        let (algo, sk32) = Self::decode_seed(secret)?;

        let pubkey33 = match algo {
            KeyAlgorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let sk = SecpSecretKey::from_byte_array(sk32).map_err(XrplError::Crypto)?;
                let pk = SecpPublicKey::from_secret_key(&secp, &sk);
                // Compressed 33-byte secp256k1 public key
                pk.serialize().to_vec()
            }
            KeyAlgorithm::Ed25519 => {
                // 32-byte Ed25519 pubkey, then prefix 0xED to form the 33-byte SigningPubKey
                let signing = Ed25519SigningKey::from_bytes(&sk32);
                let verify = signing.verifying_key();
                let mut out = Vec::with_capacity(33);
                out.push(0xED);
                out.extend_from_slice(&verify.to_bytes());
                out
            }
        };

        Ok(Self::classic_address_from_pubkey33(&pubkey33))
    }

    /// Derives the `SigningPubKey` (hex) from a secret.
    /// - secp256k1: 33-byte compressed public key (hex)
    /// - Ed25519: 0xED || 32-byte pubkey (hex)
    pub fn derive_public_key(secret: &str) -> Result<String> {
        let (algo, sk32) = Self::decode_seed(secret)?;

        let pubkey33 = match algo {
            KeyAlgorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let sk = SecpSecretKey::from_byte_array(sk32)?;
                let pk = SecpPublicKey::from_secret_key(&secp, &sk);
                pk.serialize().to_vec()
            }
            KeyAlgorithm::Ed25519 => {
                let signing = Ed25519SigningKey::from_bytes(&sk32);
                let verify = signing.verifying_key();
                let mut out = Vec::with_capacity(33);
                out.push(0xED);
                out.extend_from_slice(&verify.to_bytes());
                out
            }
        };

        Ok(hex::encode(pubkey33))
    }

    /// Signs a 32-byte transaction hash with the secret (`ECDSA` for `secp256k1`, `EdDSA` for `Ed25519`).
    /// Returns the signature as hex (64-byte compact for secp; 64-byte for `Ed25519`).
    pub fn sign_transaction_hash(tx_hash: &[u8], secret: &str) -> Result<String> {
        if tx_hash.len() != 32 {
            return Err(XrplError::Generic(
                "Hash must be exactly 32 bytes".to_string(),
            ));
        }
        let (algo, sk32) = Self::decode_seed(secret)?;

        match algo {
            KeyAlgorithm::Secp256k1 => {
                let secp = Secp256k1::new();
                let sk = SecpSecretKey::from_byte_array(sk32)?;
                let msg = Message::from_digest(
                    (*tx_hash)
                        .try_into()
                        .map_err(|_| XrplError::Generic("Digest conversion failed".to_string()))?,
                );
                let sig: SecpSignature = secp.sign_ecdsa(msg, &sk);
                Ok(hex::encode(sig.serialize_compact()))
            }
            KeyAlgorithm::Ed25519 => {
                let signing = Ed25519SigningKey::from_bytes(&sk32);
                // For XRPL prod, sign the sha512Half of the serialized tx; here we accept a 32-byte digest for tests.
                let signature: Ed25519Signature = signing.sign(tx_hash);
                Ok(hex::encode(signature.to_bytes()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Digest;

    // Canonical secp256k1 pair from XRPL docs (wallet_propose response).
    // master_seed -> account_id
    const SECP_SEED_1: &str = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";
    const SECP_ADDR_1: &str = "rUYnQuocRiJATDsyf7qBpmBmou1toiFMbv";

    // Example Ed25519 seeds (sEd…), derive addresses at runtime with derive_address.
    const ED25519_SEED_1: &str = "sEd7XBzNyArjGSXYxwf3BpfTMBtR43r";
    #[allow(dead_code)]
    const ED25519_SEED_2: &str = "rMSc7VVTyr1raWM82ywRL5KVhV7wWCtHiV";

    #[test]
    fn test_derive_public_key_secp() {
        let pk_hex = XrplCrypto::derive_public_key(SECP_SEED_1).expect("secp pubkey");
        assert_eq!(pk_hex.len(), 66);
        assert!(pk_hex.starts_with("02") || pk_hex.starts_with("03"));
    }

    #[test]
    fn test_derive_address_secp() {
        let addr1 = XrplCrypto::derive_address(SECP_SEED_1).expect("secp address");
        assert_eq!(addr1, SECP_ADDR_1);
    }

    #[test]
    fn test_derive_public_key_ed25519() {
        let pk_hex = XrplCrypto::derive_public_key(ED25519_SEED_1).expect("ed25519 pubkey");
        assert_eq!(pk_hex.len(), 66);
        assert!(pk_hex.starts_with("ed") || pk_hex.starts_with("ED"));
    }

    #[test]
    fn test_derive_address_ed25519() {
        let addr = XrplCrypto::derive_address(ED25519_SEED_1).expect("ed25519 address");
        assert!(addr.starts_with('r'));
        assert!((25..=35).contains(&addr.len()));
    }

    #[test]
    fn test_sign_and_verify_secp() -> Result<()> {
        let (algo, sk32) = XrplCrypto::decode_seed(SECP_SEED_1)?;
        assert_eq!(algo, KeyAlgorithm::Secp256k1);
        let secp = Secp256k1::new();
        let sk = SecpSecretKey::from_byte_array(sk32)?;
        let pk = SecpPublicKey::from_secret_key(&secp, &sk);

        let digest = Sha256::digest(b"test transaction");
        let sig_hex = XrplCrypto::sign_transaction_hash(&digest, SECP_SEED_1)?;
        let sig_bytes = hex::decode(sig_hex)?;
        let sig = SecpSignature::from_compact(&sig_bytes)?;

        let msg = Message::from_digest(
            (*digest)
                .try_into()
                .map_err(|_| XrplError::Generic("Digest conversion failed".to_string()))?,
        );
        assert!(secp.verify_ecdsa(msg, &sig, &pk).is_ok());
        Ok(())
    }

    #[test]
    fn test_sign_and_verify_ed25519() -> Result<()> {
        let (algo, sk32) = XrplCrypto::decode_seed(ED25519_SEED_1)?;
        assert_eq!(algo, KeyAlgorithm::Ed25519);
        let signing = Ed25519SigningKey::from_bytes(&sk32);
        let verify: Ed25519VerifyingKey = signing.verifying_key();

        let digest = Sha256::digest(b"test transaction");
        let sig_hex = XrplCrypto::sign_transaction_hash(&digest, ED25519_SEED_1)?;
        let sig_bytes = hex::decode(sig_hex)?;
        let sig = Ed25519Signature::from_slice(&sig_bytes)?;
        assert!(verify.verify(&digest, &sig).is_ok());
        Ok(())
    }
}
