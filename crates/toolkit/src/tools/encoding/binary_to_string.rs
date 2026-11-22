use encoding_rs::{GB18030, GBK};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use crate::BinaryData;

/// Text encoding used to decode binary data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TextEncoding {
    #[serde(rename = "utf-8", alias = "utf8")]
    Utf8,
    #[serde(rename = "ascii")]
    Ascii,
    #[serde(rename = "utf-16", alias = "utf16")]
    Utf16,
    #[serde(rename = "utf-32", alias = "utf32")]
    Utf32,
    #[serde(rename = "gbk")]
    Gbk,
    #[serde(rename = "gb18030")]
    Gb18030,
}

impl TextEncoding {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["utf-8", "ascii", "utf-16", "utf-32", "gbk", "gb18030"])
    }
}

/// Decodes binary data into a string.
#[mcp_tool(
    name = "binary_to_string",
    title = "Binary to String",
    description = "Decodes binary data (base64-encoded on the wire) into a string using the selected encoding."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BinaryToStringTool {
    /// Binary payload (base64-encoded on the wire).
    pub data: BinaryData,

    /// Text encoding: "utf-8" (default), "ascii", "utf-16", "utf-32", "gbk", "gb18030".
    /// UTF-16/UTF-32 honor a BOM and default to little-endian when no BOM is present.
    #[json_schema(default = "utf-8")]
    pub encoding: Option<TextEncoding>,
}

impl BinaryToStringTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let encoding = self.encoding.unwrap_or(TextEncoding::Utf8);

        Ok(CallToolResult::text_content(vec![TextContent::from(
            decode(self.data.as_slice(), encoding)?,
        )]))
    }
}

fn decode(bytes: &[u8], encoding: TextEncoding) -> Result<String, CallToolError> {
    match encoding {
        TextEncoding::Utf8 => decode_utf8(bytes),
        TextEncoding::Ascii => decode_ascii(bytes),
        TextEncoding::Utf16 => decode_utf16(bytes),
        TextEncoding::Utf32 => decode_utf32(bytes),
        TextEncoding::Gbk => decode_with_encoding_rs(GBK, bytes, "GBK"),
        TextEncoding::Gb18030 => decode_with_encoding_rs(GB18030, bytes, "GB18030"),
    }
}

fn decode_utf8(bytes: &[u8]) -> Result<String, CallToolError> {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);

    std::str::from_utf8(bytes)
        .map(ToString::to_string)
        .map_err(|err| CallToolError::from_message(format!("input is not valid UTF-8: {err}")))
}

fn decode_ascii(bytes: &[u8]) -> Result<String, CallToolError> {
    if bytes.iter().all(|byte| byte.is_ascii()) {
        // ASCII is a subset of UTF-8.
        Ok(String::from_utf8(bytes.to_vec()).expect("ASCII is valid UTF-8"))
    } else {
        Err(CallToolError::from_message(
            "input contains non-ASCII bytes".to_string(),
        ))
    }
}

fn decode_utf16(bytes: &[u8]) -> Result<String, CallToolError> {
    let (bytes, little_endian) = if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        (rest, true)
    } else if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        (rest, false)
    } else {
        (bytes, true)
    };

    if !bytes.len().is_multiple_of(2) {
        return Err(CallToolError::from_message(
            "input length is not a multiple of 2 for UTF-16".to_string(),
        ));
    }

    let units = bytes.as_chunks::<2>().0.iter().map(|chunk| {
        if little_endian {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        }
    });

    let mut decoded = String::new();
    for unit in char::decode_utf16(units) {
        let ch = unit.map_err(|err| {
            CallToolError::from_message(format!("input is not valid UTF-16: {err}"))
        })?;
        decoded.push(ch);
    }

    Ok(decoded)
}

fn decode_utf32(bytes: &[u8]) -> Result<String, CallToolError> {
    let (bytes, little_endian) = if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE, 0x00, 0x00]) {
        (rest, true)
    } else if let Some(rest) = bytes.strip_prefix(&[0x00, 0x00, 0xFE, 0xFF]) {
        (rest, false)
    } else {
        (bytes, true)
    };

    if !bytes.len().is_multiple_of(4) {
        return Err(CallToolError::from_message(
            "input length is not a multiple of 4 for UTF-32".to_string(),
        ));
    }

    let mut decoded = String::new();
    for (index, chunk) in bytes.as_chunks::<4>().0.iter().enumerate() {
        let value = if little_endian {
            u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        } else {
            u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        };

        let ch = char::from_u32(value).ok_or_else(|| {
            CallToolError::from_message(format!(
                "invalid UTF-32 code point 0x{value:08X} at byte offset {}",
                index * 4
            ))
        })?;
        decoded.push(ch);
    }

    Ok(decoded)
}

fn decode_with_encoding_rs(
    encoding: &'static encoding_rs::Encoding,
    bytes: &[u8],
    label: &str,
) -> Result<String, CallToolError> {
    let (decoded, had_errors) = encoding.decode_without_bom_handling(bytes);

    if had_errors {
        Err(CallToolError::from_message(format!(
            "input is not valid {label}"
        )))
    } else {
        Ok(decoded.into_owned())
    }
}
