use rsa::pkcs8::{EncodePublicKey, LineEnding};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};

use super::rsa_common::parse_private_key;

/// Derives an RSA public key from a private key.
#[mcp_tool(
    name = "rsa_private_to_public",
    title = "RSA Private Key to Public Key",
    description = "Derives the RSA public key (SPKI PEM) from an unencrypted private key PEM (PKCS#8 or PKCS#1)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct RsaPrivateToPublicTool {
    /// RSA private key in unencrypted PEM format (PKCS#8 or PKCS#1).
    pub private_key: String,
}

impl RsaPrivateToPublicTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let private_key = parse_private_key(&self.private_key)?;
        let public_key = private_key.to_public_key();

        let pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|err| {
                CallToolError::from_message(format!("failed to encode public key: {err}"))
            })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(pem)]))
    }
}
