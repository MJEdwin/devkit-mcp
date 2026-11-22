use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use crate::BinaryData;

use super::rsa_common::{RsaEncryptionPadding, parse_private_key, raw_decrypt};

/// Decrypts RSA-encrypted binary data.
#[mcp_tool(
    name = "rsa_decrypt",
    title = "RSA Decrypt",
    description = "Decrypts binary data with an RSA private key using PKCS#1 v1.5, OAEP-SHA256, or raw (no padding) mode. The output is base64-encoded binary."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RsaDecryptTool {
    /// Ciphertext (base64-encoded on the wire). Raw mode requires exactly the modulus byte length.
    pub data: BinaryData,

    /// RSA private key in unencrypted PEM format (PKCS#8 or PKCS#1).
    pub private_key: String,

    /// Padding scheme: "pkcs1v15", "oaep-sha256", or "raw" (unsafe, requires a full modulus-sized block).
    pub padding: RsaEncryptionPadding,
}

impl RsaDecryptTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let private_key = parse_private_key(&self.private_key)?;

        let decrypted = match self.padding {
            RsaEncryptionPadding::Pkcs1v15 => {
                private_key.pkcs1v15_decrypt(&self.data).map_err(|err| {
                    CallToolError::from_message(format!("RSA decryption failed: {err}"))
                })?
            }
            RsaEncryptionPadding::OaepSha256 => private_key
                .oaep_decrypt::<rsa::sha2::Sha256>(&self.data)
                .map_err(|err| {
                    CallToolError::from_message(format!("RSA decryption failed: {err}"))
                })?,
            RsaEncryptionPadding::Raw => raw_decrypt(&private_key, &self.data)?,
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(decrypted),
        )]))
    }
}
