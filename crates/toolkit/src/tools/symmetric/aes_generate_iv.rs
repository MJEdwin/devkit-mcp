use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// AES modes that use an IV/nonce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum AesIvMode {
    #[serde(rename = "cbc")]
    Cbc,
    #[serde(rename = "ctr")]
    Ctr,
    #[serde(rename = "cfb")]
    Cfb,
    #[serde(rename = "ofb")]
    Ofb,
    #[serde(rename = "gcm")]
    Gcm,
}

impl AesIvMode {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["cbc", "ctr", "cfb", "ofb", "gcm"])
    }

    fn bytes(self) -> usize {
        match self {
            AesIvMode::Gcm => 12,
            AesIvMode::Cbc | AesIvMode::Ctr | AesIvMode::Cfb | AesIvMode::Ofb => 16,
        }
    }
}

/// Generates a random AES IV/nonce.
#[mcp_tool(
    name = "aes_generate_iv",
    title = "AES Generate IV",
    description = "Generates a random AES IV/nonce using the operating system RNG: 16 bytes for cbc/ctr/cfb/ofb, 12 bytes for gcm. Returned as base64-encoded binary."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AesGenerateIvTool {
    /// AES mode the IV will be used with: cbc, ctr, cfb, ofb, or gcm.
    pub mode: AesIvMode,
}

impl AesGenerateIvTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let mut iv = vec![0u8; self.mode.bytes()];

        getrandom::fill(&mut iv).map_err(|err| {
            CallToolError::from_message(format!("failed to generate random AES IV: {err}"))
        })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(iv),
        )]))
    }
}
