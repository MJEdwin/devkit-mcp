use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use crate::BinaryData;

use super::aes_common::{AesMode, AesPadding, decrypt};

/// Decrypts AES-encrypted binary data.
#[mcp_tool(
    name = "aes_decrypt",
    title = "AES Decrypt",
    description = "Decrypts binary data with AES. Modes: ecb, cbc, ctr, cfb, ofb, gcm. Padding: pkcs7 (ecb/cbc only) or none. GCM input must be ciphertext||tag."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AesDecryptTool {
    /// Ciphertext as a base64-encoded string (JSON Schema format: binary).
    pub data: BinaryData,

    /// AES key as a base64-encoded string (JSON Schema format: binary): 16, 24, or 32 bytes.
    pub key: BinaryData,

    /// AES mode: "ecb", "cbc", "ctr", "cfb", "ofb", or "gcm".
    pub mode: AesMode,

    /// Padding: "pkcs7" (ecb/cbc only) or "none".
    pub padding: AesPadding,

    /// IV/nonce as a base64-encoded string (JSON Schema format: binary).
    /// 16 bytes for cbc/ctr/cfb/ofb, 12 bytes for gcm; ignored for ecb.
    pub iv: Option<BinaryData>,
}

impl AesDecryptTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let plaintext = decrypt(
            &self.data,
            &self.key,
            self.mode,
            self.padding,
            self.iv.as_deref(),
        )?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(plaintext),
        )]))
    }
}
