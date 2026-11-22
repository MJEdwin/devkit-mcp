use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// DES modes that use an IV.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DesIvMode {
    #[serde(rename = "cbc")]
    Cbc,
    #[serde(rename = "ctr")]
    Ctr,
    #[serde(rename = "cfb")]
    Cfb,
    #[serde(rename = "ofb")]
    Ofb,
}

impl DesIvMode {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["cbc", "ctr", "cfb", "ofb"])
    }
}

/// Generates a random DES/3DES IV.
#[mcp_tool(
    name = "des_generate_iv",
    title = "DES Generate IV",
    description = "Generates a random 8-byte DES/3DES IV for cbc, ctr, cfb, or ofb. Returned as base64-encoded binary."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DesGenerateIvTool {
    /// DES mode the IV will be used with: cbc, ctr, cfb, or ofb.
    pub mode: DesIvMode,
}

impl DesGenerateIvTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let mut iv = vec![0u8; 8];

        getrandom::fill(&mut iv).map_err(|err| {
            CallToolError::from_message(format!("failed to generate random DES IV: {err}"))
        })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(iv),
        )]))
    }
}
