//! MCP tools grouped by category:
//!
//! - `time`     — current time, timestamp, and lunar/solar date conversion
//! - `encoding` — URL, binary/text encodings, case conversion
//! - `json`     — JSON formatting, querying, and validation
//! - `digest`   — hash and HMAC
//! - `symmetric` — AES / DES encryption/decryption
//! - `rsa`      — RSA keys, encryption, signatures
//! - `misc`     — other generators (UUID, random values)
//!
//! To add a tool:
//! 1. create `tools/<category>/<tool_name>.rs`
//! 2. register it in `tools/<category>/mod.rs` (`mod` + `pub use`)
//! 3. add the tool type to the `tool_box!` list below
//! 4. add a match arm in `crate::handler::ToolkitHandler`

pub mod digest;
pub mod encoding;
pub mod json;
pub mod misc;
pub mod rsa;
pub mod symmetric;
pub mod time;

pub use digest::*;
pub use encoding::*;
pub use json::*;
pub use misc::*;
pub use rsa::*;
pub use symmetric::*;
pub use time::*;

use rust_mcp_sdk::schema::{Tool, ToolOutputSchema};
use rust_mcp_sdk::tool_box;
use serde_json::Value;

// Generates the `ToolkitTools` enum plus `tools()` / `try_from(params)` helpers.
tool_box!(
    ToolkitTools,
    [
        GenerateUuidTool,
        RandomNumberTool,
        RandomStringTool,
        CurrentTimeTool,
        UnixToIso8601Tool,
        Iso8601ToUnixTool,
        SolarToLunarDateTool,
        LunarToSolarDateTool,
        UrlEncodeTool,
        UrlDecodeTool,
        BinaryToHexTool,
        HexToBinaryTool,
        BinaryToStringTool,
        StringToBinaryTool,
        ChangeCaseTool,
        NamingCaseTool,
        JsonPrettyTool,
        JsonMinifyTool,
        JsonQueryTool,
        JsonValidateTool,
        JsonSchemaValidateTool,
        DigestTool,
        HmacTool,
        RsaEncryptTool,
        RsaDecryptTool,
        RsaSignTool,
        RsaVerifyTool,
        RsaGenerateKeyTool,
        RsaPrivateToPublicTool,
        RsaConvertPrivateKeyTool,
        RsaKeyComponentsTool,
        AesEncryptTool,
        AesDecryptTool,
        AesGenerateKeyTool,
        AesGenerateIvTool,
        DesEncryptTool,
        DesDecryptTool,
        DesGenerateKeyTool,
        DesGenerateIvTool
    ]
);

/// Applies tool-local `outputSchema` metadata to the listed tools.
pub(crate) fn apply_output_schemas(tools: &mut [Tool]) {
    for tool in tools {
        tool.output_schema = OUTPUT_SCHEMA_PROVIDERS
            .iter()
            .find_map(|provider| provider(&tool.name));
    }
}

/// Builds a `ToolOutputSchema` from a JSON Schema object.
pub(crate) fn object_output_schema(schema: Value) -> ToolOutputSchema {
    ToolOutputSchema {
        schema: None,
        extra: schema.as_object().cloned(),
    }
}

/// JSON schema for a string enum: a single `enum` array rather than `oneOf`.
pub(crate) fn string_enum_schema(values: &[&str]) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert("type".to_string(), Value::String("string".to_string()));
    map.insert(
        "enum".to_string(),
        Value::Array(
            values
                .iter()
                .map(|value| Value::String((*value).to_string()))
                .collect(),
        ),
    );
    map
}

type OutputSchemaProvider = fn(&str) -> Option<ToolOutputSchema>;

const OUTPUT_SCHEMA_PROVIDERS: &[OutputSchemaProvider] = &[
    rsa::generate_key_output_schema,
    rsa::key_components_output_schema,
    time::solar_to_lunar_date_output_schema,
    time::lunar_to_solar_date_output_schema,
    json::json_validate_output_schema,
    json::json_schema_validate_output_schema,
    json::json_query_output_schema,
];
