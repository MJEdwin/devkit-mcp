use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

/// Returns the current UTC time in ISO 8601 format.
#[mcp_tool(
    name = "current_time",
    title = "Current Time (UTC)",
    description = "Returns the current date and time in ISO 8601 UTC format (e.g. 2026-11-22T03:45:12.345Z)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CurrentTimeTool {}

impl CurrentTimeTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        Ok(CallToolResult::text_content(vec![TextContent::from(now)]))
    }
}
