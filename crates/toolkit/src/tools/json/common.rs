use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// How JSON tools render non-ASCII characters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum UnicodeMode {
    #[serde(rename = "characters")]
    Characters,
    #[serde(rename = "escapes")]
    Escapes,
}

impl UnicodeMode {
    pub fn json_schema() -> Map<String, Value> {
        crate::tools::string_enum_schema(&["characters", "escapes"])
    }
}

/// Replaces every non-ASCII character with a JSON `\uXXXX` escape, using a
/// surrogate pair for code points above U+FFFF.
pub(super) fn escape_non_ascii(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for character in text.chars() {
        let code = character as u32;
        if code <= 0x7F {
            result.push(character);
        } else if code <= 0xFFFF {
            result.push_str(&format!("\\u{code:04x}"));
        } else {
            let code = code - 0x1_0000;
            let high = 0xD800 + (code >> 10);
            let low = 0xDC00 + (code & 0x3FF);
            result.push_str(&format!("\\u{high:04x}\\u{low:04x}"));
        }
    }

    result
}
