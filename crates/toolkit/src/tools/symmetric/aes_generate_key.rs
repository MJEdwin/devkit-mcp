use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// AES key size in bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AesKeySize {
    Bits128,
    Bits192,
    Bits256,
}

impl AesKeySize {
    fn bits(self) -> usize {
        match self {
            AesKeySize::Bits128 => 128,
            AesKeySize::Bits192 => 192,
            AesKeySize::Bits256 => 256,
        }
    }

    fn bytes(self) -> usize {
        self.bits() / 8
    }

    pub fn json_schema() -> Map<String, Value> {
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("integer".to_string()));
        map.insert(
            "enum".to_string(),
            Value::Array(vec![
                Value::Number(128.into()),
                Value::Number(192.into()),
                Value::Number(256.into()),
            ]),
        );
        map
    }
}

impl Serialize for AesKeySize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(self.bits() as u16)
    }
}

impl<'de> Deserialize<'de> for AesKeySize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match u16::deserialize(deserializer)? {
            128 => Ok(AesKeySize::Bits128),
            192 => Ok(AesKeySize::Bits192),
            256 => Ok(AesKeySize::Bits256),
            other => Err(serde::de::Error::custom(format!(
                "unsupported AES key size {other}; expected 128, 192, or 256"
            ))),
        }
    }
}

/// Generates a random AES key.
#[mcp_tool(
    name = "aes_generate_key",
    title = "AES Generate Key",
    description = "Generates a random AES key (128, 192, or 256 bits) using the operating system RNG and returns it as base64-encoded binary."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AesGenerateKeyTool {
    /// Key size in bits: 128, 192, or 256. Defaults to 256.
    #[json_schema(default = 256)]
    pub bits: Option<AesKeySize>,
}

impl AesGenerateKeyTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let size = self.bits.unwrap_or(AesKeySize::Bits256);
        let mut key = vec![0u8; size.bytes()];

        getrandom::fill(&mut key).map_err(|err| {
            CallToolError::from_message(format!("failed to generate random AES key: {err}"))
        })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(key),
        )]))
    }
}
