mod common;

use common::{structured, text};
use toolkit::{RegexMatchTool, RegexReplaceTool, RegexTestTool};

#[test]
fn regex_match_returns_matches_and_groups() {
    let result = RegexMatchTool {
        pattern: r"(\w+)-(\d+)".to_string(),
        text: "abc-123 def-456".to_string(),
        limit: None,
        case_insensitive: None,
    }
    .call_tool()
    .expect("regex_match");

    let value = structured(&result);
    assert_eq!(value["count"].as_u64(), Some(2));
    assert_eq!(value["truncated"].as_bool(), Some(false));

    let matches = value["matches"].as_array().expect("matches");
    assert_eq!(matches[0]["text"].as_str(), Some("abc-123"));
    assert_eq!(matches[0]["start"].as_u64(), Some(0));
    assert_eq!(matches[0]["end"].as_u64(), Some(7));
    assert_eq!(matches[0]["groups"], serde_json::json!(["abc", "123"]));
    assert_eq!(matches[1]["text"].as_str(), Some("def-456"));

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn regex_match_named_groups_and_unicode_offsets() {
    let result = RegexMatchTool {
        pattern: r"(?P<word>\w+)-(?P<num>\d+)".to_string(),
        text: "abc-123".to_string(),
        limit: None,
        case_insensitive: None,
    }
    .call_tool()
    .expect("regex_match");
    let value = structured(&result);
    assert_eq!(
        value["matches"][0]["named_groups"],
        serde_json::json!({ "word": "abc", "num": "123" })
    );

    let unicode = RegexMatchTool {
        pattern: "é".to_string(),
        text: "café".to_string(),
        limit: None,
        case_insensitive: None,
    }
    .call_tool()
    .expect("regex_match");
    let value = structured(&unicode);
    assert_eq!(value["matches"][0]["start"].as_u64(), Some(3));
    assert_eq!(value["matches"][0]["end"].as_u64(), Some(5));
}

#[test]
fn regex_match_limit_and_case_insensitive() {
    let result = RegexMatchTool {
        pattern: r"\w+".to_string(),
        text: "a b c".to_string(),
        limit: Some(2),
        case_insensitive: None,
    }
    .call_tool()
    .expect("regex_match");
    let value = structured(&result);
    assert_eq!(value["count"].as_u64(), Some(2));
    assert_eq!(value["truncated"].as_bool(), Some(true));

    let insensitive = RegexMatchTool {
        pattern: "hello".to_string(),
        text: "HELLO hello".to_string(),
        limit: None,
        case_insensitive: Some(true),
    }
    .call_tool()
    .expect("regex_match");
    assert_eq!(structured(&insensitive)["count"].as_u64(), Some(2));
}

#[test]
fn regex_match_rejects_invalid_regex() {
    let result = RegexMatchTool {
        pattern: "(".to_string(),
        text: "abc".to_string(),
        limit: None,
        case_insensitive: None,
    }
    .call_tool();

    assert!(result.is_err());
}

#[test]
fn regex_replace_expands_groups() {
    let result = RegexReplaceTool {
        pattern: r"(\w+)-(\d+)".to_string(),
        text: "abc-123 def-456".to_string(),
        replacement: "$2:$1".to_string(),
        limit: None,
        case_insensitive: None,
    }
    .call_tool()
    .expect("regex_replace");

    let value = structured(&result);
    assert_eq!(value["result"].as_str(), Some("123:abc 456:def"));
    assert_eq!(value["replacements"].as_u64(), Some(2));

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn regex_replace_respects_limit_and_named_groups() {
    let result = RegexReplaceTool {
        pattern: r"(?P<word>\w+)-(?P<num>\d+)".to_string(),
        text: "abc-123 def-456".to_string(),
        replacement: "${num}_${word}".to_string(),
        limit: Some(1),
        case_insensitive: None,
    }
    .call_tool()
    .expect("regex_replace");

    let value = structured(&result);
    assert_eq!(value["result"].as_str(), Some("123_abc def-456"));
    assert_eq!(value["replacements"].as_u64(), Some(1));
}

#[test]
fn regex_replace_rejects_invalid_regex() {
    let result = RegexReplaceTool {
        pattern: "[".to_string(),
        text: "abc".to_string(),
        replacement: "x".to_string(),
        limit: None,
        case_insensitive: None,
    }
    .call_tool();

    assert!(result.is_err());
}

#[test]
fn regex_test_reports_first_match() {
    let result = RegexTestTool {
        pattern: r"(?P<num>\d+)".to_string(),
        text: "abc 123".to_string(),
        case_insensitive: None,
    }
    .call_tool()
    .expect("regex_test");

    let value = structured(&result);
    assert_eq!(value["matched"].as_bool(), Some(true));
    assert_eq!(value["match"]["text"].as_str(), Some("123"));
    assert_eq!(value["match"]["named_groups"]["num"].as_str(), Some("123"));

    let missing = RegexTestTool {
        pattern: r"\d+".to_string(),
        text: "abc".to_string(),
        case_insensitive: None,
    }
    .call_tool()
    .expect("regex_test");
    let value = structured(&missing);
    assert_eq!(value["matched"].as_bool(), Some(false));
    assert!(value["match"].is_null());
}

#[test]
fn regex_test_rejects_invalid_regex() {
    let result = RegexTestTool {
        pattern: "*".to_string(),
        text: "abc".to_string(),
        case_insensitive: None,
    }
    .call_tool();

    assert!(result.is_err());
}
