mod common;

use common::text;
use toolkit::{CurrentTimeTool, Iso8601ToUnixTool, TimestampUnit, UnixToIso8601Tool, UtcDateTime};

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
