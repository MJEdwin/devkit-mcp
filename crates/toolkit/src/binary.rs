//! Binary payload type used by MCP tool parameters.
//!
//! `BinaryData` wraps `Vec<u8>` so tool methods work with raw bytes, while
//! serde encodes/decodes it as a base64 string. Its JSON schema is
//! `{"type":"string","format":"binary"}`.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BinaryData(Vec<u8>);

impl BinaryData {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }

    pub fn json_schema() -> Map<String, Value> {
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("string".to_string()));
        map.insert("format".to_string(), Value::String("binary".to_string()));
        map
    }
}

impl Deref for BinaryData {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for BinaryData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for BinaryData {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for BinaryData {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<BinaryData> for Vec<u8> {
    fn from(value: BinaryData) -> Self {
        value.0
    }
}

impl Serialize for BinaryData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for BinaryData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        let cleaned: String = encoded.chars().filter(|ch| !ch.is_whitespace()).collect();

        STANDARD
            .decode(cleaned)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}
