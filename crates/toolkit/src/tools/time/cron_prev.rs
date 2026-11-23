use std::str::FromStr;

use chrono::{SecondsFormat, Utc};
use croner::Cron;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::cron_next::TimezoneOffset;
use crate::UtcDateTime;

/// Computes previous cron occurrences.
#[mcp_tool(
    name = "cron_prev",
    title = "Cron Previous Occurrences",
    description = "Computes the previous N execution times for a cron expression (5-7 fields, with optional seconds and year) using a fixed UTC offset."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CronPrevTool {
    /// Cron expression, e.g. "*/5 * * * *", "0 */5 * * * *", or a 7-field expression with seconds and year.
    pub expression: String,

    /// Start time as an RFC 3339 datetime with an offset. Defaults to now.
    pub from: Option<UtcDateTime>,

    /// Number of occurrences to return (1-50). Defaults to 5.
    #[json_schema(default = 5, minimum = 1, maximum = 50)]
    pub count: Option<u32>,

    /// Timezone as a fixed UTC offset ("+08:00", "-05:30", or "Z"). Defaults to "Z" (UTC).
    #[json_schema(default = "Z")]
    pub timezone: Option<TimezoneOffset>,
}

impl CronPrevTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let cron = Cron::from_str(&self.expression).map_err(|err| {
            CallToolError::from_message(format!("invalid cron expression: {err}"))
        })?;

        let count = self.count.unwrap_or(5);
        if !(1..=50).contains(&count) {
            return Err(CallToolError::from_message(format!(
                "count must be between 1 and 50, got {count}"
            )));
        }

        let timezone = self.timezone.unwrap_or_else(TimezoneOffset::utc);
        let offset = timezone.fixed_offset();
        let start = self
            .from
            .map(|from| from.with_timezone(&offset))
            .unwrap_or_else(|| Utc::now().with_timezone(&offset));

        let mut cursor = start;
        let mut times = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let previous = cron
                .find_previous_occurrence(&cursor, false)
                .map_err(|err| {
                    CallToolError::from_message(format!("no previous occurrence: {err}"))
                })?;

            times.push(Value::String(
                previous.to_rfc3339_opts(SecondsFormat::Secs, true),
            ));
            cursor = previous;
        }

        let mut structured = Map::new();
        structured.insert("timezone".to_string(), Value::String(timezone.to_string()));
        structured.insert("count".to_string(), Value::Number(u64::from(count).into()));
        structured.insert("times".to_string(), Value::Array(times));

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
                "timezone": {
                    "type": "string",
                    "pattern": "^(?:Z|[+-][0-9]{2}:[0-9]{2})$",
                    "description": "Applied UTC offset."
                },
                "count": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Number of occurrences returned."
                },
                "times": {
                    "type": "array",
                    "description": "Previous occurrences as RFC 3339 strings, most recent first.",
                    "items": { "type": "string" }
                }
            },
            "required": ["timezone", "count", "times"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == CronPrevTool::tool_name() {
        Some(CronPrevTool::output_schema())
    } else {
        None
    }
}
