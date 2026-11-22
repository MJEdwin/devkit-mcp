use rsa::traits::{PrivateKeyParts, PublicKeyParts};
use rsa::{BoxedUint, PrivateKey, PublicKey};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::rsa_common::{parse_private_key, parse_public_key};

/// Extracts the numeric components of an RSA key.
#[mcp_tool(
    name = "rsa_key_components",
    title = "RSA Key Components",
    description = "Extracts RSA key components as lowercase hex. Public keys return n/e; private keys also return d, p, q, dp, dq, and qinv. Accepts an unencrypted private or public key PEM."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RsaKeyComponentsTool {
    /// RSA private or public key in unencrypted PEM format.
    pub key: String,
}

impl RsaKeyComponentsTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let structured = if let Ok(private_key) = parse_private_key(&self.key) {
            private_components(&private_key)
        } else if let Ok(public_key) = parse_public_key(&self.key) {
            public_components(&public_key)
        } else {
            return Err(CallToolError::from_message(
                "invalid RSA private or public key PEM".to_string(),
            ));
        };

        let text =
            serde_json::to_string_pretty(&Value::Object(structured.clone())).map_err(|err| {
                CallToolError::from_message(format!("failed to serialize key components: {err}"))
            })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(text)])
            .with_structured_content(structured))
    }

    pub(crate) fn output_schema() -> ToolOutputSchema {
        crate::tools::object_output_schema(json!({
            "type": "object",
            "properties": {
                "key_type": { "type": "string", "enum": ["private", "public"] },
                "bits": { "type": "integer" },
                "n": { "type": "string" },
                "e": { "type": "string" },
                "d": { "type": "string" },
                "p": { "type": "string" },
                "q": { "type": "string" },
                "dp": { "type": "string" },
                "dq": { "type": "string" },
                "qinv": { "type": "string" }
            },
            "required": ["key_type", "bits", "n", "e"]
        }))
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == RsaKeyComponentsTool::tool_name() {
        Some(RsaKeyComponentsTool::output_schema())
    } else {
        None
    }
}

fn public_components(key: &PublicKey) -> Map<String, Value> {
    let mut map = base_components(key);
    map.insert("key_type".to_string(), Value::String("public".to_string()));
    map
}

fn private_components(key: &PrivateKey) -> Map<String, Value> {
    let mut map = base_components(key);
    map.insert("key_type".to_string(), Value::String("private".to_string()));
    map.insert("d".to_string(), Value::String(hex_component(key.d())));

    let primes = key.primes();
    if let Some(p) = primes.first() {
        map.insert("p".to_string(), Value::String(hex_component(p)));
    }
    if let Some(q) = primes.get(1) {
        map.insert("q".to_string(), Value::String(hex_component(q)));
    }
    if let Some(dp) = key.dp() {
        map.insert("dp".to_string(), Value::String(hex_component(dp)));
    }
    if let Some(dq) = key.dq() {
        map.insert("dq".to_string(), Value::String(hex_component(dq)));
    }
    if let Some(qinv) = key.qinv() {
        map.insert(
            "qinv".to_string(),
            Value::String(hex_component(&qinv.retrieve())),
        );
    }

    map
}

fn base_components<K: PublicKeyParts>(key: &K) -> Map<String, Value> {
    let n = key.n().as_ref();

    let mut map = Map::new();
    map.insert(
        "bits".to_string(),
        Value::Number((n.bit_width_vartime() as u64).into()),
    );
    map.insert("n".to_string(), Value::String(hex_component(n)));
    map.insert("e".to_string(), Value::String(hex_component(key.e())));
    map
}

fn hex_component(value: &BoxedUint) -> String {
    hex_simd::encode_to_string(
        value.to_be_bytes_trimmed_vartime(),
        hex_simd::AsciiCase::Lower,
    )
}
