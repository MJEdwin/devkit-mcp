use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// DES/3DES key size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DesKeySize {
    /// DES (8-byte key).
    #[serde(rename = "des")]
    Des,
    /// 2-key Triple DES (16-byte key).
    #[serde(rename = "3des-2key")]
    Tdes2Key,
    /// 3-key Triple DES (24-byte key).
    #[serde(rename = "3des-3key")]
    Tdes3Key,
}

impl DesKeySize {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["des", "3des-2key", "3des-3key"])
    }

    fn bytes(self) -> usize {
        match self {
            DesKeySize::Des => 8,
            DesKeySize::Tdes2Key => 16,
            DesKeySize::Tdes3Key => 24,
        }
    }
}

/// Generates a random DES or Triple DES key.
#[mcp_tool(
    name = "des_generate_key",
    title = "DES Generate Key",
    description = "Generates a random DES or Triple DES key using the operating system RNG: 8 bytes (DES), 16 bytes (2-key 3DES), or 24 bytes (3-key 3DES). Returned as base64-encoded binary."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DesGenerateKeyTool {
    /// Key size: "des" (8 bytes), "3des-2key" (16 bytes), or "3des-3key" (24 bytes). Defaults to "3des-3key".
    #[json_schema(default = "3des-3key")]
    pub key_size: Option<DesKeySize>,
}

impl DesGenerateKeyTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let size = self.key_size.unwrap_or(DesKeySize::Tdes3Key);
        let mut key = vec![0u8; size.bytes()];

        getrandom::fill(&mut key).map_err(|err| {
            CallToolError::from_message(format!("failed to generate random DES key: {err}"))
        })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(key),
        )]))
    }
}
