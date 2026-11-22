use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use crate::BinaryData;

use super::rsa_common::{RsaSignaturePadding, parse_private_key, raw_sign};

/// Signs binary data with an RSA private key.
#[mcp_tool(
    name = "rsa_sign",
    title = "RSA Sign",
    description = "Signs binary data with an RSA private key using PKCS#1 v1.5, PSS (both SHA-256), or raw (no padding) mode. The signature is returned as base64-encoded binary."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RsaSignTool {
    /// Message (base64-encoded on the wire).
    /// Raw mode expects a fully encoded EMSA block of exactly the modulus byte length.
    pub data: BinaryData,

    /// RSA private key in unencrypted PEM format (PKCS#8 or PKCS#1).
    pub private_key: String,

    /// Padding scheme: "pkcs1v15-sha256", "pss-sha256", or "raw" (unsafe, signs a caller-prepared block).
    pub padding: RsaSignaturePadding,
}

impl RsaSignTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let private_key = parse_private_key(&self.private_key)?;

        let signature = match self.padding {
            RsaSignaturePadding::Pkcs1v15Sha256 => private_key
                .pkcs1v15_sign::<rsa::sha2::Sha256>(&self.data)
                .map_err(|err| CallToolError::from_message(format!("RSA signing failed: {err}")))?,
            RsaSignaturePadding::PssSha256 => private_key
                .pss_sign::<rsa::sha2::Sha256>(&self.data)
                .map_err(|err| CallToolError::from_message(format!("RSA signing failed: {err}")))?,
            RsaSignaturePadding::Raw => raw_sign(&private_key, &self.data)?,
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(signature),
        )]))
    }
}
