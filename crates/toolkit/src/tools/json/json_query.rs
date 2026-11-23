use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use serde_json_path::JsonPath;

/// Queries a JSON document with a JSONPath expression.
#[mcp_tool(
    name = "json_query",
    title = "JSONPath Query",
    description = "Queries a JSON document with a JSONPath expression (RFC 9535) and returns the matching values."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct JsonQueryTool {
    /// JSON text to query.
    pub json: String,

    /// JSONPath expression, e.g. "$.store.book[*].title" or "$..price".
    pub path: String,

    /// Maximum number of matches to return (1-1000). Defaults to 100.
    #[json_schema(default = 100, minimum = 1, maximum = 1000)]
    pub limit: Option<u32>,
}

impl JsonQueryTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let value: Value = serde_json::from_str(&self.json).map_err(|err| {
            CallToolError::from_message(format!(
                "invalid JSON at line {}, column {}: {err}",
                err.line().max(1),
                err.column().max(1)
            ))
        })?;

        let path = JsonPath::parse(&self.path)
            .map_err(|err| CallToolError::from_message(format!("invalid JSONPath: {err}")))?;

        let limit = self.limit.unwrap_or(100);
        if !(1..=1000).contains(&limit) {
            return Err(CallToolError::from_message(format!(
                "limit must be between 1 and 1000, got {limit}"
            )));
        }

        let nodes = path.query(&value).all();
        let truncated = nodes.len() > limit as usize;
        let matches: Vec<Value> = nodes.into_iter().take(limit as usize).cloned().collect();

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
                    "description": "Matched JSON values.",
                    "items": {}
                }
            },
            "required": ["count", "truncated", "matches"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == JsonQueryTool::tool_name() {
        Some(JsonQueryTool::output_schema())
    } else {
        None
    }
}
