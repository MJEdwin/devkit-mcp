use chrono::{Datelike, FixedOffset, SecondsFormat};
use lunar_lite::{SolarDate, lunar_year_stem_branch, solar_to_lunar};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::UtcDateTime;

const SHANGHAI_UTC_OFFSET_SECONDS: i32 = 8 * 60 * 60;

const LUNAR_MONTH_NAMES: [&str; 12] = [
    "正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "冬月", "腊月",
];

const LUNAR_DAY_NAMES: [&str; 30] = [
    "初一", "初二", "初三", "初四", "初五", "初六", "初七", "初八", "初九", "初十", "十一", "十二",
    "十三", "十四", "十五", "十六", "十七", "十八", "十九", "二十", "廿一", "廿二", "廿三", "廿四",
    "廿五", "廿六", "廿七", "廿八", "廿九", "三十",
];

const HEAVENLY_STEM_NAMES: [&str; 10] =
    ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

const EARTHLY_BRANCH_NAMES: [&str; 12] = [
    "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥",
];

/// Converts a time to China Standard Time and returns its Chinese lunar date.
#[mcp_tool(
    name = "shanghai_lunar_date",
    title = "Shanghai Time to Lunar Date",
    description = "Converts an RFC 3339 datetime to China Standard Time (Asia/Shanghai, UTC+8) and returns the Chinese lunar year, month, day, and sexagenary (Heavenly Stem / Earthly Branch) year for that local date."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ShanghaiLunarDateTool {
    /// RFC 3339 datetime with a timezone offset, e.g. 2024-02-09T16:00:00Z or 2024-02-10T00:00:00+08:00.
    pub datetime: UtcDateTime,
}

impl ShanghaiLunarDateTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let offset = FixedOffset::east_opt(SHANGHAI_UTC_OFFSET_SECONDS)
            .expect("the UTC+8 offset is always valid");
        let shanghai = self.datetime.with_timezone(&offset);
        let date = shanghai.date_naive();

        let solar = SolarDate {
            year: date.year(),
            month: date.month() as u8,
            day: date.day() as u8,
        };

        let lunar = solar_to_lunar(solar).map_err(|err| {
            CallToolError::from_message(format!(
                "cannot convert {}-{:02}-{:02} (Asia/Shanghai) to a lunar date: {err}",
                date.year(),
                date.month(),
                date.day()
            ))
        })?;

        let month_name = lunar_month_name(lunar.month, lunar.is_leap_month);
        let day_name = lunar_day_name(lunar.day);
        let lunar_date = format!("{}年{}{}", chinese_digits(lunar.year), month_name, day_name);

        let stem_branch = lunar_year_stem_branch(lunar.year);
        let stem_name = HEAVENLY_STEM_NAMES[stem_branch.stem().index()].to_string();
        let branch_name = EARTHLY_BRANCH_NAMES[stem_branch.branch().index()].to_string();
        let lunar_year_ganzhi = format!("{stem_name}{branch_name}");

        let mut structured = Map::new();
        structured.insert(
            "shanghai_time".to_string(),
            Value::String(shanghai.to_rfc3339_opts(SecondsFormat::AutoSi, false)),
        );
        structured.insert(
            "lunar_year".to_string(),
            Value::Number(i64::from(lunar.year).into()),
        );
        structured.insert(
            "lunar_month".to_string(),
            Value::Number(u64::from(lunar.month).into()),
        );
        structured.insert(
            "lunar_day".to_string(),
            Value::Number(u64::from(lunar.day).into()),
        );
        structured.insert(
            "is_leap_month".to_string(),
            Value::Bool(lunar.is_leap_month),
        );
        structured.insert("lunar_month_name".to_string(), Value::String(month_name));
        structured.insert(
            "lunar_day_name".to_string(),
            Value::String(day_name.to_string()),
        );
        structured.insert("lunar_date".to_string(), Value::String(lunar_date));
        structured.insert(
            "lunar_year_ganzhi".to_string(),
            Value::String(lunar_year_ganzhi),
        );
        structured.insert("lunar_year_stem".to_string(), Value::String(stem_name));
        structured.insert("lunar_year_branch".to_string(), Value::String(branch_name));

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
                "shanghai_time": {
                    "type": "string",
                    "description": "The input time converted to Asia/Shanghai (UTC+8), as RFC 3339."
                },
                "lunar_year": {
                    "type": "integer",
                    "description": "Chinese lunar year."
                },
                "lunar_month": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 12,
                    "description": "Lunar month number (1-12)."
                },
                "lunar_day": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 30,
                    "description": "Lunar day of month (1-30)."
                },
                "is_leap_month": {
                    "type": "boolean",
                    "description": "Whether the month is a leap (intercalary) month."
                },
                "lunar_month_name": {
                    "type": "string",
                    "description": "Chinese month name, prefixed with 闰 for leap months."
                },
                "lunar_day_name": {
                    "type": "string",
                    "description": "Chinese day name, e.g. 初一."
                },
                "lunar_date": {
                    "type": "string",
                    "description": "Full Chinese lunar date, e.g. 二〇二四年正月初一."
                },
                "lunar_year_ganzhi": {
                    "type": "string",
                    "description": "Sexagenary (天干地支) name of the lunar year, e.g. 甲辰."
                },
                "lunar_year_stem": {
                    "type": "string",
                    "description": "Heavenly Stem (天干) of the lunar year, e.g. 甲."
                },
                "lunar_year_branch": {
                    "type": "string",
                    "description": "Earthly Branch (地支) of the lunar year, e.g. 辰."
                }
            },
            "required": [
                "shanghai_time",
                "lunar_year",
                "lunar_month",
                "lunar_day",
                "is_leap_month",
                "lunar_month_name",
                "lunar_day_name",
                "lunar_date",
                "lunar_year_ganzhi",
                "lunar_year_stem",
                "lunar_year_branch"
            ]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == ShanghaiLunarDateTool::tool_name() {
        Some(ShanghaiLunarDateTool::output_schema())
    } else {
        None
    }
}

fn lunar_month_name(month: u8, is_leap_month: bool) -> String {
    let name = LUNAR_MONTH_NAMES
        .get(month.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("未知月");

    if is_leap_month {
        format!("闰{name}")
    } else {
        name.to_string()
    }
}

fn lunar_day_name(day: u8) -> &'static str {
    LUNAR_DAY_NAMES
        .get(day.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("未知日")
}

fn chinese_digits(value: i32) -> String {
    const DIGITS: [char; 10] = ['〇', '一', '二', '三', '四', '五', '六', '七', '八', '九'];

    let mut result = String::new();
    if value < 0 {
        result.push('负');
    }
    for ch in value.unsigned_abs().to_string().chars() {
        let index = ch.to_digit(10).unwrap_or(0) as usize;
        result.push(DIGITS[index]);
    }
    result
}
