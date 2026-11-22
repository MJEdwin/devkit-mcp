#![allow(dead_code)]

use rust_mcp_sdk::schema::{CallToolResult, ContentBlock};
use serde_json::Value;
use std::sync::OnceLock;
use toolkit::{BinaryData, RsaConvertPrivateKeyTool, RsaGenerateKeyTool, RsaPrivateKeyFormat};

pub fn text(result: &CallToolResult) -> &str {
    result
        .content
        .iter()
        .find_map(|block| match block {
            ContentBlock::TextContent(text) => Some(text.text.as_str()),
            _ => None,
        })
        .expect("expected a text content block")
}

pub fn structured(result: &CallToolResult) -> &Value {
    result
        .structured_content
        .as_ref()
        .expect("expected structuredContent")
}

pub fn base64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub fn bin(bytes: &[u8]) -> BinaryData {
    BinaryData::from(bytes)
}

pub fn unbase64(text: &str) -> Vec<u8> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(text)
        .expect("valid base64")
}

pub fn from_hex(text: &str) -> Vec<u8> {
    hex_simd::decode_to_vec(text).expect("valid hex")
}

pub struct TestKeys {
    pub pkcs8: String,
    pub pkcs1: String,
    pub public: String,
}

/// One 1024-bit RSA key pair shared by all tests in a binary (keygen is slow).
pub fn rsa_test_keys() -> &'static TestKeys {
    static KEYS: OnceLock<TestKeys> = OnceLock::new();

    KEYS.get_or_init(|| {
        let generated = RsaGenerateKeyTool {
            bits: Some(1024),
            format: Some(RsaPrivateKeyFormat::Pkcs8),
        }
        .call_tool()
        .expect("RSA key generation");

        let structured = structured(&generated);
        let pkcs8 = structured["private_key"]
            .as_str()
            .expect("private_key")
            .to_string();
        let public = structured["public_key"]
            .as_str()
            .expect("public_key")
            .to_string();

        let converted = RsaConvertPrivateKeyTool {
            private_key: pkcs8.clone(),
            format: RsaPrivateKeyFormat::Pkcs1,
        }
        .call_tool()
        .expect("PKCS#1 conversion");
        let pkcs1 = text(&converted).to_string();

        TestKeys {
            pkcs8,
            pkcs1,
            public,
        }
    })
}
