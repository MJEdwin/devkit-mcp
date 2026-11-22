use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use crate::BinaryData;

/// Converts binary data to a hex string.
#[mcp_tool(
    name = "binary_to_hex",
    title = "Binary to Hex",
    description = "Converts binary data (base64-encoded on the wire) to a hex string."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BinaryToHexTool {
    /// Binary payload (base64-encoded on the wire).
    pub data: BinaryData,

    /// Return the hex string in uppercase. Defaults to false (lowercase).
    #[json_schema(default = false)]
    pub uppercase: Option<bool>,
}

impl BinaryToHexTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let bytes = self.data.as_slice();

        let case = if self.uppercase.unwrap_or(false) {
            hex_simd::AsciiCase::Upper
        } else {
            hex_simd::AsciiCase::Lower
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            hex_simd::encode_to_string(bytes, case),
        )]))
    }
}
