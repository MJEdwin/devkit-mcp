use base64::{Engine as _, engine::general_purpose::STANDARD};
use encoding_rs::{GB18030, GBK};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use super::TextEncoding;

/// Encodes a string into binary data.
#[mcp_tool(
    name = "string_to_binary",
    title = "String to Binary",
    description = "Encodes a string into binary data using the selected encoding, returned as a base64-encoded string (JSON Schema format: binary)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct StringToBinaryTool {
    /// Text to encode.
    pub text: String,

    /// Text encoding: "utf-8" (default), "ascii", "utf-16", "utf-32", "gbk", "gb18030".
    /// UTF-16/UTF-32 are emitted as little-endian with a BOM.
    #[json_schema(default = "utf-8")]
    pub encoding: Option<TextEncoding>,
}

impl StringToBinaryTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let encoding = self.encoding.unwrap_or(TextEncoding::Utf8);
        let bytes = encode(&self.text, encoding)?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(bytes),
        )]))
    }
}

fn encode(text: &str, encoding: TextEncoding) -> Result<Vec<u8>, CallToolError> {
    match encoding {
        TextEncoding::Utf8 => Ok(text.as_bytes().to_vec()),
        TextEncoding::Ascii => {
            if text.is_ascii() {
                Ok(text.as_bytes().to_vec())
            } else {
                Err(CallToolError::from_message(
                    "text contains non-ASCII characters".to_string(),
                ))
            }
        }
        TextEncoding::Utf16 => {
            let mut bytes = vec![0xFF, 0xFE];
            for unit in text.encode_utf16() {
                bytes.extend_from_slice(&unit.to_le_bytes());
            }
            Ok(bytes)
        }
        TextEncoding::Utf32 => {
            let mut bytes = vec![0xFF, 0xFE, 0x00, 0x00];
            for ch in text.chars() {
                bytes.extend_from_slice(&(ch as u32).to_le_bytes());
            }
            Ok(bytes)
        }
        TextEncoding::Gbk => encode_with_encoding_rs(GBK, text, "GBK"),
        TextEncoding::Gb18030 => encode_with_encoding_rs(GB18030, text, "GB18030"),
    }
}

fn encode_with_encoding_rs(
    encoding: &'static encoding_rs::Encoding,
    text: &str,
    label: &str,
) -> Result<Vec<u8>, CallToolError> {
    let (encoded, _, had_errors) = encoding.encode(text);

    if had_errors {
        Err(CallToolError::from_message(format!(
            "text contains characters not representable in {label}"
        )))
    } else {
        Ok(encoded.into_owned())
    }
}
