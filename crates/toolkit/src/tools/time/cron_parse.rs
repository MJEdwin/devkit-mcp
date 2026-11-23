use std::str::FromStr;

use croner::Cron;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::cron_generate::{CronField, CronTerm};

/// Parses a cron expression into its fields and description.
#[mcp_tool(
    name = "cron_parse",
    title = "Cron Parse",
    description = "Parses a cron expression (5-7 fields) into its individual fields and returns an English description as JSON."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CronParseTool {
    /// Cron expression to parse, e.g. "*/5 * * * *" or "0 */5 * * * *".
    pub expression: String,
}

impl CronParseTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let parts: Vec<&str> = self.expression.split_whitespace().collect();

        if !(5..=7).contains(&parts.len()) {
            return Err(CallToolError::from_message(format!(
                "cron expression must have between 5 and 7 fields, got {}",
                parts.len()
            )));
        }

        let normalized = parts.join(" ");
        let cron = Cron::from_str(&normalized).map_err(|err| {
            CallToolError::from_message(format!("invalid cron expression '{normalized}': {err}"))
        })?;

        let (second, minute, hour, day_of_month, month, day_of_week, year) = match parts.len() {
            5 => (None, parts[0], parts[1], parts[2], parts[3], parts[4], None),
            6 => (
                Some(parts[0]),
                parts[1],
                parts[2],
                parts[3],
                parts[4],
                parts[5],
                None,
            ),
            _ => (
                Some(parts[0]),
                parts[1],
                parts[2],
                parts[3],
                parts[4],
                parts[5],
                Some(parts[6]),
            ),
        };

        let mut fields = Map::new();
        let mut raw_fields = Map::new();

        for (name, raw) in [
            ("second", second),
            ("minute", Some(minute)),
            ("hour", Some(hour)),
            ("day_of_month", Some(day_of_month)),
            ("month", Some(month)),
            ("day_of_week", Some(day_of_week)),
            ("year", year),
        ] {
            fields.insert(name.to_string(), structured_field(raw)?);
            raw_fields.insert(name.to_string(), optional_field(raw));
        }

        let mut structured = Map::new();
        structured.insert("expression".to_string(), Value::String(normalized));
        structured.insert(
            "field_count".to_string(),
            Value::Number((parts.len() as u64).into()),
        );
        structured.insert("fields".to_string(), Value::Object(fields));
        structured.insert("raw_fields".to_string(), Value::Object(raw_fields));
        structured.insert("description".to_string(), Value::String(cron.describe()));

        let text =
            serde_json::to_string_pretty(&Value::Object(structured.clone())).map_err(|err| {
                CallToolError::from_message(format!("failed to serialize result: {err}"))
            })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(text)])
            .with_structured_content(structured))
    }

    pub(crate) fn output_schema() -> ToolOutputSchema {
        let field_schema = Value::Object(CronField::json_schema());
        let optional_field_schema = json!({
            "oneOf": [field_schema.clone(), { "type": "null" }]
        });

        crate::tools::object_output_schema(json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Normalized cron expression."
                },
                "field_count": {
                    "type": "integer",
                    "minimum": 5,
                    "maximum": 7,
                    "description": "Number of fields in the expression."
                },
                "fields": {
                    "type": "object",
                    "description": "Parsed fields; names and special syntax are returned as raw strings.",
                    "properties": {
                        "second": optional_field_schema.clone(),
                        "minute": field_schema.clone(),
                        "hour": field_schema.clone(),
                        "day_of_month": field_schema.clone(),
                        "month": field_schema.clone(),
                        "day_of_week": field_schema.clone(),
                        "year": optional_field_schema
                    },
                    "required": [
                        "second",
                        "minute",
                        "hour",
                        "day_of_month",
                        "month",
                        "day_of_week",
                        "year"
                    ]
                },
                "raw_fields": {
                    "type": "object",
                    "description": "Original field strings.",
                    "properties": {
                        "second": { "type": ["string", "null"] },
                        "minute": { "type": "string" },
                        "hour": { "type": "string" },
                        "day_of_month": { "type": "string" },
                        "month": { "type": "string" },
                        "day_of_week": { "type": "string" },
                        "year": { "type": ["string", "null"] }
                    },
                    "required": [
                        "second",
                        "minute",
                        "hour",
                        "day_of_month",
                        "month",
                        "day_of_week",
                        "year"
                    ]
                },
                "description": {
                    "type": "string",
                    "description": "English description of the schedule."
                }
            },
            "required": ["expression", "field_count", "fields", "raw_fields", "description"]
        }))
    }
}

fn structured_field(raw: Option<&str>) -> Result<Value, CallToolError> {
    match raw {
        Some(raw) => serde_json::to_value(parse_field(raw)).map_err(|err| {
            CallToolError::from_message(format!("failed to serialize cron field: {err}"))
        }),
        None => Ok(Value::Null),
    }
}

/// Converts a numeric cron field into the structured `CronField` form. Names
/// and special syntax fall back to a raw string.
fn parse_field(raw: &str) -> CronField {
    let raw = raw.trim();

    if raw.contains(',') {
        let terms: Option<Vec<CronTerm>> = raw.split(',').map(parse_term).collect();

        return terms.map_or_else(|| CronField::Raw(raw.to_string()), CronField::List);
    }

    parse_term(raw).map_or_else(|| CronField::Raw(raw.to_string()), CronField::Term)
}

fn parse_term(term: &str) -> Option<CronTerm> {
    if term == "*" {
        return Some(CronTerm::Wildcard);
    }

    if let Some((base, step)) = term.split_once('/') {
        let step: u32 = step.parse().ok()?;
        if step == 0 {
            return None;
        }

        return match base {
            "*" => Some(CronTerm::Step {
                start: None,
                end: None,
                step,
            }),
            _ => match base.split_once('-') {
                Some((start, end)) => Some(CronTerm::Step {
                    start: Some(start.parse().ok()?),
                    end: Some(end.parse().ok()?),
                    step,
                }),
                None => Some(CronTerm::Step {
                    start: Some(base.parse().ok()?),
                    end: None,
                    step,
                }),
            },
        };
    }

    if let Some((start, end)) = term.split_once('-') {
        return Some(CronTerm::Range {
            start: start.parse().ok()?,
            end: end.parse().ok()?,
        });
    }

    term.parse::<u32>()
        .ok()
        .map(|value| CronTerm::Value { value })
}

fn optional_field(value: Option<&str>) -> Value {
    value.map_or(Value::Null, |value| Value::String(value.to_string()))
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == CronParseTool::tool_name() {
        Some(CronParseTool::output_schema())
    } else {
        None
    }
}
