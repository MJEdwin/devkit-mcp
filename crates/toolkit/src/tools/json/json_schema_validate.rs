use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::json_validate::parser_error;

/// Upper bound on reported schema violations.
const MAX_SCHEMA_ERRORS: usize = 100;

/// Validates a JSON document against a JSON Schema.
#[mcp_tool(
    name = "json_schema_validate",
    title = "JSON Schema Validate",
    description = "Validates a JSON document against a JSON Schema and reports violations with instance and schema paths."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct JsonSchemaValidateTool {
    /// JSON text to validate.
    pub json: String,

    /// JSON Schema to validate against. Accepts a JSON object/boolean or a
    /// JSON-encoded string containing the schema.
    pub json_schema: Value,
}

impl JsonSchemaValidateTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let schema = parse_schema(&self.json_schema)?;
        let validator = jsonschema::validator_for(&schema)
            .map_err(|err| CallToolError::from_message(format!("invalid JSON Schema: {err}")))?;

        let value: Value = serde_json::from_str(&self.json).map_err(|err| {
            CallToolError::from_message(format!(
                "invalid JSON at line {}, column {}: {}",
                err.line().max(1),
                err.column().max(1),
                parser_error(&err)
            ))
        })?;

        let schema_errors: Vec<Value> = validator
            .iter_errors(&value)
            .take(MAX_SCHEMA_ERRORS)
            .map(|err| {
                json!({
                    "path": err.instance_path().as_str(),
                    "schema_path": err.schema_path().as_str(),
                    "message": err.to_string(),
                })
            })
            .collect();

        let mut structured = Map::new();
        let valid = schema_errors.is_empty();
        structured.insert("valid".to_string(), Value::Bool(valid));
        structured.insert("schema_errors".to_string(), Value::Array(schema_errors));

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
                    "description": "Whether the JSON conforms to the schema."
                },
                "schema_errors": {
                    "type": "array",
                    "description": "Schema violations (up to 100); empty when the document is valid.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "JSON Pointer to the offending value in the instance."
                            },
                            "schema_path": {
                                "type": "string",
                                "description": "JSON Pointer to the failing keyword in the schema."
                            },
                            "message": {
                                "type": "string",
                                "description": "Human-readable validation message."
                            }
                        },
                        "required": ["path", "schema_path", "message"]
                    }
                }
            },
            "required": ["valid", "schema_errors"]
        }))
    }
}

/// Accepts either a schema object/boolean or a JSON-encoded schema string.
fn parse_schema(schema: &Value) -> Result<Value, CallToolError> {
    match schema {
        Value::String(text) => serde_json::from_str(text).map_err(|err| {
            CallToolError::from_message(format!("invalid json_schema string: {err}"))
        }),
        other => Ok(other.clone()),
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == JsonSchemaValidateTool::tool_name() {
        Some(JsonSchemaValidateTool::output_schema())
    } else {
        None
    }
}
