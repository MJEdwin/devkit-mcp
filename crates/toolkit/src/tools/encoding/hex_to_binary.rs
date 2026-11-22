use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// Converts a hex string to binary data.
#[mcp_tool(
    name = "hex_to_binary",
    title = "Hex to Binary",
    description = "Converts a hex string to binary data, returned as a base64-encoded string (JSON Schema format: binary)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct HexToBinaryTool {
    /// Hex string to decode. Case-insensitive, whitespace is ignored.
    pub hex: String,
}

impl HexToBinaryTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let cleaned: String = self.hex.chars().filter(|ch| !ch.is_whitespace()).collect();

        let bytes = decode_hex(&cleaned)?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(bytes),
        )]))
    }
}

fn decode_hex(input: &str) -> Result<Vec<u8>, CallToolError> {
    if let Some((offset, ch)) = input.char_indices().find(|(_, ch)| !ch.is_ascii_hexdigit()) {
        return Err(CallToolError::from_message(format!(
            "invalid hex string: invalid character '{ch}' at byte offset {offset}"
        )));
    }

    if !input.len().is_multiple_of(2) {
        return Err(CallToolError::from_message(
            "invalid hex string: odd number of digits".to_string(),
        ));
    }

    hex_simd::decode_to_vec(input.as_bytes())
        .map_err(|err| CallToolError::from_message(format!("invalid hex string: {err}")))
}
