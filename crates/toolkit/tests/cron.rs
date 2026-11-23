mod common;

use common::{structured, text};
use toolkit::{
    CronField, CronGenerateTool, CronNextTool, CronParseTool, CronPrevTool, CronTerm,
    CronValidateTool, TimezoneOffset,
};

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
    assert_eq!(value["timezone"].as_str(), Some("Z"));
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
        timezone: Some("+08:00".parse().expect("valid offset")),
    }
    .call_tool()
    .expect("cron_next");

    let value = structured(&shanghai);
    assert_eq!(value["timezone"].as_str(), Some("+08:00"));
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

#[test]
fn cron_next_supports_arbitrary_utc_offsets() {
    let result = CronNextTool {
        expression: "0 9 * * *".to_string(),
        from: Some("2026-01-01T00:00:00-05:30".parse().expect("valid datetime")),
        count: Some(1),
        timezone: Some("-05:30".parse().expect("valid offset")),
    }
    .call_tool()
    .expect("cron_next");

    let value = structured(&result);
    assert_eq!(value["timezone"].as_str(), Some("-05:30"));
    assert_eq!(
        value["times"],
        serde_json::json!(["2026-01-01T09:00:00-05:30"])
    );
}

#[test]
fn timezone_offset_parsing() {
    assert_eq!("Z".parse::<TimezoneOffset>().unwrap().to_string(), "Z");
    assert_eq!(
        "+08:00".parse::<TimezoneOffset>().unwrap().to_string(),
        "+08:00"
    );
    assert_eq!(
        "-05:30".parse::<TimezoneOffset>().unwrap().to_string(),
        "-05:30"
    );
    assert_eq!(
        "+14:00".parse::<TimezoneOffset>().unwrap().to_string(),
        "+14:00"
    );
    assert_eq!(
        "-12:00".parse::<TimezoneOffset>().unwrap().to_string(),
        "-12:00"
    );

    assert!("+14:30".parse::<TimezoneOffset>().is_err());
    assert!("-12:30".parse::<TimezoneOffset>().is_err());
    assert!("+25:00".parse::<TimezoneOffset>().is_err());
    assert!("not-a-zone".parse::<TimezoneOffset>().is_err());
}

#[test]
fn cron_generate_five_fields() {
    let result = CronGenerateTool {
        second: None,
        minute: CronField::Raw("*/5".to_string()),
        hour: CronField::Raw("*".to_string()),
        day_of_month: CronField::Raw("*".to_string()),
        month: CronField::Raw("*".to_string()),
        day_of_week: CronField::Raw("*".to_string()),
        year: None,
    }
    .call_tool()
    .expect("cron_generate");

    let value = structured(&result);
    assert_eq!(value["expression"].as_str(), Some("*/5 * * * *"));
    assert_eq!(value["field_count"].as_u64(), Some(5));
    assert!(
        !value["description"]
            .as_str()
            .expect("description")
            .is_empty()
    );

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn cron_generate_six_and_seven_fields() {
    let six = CronGenerateTool {
        second: Some(CronField::Raw("0".to_string())),
        minute: CronField::Raw("*/5".to_string()),
        hour: CronField::Raw("*".to_string()),
        day_of_month: CronField::Raw("*".to_string()),
        month: CronField::Raw("*".to_string()),
        day_of_week: CronField::Raw("*".to_string()),
        year: None,
    }
    .call_tool()
    .expect("cron_generate");
    let value = structured(&six);
    assert_eq!(value["expression"].as_str(), Some("0 */5 * * * *"));
    assert_eq!(value["field_count"].as_u64(), Some(6));

    let seven = CronGenerateTool {
        second: Some(CronField::Raw("0".to_string())),
        minute: CronField::Raw("0".to_string()),
        hour: CronField::Raw("9".to_string()),
        day_of_month: CronField::Raw("1".to_string()),
        month: CronField::Raw("1".to_string()),
        day_of_week: CronField::Raw("*".to_string()),
        year: Some(CronField::Raw("2026".to_string())),
    }
    .call_tool()
    .expect("cron_generate");
    let value = structured(&seven);
    assert_eq!(value["expression"].as_str(), Some("0 0 9 1 1 * 2026"));
    assert_eq!(value["field_count"].as_u64(), Some(7));
}

#[test]
fn cron_generate_rejects_invalid_input() {
    // Empty field.
    assert!(
        CronGenerateTool {
            second: None,
            minute: CronField::Raw(String::new()),
            hour: CronField::Raw("*".to_string()),
            day_of_month: CronField::Raw("*".to_string()),
            month: CronField::Raw("*".to_string()),
            day_of_week: CronField::Raw("*".to_string()),
            year: None,
        }
        .call_tool()
        .is_err()
    );

    // Whitespace inside a field would add another cron field.
    assert!(
        CronGenerateTool {
            second: None,
            minute: CronField::Raw("1 2".to_string()),
            hour: CronField::Raw("*".to_string()),
            day_of_month: CronField::Raw("*".to_string()),
            month: CronField::Raw("*".to_string()),
            day_of_week: CronField::Raw("*".to_string()),
            year: None,
        }
        .call_tool()
        .is_err()
    );

    // Year requires seconds.
    assert!(
        CronGenerateTool {
            second: None,
            minute: CronField::Raw("0".to_string()),
            hour: CronField::Raw("9".to_string()),
            day_of_month: CronField::Raw("1".to_string()),
            month: CronField::Raw("1".to_string()),
            day_of_week: CronField::Raw("*".to_string()),
            year: Some(CronField::Raw("2026".to_string())),
        }
        .call_tool()
        .is_err()
    );

    // Invalid field value.
    assert!(
        CronGenerateTool {
            second: None,
            minute: CronField::Raw("99".to_string()),
            hour: CronField::Raw("*".to_string()),
            day_of_month: CronField::Raw("*".to_string()),
            month: CronField::Raw("*".to_string()),
            day_of_week: CronField::Raw("*".to_string()),
            year: None,
        }
        .call_tool()
        .is_err()
    );
}

#[test]
fn cron_generate_structured_terms() {
    let step = CronGenerateTool {
        second: None,
        minute: CronField::Term(CronTerm::Step {
            start: None,
            end: None,
            step: 5,
        }),
        hour: CronField::Raw("*".to_string()),
        day_of_month: CronField::Raw("*".to_string()),
        month: CronField::Raw("*".to_string()),
        day_of_week: CronField::Raw("*".to_string()),
        year: None,
    }
    .call_tool()
    .expect("cron_generate");
    assert_eq!(structured(&step)["expression"], "*/5 * * * *");

    let range_step = CronGenerateTool {
        second: None,
        minute: CronField::Term(CronTerm::Step {
            start: Some(1),
            end: Some(10),
            step: 2,
        }),
        hour: CronField::Raw("*".to_string()),
        day_of_month: CronField::Raw("*".to_string()),
        month: CronField::Raw("*".to_string()),
        day_of_week: CronField::Raw("*".to_string()),
        year: None,
    }
    .call_tool()
    .expect("cron_generate");
    assert_eq!(structured(&range_step)["expression"], "1-10/2 * * * *");

    let list = CronGenerateTool {
        second: None,
        minute: CronField::List(vec![
            CronTerm::Range { start: 0, end: 30 },
            CronTerm::Value { value: 45 },
        ]),
        hour: CronField::Raw("*".to_string()),
        day_of_month: CronField::Raw("*".to_string()),
        month: CronField::Raw("*".to_string()),
        day_of_week: CronField::Raw("*".to_string()),
        year: None,
    }
    .call_tool()
    .expect("cron_generate");
    assert_eq!(structured(&list)["expression"], "0-30,45 * * * *");

    let weekdays = CronGenerateTool {
        second: None,
        minute: CronField::Number(0),
        hour: CronField::Number(9),
        day_of_month: CronField::Raw("*".to_string()),
        month: CronField::Raw("*".to_string()),
        day_of_week: CronField::Term(CronTerm::Range { start: 1, end: 5 }),
        year: None,
    }
    .call_tool()
    .expect("cron_generate");
    assert_eq!(structured(&weekdays)["expression"], "0 9 * * 1-5");
}

#[test]
fn cron_generate_rejects_invalid_structured_terms() {
    // step must be positive
    assert!(
        CronGenerateTool {
            second: None,
            minute: CronField::Term(CronTerm::Step {
                start: None,
                end: None,
                step: 0,
            }),
            hour: CronField::Raw("*".to_string()),
            day_of_month: CronField::Raw("*".to_string()),
            month: CronField::Raw("*".to_string()),
            day_of_week: CronField::Raw("*".to_string()),
            year: None,
        }
        .call_tool()
        .is_err()
    );

    // end requires start
    assert!(
        CronGenerateTool {
            second: None,
            minute: CronField::Term(CronTerm::Step {
                start: None,
                end: Some(10),
                step: 2,
            }),
            hour: CronField::Raw("*".to_string()),
            day_of_month: CronField::Raw("*".to_string()),
            month: CronField::Raw("*".to_string()),
            day_of_week: CronField::Raw("*".to_string()),
            year: None,
        }
        .call_tool()
        .is_err()
    );

    // named values are not supported
    assert!(
        CronGenerateTool {
            second: None,
            minute: CronField::Raw("MON".to_string()),
            hour: CronField::Raw("*".to_string()),
            day_of_month: CronField::Raw("*".to_string()),
            month: CronField::Raw("*".to_string()),
            day_of_week: CronField::Raw("*".to_string()),
            year: None,
        }
        .call_tool()
        .is_err()
    );

    // empty list
    assert!(
        CronGenerateTool {
            second: None,
            minute: CronField::List(Vec::new()),
            hour: CronField::Raw("*".to_string()),
            day_of_month: CronField::Raw("*".to_string()),
            month: CronField::Raw("*".to_string()),
            day_of_week: CronField::Raw("*".to_string()),
            year: None,
        }
        .call_tool()
        .is_err()
    );
}

#[test]
fn cron_parse_five_fields() {
    let result = CronParseTool {
        expression: "*/5 * * * *".to_string(),
    }
    .call_tool()
    .expect("cron_parse");

    let value = structured(&result);
    assert_eq!(value["expression"].as_str(), Some("*/5 * * * *"));
    assert_eq!(value["field_count"].as_u64(), Some(5));
    assert!(value["fields"]["second"].is_null());
    assert_eq!(
        value["fields"]["minute"],
        serde_json::json!({ "type": "step", "step": 5 })
    );
    assert_eq!(
        value["fields"]["hour"],
        serde_json::json!({ "type": "wildcard" })
    );
    assert_eq!(
        value["fields"]["day_of_month"],
        serde_json::json!({ "type": "wildcard" })
    );
    assert_eq!(
        value["fields"]["month"],
        serde_json::json!({ "type": "wildcard" })
    );
    assert_eq!(
        value["fields"]["day_of_week"],
        serde_json::json!({ "type": "wildcard" })
    );
    assert!(value["fields"]["year"].is_null());
    assert_eq!(value["raw_fields"]["minute"].as_str(), Some("*/5"));
    assert!(
        !value["description"]
            .as_str()
            .expect("description")
            .is_empty()
    );

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn cron_parse_six_and_seven_fields() {
    let six = CronParseTool {
        expression: "0 */5 * * * *".to_string(),
    }
    .call_tool()
    .expect("cron_parse");
    let value = structured(&six);
    assert_eq!(value["field_count"].as_u64(), Some(6));
    assert_eq!(
        value["fields"]["second"],
        serde_json::json!({ "type": "value", "value": 0 })
    );
    assert_eq!(value["raw_fields"]["second"].as_str(), Some("0"));
    assert!(value["fields"]["year"].is_null());

    let seven = CronParseTool {
        expression: "  0  0 9 1 1 * 2026 ".to_string(),
    }
    .call_tool()
    .expect("cron_parse");
    let value = structured(&seven);
    assert_eq!(value["expression"].as_str(), Some("0 0 9 1 1 * 2026"));
    assert_eq!(value["field_count"].as_u64(), Some(7));
    assert_eq!(
        value["fields"]["second"],
        serde_json::json!({ "type": "value", "value": 0 })
    );
    assert_eq!(
        value["fields"]["year"],
        serde_json::json!({ "type": "value", "value": 2026 })
    );
    assert_eq!(value["raw_fields"]["year"].as_str(), Some("2026"));
}

#[test]
fn cron_parse_returns_structured_fields() {
    let result = CronParseTool {
        expression: "1-5,10,*/15 9 * * 1-5".to_string(),
    }
    .call_tool()
    .expect("cron_parse");

    let value = structured(&result);
    assert_eq!(
        value["fields"]["minute"],
        serde_json::json!([
            { "type": "range", "start": 1, "end": 5 },
            { "type": "value", "value": 10 },
            { "type": "step", "step": 15 }
        ])
    );
    assert_eq!(
        value["fields"]["hour"],
        serde_json::json!({ "type": "value", "value": 9 })
    );
    assert_eq!(
        value["fields"]["day_of_week"],
        serde_json::json!({ "type": "range", "start": 1, "end": 5 })
    );
    assert_eq!(value["raw_fields"]["minute"].as_str(), Some("1-5,10,*/15"));

    let named = CronParseTool {
        expression: "0 9 * * MON".to_string(),
    }
    .call_tool()
    .expect("cron_parse");
    assert_eq!(structured(&named)["fields"]["day_of_week"], "MON");
}

#[test]
fn cron_parse_rejects_invalid_expression() {
    assert!(
        CronParseTool {
            expression: "bad".to_string(),
        }
        .call_tool()
        .is_err()
    );

    assert!(
        CronParseTool {
            expression: "0 0 0 0 0 0 0 0".to_string(),
        }
        .call_tool()
        .is_err()
    );
}

#[test]
fn cron_prev_utc() {
    let result = CronPrevTool {
        expression: "0 * * * *".to_string(),
        from: Some("2026-01-01T03:00:00Z".parse().expect("valid datetime")),
        count: Some(3),
        timezone: None,
    }
    .call_tool()
    .expect("cron_prev");

    let value = structured(&result);
    assert_eq!(value["timezone"].as_str(), Some("Z"));
    assert_eq!(value["count"].as_u64(), Some(3));
    assert_eq!(
        value["times"],
        serde_json::json!([
            "2026-01-01T02:00:00Z",
            "2026-01-01T01:00:00Z",
            "2026-01-01T00:00:00Z"
        ])
    );
}

#[test]
fn cron_prev_with_offset() {
    let result = CronPrevTool {
        expression: "0 9 * * *".to_string(),
        from: Some("2026-01-01T09:00:00+08:00".parse().expect("valid datetime")),
        count: Some(1),
        timezone: Some("+08:00".parse().expect("valid offset")),
    }
    .call_tool()
    .expect("cron_prev");

    let value = structured(&result);
    assert_eq!(value["timezone"].as_str(), Some("+08:00"));
    assert_eq!(
        value["times"],
        serde_json::json!(["2025-12-31T09:00:00+08:00"])
    );
}

#[test]
fn cron_prev_rejects_invalid_input() {
    assert!(
        CronPrevTool {
            expression: "bad".to_string(),
            from: None,
            count: None,
            timezone: None,
        }
        .call_tool()
        .is_err()
    );

    assert!(
        CronPrevTool {
            expression: "0 * * * *".to_string(),
            from: None,
            count: Some(0),
            timezone: None,
        }
        .call_tool()
        .is_err()
    );
}
