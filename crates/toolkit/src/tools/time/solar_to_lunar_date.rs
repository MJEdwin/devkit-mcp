use chrono::NaiveDate;
use lunar_lite::{SolarDate, lunar_year_stem_branch, solar_to_lunar};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

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

const ZODIAC_NAMES: [&str; 12] = [
    "鼠", "牛", "虎", "兔", "龙", "蛇", "马", "羊", "猴", "鸡", "狗", "猪",
];

/// Converts a Gregorian (solar) date to its Chinese lunar date.
#[mcp_tool(
    name = "solar_to_lunar_date",
    title = "Solar Date to Lunar Date",
    description = "Converts a Gregorian (solar) year, month, and day to the Chinese lunar date, sexagenary (Heavenly Stem / Earthly Branch) year, and Chinese zodiac."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SolarToLunarDateTool {
    /// Gregorian year, e.g. 2024.
    #[json_schema(minimum = 1, maximum = 9999)]
    pub solar_year: i32,

    /// Gregorian month, 1-12.
    #[json_schema(minimum = 1, maximum = 12)]
    pub solar_month: u8,

    /// Gregorian day, 1-31.
    #[json_schema(minimum = 1, maximum = 31)]
    pub solar_day: u8,
}

impl SolarToLunarDateTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let solar_date = format!(
            "{:04}-{:02}-{:02}",
            self.solar_year, self.solar_month, self.solar_day
        );

        NaiveDate::from_ymd_opt(
            self.solar_year,
            u32::from(self.solar_month),
            u32::from(self.solar_day),
        )
        .ok_or_else(|| {
            CallToolError::from_message(format!("invalid Gregorian date: {solar_date}"))
        })?;

        let solar = SolarDate {
            year: self.solar_year,
            month: self.solar_month,
            day: self.solar_day,
        };

        let lunar = solar_to_lunar(solar).map_err(|err| {
            CallToolError::from_message(format!(
                "cannot convert {solar_date} to a lunar date: {err}"
            ))
        })?;

        let month_name = lunar_month_name(lunar.month, lunar.is_leap_month);
        let day_name = lunar_day_name(lunar.day);
        let lunar_date = format!("{}年{}{}", chinese_digits(lunar.year), month_name, day_name);

        let stem_branch = lunar_year_stem_branch(lunar.year);
        let stem_name = HEAVENLY_STEM_NAMES[stem_branch.stem().index()].to_string();
        let branch_name = EARTHLY_BRANCH_NAMES[stem_branch.branch().index()].to_string();
        let lunar_year_ganzhi = format!("{stem_name}{branch_name}");
        let zodiac = ZODIAC_NAMES[stem_branch.branch().index()].to_string();

        let mut structured = Map::new();
        structured.insert("solar_date".to_string(), Value::String(solar_date));
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
        structured.insert("zodiac".to_string(), Value::String(zodiac));

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
                    "description": "The input Gregorian date in YYYY-MM-DD form."
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
                },
                "zodiac": {
                    "type": "string",
                    "description": "Chinese zodiac animal for the lunar year, e.g. 龙."
                }
            },
            "required": [
                "solar_date",
                "lunar_year",
                "lunar_month",
                "lunar_day",
                "is_leap_month",
                "lunar_month_name",
                "lunar_day_name",
                "lunar_date",
                "lunar_year_ganzhi",
                "lunar_year_stem",
                "lunar_year_branch",
                "zodiac"
            ]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == SolarToLunarDateTool::tool_name() {
        Some(SolarToLunarDateTool::output_schema())
    } else {
        None
    }
}

pub(super) fn lunar_month_name(month: u8, is_leap_month: bool) -> String {
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

pub(super) fn lunar_day_name(day: u8) -> &'static str {
    LUNAR_DAY_NAMES
        .get(day.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("未知日")
}

pub(super) fn chinese_digits(value: i32) -> String {
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
