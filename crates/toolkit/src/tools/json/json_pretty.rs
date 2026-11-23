use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};
use serde_json::ser::{PrettyFormatter, Serializer};
use serde_json::{Map, Value};

/// How `json_pretty` renders non-ASCII characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum UnicodeMode {
    #[serde(rename = "characters")]
    Characters,
    #[serde(rename = "escapes")]
    Escapes,
}

impl UnicodeMode {
    pub fn json_schema() -> Map<String, Value> {
        crate::tools::string_enum_schema(&["characters", "escapes"])
    }
}

/// Pretty-prints a JSON string.
#[mcp_tool(
    name = "json_pretty",
    title = "JSON Pretty Print",
    description = "Parses a JSON string and returns it pretty-printed with newlines and configurable indentation. By default \\uXXXX escapes are decoded to real characters; set `unicode` to \"escapes\" to emit non-ASCII characters as \\uXXXX."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct JsonPrettyTool {
    /// JSON text to format.
    pub json: String,

    /// Number of spaces per indentation level, 0-8. Defaults to 2.
    #[json_schema(default = 2, minimum = 0, maximum = 8)]
    pub indent: Option<u8>,

    /// Unicode rendering: "characters" decodes \uXXXX escapes to real characters (default); "escapes" emits non-ASCII as \uXXXX.
    #[json_schema(default = "characters")]
    pub unicode: Option<UnicodeMode>,
}

impl JsonPrettyTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let indent = self.indent.unwrap_or(2);
        if indent > 8 {
            return Err(CallToolError::from_message(format!(
                "indent must be between 0 and 8, got {indent}"
            )));
        }

        let value: Value = serde_json::from_str(&self.json).map_err(|err| {
            CallToolError::from_message(format!(
                "invalid JSON at line {}, column {}: {err}",
                err.line(),
                err.column()
            ))
        })?;

        let indent_bytes = vec![b' '; usize::from(indent)];
        let formatter = PrettyFormatter::with_indent(&indent_bytes);
        let mut output = Vec::new();
        let mut serializer = Serializer::with_formatter(&mut output, formatter);

        value.serialize(&mut serializer).map_err(|err| {
            CallToolError::from_message(format!("failed to serialize JSON: {err}"))
        })?;

        let mut text = String::from_utf8(output).map_err(|err| {
            CallToolError::from_message(format!("failed to decode formatted JSON: {err}"))
        })?;

        if matches!(self.unicode, Some(UnicodeMode::Escapes)) {
            text = escape_non_ascii(&text);
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(text)]))
    }
}

/// Replaces every non-ASCII character with a JSON `\uXXXX` escape, using a
/// surrogate pair for code points above U+FFFF.
fn escape_non_ascii(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for character in text.chars() {
        let code = character as u32;
        if code <= 0x7F {
            result.push(character);
        } else if code <= 0xFFFF {
            result.push_str(&format!("\\u{code:04x}"));
        } else {
            let code = code - 0x1_0000;
            let high = 0xD800 + (code >> 10);
            let low = 0xDC00 + (code & 0x3FF);
            result.push_str(&format!("\\u{high:04x}\\u{low:04x}"));
        }
    }

    result
}
