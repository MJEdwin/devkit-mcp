mod common;

use common::{structured, text};
use toolkit::{
    JsonMinifyTool, JsonPrettyTool, JsonQueryTool, JsonSchemaValidateTool, JsonValidateTool,
    UnicodeMode,
};

#[test]
fn json_pretty_formats_with_default_indent() {
    let result = JsonPrettyTool {
        json: r#"{"a":1,"b":[true,null,"x"]}"#.to_string(),
        indent: None,
        unicode: None,
    }
    .call_tool()
    .expect("json_pretty");

    assert_eq!(
        text(&result),
        "{\n  \"a\": 1,\n  \"b\": [\n    true,\n    null,\n    \"x\"\n  ]\n}"
    );
}

#[test]
fn json_pretty_custom_and_zero_indent() {
    let four = JsonPrettyTool {
        json: r#"{"a":1}"#.to_string(),
        indent: Some(4),
        unicode: None,
    }
    .call_tool()
    .expect("json_pretty");
    assert_eq!(text(&four), "{\n    \"a\": 1\n}");

    let zero = JsonPrettyTool {
        json: r#"{"a":1}"#.to_string(),
        indent: Some(0),
        unicode: None,
    }
    .call_tool()
    .expect("json_pretty");
    assert_eq!(text(&zero), "{\n\"a\": 1\n}");
}

#[test]
fn json_pretty_preserves_key_order() {
    let result = JsonPrettyTool {
        json: r#"{"z":1,"a":2,"m":3}"#.to_string(),
        indent: None,
        unicode: None,
    }
    .call_tool()
    .expect("json_pretty");

    let rendered = text(&result);
    let z = rendered.find("\"z\"").expect("z key");
    let a = rendered.find("\"a\"").expect("a key");
    let m = rendered.find("\"m\"").expect("m key");
    assert!(z < a && a < m, "keys were reordered: {rendered}");
}

#[test]
fn json_pretty_unicode_conversion() {
    let input = r#"{"msg":"你好","emoji":"😀"}"#;

    let decoded = JsonPrettyTool {
        json: input.to_string(),
        indent: None,
        unicode: None,
    }
    .call_tool()
    .expect("json_pretty");
    let decoded_text = text(&decoded);
    assert!(decoded_text.contains("你好"));
    assert!(decoded_text.contains("😀"));

    let explicit = JsonPrettyTool {
        json: input.to_string(),
        indent: None,
        unicode: Some(UnicodeMode::Characters),
    }
    .call_tool()
    .expect("json_pretty");
    assert!(text(&explicit).contains("你好"));

    let escaped = JsonPrettyTool {
        json: input.to_string(),
        indent: None,
        unicode: Some(UnicodeMode::Escapes),
    }
    .call_tool()
    .expect("json_pretty");
    let escaped_text = text(&escaped);
    assert!(escaped_text.contains("\\u4f60\\u597d"));
    assert!(escaped_text.contains("\\ud83d\\ude00"));

    let parsed: serde_json::Value = serde_json::from_str(escaped_text).expect("valid JSON");
    let original: serde_json::Value = serde_json::from_str(input).expect("valid JSON");
    assert_eq!(parsed, original);
}

#[test]
fn json_validate_accepts_valid_json() {
    for input in [
        "{}",
        "[1,2,3]",
        "\"hello\"",
        "42",
        "null",
        "{\"a\": [true, null]}",
    ] {
        let result = JsonValidateTool {
            json: input.to_string(),
        }
        .call_tool()
        .expect("json_validate");

        let value = structured(&result);
        assert_eq!(value["valid"].as_bool(), Some(true), "input: {input}");
        assert!(value["line"].is_null());
        assert!(value["column"].is_null());
        assert!(value["error"].is_null());

        let pretty = serde_json::to_string_pretty(value).expect("serialize");
        assert_eq!(text(&result), pretty);
    }
}

#[test]
fn json_validate_reports_position_and_reason() {
    let result = JsonValidateTool {
        json: "{\"a\":}".to_string(),
    }
    .call_tool()
    .expect("json_validate");

    let value = structured(&result);
    assert_eq!(value["valid"].as_bool(), Some(false));
    assert_eq!(value["line"].as_u64(), Some(1));
    assert_eq!(value["column"].as_u64(), Some(6));

    let error = value["error"].as_str().expect("error message");
    assert!(
        error.contains("expected value"),
        "unexpected error: {error}"
    );
}

#[test]
fn json_validate_reports_multiline_position() {
    let input = "{\n  \"a\": 1,\n  \"b\": }";
    let result = JsonValidateTool {
        json: input.to_string(),
    }
    .call_tool()
    .expect("json_validate");

    let value = structured(&result);
    assert_eq!(value["valid"].as_bool(), Some(false));
    assert_eq!(value["line"].as_u64(), Some(3));
    assert_eq!(value["column"].as_u64(), Some(8));

    let error = value["error"].as_str().expect("error message");
    assert!(
        error.contains("expected value"),
        "unexpected error: {error}"
    );
}

#[test]
fn json_validate_rejects_empty_input() {
    let result = JsonValidateTool {
        json: String::new(),
    }
    .call_tool()
    .expect("json_validate");

    let value = structured(&result);
    assert_eq!(value["valid"].as_bool(), Some(false));
    assert_eq!(value["line"].as_u64(), Some(1));
    assert_eq!(value["column"].as_u64(), Some(1));
}

#[test]
fn json_pretty_rejects_invalid_input() {
    let invalid = JsonPrettyTool {
        json: "{\"a\":".to_string(),
        indent: None,
        unicode: None,
    }
    .call_tool();
    assert!(invalid.is_err());

    let bad_indent = JsonPrettyTool {
        json: "{}".to_string(),
        indent: Some(9),
        unicode: None,
    }
    .call_tool();
    assert!(bad_indent.is_err());
}

#[test]
fn json_schema_validate_passes() {
    let result = JsonSchemaValidateTool {
        json: r#"{"age":42,"name":"Ada"}"#.to_string(),
        json_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "age": { "type": "integer", "minimum": 0 },
                "name": { "type": "string" }
            },
            "required": ["age", "name"]
        }),
    }
    .call_tool()
    .expect("json_schema_validate");

    let value = structured(&result);
    assert_eq!(value["valid"].as_bool(), Some(true));
    assert_eq!(value["schema_errors"].as_array().map(Vec::len), Some(0));

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn json_schema_validate_reports_violations() {
    let result = JsonSchemaValidateTool {
        json: r#"{"age":"old"}"#.to_string(),
        json_schema: serde_json::json!({
            "type": "object",
            "properties": { "age": { "type": "integer" } },
            "required": ["age"]
        }),
    }
    .call_tool()
    .expect("json_schema_validate");

    let value = structured(&result);
    assert_eq!(value["valid"].as_bool(), Some(false));

    let errors = value["schema_errors"].as_array().expect("schema errors");
    assert!(!errors.is_empty());
    assert_eq!(errors[0]["path"].as_str(), Some("/age"));
    assert!(errors[0]["schema_path"].as_str().is_some());
    assert!(
        errors[0]["message"]
            .as_str()
            .expect("message")
            .contains("integer")
    );
}

#[test]
fn json_schema_validate_accepts_schema_as_string() {
    let result = JsonSchemaValidateTool {
        json: r#"{"age":42}"#.to_string(),
        json_schema: serde_json::json!(r#"{"type":"object","required":["age"]}"#),
    }
    .call_tool()
    .expect("json_schema_validate");

    assert_eq!(structured(&result)["valid"].as_bool(), Some(true));
}

#[test]
fn json_schema_validate_rejects_invalid_schema() {
    let result = JsonSchemaValidateTool {
        json: "{}".to_string(),
        json_schema: serde_json::json!({ "type": 123 }),
    }
    .call_tool();

    assert!(result.is_err());
}

#[test]
fn json_schema_validate_rejects_invalid_json() {
    let result = JsonSchemaValidateTool {
        json: "{".to_string(),
        json_schema: serde_json::json!({ "type": "object" }),
    }
    .call_tool();

    assert!(result.is_err());
}

#[test]
fn json_minify_removes_whitespace() {
    let result = JsonMinifyTool {
        json: "{\n  \"a\": 1,\n  \"b\": [true, null, \"x\"]\n}".to_string(),
        unicode: None,
    }
    .call_tool()
    .expect("json_minify");

    assert_eq!(text(&result), r#"{"a":1,"b":[true,null,"x"]}"#);
}

#[test]
fn json_minify_preserves_key_order() {
    let result = JsonMinifyTool {
        json: r#"{ "z": 1, "a": 2, "m": 3 }"#.to_string(),
        unicode: None,
    }
    .call_tool()
    .expect("json_minify");

    assert_eq!(text(&result), r#"{"z":1,"a":2,"m":3}"#);
}

#[test]
fn json_minify_unicode_modes() {
    let input = r#"{"msg":"你好","emoji":"😀"}"#;

    let default = JsonMinifyTool {
        json: input.to_string(),
        unicode: None,
    }
    .call_tool()
    .expect("json_minify");
    assert!(text(&default).contains("你好"));
    assert!(!text(&default).contains('\n'));

    let escaped = JsonMinifyTool {
        json: input.to_string(),
        unicode: Some(UnicodeMode::Escapes),
    }
    .call_tool()
    .expect("json_minify");
    assert!(text(&escaped).contains("\\u4f60\\u597d"));
    assert!(text(&escaped).contains("\\ud83d\\ude00"));

    let parsed: serde_json::Value = serde_json::from_str(text(&escaped)).expect("valid JSON");
    let original: serde_json::Value = serde_json::from_str(input).expect("valid JSON");
    assert_eq!(parsed, original);
}

#[test]
fn json_minify_rejects_invalid_input() {
    let result = JsonMinifyTool {
        json: "{".to_string(),
        unicode: None,
    }
    .call_tool();

    assert!(result.is_err());
}

#[test]
fn json_query_selects_values() {
    let result = JsonQueryTool {
        json: r#"{"store":{"book":[{"title":"A","price":10},{"title":"B","price":20}]}}"#
            .to_string(),
        path: "$.store.book[*].title".to_string(),
        limit: None,
    }
    .call_tool()
    .expect("json_query");

    let value = structured(&result);
    assert_eq!(value["count"].as_u64(), Some(2));
    assert_eq!(value["truncated"].as_bool(), Some(false));
    assert_eq!(value["matches"], serde_json::json!(["A", "B"]));

    let pretty = serde_json::to_string_pretty(value).expect("serialize");
    assert_eq!(text(&result), pretty);
}

#[test]
fn json_query_supports_filters_and_recursion() {
    let input = r#"{"store":{"book":[{"title":"A","price":10},{"title":"B","price":20}]}}"#;

    let filtered = JsonQueryTool {
        json: input.to_string(),
        path: "$.store.book[?(@.price > 15)].title".to_string(),
        limit: None,
    }
    .call_tool()
    .expect("json_query");
    assert_eq!(structured(&filtered)["matches"], serde_json::json!(["B"]));

    let prices = JsonQueryTool {
        json: input.to_string(),
        path: "$..price".to_string(),
        limit: None,
    }
    .call_tool()
    .expect("json_query");
    assert_eq!(structured(&prices)["matches"], serde_json::json!([10, 20]));
}

#[test]
fn json_query_no_matches() {
    let result = JsonQueryTool {
        json: r#"{"a":1}"#.to_string(),
        path: "$.missing".to_string(),
        limit: None,
    }
    .call_tool()
    .expect("json_query");

    let value = structured(&result);
    assert_eq!(value["count"].as_u64(), Some(0));
    assert_eq!(value["truncated"].as_bool(), Some(false));
    assert_eq!(value["matches"], serde_json::json!([]));
}

#[test]
fn json_query_truncates_at_limit() {
    let result = JsonQueryTool {
        json: r#"{"a":1,"b":2,"c":3}"#.to_string(),
        path: "$.*".to_string(),
        limit: Some(2),
    }
    .call_tool()
    .expect("json_query");

    let value = structured(&result);
    assert_eq!(value["count"].as_u64(), Some(2));
    assert_eq!(value["truncated"].as_bool(), Some(true));
    assert_eq!(value["matches"], serde_json::json!([1, 2]));
}

#[test]
fn json_query_rejects_invalid_input() {
    assert!(
        JsonQueryTool {
            json: "{".to_string(),
            path: "$".to_string(),
            limit: None,
        }
        .call_tool()
        .is_err()
    );

    assert!(
        JsonQueryTool {
            json: "{}".to_string(),
            path: "$[".to_string(),
            limit: None,
        }
        .call_tool()
        .is_err()
    );

    assert!(
        JsonQueryTool {
            json: "{}".to_string(),
            path: "$".to_string(),
            limit: Some(0),
        }
        .call_tool()
        .is_err()
    );
}
