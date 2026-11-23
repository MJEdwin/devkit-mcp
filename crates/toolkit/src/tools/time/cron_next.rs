use std::str::FromStr;

use chrono::{FixedOffset, SecondsFormat, Utc};
use croner::Cron;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::UtcDateTime;

/// Timezone used to evaluate a cron expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum CronTimezone {
    #[serde(rename = "utc")]
    Utc,
    #[serde(rename = "asia_shanghai")]
    AsiaShanghai,
}

impl CronTimezone {
    pub fn json_schema() -> Map<String, Value> {
        crate::tools::string_enum_schema(&["utc", "asia_shanghai"])
    }

    fn offset(self) -> FixedOffset {
        let seconds = match self {
            Self::Utc => 0,
            Self::AsiaShanghai => 8 * 60 * 60,
        };

        FixedOffset::east_opt(seconds).expect("the selected offset is always valid")
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Utc => "utc",
            Self::AsiaShanghai => "asia_shanghai",
        }
    }
}

/// Computes upcoming cron occurrences.
#[mcp_tool(
    name = "cron_next",
    title = "Cron Next Occurrences",
    description = "Computes the next N execution times for a cron expression (5-7 fields, with optional seconds and year) in UTC or Asia/Shanghai."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CronNextTool {
    /// Cron expression, e.g. "*/5 * * * *", "0 */5 * * * *", or a 7-field expression with seconds and year.
    pub expression: String,

    /// Start time as an RFC 3339 datetime with an offset. Defaults to now.
    pub from: Option<UtcDateTime>,

    /// Number of occurrences to return (1-50). Defaults to 5.
    #[json_schema(default = 5, minimum = 1, maximum = 50)]
    pub count: Option<u32>,

    /// Timezone for evaluation: "utc" (default) or "asia_shanghai".
    #[json_schema(default = "utc")]
    pub timezone: Option<CronTimezone>,
}

impl CronNextTool {
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

        let timezone = self.timezone.unwrap_or(CronTimezone::Utc);
        let offset = timezone.offset();
        let start = self
            .from
            .map(|from| from.with_timezone(&offset))
            .unwrap_or_else(|| Utc::now().with_timezone(&offset));

        let mut cursor = start;
        let mut times = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let next = cron
                .find_next_occurrence(&cursor, false)
                .map_err(|err| CallToolError::from_message(format!("no next occurrence: {err}")))?;

            times.push(Value::String(
                next.to_rfc3339_opts(SecondsFormat::Secs, true),
            ));
            cursor = next;
        }

        let mut structured = Map::new();
        structured.insert(
            "timezone".to_string(),
            Value::String(timezone.as_str().to_string()),
        );
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
                    "enum": ["utc", "asia_shanghai"],
                    "description": "Timezone used for evaluation."
                },
                "count": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Number of occurrences returned."
                },
                "times": {
                    "type": "array",
                    "description": "Next occurrences as RFC 3339 strings.",
                    "items": { "type": "string" }
                }
            },
            "required": ["timezone", "count", "times"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == CronNextTool::tool_name() {
        Some(CronNextTool::output_schema())
    } else {
        None
    }
}
