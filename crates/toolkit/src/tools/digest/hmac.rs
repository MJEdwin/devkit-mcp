use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use md5::Md5;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha384, Sha512};

use crate::BinaryData;

use super::DigestAlgorithm;

macro_rules! compute_hmac {
    ($digest:ty, $key:expr, $data:expr) => {{
        let mut mac = Hmac::<$digest>::new_from_slice($key)
            .map_err(|err| CallToolError::from_message(format!("invalid HMAC key: {err}")))?;
        mac.update($data);
        Ok::<Vec<u8>, CallToolError>(mac.finalize().into_bytes().to_vec())
    }};
}

/// Computes a keyed-hash message authentication code (HMAC).
#[mcp_tool(
    name = "hmac",
    title = "HMAC",
    description = "Computes an HMAC (keyed-hash message authentication code) over binary data using MD5, SHA-1, SHA-256, SHA-384, or SHA-512. The message and key are base64-encoded binary; the MAC is returned as base64-encoded binary."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct HmacTool {
    /// Message (base64-encoded on the wire).
    pub data: BinaryData,

    /// Secret key (base64-encoded on the wire). Any length is accepted.
    pub key: BinaryData,

    /// Hash algorithm: "md5", "sha1", "sha256", "sha384", or "sha512".
    pub algorithm: DigestAlgorithm,
}

impl HmacTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let data = self.data.as_slice();
        let key = self.key.as_slice();

        let mac = match self.algorithm {
            DigestAlgorithm::Md5 => compute_hmac!(Md5, key, data)?,
            DigestAlgorithm::Sha1 => compute_hmac!(Sha1, key, data)?,
            DigestAlgorithm::Sha256 => compute_hmac!(Sha256, key, data)?,
            DigestAlgorithm::Sha384 => compute_hmac!(Sha384, key, data)?,
            DigestAlgorithm::Sha512 => compute_hmac!(Sha512, key, data)?,
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(mac),
        )]))
    }
}
