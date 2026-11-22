use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use crate::UtcDateTime;

use super::TimestampUnit;

/// Converts an ISO 8601 datetime to a Unix timestamp.
#[mcp_tool(
    name = "iso8601_to_unix",
    title = "ISO 8601 to Unix Timestamp",
    description = "Converts an ISO 8601 datetime with a timezone offset (e.g. 2023-11-14T22:13:20Z or 2023-11-15T06:13:20+08:00) to a Unix timestamp (seconds or milliseconds)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct Iso8601ToUnixTool {
    /// ISO 8601 datetime with a timezone offset (RFC 3339), e.g. 2023-11-14T22:13:20Z or 2023-11-15T06:13:20+08:00. Offsets are required.
    pub datetime: UtcDateTime,

    /// Unit of the returned timestamp: "seconds" (default) or "milliseconds".
    #[json_schema(default = "seconds")]
    pub unit: Option<TimestampUnit>,
}

impl Iso8601ToUnixTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let timestamp = match self.unit.unwrap_or(TimestampUnit::Seconds) {
            TimestampUnit::Seconds => self.datetime.timestamp(),
            TimestampUnit::Milliseconds => self.datetime.timestamp_millis(),
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            timestamp.to_string(),
        )]))
    }
}
