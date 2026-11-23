use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::regex_common::build_regex;

const MAX_REPLACEMENTS: u32 = 10_000;

/// Replaces regular-expression matches in text.
#[mcp_tool(
    name = "regex_replace",
    title = "Regex Replace",
    description = "Replaces regular-expression matches in text. The replacement supports $1 / ${name} expansion; use $$ for a literal $. Uses Rust regex syntax (no backreferences or look-around)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RegexReplaceTool {
    /// Regular expression (Rust regex syntax).
    pub pattern: String,

    /// Text to transform.
    pub text: String,

    /// Replacement string; supports $1, ${name}, and $$ for a literal $.
    pub replacement: String,

    /// Maximum replacements; 0 means replace all. Defaults to 0.
    #[json_schema(default = 0, minimum = 0, maximum = 10000)]
    pub limit: Option<u32>,

    /// Case-insensitive matching. Defaults to false.
    #[json_schema(default = false)]
    pub case_insensitive: Option<bool>,
}

impl RegexReplaceTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let regex = build_regex(&self.pattern, self.case_insensitive.unwrap_or(false))?;

        let limit = self.limit.unwrap_or(0);
        if limit > MAX_REPLACEMENTS {
            return Err(CallToolError::from_message(format!(
                "limit must be between 0 and {MAX_REPLACEMENTS}, got {limit}"
            )));
        }

        let (result, replacements) = if limit == 0 {
            let replacements = regex.find_iter(&self.text).count();
            (
                regex
                    .replace_all(&self.text, self.replacement.as_str())
                    .to_string(),
                replacements,
            )
        } else {
            let limit = limit as usize;
            let replacements = regex.find_iter(&self.text).take(limit).count();
            (
                regex
                    .replacen(&self.text, limit, self.replacement.as_str())
                    .to_string(),
                replacements,
            )
        };

        let mut structured = Map::new();
        structured.insert("result".to_string(), Value::String(result));
        structured.insert(
            "replacements".to_string(),
            Value::Number((replacements as u64).into()),
        );

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
                "result": {
                    "type": "string",
                    "description": "Text after replacements."
                },
                "replacements": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Number of replacements performed."
                }
            },
            "required": ["result", "replacements"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == RegexReplaceTool::tool_name() {
        Some(RegexReplaceTool::output_schema())
    } else {
        None
    }
}
