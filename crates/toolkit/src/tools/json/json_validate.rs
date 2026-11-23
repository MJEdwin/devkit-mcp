use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// Validates whether a string is legal JSON.
#[mcp_tool(
    name = "json_validate",
    title = "JSON Validate",
    description = "Validates whether a string is legal JSON. On failure it returns the 1-based line, column, and parser error message."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct JsonValidateTool {
    /// JSON text to validate.
    pub json: String,
}

impl JsonValidateTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let mut structured = Map::new();

        match serde_json::from_str::<Value>(&self.json) {
            Ok(_) => {
                structured.insert("valid".to_string(), Value::Bool(true));
                structured.insert("line".to_string(), Value::Null);
                structured.insert("column".to_string(), Value::Null);
                structured.insert("error".to_string(), Value::Null);
            }
            Err(err) => {
                structured.insert("valid".to_string(), Value::Bool(false));
                // serde_json reports 0/0 for empty input; use 1-based positions.
                structured.insert(
                    "line".to_string(),
                    Value::Number((err.line().max(1) as u64).into()),
                );
                structured.insert(
                    "column".to_string(),
                    Value::Number((err.column().max(1) as u64).into()),
                );
                structured.insert("error".to_string(), Value::String(parser_error(&err)));
            }
        }

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
                "valid": {
                    "type": "boolean",
                    "description": "Whether the input is legal JSON."
                },
                "line": {
                    "type": ["integer", "null"],
                    "minimum": 1,
                    "description": "1-based line of the error, or null when valid."
                },
                "column": {
                    "type": ["integer", "null"],
                    "minimum": 1,
                    "description": "1-based column of the error, or null when valid."
                },
                "error": {
                    "type": ["string", "null"],
                    "description": "Parser error message, or null when valid."
                }
            },
            "required": ["valid", "line", "column", "error"]
        }))
    }
}

/// Drops serde_json's " at line N column M" suffix: the position is reported
/// separately in the structured result.
pub(super) fn parser_error(err: &serde_json::Error) -> String {
    let message = err.to_string();
    match message.rsplit_once(" at line ") {
        Some((reason, _)) => reason.to_string(),
        None => message,
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == JsonValidateTool::tool_name() {
        Some(JsonValidateTool::output_schema())
    } else {
        None
    }
}
