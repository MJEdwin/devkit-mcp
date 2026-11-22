use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// Supported UUID versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UuidVersion {
    V4 = 4,
    V7 = 7,
}

impl UuidVersion {
    /// JSON schema for the enum, used by `GenerateUuidTool`'s input schema.
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "type".to_string(),
            serde_json::Value::String("integer".to_string()),
        );
        map.insert(
            "enum".to_string(),
            serde_json::Value::Array(vec![
                serde_json::Value::Number(4.into()),
                serde_json::Value::Number(7.into()),
            ]),
        );
        map
    }
}

impl Serialize for UuidVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for UuidVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match u8::deserialize(deserializer)? {
            4 => Ok(UuidVersion::V4),
            7 => Ok(UuidVersion::V7),
            other => Err(serde::de::Error::custom(format!(
                "unsupported UUID version {other}; expected 4 or 7"
            ))),
        }
    }
}

/// Generates a UUID/GUID.
#[mcp_tool(
    name = "generate_uuid",
    title = "Generate UUID/GUID",
    description = "Generates a UUID/GUID of the requested version. Supported versions: 4 (random) and 7 (time-ordered)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GenerateUuidTool {
    /// UUID version to generate: 4 (random) or 7 (time-ordered).
    pub version: UuidVersion,

    /// Return the UUID in uppercase. Defaults to false (lowercase).
    #[json_schema(default = false)]
    pub uppercase: Option<bool>,

    /// Remove hyphens from the UUID. Defaults to false, which keeps the hyphens.
    #[json_schema(default = false)]
    pub no_hyphens: Option<bool>,
}

impl GenerateUuidTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let id = match self.version {
            UuidVersion::V4 => uuid::Uuid::new_v4(),
            UuidVersion::V7 => uuid::Uuid::now_v7(),
        };

        let mut text = if self.no_hyphens.unwrap_or(false) {
            id.simple().to_string()
        } else {
            id.to_string()
        };

        if self.uppercase.unwrap_or(false) {
            text = text.to_uppercase();
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(text)]))
    }
}
