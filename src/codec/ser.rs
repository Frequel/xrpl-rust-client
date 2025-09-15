//! # Binary Serialization
//!
//! Implements the binary serialization for XRPL types.

#![allow(dead_code)] // TODO: Remove this when the module is complete.
#![allow(unused_imports)] // TODO: Remove this when all imports are used.

use crate::{
    codec::{get_definitions, FieldInfo},
    types::{AmountType, Payment},
    Result, XrplError,
};
use bs58::Alphabet;
use rust_decimal::prelude::*;
use std::io::Write;

/// A helper struct to build the binary representation of a transaction.
#[derive(Debug, Default)]
pub struct BinarySerializer {
    sink: Vec<u8>,
}

impl BinarySerializer {
    /// Creates a new `BinarySerializer`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a single byte.
    pub fn write_u8(&mut self, value: u8) {
        self.sink.push(value);
    }

    /// Appends a 16-bit unsigned integer in big-endian format.
    pub fn write_u16(&mut self, value: u16) {
        self.sink.extend_from_slice(&value.to_be_bytes());
    }

    /// Appends a 32-bit unsigned integer in big-endian format.
    pub fn write_u32(&mut self, value: u32) {
        self.sink.extend_from_slice(&value.to_be_bytes());
    }

    /// Appends a 64-bit unsigned integer in big-endian format.
    pub fn write_u64(&mut self, value: u64) {
        self.sink.extend_from_slice(&value.to_be_bytes());
    }

    /// Appends a slice of bytes.
    pub fn write_bytes(&mut self, value: &[u8]) {
        self.sink.extend_from_slice(value);
    }

    /// Writes variable-length encoded data.
    pub fn write_vl(&mut self, value: &[u8]) {
        let len = value.len();
        if len <= 192 {
            self.write_u8(len as u8);
        } else if len <= 12480 {
            let len = len - 193;
            self.write_u8(193 + (len >> 8) as u8);
            self.write_u8((len & 0xFF) as u8);
        } else if len <= 918744 {
            let len = len - 12481;
            self.write_u8(241 + (len >> 16) as u8);
            self.write_u8(((len >> 8) & 0xFF) as u8);
            self.write_u8((len & 0xFF) as u8);
        } else {
            // This should be an error, but let's panic for now during development.
            // In a real implementation, this should return a Result.
            panic!("Variable length field is too large: {}", len);
        }
        self.write_bytes(value);
    }

    /// Writes a field ID based on type and field codes.
    fn write_field_id(&mut self, type_code: i32, field_code: i32) {
        let type_code = type_code as u8;
        let field_code = field_code as u8;

        if type_code < 16 && field_code < 16 {
            self.write_u8((type_code << 4) | field_code);
        } else if type_code < 16 && field_code >= 16 {
            self.write_u8(type_code << 4);
            self.write_u8(field_code);
        } else if type_code >= 16 && field_code < 16 {
            self.write_u8(field_code);
            self.write_u8(type_code);
        } else {
            self.write_u8(0);
            self.write_u8(type_code);
            self.write_u8(field_code);
        }
    }

    /// Writes an `AccountID` field.
    pub fn write_account_id(&mut self, address: &str) -> Result<()> {
        let decoded = bs58::decode(address)
            .with_alphabet(Alphabet::RIPPLE)
            .into_vec()
            .map_err(|_| XrplError::InvalidAddress("Invalid address format or checksum".into()))?;

        // The decoded address includes a 1-byte prefix (0x00) and a 4-byte checksum.
        // We only need the 20-byte account ID part.
        if decoded.len() != 25 {
            return Err(XrplError::InvalidAddress(
                "Decoded address has incorrect length".into(),
            ));
        }
        let account_id = &decoded[1..21];
        self.write_bytes(account_id);
        Ok(())
    }

    /// Writes a `Blob` field.
    pub fn write_blob(&mut self, blob: &[u8]) {
        self.write_vl(blob);
    }

    /// Writes a `Currency` field.
    pub fn write_currency(&mut self, currency_code: &str) -> Result<()> {
        if currency_code.len() == 3 {
            // Standard currency code
            let mut currency_bytes = [0u8; 20];
            currency_bytes[12..15].copy_from_slice(currency_code.as_bytes());
            self.write_bytes(&currency_bytes);
        } else if currency_code.len() == 40 {
            // Hex currency code
            let bytes = hex::decode(currency_code)
                .map_err(|_| XrplError::Validation("Invalid hex currency".into()))?;
            self.write_bytes(&bytes);
        } else {
            return Err(XrplError::Validation(
                "Currency must be 3-char or 40-hex".into(),
            ));
        }
        Ok(())
    }

    /// Writes an `Amount` field.
    pub fn write_amount(&mut self, amount: &AmountType) -> Result<()> {
        match amount {
            AmountType::Xrp(drops_str) => {
                let drops = drops_str
                    .parse::<u64>()
                    .map_err(|_| XrplError::Validation("Invalid XRP amount string".into()))?;
                // XRP amount is a 64-bit unsigned integer with the 62nd bit set.
                self.write_u64(drops | 0x4000_0000_0000_0000);
            }
            AmountType::IssuedToken {
                value,
                currency,
                issuer,
            } => {
                self.write_issued_token_amount(value, currency, issuer)?;
            }
        }
        Ok(())
    }

    fn write_issued_token_amount(
        &mut self,
        value_str: &str,
        currency: &str,
        issuer: &str,
    ) -> Result<()> {
        // 1. Serialize the amount value
        let value_dec = Decimal::from_str(value_str)
            .map_err(|_| XrplError::Validation("Invalid token amount string".into()))?;

        if value_dec.is_zero() {
            // Special case for 0, only the "not XRP" bit is set.
            self.write_u64(0x8000_0000_0000_0000);
        } else {
            let is_sign_positive = value_dec.is_sign_positive();
            let scale = value_dec.scale();
            let mut mantissa_u64 = value_dec.mantissa().abs() as u64;

            // Normalize the mantissa and exponent
            let mut exponent_i32 = -(scale as i32);
            while mantissa_u64 > 0 && mantissa_u64 < 1_000_000_000_000_000 {
                if exponent_i32 <= -96 {
                    break;
                }
                mantissa_u64 *= 10;
                exponent_i32 -= 1;
            }
            while mantissa_u64 > 9_999_999_999_999_999 {
                if exponent_i32 >= 80 {
                    break;
                }
                mantissa_u64 /= 10;
                exponent_i32 += 1;
            }

            if exponent_i32 < -96 || exponent_i32 > 80 {
                return Err(XrplError::Validation("Amount out of range".into()));
            }
            if mantissa_u64 > 9_999_999_999_999_999 {
                return Err(XrplError::Validation("Mantissa out of range".into()));
            }

            let mut amount_u64: u64 = 1 << 63; // "not XRP" bit
            if is_sign_positive {
                amount_u64 |= 1 << 62; // sign bit
            }
            amount_u64 |= ((exponent_i32 + 97) as u64) << 54;
            amount_u64 |= mantissa_u64;
            self.write_u64(amount_u64);
        }

        // 2. Serialize the currency
        self.write_currency(currency)?;

        // 3. Serialize the issuer
        self.write_account_id(issuer)?;

        Ok(())
    }

    /// Consumes the serializer and returns the byte buffer.
    pub fn to_vec(self) -> Vec<u8> {
        self.sink
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex;

    #[test]
    fn test_write_vl() {
        let mut serializer = BinarySerializer::new();
        let data = vec![0x01, 0x02, 0x03];
        serializer.write_vl(&data);
        assert_eq!(serializer.to_vec(), vec![0x03, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_write_field_id() {
        let mut serializer = BinarySerializer::new();
        // Type code 2 (UInt32), field code 4 (Sequence)
        serializer.write_field_id(2, 4);
        assert_eq!(serializer.to_vec(), vec![0x24]);
    }

    #[test]
    fn test_write_account_id() -> Result<()> {
        let mut serializer = BinarySerializer::new();
        let address = "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH";
        serializer.write_account_id(address)?;
        let expected_hex = "93B89AFCAD4C8EAC2B131C1331FEF12AE1522BBE";
        assert_eq!(hex::encode_upper(serializer.to_vec()), expected_hex);
        Ok(())
    }

    #[test]
    fn test_write_blob() {
        let mut serializer = BinarySerializer::new();
        let data = hex::decode("0102030405").unwrap();
        serializer.write_blob(&data);
        // VL prefix (0x05) + data
        assert_eq!(hex::encode_upper(serializer.to_vec()), "050102030405");
    }

    #[test]
    fn test_write_currency_standard() -> Result<()> {
        let mut serializer = BinarySerializer::new();
        serializer.write_currency("USD")?;
        let expected_hex = "0000000000000000000000005553440000000000";
        assert_eq!(hex::encode_upper(serializer.to_vec()), expected_hex);
        Ok(())
    }

    #[test]
    fn test_write_currency_hex() -> Result<()> {
        let mut serializer = BinarySerializer::new();
        let hex_code = "015841551A748AD2C1F76FF6ECB0CCCD00000000";
        serializer.write_currency(hex_code)?;
        assert_eq!(hex::encode_upper(serializer.to_vec()), hex_code);
        Ok(())
    }

    #[test]
    fn test_write_amount_xrp() -> Result<()> {
        let mut serializer = BinarySerializer::new();
        let amount = AmountType::Xrp("1000000".to_string()); // 1 XRP
        serializer.write_amount(&amount)?;
        let expected_hex = "40000000000F4240"; // 1,000,000 drops with the 62nd bit set
        assert_eq!(hex::encode_upper(serializer.to_vec()), expected_hex);
        Ok(())
    }

    #[test]
    fn test_write_amount_issued_token() -> Result<()> {
        let mut serializer = BinarySerializer::new();
        let amount = AmountType::IssuedToken {
            value: "123.45".to_string(),
            currency: "USD".to_string(),
            issuer: "rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH".to_string(),
        };
        serializer.write_amount(&amount)?;
        let expected_hex = "D50462C56DF9A800000000000000000000000000555344000000000093B89AFCAD4C8EAC2B131C1331FEF12AE1522BBE";
        assert_eq!(hex::encode_upper(serializer.to_vec()), expected_hex);
        Ok(())
    }
}
