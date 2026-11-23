use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::regex_common::{build_regex, capture_value, match_item_schema};

/// Tests whether text matches a regular expression.
#[mcp_tool(
    name = "regex_test",
    title = "Regex Test",
    description = "Tests whether text matches a regular expression and returns details of the first match. Uses Rust regex syntax (no backreferences or look-around)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RegexTestTool {
    /// Regular expression (Rust regex syntax).
    pub pattern: String,

    /// Text to test.
    pub text: String,

    /// Case-insensitive matching. Defaults to false.
    #[json_schema(default = false)]
    pub case_insensitive: Option<bool>,
}

impl RegexTestTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let regex = build_regex(&self.pattern, self.case_insensitive.unwrap_or(false))?;

        let captures = regex.captures(&self.text);
        let matched = captures.is_some();
        let match_value = captures
            .as_ref()
            .map(|captures| capture_value(&regex, captures))
            .unwrap_or(Value::Null);

        let mut structured = Map::new();
        structured.insert("matched".to_string(), Value::Bool(matched));
        structured.insert("match".to_string(), match_value);

        let text =
            serde_json::to_string_pretty(&Value::Object(structured.clone())).map_err(|err| {
                CallToolError::from_message(format!("failed to serialize result: {err}"))
            })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(text)])
            .with_structured_content(structured))
    }

    pub(crate) fn output_schema() -> ToolOutputSchema {
        crate::tools::object_output_schema(json!({
            "type": "object",
            "properties": {
                "matched": {
                    "type": "boolean",
                    "description": "Whether the pattern matched anywhere in the text."
                },
                "match": {
                    "description": "Details of the first match, or null when there is no match.",
                    "anyOf": [match_item_schema(), { "type": "null" }]
                }
            },
            "required": ["matched", "match"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == RegexTestTool::tool_name() {
        Some(RegexTestTool::output_schema())
    } else {
        None
    }
}
