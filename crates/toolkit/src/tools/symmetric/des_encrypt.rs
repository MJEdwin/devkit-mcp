use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use crate::BinaryData;

use super::des_common::{DesMode, DesPadding, encrypt};

/// Encrypts binary data with DES or Triple DES.
#[mcp_tool(
    name = "des_encrypt",
    title = "DES Encrypt",
    description = "Encrypts binary data with DES or Triple DES. Key length selects DES (8 bytes), 2-key 3DES (16), or 3-key 3DES (24). Modes: ecb, cbc, ctr, cfb, ofb. Legacy algorithms; prefer AES for new designs."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DesEncryptTool {
    /// Plaintext as a base64-encoded string (JSON Schema format: binary).
    pub data: BinaryData,

    /// Key as a base64-encoded string (JSON Schema format: binary): 8 (DES), 16 (2-key 3DES), or 24 (3-key 3DES) bytes.
    pub key: BinaryData,

    /// Mode: "ecb", "cbc", "ctr", "cfb", or "ofb".
    pub mode: DesMode,

    /// Padding: "pkcs7" (ecb/cbc only) or "none".
    pub padding: DesPadding,

    /// IV as a base64-encoded string (JSON Schema format: binary).
    /// 8 bytes for cbc/ctr/cfb/ofb; ignored for ecb.
    pub iv: Option<BinaryData>,
}

impl DesEncryptTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ciphertext = encrypt(
            &self.data,
            &self.key,
            self.mode,
            self.padding,
            self.iv.as_deref(),
        )?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(ciphertext),
        )]))
    }
}
