use std::fmt;
use std::str::FromStr;

use chrono::{FixedOffset, SecondsFormat, Utc};
use croner::Cron;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value, json};

use crate::UtcDateTime;

/// A fixed UTC offset such as `+08:00`, `-05:30`, or `Z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimezoneOffset(FixedOffset);

impl TimezoneOffset {
    pub(super) fn utc() -> Self {
        Self(FixedOffset::east_opt(0).expect("zero offset is always valid"))
    }

    pub fn json_schema() -> Map<String, Value> {
        let mut map = Map::new();
        map.insert("type".to_string(), Value::String("string".to_string()));
        map.insert(
            "pattern".to_string(),
            Value::String("^(?:Z|[+-][0-9]{2}:[0-9]{2})$".to_string()),
        );
        map.insert(
            "description".to_string(),
            Value::String("UTC offset such as +08:00, -05:30, or Z".to_string()),
        );
        map
    }

    pub(super) fn fixed_offset(self) -> FixedOffset {
        self.0
    }
}

impl FromStr for TimezoneOffset {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        if value.eq_ignore_ascii_case("z") {
            return Ok(Self::utc());
        }

        if !value.is_ascii() || value.len() != 6 {
            return Err("expected a UTC offset like +08:00, -05:30, or Z".to_string());
        }

        let bytes = value.as_bytes();
        if (bytes[0] != b'+' && bytes[0] != b'-') || bytes[3] != b':' {
            return Err("expected a UTC offset like +08:00, -05:30, or Z".to_string());
        }

        let hours: i32 = value[1..3]
            .parse()
            .map_err(|_| "invalid UTC offset hours".to_string())?;
        let minutes: i32 = value[4..6]
            .parse()
            .map_err(|_| "invalid UTC offset minutes".to_string())?;

        if minutes > 59 {
            return Err(format!("UTC offset minutes must be 0-59, got {minutes}"));
        }

        let signed_minutes = if bytes[0] == b'-' {
            -(hours * 60 + minutes)
        } else {
            hours * 60 + minutes
        };

        // All real-world zones fall between UTC-12:00 and UTC+14:00.
        if !(-12 * 60..=14 * 60).contains(&signed_minutes) {
            return Err(format!(
                "UTC offset must be between -12:00 and +14:00, got {value}"
            ));
        }

        FixedOffset::east_opt(signed_minutes * 60)
            .map(Self)
            .ok_or_else(|| format!("invalid UTC offset: {value}"))
    }
}

impl fmt::Display for TimezoneOffset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.0.local_minus_utc();

        if total == 0 {
            return formatter.write_str("Z");
        }

        let sign = if total < 0 { '-' } else { '+' };
        let absolute = total.abs();
        write!(
            formatter,
            "{sign}{:02}:{:02}",
            absolute / 3600,
            (absolute % 3600) / 60
        )
    }
}

impl Serialize for TimezoneOffset {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TimezoneOffset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

/// Computes upcoming cron occurrences.
#[mcp_tool(
    name = "cron_next",
    title = "Cron Next Occurrences",
    description = "Computes the next N execution times for a cron expression (5-7 fields, with optional seconds and year) using a fixed UTC offset."
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

    /// Timezone as a fixed UTC offset ("+08:00", "-05:30", or "Z"). Defaults to "Z" (UTC).
    #[json_schema(default = "Z")]
    pub timezone: Option<TimezoneOffset>,
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

        let timezone = self.timezone.unwrap_or_else(TimezoneOffset::utc);
        let offset = timezone.fixed_offset();
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
