//! # Binary Codec
//!
//! This module provides the binary serialization and deserialization for
//! XRPL transactions.

pub mod ser;

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Describes a single field in the XRPL protocol.
#[derive(Deserialize, Debug)]
pub struct FieldInfo {
    /// The field's unique identifier, used for sorting.
    pub nth: i32,
    /// Whether the field is variable-length encoded.
    #[serde(rename = "isVLEncoded")]
    pub is_vl_encoded: bool,
    /// Whether the field is serialized.
    #[serde(rename = "isSerialized")]
    pub is_serialized: bool,
    /// Whether the field is included in the signature.
    #[serde(rename = "isSigningField")]
    pub is_signing_field: bool,
    /// The field's data type (e.g., "Amount", "AccountID").
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Contains the definitions for all XRPL protocol fields and types.
#[derive(Deserialize, Debug)]
pub struct Definitions {
    /// A map of type names to their internal type codes.
    #[serde(rename = "TYPES")]
    pub types: HashMap<String, i32>,
    /// A map of transaction type names to their internal type codes.
    #[serde(rename = "TRANSACTION_TYPES")]
    pub transaction_types: HashMap<String, i32>,
    /// A list of all fields with their associated metadata.
    #[serde(rename = "FIELDS")]
    pub fields: Vec<(String, FieldInfo)>,
}

static DEFINITIONS: OnceLock<Definitions> = OnceLock::new();

/// Parses and returns a static reference to the XRPL definitions.
///
/// The definitions are loaded from `definitions.json` and parsed only once.
pub fn get_definitions() -> &'static Definitions {
    DEFINITIONS.get_or_init(|| {
        let json_str = include_str!("codec/definitions.json");
        serde_json::from_str(json_str).expect("Failed to parse definitions.json")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_definitions() {
        let defs = get_definitions();
        assert!(!defs.types.is_empty());
        assert!(!defs.transaction_types.is_empty());
        assert!(!defs.fields.is_empty());

        let payment_type_code = defs.transaction_types.get("Payment");
        assert_eq!(payment_type_code, Some(&0));

        let amount_field_info = defs
            .fields
            .iter()
            .find(|(name, _)| name == "Amount")
            .map(|(_, info)| info);
        assert!(amount_field_info.is_some());
        assert_eq!(amount_field_info.unwrap().r#type, "Amount");
    }
}
