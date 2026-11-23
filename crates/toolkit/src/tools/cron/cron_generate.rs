use std::str::FromStr;

use croner::Cron;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// One term of a cron field: a numeric value, a range, or a step.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CronTerm {
    /// A wildcard, `*`.
    Wildcard,

    /// A single numeric value, e.g. `5`.
    Value { value: u32 },

    /// A numeric range, e.g. `1-5`.
    Range { start: u32, end: u32 },

    /// A step, e.g. `*/5`, `1-10/2`, or `5/10`.
    Step {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        start: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        end: Option<u32>,
        step: u32,
    },
}

impl CronTerm {
    fn render(&self) -> Result<String, CallToolError> {
        match self {
            Self::Wildcard => Ok("*".to_string()),
            Self::Value { value } => Ok(value.to_string()),
            Self::Range { start, end } => Ok(format!("{start}-{end}")),
            Self::Step { start, end, step } => {
                if *step == 0 {
                    return Err(CallToolError::from_message("step must be greater than 0"));
                }

                match (start, end) {
                    (None, None) => Ok(format!("*/{step}")),
                    (Some(start), None) => Ok(format!("{start}/{step}")),
                    (Some(start), Some(end)) => Ok(format!("{start}-{end}/{step}")),
                    (None, Some(_)) => Err(CallToolError::from_message(
                        "step end requires a start value",
                    )),
                }
            }
        }
    }
}

/// A cron field: raw syntax, a single value, one structured term, or a list.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CronField {
    /// Raw numeric cron field syntax such as `*/5`, `1-5`, or `1,3,5`.
    Raw(String),

    /// A single numeric value.
    Number(u32),

    /// One structured term.
    Term(CronTerm),

    /// An enumeration of terms, joined with commas.
    List(Vec<CronTerm>),
}

impl CronField {
    fn render(&self) -> Result<String, CallToolError> {
        match self {
            Self::Raw(value) => {
                let value = value.trim();
                if value.is_empty() {
                    return Err(CallToolError::from_message("cron field must not be empty"));
                }
                if value.chars().any(char::is_whitespace) {
                    return Err(CallToolError::from_message(
                        "cron field must not contain whitespace",
                    ));
                }
                if value
                    .chars()
                    .any(|character| character.is_ascii_alphabetic())
                {
                    return Err(CallToolError::from_message(
                        "named cron values are not supported; use numbers or structured terms",
                    ));
                }
                Ok(value.to_string())
            }
            Self::Number(value) => Ok(value.to_string()),
            Self::Term(term) => term.render(),
            Self::List(terms) => {
                if terms.is_empty() {
                    return Err(CallToolError::from_message(
                        "cron field list must not be empty",
                    ));
                }

                let rendered: Vec<String> = terms
                    .iter()
                    .map(CronTerm::render)
                    .collect::<Result<_, _>>()?;

                Ok(rendered.join(","))
            }
        }
    }

    pub fn json_schema() -> Map<String, Value> {
        json!({
            "oneOf": [
                {
                    "type": "string",
                    "description": "Raw numeric cron field syntax, e.g. \"*/5\", \"1-5\", or \"1,3,5\"."
                },
                {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Single numeric value."
                },
                cron_term_schema(),
                {
                    "type": "array",
                    "description": "Enumeration of terms, joined with commas.",
                    "minItems": 1,
                    "items": cron_term_schema()
                }
            ]
        })
        .as_object()
        .cloned()
        .expect("the cron field schema is always an object")
    }
}

fn cron_number_schema() -> Value {
    json!({ "type": "integer", "minimum": 0 })
}

fn cron_term_schema() -> Value {
    json!({
        "oneOf": [
            {
                "type": "object",
                "properties": {
                    "type": { "const": "wildcard" }
                },
                "required": ["type"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "type": { "const": "value" },
                    "value": cron_number_schema()
                },
                "required": ["type", "value"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "type": { "const": "range" },
                    "start": cron_number_schema(),
                    "end": cron_number_schema()
                },
                "required": ["type", "start", "end"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "type": { "const": "step" },
                    "start": cron_number_schema(),
                    "end": cron_number_schema(),
                    "step": { "type": "integer", "minimum": 1 }
                },
                "required": ["type", "step"],
                "additionalProperties": false
            }
        ]
    })
}

/// Builds a cron expression from per-field values.
#[mcp_tool(
    name = "cron_generate",
    title = "Cron Generate",
    description = "Builds a cron expression from per-field values. Fields accept raw syntax, single values, or structured value/range/step terms and lists. Provide second (optional) for a 6-field expression; provide year for a 7-field expression."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CronGenerateTool {
    /// Seconds field, optional. Include it to build a 6- or 7-field expression.
    pub second: Option<CronField>,

    /// Minutes field, e.g. { "type": "step", "step": 5 } for */5.
    pub minute: CronField,

    /// Hours field, e.g. { "type": "value", "value": 9 } for 9.
    pub hour: CronField,

    /// Day-of-month field, e.g. { "type": "range", "start": 1, "end": 15 }.
    pub day_of_month: CronField,

    /// Month field (1-12), e.g. { "type": "range", "start": 1, "end": 6 }.
    pub month: CronField,

    /// Day-of-week field (0-7), e.g. { "type": "range", "start": 1, "end": 5 }.
    pub day_of_week: CronField,

    /// Year field, optional. Requires `second` to be provided.
    pub year: Option<CronField>,
}

impl CronGenerateTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let second = self.second.as_ref().map(CronField::render).transpose()?;
        let minute = self.minute.render()?;
        let hour = self.hour.render()?;
        let day_of_month = self.day_of_month.render()?;
        let month = self.month.render()?;
        let day_of_week = self.day_of_week.render()?;
        let year = self.year.as_ref().map(CronField::render).transpose()?;

        if year.is_some() && second.is_none() {
            return Err(CallToolError::from_message(
                "year requires second to be provided",
            ));
        }

        let mut fields = Vec::new();
        if let Some(second) = &second {
            fields.push(second.as_str());
        }
        fields.push(minute.as_str());
        fields.push(hour.as_str());
        fields.push(day_of_month.as_str());
        fields.push(month.as_str());
        fields.push(day_of_week.as_str());
        if let Some(year) = &year {
            fields.push(year.as_str());
        }

        let expression = fields.join(" ");
        let cron = Cron::from_str(&expression).map_err(|err| {
            CallToolError::from_message(format!("invalid cron expression '{expression}': {err}"))
        })?;

        let mut structured = Map::new();
        structured.insert("expression".to_string(), Value::String(expression));
        structured.insert(
            "field_count".to_string(),
            Value::Number((fields.len() as u64).into()),
        );
        structured.insert("description".to_string(), Value::String(cron.describe()));

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
                "expression": {
                    "type": "string",
                    "description": "Generated cron expression."
                },
                "field_count": {
                    "type": "integer",
                    "minimum": 5,
                    "maximum": 7,
                    "description": "Number of fields in the expression."
                },
                "description": {
                    "type": "string",
                    "description": "English description of the schedule."
                }
            },
            "required": ["expression", "field_count", "description"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == CronGenerateTool::tool_name() {
        Some(CronGenerateTool::output_schema())
    } else {
        None
    }
}
