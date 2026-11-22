//! Tool library shared by the `cli` (stdio) and `api` (HTTP) MCP servers.

pub mod binary;
pub mod datetime;
pub mod handler;
pub mod tools;

pub use binary::BinaryData;
pub use datetime::UtcDateTime;
pub use handler::ToolkitHandler;
pub use tools::*;

use rust_mcp_sdk::ServerDetails;
use rust_mcp_sdk::schema::{Implementation, ServerCapabilities, ServerCapabilitiesTools};

/// Server name reported to MCP clients.
pub const SERVER_NAME: &str = "devkit-mcp";

/// Build the server metadata advertised by every transport.
pub fn server_details(transport: &str) -> ServerDetails {
    ServerDetails {
        server_info: Implementation {
            name: SERVER_NAME.into(),
            version: env!("CARGO_PKG_VERSION").into(),
            title: Some("Devkit MCP".into()),
            description: Some(format!(
                "Toolkit MCP server exposing shared tools over {transport}."
            )),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        instructions: Some(
            "Shared devkit tools: generate_uuid, current_time, unix_to_iso8601, iso8601_to_unix, url_encode, url_decode, binary_to_hex, hex_to_binary, binary_to_string, string_to_binary, digest, hmac, rsa_encrypt, rsa_decrypt, rsa_sign, rsa_verify, rsa_generate_key, rsa_private_to_public, rsa_convert_private_key, rsa_key_components, aes_encrypt, aes_decrypt, aes_generate_key, aes_generate_iv, des_encrypt, des_decrypt, des_generate_key, des_generate_iv."
                .into(),
        ),
        meta: None,
    }
}
