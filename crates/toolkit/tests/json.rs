mod common;

use common::{structured, text};
use toolkit::{JsonPrettyTool, JsonSchemaValidateTool, JsonValidateTool, UnicodeMode};

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
