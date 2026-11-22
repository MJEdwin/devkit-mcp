mod common;

use common::{structured, text};
use toolkit::{
    CurrentTimeTool, Iso8601ToUnixTool, ShanghaiLunarDateTool, TimestampUnit, UnixToIso8601Tool,
    UtcDateTime,
};

#[test]
fn current_time_is_iso8601_utc() {
    let result = CurrentTimeTool {}.call_tool().expect("current_time");
    let parsed = chrono::DateTime::parse_from_rfc3339(text(&result)).expect("RFC 3339");
    assert_eq!(parsed.offset().local_minus_utc(), 0);
}

#[test]
fn unix_seconds_to_iso8601() {
    let result = UnixToIso8601Tool {
        timestamp: 1_700_000_000,
        unit: None,
    }
    .call_tool()
    .expect("unix_to_iso8601");
    assert_eq!(text(&result), "2023-11-14T22:13:20Z");
}

#[test]
fn unix_milliseconds_to_iso8601() {
    let result = UnixToIso8601Tool {
        timestamp: 1_700_000_000_123,
        unit: Some(TimestampUnit::Milliseconds),
    }
    .call_tool()
    .expect("unix_to_iso8601");
    assert_eq!(text(&result), "2023-11-14T22:13:20.123Z");
}

#[test]
fn unix_to_iso8601_negative_timestamp() {
    let result = UnixToIso8601Tool {
        timestamp: -1,
        unit: None,
    }
    .call_tool()
    .expect("unix_to_iso8601");
    assert_eq!(text(&result), "1969-12-31T23:59:59Z");
}

#[test]
fn iso8601_to_unix_with_offset() {
    let result = Iso8601ToUnixTool {
        datetime: "2023-11-15T06:13:20+08:00"
            .parse::<UtcDateTime>()
            .expect("valid datetime"),
        unit: None,
    }
    .call_tool()
    .expect("iso8601_to_unix");
    assert_eq!(text(&result), "1700000000");
}

#[test]
fn iso8601_to_unix_milliseconds() {
    let result = Iso8601ToUnixTool {
        datetime: "2023-11-14T22:13:20.123Z"
            .parse::<UtcDateTime>()
            .expect("valid datetime"),
        unit: Some(TimestampUnit::Milliseconds),
    }
    .call_tool()
    .expect("iso8601_to_unix");
    assert_eq!(text(&result), "1700000000123");
}

#[test]
fn shanghai_lunar_date_spring_festival() {
    let result = ShanghaiLunarDateTool {
        datetime: "2024-02-09T16:00:00Z".parse().expect("valid datetime"),
    }
    .call_tool()
    .expect("shanghai_lunar_date");

    let value = structured(&result);
    assert_eq!(
        value["shanghai_time"].as_str(),
        Some("2024-02-10T00:00:00+08:00")
    );
    assert_eq!(value["lunar_year"].as_i64(), Some(2024));
    assert_eq!(value["lunar_month"].as_u64(), Some(1));
    assert_eq!(value["lunar_day"].as_u64(), Some(1));
    assert_eq!(value["is_leap_month"].as_bool(), Some(false));
    assert_eq!(value["lunar_month_name"].as_str(), Some("正月"));
    assert_eq!(value["lunar_day_name"].as_str(), Some("初一"));
    assert_eq!(value["lunar_date"].as_str(), Some("二〇二四年正月初一"));
    assert_eq!(value["lunar_year_ganzhi"].as_str(), Some("甲辰"));
    assert_eq!(value["lunar_year_stem"].as_str(), Some("甲"));
    assert_eq!(value["lunar_year_branch"].as_str(), Some("辰"));
    assert_eq!(value["zodiac"].as_str(), Some("龙"));

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn shanghai_lunar_date_leap_month() {
    let result = ShanghaiLunarDateTool {
        datetime: "2023-03-22T12:00:00+08:00".parse().expect("valid datetime"),
    }
    .call_tool()
    .expect("shanghai_lunar_date");

    let value = structured(&result);
    assert_eq!(
        value["shanghai_time"].as_str(),
        Some("2023-03-22T12:00:00+08:00")
    );
    assert_eq!(value["lunar_year"].as_i64(), Some(2023));
    assert_eq!(value["lunar_month"].as_u64(), Some(2));
    assert_eq!(value["lunar_day"].as_u64(), Some(1));
    assert_eq!(value["is_leap_month"].as_bool(), Some(true));
    assert_eq!(value["lunar_month_name"].as_str(), Some("闰二月"));
    assert_eq!(value["lunar_date"].as_str(), Some("二〇二三年闰二月初一"));
    assert_eq!(value["lunar_year_ganzhi"].as_str(), Some("癸卯"));
    assert_eq!(value["lunar_year_stem"].as_str(), Some("癸"));
    assert_eq!(value["lunar_year_branch"].as_str(), Some("卯"));
    assert_eq!(value["zodiac"].as_str(), Some("兔"));
}

#[test]
fn shanghai_lunar_date_timezone_boundary() {
    // 2024-02-09T15:59:59Z is 2024-02-09 23:59:59 in Shanghai (除夕).
    let result = ShanghaiLunarDateTool {
        datetime: "2024-02-09T15:59:59Z".parse().expect("valid datetime"),
    }
    .call_tool()
    .expect("shanghai_lunar_date");

    let value = structured(&result);
    assert_eq!(
        value["shanghai_time"].as_str(),
        Some("2024-02-09T23:59:59+08:00")
    );
    assert_eq!(value["lunar_year"].as_i64(), Some(2023));
    assert_eq!(value["lunar_month"].as_u64(), Some(12));
    assert_eq!(value["lunar_day"].as_u64(), Some(30));
    assert_eq!(value["lunar_date"].as_str(), Some("二〇二三年腊月三十"));
    assert_eq!(value["lunar_year_ganzhi"].as_str(), Some("癸卯"));
    assert_eq!(value["zodiac"].as_str(), Some("兔"));
}

#[test]
fn shanghai_lunar_date_out_of_range() {
    let result = ShanghaiLunarDateTool {
        datetime: "9999-12-31T20:00:00Z".parse().expect("valid datetime"),
    }
    .call_tool();

    assert!(result.is_err());
}

#[test]
fn utc_datetime_requires_offset() {
    assert!("2023-11-14T22:13:20".parse::<UtcDateTime>().is_err());
    assert!(
        serde_json::from_value::<UtcDateTime>(serde_json::json!("2023-11-14T22:13:20")).is_err()
    );
}

#[test]
fn utc_datetime_serializes_as_utc() {
    let datetime: UtcDateTime = "2023-11-15T06:13:20+08:00".parse().expect("valid datetime");

    assert_eq!(
        serde_json::to_value(datetime).expect("serialize"),
        serde_json::json!("2023-11-14T22:13:20Z")
    );
}
