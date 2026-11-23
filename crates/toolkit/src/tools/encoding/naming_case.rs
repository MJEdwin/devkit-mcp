use convert_case::{Case, Casing};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Target naming convention for `naming_case`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum NamingCase {
    #[serde(rename = "pascal")]
    Pascal,
    #[serde(rename = "camel")]
    Camel,
    #[serde(rename = "lower_snake")]
    LowerSnake,
    #[serde(rename = "upper_snake")]
    UpperSnake,
}

impl NamingCase {
    pub fn json_schema() -> Map<String, Value> {
        crate::tools::string_enum_schema(&["pascal", "camel", "lower_snake", "upper_snake"])
    }

    fn to_convert_case(self) -> Case<'static> {
        match self {
            Self::Pascal => Case::Pascal,
            Self::Camel => Case::Camel,
            Self::LowerSnake => Case::Snake,
            Self::UpperSnake => Case::UpperSnake,
        }
    }
}

/// Converts text between common naming conventions.
#[mcp_tool(
    name = "naming_case",
    title = "Naming Convention Converter",
    description = "Converts text between PascalCase, camelCase, lower_snake_case, and UPPER_SNAKE_CASE."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct NamingCaseTool {
    /// Text to convert, e.g. "my_var_name", "myVarName", or "HTTPRequest".
    pub text: String,

    /// Target naming convention: "pascal", "camel", "lower_snake", or "upper_snake".
    pub case: NamingCase,
}

impl NamingCaseTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let converted = self.text.to_case(self.case.to_convert_case());

        Ok(CallToolResult::text_content(vec![TextContent::from(
            converted,
        )]))
    }
}
