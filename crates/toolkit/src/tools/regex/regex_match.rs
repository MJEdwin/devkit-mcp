use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::regex_common::{build_regex, capture_value, match_item_schema};

/// Finds all regular-expression matches in text.
#[mcp_tool(
    name = "regex_match",
    title = "Regex Match",
    description = "Finds all non-overlapping regular-expression matches in text, returning match text, byte offsets, capture groups, and named groups. Uses Rust regex syntax (no backreferences or look-around)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RegexMatchTool {
    /// Regular expression (Rust regex syntax).
    pub pattern: String,

    /// Text to search.
    pub text: String,

    /// Maximum number of matches to return (1-1000). Defaults to 100.
    #[json_schema(default = 100, minimum = 1, maximum = 1000)]
    pub limit: Option<u32>,

    /// Case-insensitive matching. Defaults to false.
    #[json_schema(default = false)]
    pub case_insensitive: Option<bool>,
}

impl RegexMatchTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let regex = build_regex(&self.pattern, self.case_insensitive.unwrap_or(false))?;

        let limit = self.limit.unwrap_or(100);
        if !(1..=1000).contains(&limit) {
            return Err(CallToolError::from_message(format!(
                "limit must be between 1 and 1000, got {limit}"
            )));
        }

        let mut matches = Vec::new();
        let mut truncated = false;
        for captures in regex.captures_iter(&self.text) {
            if matches.len() >= limit as usize {
                truncated = true;
                break;
            }
            matches.push(capture_value(&regex, &captures));
        }

        let mut structured = Map::new();
        structured.insert(
            "count".to_string(),
            Value::Number((matches.len() as u64).into()),
        );
        structured.insert("truncated".to_string(), Value::Bool(truncated));
        structured.insert("matches".to_string(), Value::Array(matches));

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
                "count": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Number of matches returned."
                },
                "truncated": {
                    "type": "boolean",
                    "description": "Whether more matches existed than the limit allowed."
                },
                "matches": {
                    "type": "array",
                    "items": match_item_schema()
                }
            },
            "required": ["count", "truncated", "matches"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == RegexMatchTool::tool_name() {
        Some(RegexMatchTool::output_schema())
    } else {
        None
    }
}
