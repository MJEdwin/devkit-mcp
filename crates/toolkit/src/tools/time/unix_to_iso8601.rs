use chrono::{DateTime, SecondsFormat, Utc};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// Unit of the supplied Unix timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum TimestampUnit {
    #[serde(rename = "seconds")]
    Seconds,
    #[serde(rename = "milliseconds")]
    Milliseconds,
}

impl TimestampUnit {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["seconds", "milliseconds"])
    }
}

/// Converts a Unix timestamp to an ISO 8601 UTC time.
#[mcp_tool(
    name = "unix_to_iso8601",
    title = "Unix Timestamp to ISO 8601",
    description = "Converts a Unix timestamp (seconds or milliseconds) to an ISO 8601 UTC string."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct UnixToIso8601Tool {
    /// Unix timestamp to convert. May be negative for times before 1970.
    pub timestamp: i64,

    /// Unit of `timestamp`: "seconds" (default) or "milliseconds".
    #[json_schema(default = "seconds")]
    pub unit: Option<TimestampUnit>,
}

impl UnixToIso8601Tool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let unit = self.unit.unwrap_or(TimestampUnit::Seconds);

        let (seconds, nanos) = match unit {
            TimestampUnit::Seconds => (self.timestamp, 0),
            TimestampUnit::Milliseconds => (
                self.timestamp.div_euclid(1000),
                self.timestamp.rem_euclid(1000) as u32 * 1_000_000,
            ),
        };

        let datetime = DateTime::<Utc>::from_timestamp(seconds, nanos).ok_or_else(|| {
            CallToolError::from_message(format!(
                "timestamp {} is out of the supported range",
                self.timestamp
            ))
        })?;

        let precision = match unit {
            TimestampUnit::Seconds => SecondsFormat::Secs,
            TimestampUnit::Milliseconds => SecondsFormat::Millis,
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            datetime.to_rfc3339_opts(precision, true),
        )]))
    }
}
