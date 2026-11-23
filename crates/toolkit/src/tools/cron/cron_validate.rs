use std::str::FromStr;

use croner::Cron;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// Validates a cron expression.
#[mcp_tool(
    name = "cron_validate",
    title = "Cron Validate",
    description = "Validates a cron expression (5-7 fields, with optional seconds and year). On success it also returns an English description."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CronValidateTool {
    /// Cron expression, e.g. "*/5 * * * *", "0 */5 * * * *", or a 7-field expression with seconds and year.
    pub expression: String,
}

impl CronValidateTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let mut structured = Map::new();

        match Cron::from_str(&self.expression) {
            Ok(cron) => {
                structured.insert("valid".to_string(), Value::Bool(true));
                structured.insert("error".to_string(), Value::Null);
                structured.insert("description".to_string(), Value::String(cron.describe()));
            }
            Err(err) => {
                structured.insert("valid".to_string(), Value::Bool(false));
                structured.insert("error".to_string(), Value::String(err.to_string()));
                structured.insert("description".to_string(), Value::Null);
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
                    "description": "Whether the expression is a valid cron pattern."
                },
                "error": {
                    "type": ["string", "null"],
                    "description": "Parser error message, or null when valid."
                },
                "description": {
                    "type": ["string", "null"],
                    "description": "English description of the schedule, or null when invalid."
                }
            },
            "required": ["valid", "error", "description"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == CronValidateTool::tool_name() {
        Some(CronValidateTool::output_schema())
    } else {
        None
    }
}
