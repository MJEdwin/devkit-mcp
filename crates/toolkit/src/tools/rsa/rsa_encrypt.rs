use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use crate::BinaryData;

use super::rsa_common::{RsaEncryptionPadding, parse_public_key, raw_encrypt};

/// Encrypts binary data with an RSA public key.
#[mcp_tool(
    name = "rsa_encrypt",
    title = "RSA Encrypt",
    description = "Encrypts binary data with an RSA public key using PKCS#1 v1.5, OAEP-SHA256, or raw (no padding) mode. The output is base64-encoded binary."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RsaEncryptTool {
    /// Plaintext (base64-encoded on the wire). Raw mode requires exactly the modulus byte length.
    pub data: BinaryData,

    /// RSA public key in unencrypted PEM format (SPKI or PKCS#1).
    pub public_key: String,

    /// Padding scheme: "pkcs1v15", "oaep-sha256", or "raw" (unsafe, requires a full modulus-sized block).
    pub padding: RsaEncryptionPadding,
}

impl RsaEncryptTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let public_key = parse_public_key(&self.public_key)?;

        let encrypted = match self.padding {
            RsaEncryptionPadding::Pkcs1v15 => {
                public_key.pkcs1v15_encrypt(&self.data).map_err(|err| {
                    CallToolError::from_message(format!("RSA encryption failed: {err}"))
                })?
            }
            RsaEncryptionPadding::OaepSha256 => public_key
                .oaep_encrypt::<rsa::sha2::Sha256>(&self.data)
                .map_err(|err| {
                    CallToolError::from_message(format!("RSA encryption failed: {err}"))
                })?,
            RsaEncryptionPadding::Raw => raw_encrypt(&public_key, &self.data)?,
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(encrypted),
        )]))
    }
}
