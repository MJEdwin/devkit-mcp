use percent_encoding::percent_decode_str;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use super::EncodingStandard;

/// Decodes a percent-encoded string.
#[mcp_tool(
    name = "url_decode",
    title = "URL Decode",
    description = "Decodes a percent-encoded string using either query-string (RFC 3986) or form (application/x-www-form-urlencoded) rules."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UrlDecodeTool {
    /// Percent-encoded string to decode.
    pub input: String,

    /// Decoding standard: "query" (leaves + as-is) or "form" (treats + as space).
    pub standard: EncodingStandard,
}

impl UrlDecodeTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        if !has_valid_percent_escapes(&self.input) {
            return Err(CallToolError::from_message(
                "malformed input: '%' must be followed by two hex digits".to_string(),
            ));
        }

        let input = match self.standard {
            EncodingStandard::Query => self.input.clone(),
            EncodingStandard::Form => self.input.replace('+', " "),
        };

        let decoded = percent_decode_str(&input).decode_utf8().map_err(|err| {
            CallToolError::from_message(format!("decoded bytes are not valid UTF-8: {err}"))
        })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            decoded.into_owned(),
        )]))
    }
}

fn has_valid_percent_escapes(input: &str) -> bool {
    let bytes = input.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' {
            let Some(high) = bytes.get(index + 1) else {
                return false;
            };
            let Some(low) = bytes.get(index + 2) else {
                return false;
            };
            if !high.is_ascii_hexdigit() || !low.is_ascii_hexdigit() {
                return false;
            }
            index += 3;
        } else {
            index += 1;
        }
    }

    true
}
