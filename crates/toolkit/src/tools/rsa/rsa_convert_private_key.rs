use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::{EncodePrivateKey, LineEnding};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use super::rsa_common::parse_private_key;

/// Target format for `rsa_convert_private_key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum RsaPrivateKeyFormat {
    /// PKCS#1 (`BEGIN RSA PRIVATE KEY`).
    #[serde(rename = "pkcs1")]
    Pkcs1,
    /// PKCS#8 (`BEGIN PRIVATE KEY`).
    #[serde(rename = "pkcs8")]
    Pkcs8,
}

impl RsaPrivateKeyFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            RsaPrivateKeyFormat::Pkcs1 => "pkcs1",
            RsaPrivateKeyFormat::Pkcs8 => "pkcs8",
        }
    }

    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["pkcs1", "pkcs8"])
    }
}

/// Converts an RSA private key between PEM formats.
#[mcp_tool(
    name = "rsa_convert_private_key",
    title = "RSA Convert Private Key",
    description = "Converts an unencrypted RSA private key between PKCS#1 and PKCS#8 PEM formats."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RsaConvertPrivateKeyTool {
    /// RSA private key in unencrypted PEM format (PKCS#1 or PKCS#8).
    pub private_key: String,

    /// Target format: "pkcs1" or "pkcs8".
    pub format: RsaPrivateKeyFormat,
}

impl RsaConvertPrivateKeyTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let private_key = parse_private_key(&self.private_key)?;

        let pem = match self.format {
            RsaPrivateKeyFormat::Pkcs1 => private_key
                .to_pkcs1_pem(LineEnding::LF)
                .map_err(|err| {
                    CallToolError::from_message(format!(
                        "failed to encode PKCS#1 private key: {err}"
                    ))
                })?
                .as_str()
                .to_owned(),
            RsaPrivateKeyFormat::Pkcs8 => private_key
                .to_pkcs8_pem(LineEnding::LF)
                .map_err(|err| {
                    CallToolError::from_message(format!(
                        "failed to encode PKCS#8 private key: {err}"
                    ))
                })?
                .as_str()
                .to_owned(),
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(pem)]))
    }
}
