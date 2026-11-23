use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::common::{UnicodeMode, escape_non_ascii};

/// Minifies a JSON string.
#[mcp_tool(
    name = "json_minify",
    title = "JSON Minify",
    description = "Parses a JSON string and returns the compact form without insignificant whitespace. By default non-ASCII characters are kept; set `unicode` to \"escapes\" to emit \\uXXXX escapes."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct JsonMinifyTool {
    /// JSON text to minify.
    pub json: String,

    /// Unicode rendering: "characters" keeps real characters (default); "escapes" emits non-ASCII as \uXXXX.
    #[json_schema(default = "characters")]
    pub unicode: Option<UnicodeMode>,
}

impl JsonMinifyTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let value: Value = serde_json::from_str(&self.json).map_err(|err| {
            CallToolError::from_message(format!(
                "invalid JSON at line {}, column {}: {err}",
                err.line(),
                err.column()
            ))
        })?;

        let mut text = serde_json::to_string(&value).map_err(|err| {
            CallToolError::from_message(format!("failed to serialize JSON: {err}"))
        })?;

        if matches!(self.unicode, Some(UnicodeMode::Escapes)) {
            text = escape_non_ascii(&text);
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(text)]))
    }
}
