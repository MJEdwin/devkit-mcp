use lunar_lite::{LunarDate, lunar_to_solar, validate_lunar_date};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::solar_to_lunar_date::{chinese_digits, lunar_day_name, lunar_month_name};

/// Converts a Chinese lunar date to the corresponding Gregorian date.
#[mcp_tool(
    name = "lunar_to_solar_date",
    title = "Lunar Date to Solar Date",
    description = "Converts a Chinese lunar date (year, month, day, and leap-month flag) to the corresponding Gregorian (solar) date, returned as YYYY-MM-DD."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct LunarToSolarDateTool {
    /// Chinese lunar year, e.g. 2024.
    #[json_schema(minimum = 1, maximum = 9999)]
    pub lunar_year: i32,

    /// Lunar month, 1-12.
    #[json_schema(minimum = 1, maximum = 12)]
    pub lunar_month: u8,

    /// Lunar day, 1-30.
    #[json_schema(minimum = 1, maximum = 30)]
    pub lunar_day: u8,

    /// Whether the month is a leap (intercalary) month.
    #[json_schema(default = false)]
    pub is_leap_month: Option<bool>,
}

impl LunarToSolarDateTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        if !(1..=12).contains(&self.lunar_month) {
            return Err(CallToolError::from_message(format!(
                "lunar_month must be between 1 and 12, got {}",
                self.lunar_month
            )));
        }

        if !(1..=30).contains(&self.lunar_day) {
            return Err(CallToolError::from_message(format!(
                "lunar_day must be between 1 and 30, got {}",
                self.lunar_day
            )));
        }

        let lunar = LunarDate {
            year: self.lunar_year,
            month: self.lunar_month,
            day: self.lunar_day,
            is_leap_month: self.is_leap_month.unwrap_or(false),
        };

        validate_lunar_date(lunar)
            .map_err(|err| CallToolError::from_message(format!("invalid lunar date: {err}")))?;

        let solar = lunar_to_solar(lunar).map_err(|err| {
            CallToolError::from_message(format!(
                "cannot convert lunar date {} to a solar date: {err}",
                lunar_text(lunar)
            ))
        })?;

        let solar_date = format!("{:04}-{:02}-{:02}", solar.year, solar.month, solar.day);

        let mut structured = Map::new();
        structured.insert("solar_date".to_string(), Value::String(solar_date));
        structured.insert("lunar_date".to_string(), Value::String(lunar_text(lunar)));

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
                "solar_date": {
                    "type": "string",
                    "description": "Gregorian date in YYYY-MM-DD form."
                },
                "lunar_date": {
                    "type": "string",
                    "description": "The lunar date in Chinese, e.g. 二〇二三年闰二月初一."
                }
            },
            "required": ["solar_date", "lunar_date"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == LunarToSolarDateTool::tool_name() {
        Some(LunarToSolarDateTool::output_schema())
    } else {
        None
    }
}

fn lunar_text(lunar: LunarDate) -> String {
    format!(
        "{}年{}{}",
        chinese_digits(lunar.year),
        lunar_month_name(lunar.month, lunar.is_leap_month),
        lunar_day_name(lunar.day)
    )
}
