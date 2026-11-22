use base64::{Engine as _, engine::general_purpose::STANDARD};
use md5::Md5;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

use crate::BinaryData;

/// Hash algorithm used by the `digest` tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum DigestAlgorithm {
    #[serde(rename = "md5")]
    Md5,
    #[serde(rename = "sha1")]
    Sha1,
    #[serde(rename = "sha256")]
    Sha256,
    #[serde(rename = "sha384")]
    Sha384,
    #[serde(rename = "sha512")]
    Sha512,
}

impl DigestAlgorithm {
    pub fn json_schema() -> serde_json::Map<String, serde_json::Value> {
        crate::tools::string_enum_schema(&["md5", "sha1", "sha256", "sha384", "sha512"])
    }
}

/// Computes a hash digest of binary data.
#[mcp_tool(
    name = "digest",
    title = "Binary Digest",
    description = "Computes a hash digest of binary data using MD5, SHA-1, SHA-256, SHA-384, or SHA-512. The digest is returned as base64-encoded binary."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DigestTool {
    /// Binary payload (base64-encoded on the wire).
    pub data: BinaryData,

    /// Hash algorithm: "md5", "sha1", "sha256", "sha384", or "sha512".
    pub algorithm: DigestAlgorithm,
}

impl DigestTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let bytes = self.data.as_slice();

        let digest = match self.algorithm {
            DigestAlgorithm::Md5 => Md5::digest(bytes).to_vec(),
            DigestAlgorithm::Sha1 => Sha1::digest(bytes).to_vec(),
            DigestAlgorithm::Sha256 => Sha256::digest(bytes).to_vec(),
            DigestAlgorithm::Sha384 => Sha384::digest(bytes).to_vec(),
            DigestAlgorithm::Sha512 => Sha512::digest(bytes).to_vec(),
        };

        Ok(CallToolResult::text_content(vec![TextContent::from(
            STANDARD.encode(digest),
        )]))
    }
}
