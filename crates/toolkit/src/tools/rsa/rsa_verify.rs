use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use crate::BinaryData;

use super::rsa_common::{RsaSignaturePadding, parse_public_key, raw_verify, verification_result};

/// Verifies an RSA signature.
#[mcp_tool(
    name = "rsa_verify",
    title = "RSA Verify",
    description = "Verifies an RSA signature over binary data using PKCS#1 v1.5, PSS (both SHA-256), or raw (no padding) mode. Returns \"true\" or \"false\"."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RsaVerifyTool {
    /// Message (base64-encoded on the wire).
    /// Raw mode expects a fully encoded EMSA block of exactly the modulus byte length.
    pub data: BinaryData,

    /// Signature (base64-encoded on the wire). Raw mode requires exactly the modulus byte length.
    pub signature: BinaryData,

    /// RSA public key in unencrypted PEM format (SPKI or PKCS#1).
    pub public_key: String,

    /// Padding scheme: "pkcs1v15-sha256", "pss-sha256", or "raw" (compares s^e mod n to data).
    pub padding: RsaSignaturePadding,
}

impl RsaVerifyTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let public_key = parse_public_key(&self.public_key)?;

        let valid = match self.padding {
            RsaSignaturePadding::Pkcs1v15Sha256 => verification_result(
                public_key.pkcs1v15_verify::<rsa::sha2::Sha256>(&self.data, &self.signature),
            )?,
            RsaSignaturePadding::PssSha256 => verification_result(
                public_key.pss_verify::<rsa::sha2::Sha256>(&self.data, &self.signature),
            )?,
            RsaSignaturePadding::Raw => raw_verify(&public_key, &self.data, &self.signature)?,
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            if valid { "true" } else { "false" }.to_string(),
        )]))
    }
}
