use getrandom::SysRng;
use rsa::PrivateKey;
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::RsaPrivateKeyFormat;

/// Generates a new RSA private/public key pair.
#[mcp_tool(
    name = "rsa_generate_key",
    title = "RSA Generate Key",
    description = "Generates a new unencrypted RSA private/public key pair. The private key format is selected by `format` (PKCS#8 by default); the public key is SPKI PEM."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RsaGenerateKeyTool {
    /// RSA modulus size in bits. Defaults to 2048; values must be between 1024 and 8192.
    #[json_schema(default = 2048, minimum = 1024, maximum = 8192)]
    pub bits: Option<u32>,

    /// Private key format to return: "pkcs8" (default) or "pkcs1".
    #[json_schema(default = "pkcs8")]
    pub format: Option<RsaPrivateKeyFormat>,
}

impl RsaGenerateKeyTool {
    /// Synchronous entry point, used by tests.
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        generate_key(self.bits, self.format)
    }

    /// Async entry point for the MCP handler. Key generation is CPU-heavy, so
    /// it runs on Tokio's blocking pool instead of the async executor.
    pub async fn call_tool_async(&self) -> Result<CallToolResult, CallToolError> {
        let bits = self.bits;
        let format = self.format;

        tokio::task::spawn_blocking(move || {
            generate_key(bits, format).map_err(|err| err.to_string())
        })
        .await
        .map_err(|err| {
            CallToolError::from_message(format!("RSA key generation task failed: {err}"))
        })?
        .map_err(CallToolError::from_message)
    }

    pub(crate) fn output_schema() -> ToolOutputSchema {
        crate::tools::object_output_schema(json!({
            "type": "object",
            "properties": {
                "bits": { "type": "integer" },
                "format": { "type": "string", "enum": ["pkcs1", "pkcs8"] },
                "private_key": { "type": "string" },
                "public_key": { "type": "string" }
            },
            "required": ["bits", "format", "private_key", "public_key"]
        }))
    }
}

fn generate_key(
    bits: Option<u32>,
    format: Option<RsaPrivateKeyFormat>,
) -> Result<CallToolResult, CallToolError> {
    let bits = bits.unwrap_or(2048) as usize;

    if !(1024..=8192).contains(&bits) {
        return Err(CallToolError::from_message(
            "bits must be between 1024 and 8192".to_string(),
        ));
    }

    let format = format.unwrap_or(RsaPrivateKeyFormat::Pkcs8);

    let private_key = PrivateKey::new(&mut SysRng, bits)
        .map_err(|err| CallToolError::from_message(format!("RSA key generation failed: {err}")))?;
    let public_key = private_key.to_public_key();

    let private_pem = match format {
        RsaPrivateKeyFormat::Pkcs1 => private_key.to_pkcs1_pem(LineEnding::LF).map_err(|err| {
            CallToolError::from_message(format!("failed to encode PKCS#1 private key: {err}"))
        })?,
        RsaPrivateKeyFormat::Pkcs8 => private_key.to_pkcs8_pem(LineEnding::LF).map_err(|err| {
            CallToolError::from_message(format!("failed to encode PKCS#8 private key: {err}"))
        })?,
    };
    let public_pem = public_key
        .to_public_key_pem(LineEnding::LF)
        .map_err(|err| {
            CallToolError::from_message(format!("failed to encode public key: {err}"))
        })?;

    let mut structured = Map::new();
    structured.insert("bits".to_string(), Value::Number((bits as u64).into()));
    structured.insert(
        "format".to_string(),
        Value::String(format.as_str().to_string()),
    );
    structured.insert(
        "private_key".to_string(),
        Value::String(private_pem.as_str().to_owned()),
    );
    structured.insert("public_key".to_string(), Value::String(public_pem));

    let text = serde_json::to_string_pretty(&Value::Object(structured.clone()))
        .map_err(|err| CallToolError::from_message(format!("failed to serialize keys: {err}")))?;

    Ok(CallToolResult::text_content(vec![TextContent::from(text)])
        .with_structured_content(structured))
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == RsaGenerateKeyTool::tool_name() {
        Some(RsaGenerateKeyTool::output_schema())
    } else {
        None
    }
}
