use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// RFC 3986 unreserved characters are left as-is; everything else is percent-encoded.
const QUERY_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// application/x-www-form-urlencoded: alphanumerics plus `*`, `-`, `.`, `_` are left as-is.
const FORM_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

/// Encoding standard used by the `url_encode` tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum EncodingStandard {
    /// Query-string encoding (RFC 3986): space becomes `%20`, `~` is kept.
    #[serde(rename = "query")]
    Query,
    /// Form encoding (application/x-www-form-urlencoded): space becomes `+`, `~` is encoded.
    #[serde(rename = "form")]
    Form,
}

impl EncodingStandard {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["query", "form"])
    }
}

/// Percent-encodes a string.
#[mcp_tool(
    name = "url_encode",
    title = "URL Encode",
    description = "Percent-encodes a string using either query-string (RFC 3986) or form (application/x-www-form-urlencoded) rules."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UrlEncodeTool {
    /// Raw string to encode.
    pub input: String,

    /// Encoding standard: "query" (space becomes %20) or "form" (space becomes +).
    pub standard: EncodingStandard,
}

impl UrlEncodeTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let encoded = match self.standard {
            EncodingStandard::Query => utf8_percent_encode(&self.input, QUERY_SET).to_string(),
            EncodingStandard::Form => utf8_percent_encode(&self.input, FORM_SET)
                .to_string()
                .replace("%20", "+"),
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            encoded,
        )]))
    }
}
