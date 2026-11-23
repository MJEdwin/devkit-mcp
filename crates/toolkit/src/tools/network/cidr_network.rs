use std::net::Ipv4Addr;
use std::str::FromStr;

use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{
    CallToolResult, TextContent, ToolOutputSchema, schema_utils::CallToolError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

/// Calculates the network for an IPv4 address and subnet mask.
#[mcp_tool(
    name = "cidr_network",
    title = "CIDR Network Calculator",
    description = "Calculates the network address, broadcast address, usable host range, and host counts for an IPv4 address and subnet mask (dotted or prefix length)."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CidrNetworkTool {
    /// IPv4 address, e.g. "192.168.1.10".
    pub ip: String,

    /// Subnet mask: dotted (e.g. "255.255.255.0") or prefix length (e.g. "24" or "/24").
    pub mask: String,
}

impl CidrNetworkTool {
    pub fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let ip = Ipv4Addr::from_str(&self.ip).map_err(|err| {
            CallToolError::from_message(format!("invalid IPv4 address '{}': {err}", self.ip))
        })?;

        let prefix = parse_prefix(&self.mask)?;
        let mask_bits = prefix_bits(prefix);

        let ip_bits = u32::from(ip);
        let network_bits = ip_bits & mask_bits;
        let broadcast_bits = network_bits | !mask_bits;

        let address_count = 1u64 << (32 - prefix);
        let (first_host_bits, last_host_bits, usable_hosts) = match prefix {
            32 => (network_bits, network_bits, 1),
            31 => (network_bits, broadcast_bits, 2),
            _ => (network_bits + 1, broadcast_bits - 1, address_count - 2),
        };

        let mask = Ipv4Addr::from(mask_bits);
        let network = Ipv4Addr::from(network_bits);
        let broadcast = Ipv4Addr::from(broadcast_bits);

        let mut structured = Map::new();
        structured.insert("ip".to_string(), Value::String(ip.to_string()));
        structured.insert("mask".to_string(), Value::String(mask.to_string()));
        structured.insert(
            "prefix_length".to_string(),
            Value::Number(u64::from(prefix).into()),
        );
        structured.insert("network".to_string(), Value::String(network.to_string()));
        structured.insert(
            "broadcast".to_string(),
            Value::String(broadcast.to_string()),
        );
        structured.insert(
            "first_host".to_string(),
            Value::String(Ipv4Addr::from(first_host_bits).to_string()),
        );
        structured.insert(
            "last_host".to_string(),
            Value::String(Ipv4Addr::from(last_host_bits).to_string()),
        );
        structured.insert(
            "address_count".to_string(),
            Value::Number(address_count.into()),
        );
        structured.insert(
            "usable_hosts".to_string(),
            Value::Number(usable_hosts.into()),
        );
        structured.insert(
            "cidr".to_string(),
            Value::String(format!("{network}/{prefix}")),
        );

        let text =
            serde_json::to_string_pretty(&Value::Object(structured.clone())).map_err(|err| {
                CallToolError::from_message(format!("failed to serialize result: {err}"))
            })?;

        Ok(CallToolResult::text_content(vec![TextContent::from(text)])
            .with_structured_content(structured))
    }

    pub(crate) fn output_schema() -> ToolOutputSchema {
        crate::tools::object_output_schema(json!({
            "type": "object",
            "properties": {
                "ip": { "type": "string", "description": "Input IPv4 address." },
                "mask": { "type": "string", "description": "Normalized dotted subnet mask." },
                "prefix_length": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 32,
                    "description": "CIDR prefix length."
                },
                "network": { "type": "string", "description": "Network address." },
                "broadcast": { "type": "string", "description": "Broadcast address." },
                "first_host": { "type": "string", "description": "First usable host." },
                "last_host": { "type": "string", "description": "Last usable host." },
                "address_count": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Total addresses in the network."
                },
                "usable_hosts": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Usable host count (/31 and /32 follow RFC 3021 and host-route semantics)."
                },
                "cidr": { "type": "string", "description": "Network in CIDR notation." }
            },
            "required": [
                "ip",
                "mask",
                "prefix_length",
                "network",
                "broadcast",
                "first_host",
                "last_host",
                "address_count",
                "usable_hosts",
                "cidr"
            ]
        }))
    }
}

fn parse_prefix(mask: &str) -> Result<u8, CallToolError> {
    let mask = mask.trim();
    let mask = mask.strip_prefix('/').unwrap_or(mask);

    if let Ok(prefix) = mask.parse::<u8>() {
        if prefix <= 32 {
            return Ok(prefix);
        }

        return Err(CallToolError::from_message(format!(
            "prefix length must be between 0 and 32, got {prefix}"
        )));
    }

    let dotted = Ipv4Addr::from_str(mask).map_err(|_| {
        CallToolError::from_message(
            "mask must be a dotted IPv4 mask (e.g. 255.255.255.0) or a prefix length 0-32",
        )
    })?;

    let bits = u32::from(dotted);
    let prefix = bits.leading_ones() as u8;
    let expected = prefix_bits(prefix);

    if bits != expected {
        return Err(CallToolError::from_message(format!(
            "invalid subnet mask (bits must be contiguous): {dotted}"
        )));
    }

    Ok(prefix)
}

fn prefix_bits(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}

pub(crate) fn output_schema_provider(name: &str) -> Option<ToolOutputSchema> {
    if name == CidrNetworkTool::tool_name() {
        Some(CidrNetworkTool::output_schema())
    } else {
        None
    }
}
