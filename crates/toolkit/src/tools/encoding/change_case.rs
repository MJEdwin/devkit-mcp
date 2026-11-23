use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Target case for `change_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum CaseMode {
    #[serde(rename = "lowercase")]
    Lowercase,
    #[serde(rename = "uppercase")]
    Uppercase,
    #[serde(rename = "titlecase")]
    Titlecase,
}

impl CaseMode {
    pub fn json_schema() -> Map<String, Value> {
        crate::tools::string_enum_schema(&["lowercase", "uppercase", "titlecase"])
    }
}

/// Converts text between lowercase, uppercase, and title case.
#[mcp_tool(
    name = "change_case",
    title = "Change Letter Case",
    description = "Converts text to lowercase, uppercase, or title case using Unicode-aware case mappings."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ChangeCaseTool {
    /// Text to convert.
    pub text: String,

    /// Target case: "lowercase", "uppercase", or "titlecase".
    pub case: CaseMode,
}

impl ChangeCaseTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let converted = match self.case {
            CaseMode::Lowercase => self.text.to_lowercase(),
            CaseMode::Uppercase => self.text.to_uppercase(),
            CaseMode::Titlecase => title_case(&self.text),
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            converted,
        )]))
    }
}

/// Uppercases the first character of each whitespace-separated word and
/// lowercases the rest. Non-alphabetic characters are preserved.
fn title_case(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut at_word_start = true;

    for character in text.chars() {
        if character.is_whitespace() {
            result.push(character);
            at_word_start = true;
        } else if at_word_start {
            result.extend(character.to_uppercase());
            at_word_start = false;
        } else {
            result.extend(character.to_lowercase());
        }
    }

    result
}
