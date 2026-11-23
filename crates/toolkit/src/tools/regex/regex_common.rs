use regex::{Captures, Regex, RegexBuilder};
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde_json::{Map, Value, json};

pub(super) fn build_regex(pattern: &str, case_insensitive: bool) -> Result<Regex, CallToolError> {
    RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .build()
        .map_err(|err| CallToolError::from_message(format!("invalid regex: {err}")))
}

/// JSON object describing one match: text, byte offsets, captures, named captures.
pub(super) fn capture_value(regex: &Regex, captures: &Captures<'_>) -> Value {
    let matched = captures.get(0).expect("capture group 0 always exists");

    let groups: Vec<Value> = (1..captures.len())
        .map(|index| match captures.get(index) {
            Some(value) => Value::String(value.as_str().to_string()),
            None => Value::Null,
        })
        .collect();

    let mut named_groups = Map::new();
    for name in regex.capture_names().flatten() {
        named_groups.insert(
            name.to_string(),
            match captures.name(name) {
                Some(value) => Value::String(value.as_str().to_string()),
                None => Value::Null,
            },
        );
    }

    json!({
        "text": matched.as_str(),
        "start": matched.start(),
        "end": matched.end(),
        "groups": groups,
        "named_groups": named_groups,
    })
}

pub(super) fn match_item_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "text": {
                "type": "string",
                "description": "Matched text."
            },
            "start": {
                "type": "integer",
                "minimum": 0,
                "description": "Start byte offset in the UTF-8 input."
            },
            "end": {
                "type": "integer",
                "minimum": 0,
                "description": "End byte offset (exclusive) in the UTF-8 input."
            },
            "groups": {
                "type": "array",
                "description": "Capture groups 1..N; null for unmatched groups.",
                "items": { "type": ["string", "null"] }
            },
            "named_groups": {
                "type": "object",
                "description": "Named capture groups; null for unmatched groups.",
                "additionalProperties": { "type": ["string", "null"] }
            }
        },
        "required": ["text", "start", "end", "groups", "named_groups"]
    })
}
