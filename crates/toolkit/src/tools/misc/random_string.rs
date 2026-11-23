use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Character set used for random strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum RandomCharset {
    #[serde(rename = "alphanumeric")]
    Alphanumeric,
    #[serde(rename = "alphabetic")]
    Alphabetic,
    #[serde(rename = "numeric")]
    Numeric,
    #[serde(rename = "hex")]
    Hex,
}

impl RandomCharset {
    pub fn json_schema() -> Map<String, Value> {
        crate::tools::string_enum_schema(&["alphanumeric", "alphabetic", "numeric", "hex"])
    }

    fn characters(self) -> &'static [u8] {
        match self {
            Self::Alphanumeric => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            Self::Alphabetic => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
            Self::Numeric => b"0123456789",
            Self::Hex => b"0123456789abcdef",
        }
    }
}

/// Generates a random string using the OS RNG.
#[mcp_tool(
    name = "random_string",
    title = "Random String Generator",
    description = "Generates a random string of the requested length from the operating system RNG."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RandomStringTool {
    /// String length, 1-1024. Defaults to 16.
    #[json_schema(default = 16, minimum = 1, maximum = 1024)]
    pub length: Option<u32>,

    /// String character set. Defaults to "alphanumeric".
    #[json_schema(default = "alphanumeric")]
    pub charset: Option<RandomCharset>,
}

impl RandomStringTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let length = self.length.unwrap_or(16);
        if !(1..=1024).contains(&length) {
            return Err(CallToolError::from_message(format!(
                "length must be between 1 and 1024, got {length}"
            )));
        }

        let value = random_string(
            length as usize,
            self.charset.unwrap_or(RandomCharset::Alphanumeric),
        )?;

        Ok(CallToolResult::text_content(vec![TextContent::from(value)]))
    }
}

fn random_string(length: usize, charset: RandomCharset) -> Result<String, CallToolError> {
    let characters = charset.characters();
    let count = characters.len() as u32;
    let zone = u32::MAX - (u32::MAX % count);

    let mut result = String::with_capacity(length);
    while result.len() < length {
        let value = getrandom::u32().map_err(rng_error)?;
        if value < zone {
            let index = (value % count) as usize;
            result.push(characters[index] as char);
        }
    }

    Ok(result)
}

fn rng_error(err: getrandom::Error) -> CallToolError {
    CallToolError::from_message(format!("random string generation failed: {err}"))
}
