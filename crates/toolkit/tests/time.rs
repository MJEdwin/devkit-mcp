mod common;

use common::{structured, text};
use toolkit::{
    CronNextTool, CronTimezone, CronValidateTool, CurrentTimeTool, Iso8601ToUnixTool,
    LunarToSolarDateTool, SolarToLunarDateTool, TimestampUnit, UnixToIso8601Tool, UtcDateTime,
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
fn solar_to_lunar_date_spring_festival() {
    let result = SolarToLunarDateTool {
        solar_year: 2024,
        solar_month: 2,
        solar_day: 10,
    }
    .call_tool()
    .expect("solar_to_lunar_date");

    let value = structured(&result);
    assert_eq!(value["solar_date"].as_str(), Some("2024-02-10"));
    assert_eq!(value["lunar_year"].as_i64(), Some(2024));
    assert_eq!(value["lunar_month"].as_u64(), Some(1));
    assert_eq!(value["lunar_day"].as_u64(), Some(1));
    assert_eq!(value["is_leap_month"].as_bool(), Some(false));
    assert_eq!(value["lunar_month_name"].as_str(), Some("正月"));
    assert_eq!(value["lunar_day_name"].as_str(), Some("初一"));
    assert_eq!(value["lunar_date"].as_str(), Some("二〇二四年正月初一"));
    assert_eq!(value["lunar_year_ganzhi"].as_str(), Some("甲辰"));
    assert_eq!(value["zodiac"].as_str(), Some("龙"));

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn solar_to_lunar_date_leap_month() {
    let result = SolarToLunarDateTool {
        solar_year: 2023,
        solar_month: 3,
        solar_day: 22,
    }
    .call_tool()
    .expect("solar_to_lunar_date");

    let value = structured(&result);
    assert_eq!(value["solar_date"].as_str(), Some("2023-03-22"));
    assert_eq!(value["lunar_year"].as_i64(), Some(2023));
    assert_eq!(value["lunar_month"].as_u64(), Some(2));
    assert_eq!(value["lunar_day"].as_u64(), Some(1));
    assert_eq!(value["is_leap_month"].as_bool(), Some(true));
    assert_eq!(value["lunar_month_name"].as_str(), Some("闰二月"));
    assert_eq!(value["lunar_date"].as_str(), Some("二〇二三年闰二月初一"));
    assert_eq!(value["lunar_year_ganzhi"].as_str(), Some("癸卯"));
    assert_eq!(value["zodiac"].as_str(), Some("兔"));
}

#[test]
fn solar_to_lunar_date_accepts_gregorian_leap_day() {
    let result = SolarToLunarDateTool {
        solar_year: 2024,
        solar_month: 2,
        solar_day: 29,
    }
    .call_tool()
    .expect("solar_to_lunar_date");

    let value = structured(&result);
    assert_eq!(value["solar_date"].as_str(), Some("2024-02-29"));
    assert_eq!(value["lunar_year"].as_i64(), Some(2024));
    assert_eq!(value["lunar_month"].as_u64(), Some(1));
    assert_eq!(value["lunar_day"].as_u64(), Some(20));
    assert_eq!(value["lunar_date"].as_str(), Some("二〇二四年正月二十"));
}

#[test]
fn solar_to_lunar_date_rejects_invalid_gregorian_date() {
    // 2023 is not a leap year.
    assert!(
        SolarToLunarDateTool {
            solar_year: 2023,
            solar_month: 2,
            solar_day: 29,
        }
        .call_tool()
        .is_err()
    );

    // April has 30 days.
    assert!(
        SolarToLunarDateTool {
            solar_year: 2024,
            solar_month: 4,
            solar_day: 31,
        }
        .call_tool()
        .is_err()
    );

    // Month is out of range.
    assert!(
        SolarToLunarDateTool {
            solar_year: 2024,
            solar_month: 13,
            solar_day: 1,
        }
        .call_tool()
        .is_err()
    );
}

#[test]
fn solar_to_lunar_date_out_of_range() {
    let result = SolarToLunarDateTool {
        solar_year: 10_000,
        solar_month: 12,
        solar_day: 31,
    }
    .call_tool();

    assert!(result.is_err());
}

#[test]
fn lunar_to_solar_date_spring_festival() {
    let result = LunarToSolarDateTool {
        lunar_year: 2024,
        lunar_month: 1,
        lunar_day: 1,
        is_leap_month: None,
    }
    .call_tool()
    .expect("lunar_to_solar_date");

    let value = structured(&result);
    assert_eq!(value["solar_date"].as_str(), Some("2024-02-10"));
    assert_eq!(value["lunar_date"].as_str(), Some("二〇二四年正月初一"));

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn lunar_to_solar_date_leap_month() {
    let result = LunarToSolarDateTool {
        lunar_year: 2023,
        lunar_month: 2,
        lunar_day: 1,
        is_leap_month: Some(true),
    }
    .call_tool()
    .expect("lunar_to_solar_date");

    let value = structured(&result);
    assert_eq!(value["solar_date"].as_str(), Some("2023-03-22"));
    assert_eq!(value["lunar_date"].as_str(), Some("二〇二三年闰二月初一"));
}

#[test]
fn lunar_to_solar_date_rejects_invalid_input() {
    // 2024 has no leap second month.
    assert!(
        LunarToSolarDateTool {
            lunar_year: 2024,
            lunar_month: 2,
            lunar_day: 1,
            is_leap_month: Some(true),
        }
        .call_tool()
        .is_err()
    );

    // Lunar months never have 31 days.
    assert!(
        LunarToSolarDateTool {
            lunar_year: 2024,
            lunar_month: 1,
            lunar_day: 31,
            is_leap_month: None,
        }
        .call_tool()
        .is_err()
    );

    // Month and year must be in range.
    assert!(
        LunarToSolarDateTool {
            lunar_year: 2024,
            lunar_month: 13,
            lunar_day: 1,
            is_leap_month: None,
        }
        .call_tool()
        .is_err()
    );
    assert!(
        LunarToSolarDateTool {
            lunar_year: 0,
            lunar_month: 1,
            lunar_day: 1,
            is_leap_month: None,
        }
        .call_tool()
        .is_err()
    );
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

#[test]
fn cron_validate_accepts_and_describes() {
    let five = CronValidateTool {
        expression: "*/5 * * * *".to_string(),
    }
    .call_tool()
    .expect("cron_validate");

    let value = structured(&five);
    assert_eq!(value["valid"].as_bool(), Some(true));
    assert!(value["error"].is_null());
    assert!(
        !value["description"]
            .as_str()
            .expect("description")
            .is_empty()
    );

    let six = CronValidateTool {
        expression: "0 */5 * * * *".to_string(),
    }
    .call_tool()
    .expect("cron_validate");
    assert_eq!(structured(&six)["valid"].as_bool(), Some(true));
}

#[test]
fn cron_validate_rejects_invalid_expression() {
    let result = CronValidateTool {
        expression: "not a cron".to_string(),
    }
    .call_tool()
    .expect("cron_validate");

    let value = structured(&result);
    assert_eq!(value["valid"].as_bool(), Some(false));
    assert!(value["error"].as_str().is_some());
    assert!(value["description"].is_null());
}

#[test]
fn cron_next_utc() {
    let result = CronNextTool {
        expression: "0 * * * *".to_string(),
        from: Some("2026-01-01T00:00:00Z".parse().expect("valid datetime")),
        count: Some(3),
        timezone: None,
    }
    .call_tool()
    .expect("cron_next");

    let value = structured(&result);
    assert_eq!(value["timezone"].as_str(), Some("utc"));
    assert_eq!(value["count"].as_u64(), Some(3));
    assert_eq!(
        value["times"],
        serde_json::json!([
            "2026-01-01T01:00:00Z",
            "2026-01-01T02:00:00Z",
            "2026-01-01T03:00:00Z"
        ])
    );
}

#[test]
fn cron_next_shanghai_and_seconds() {
    let shanghai = CronNextTool {
        expression: "0 9 * * *".to_string(),
        from: Some("2026-01-01T00:00:00+08:00".parse().expect("valid datetime")),
        count: Some(1),
        timezone: Some(CronTimezone::AsiaShanghai),
    }
    .call_tool()
    .expect("cron_next");

    let value = structured(&shanghai);
    assert_eq!(value["timezone"].as_str(), Some("asia_shanghai"));
    assert_eq!(
        value["times"],
        serde_json::json!(["2026-01-01T09:00:00+08:00"])
    );

    let seconds = CronNextTool {
        expression: "*/30 * * * * *".to_string(),
        from: Some("2026-01-01T00:00:00Z".parse().expect("valid datetime")),
        count: Some(3),
        timezone: None,
    }
    .call_tool()
    .expect("cron_next");

    assert_eq!(
        structured(&seconds)["times"],
        serde_json::json!([
            "2026-01-01T00:00:30Z",
            "2026-01-01T00:01:00Z",
            "2026-01-01T00:01:30Z"
        ])
    );
}

#[test]
fn cron_next_rejects_invalid_input() {
    assert!(
        CronNextTool {
            expression: "bad".to_string(),
            from: None,
            count: None,
            timezone: None,
        }
        .call_tool()
        .is_err()
    );

    assert!(
        CronNextTool {
            expression: "0 * * * *".to_string(),
            from: None,
            count: Some(0),
            timezone: None,
        }
        .call_tool()
        .is_err()
    );
}
