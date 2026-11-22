//! Unified MCP server entry point.
//!
//! - `local`  -> JSON-RPC over stdin/stdout
//! - `remote` -> streamable HTTP

mod auth;

use std::io::IsTerminal;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use rust_mcp_axum::{AxumServerOptions, create_axum_server};
use rust_mcp_sdk::{
    McpObserver, McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
    auth::AuthProvider,
    error::SdkResult,
    mcp_server::{McpServerOptions, server_runtime},
    schema::{ClientMessage, McpMessage, RequestId, RpcMessage, ServerMessage},
};
use toolkit::{ToolkitHandler, server_details};
use tracing_subscriber::EnvFilter;

use crate::auth::ApiKeyAuthProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Transport {
    /// Serve MCP over stdin/stdout.
    Local,
    /// Serve MCP over streamable HTTP.
    Remote,
}

#[derive(Debug, Parser)]
#[command(name = "devkit-mcp", version, about = "devkit MCP server")]
struct Cli {
    /// Transport to use.
    transport: Transport,

    /// Host to bind in `remote` mode.
    #[arg(long, env = "MCP_HOST", default_value = "127.0.0.1")]
    host: String,

    /// Port to bind in `remote` mode.
    #[arg(long, env = "MCP_PORT", default_value_t = 8080)]
    port: u16,

    /// API key for `remote` mode. If unset, no authentication is required.
    #[arg(long, env = "MCP_API_KEY")]
    api_key: Option<String>,
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    let cli = Cli::parse();

    match cli.transport {
        Transport::Local => run_stdio().await,
        Transport::Remote => {
            init_tracing();
            run_http(&cli.host, cli.port, cli.api_key).await
        }
    }
}

/// Installs a stderr tracing subscriber. `RUST_LOG` controls the level;
/// defaults to `info`. Use `RUST_LOG=trace` to also see full MCP payloads.
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .init();
}

/// Logs every MCP message that passes through the HTTP transport.
struct TracingServerObserver;

impl McpObserver<ClientMessage, ServerMessage> for TracingServerObserver {
    fn on_receive(&self, message: &ClientMessage) {
        log_message(
            "recv",
            &message.message_type().to_string(),
            message.method(),
            message.request_id(),
            message,
        );
    }

    fn on_send(&self, message: &ServerMessage) {
        log_message(
            "send",
            &message.message_type().to_string(),
            message.method(),
            message.request_id(),
            message,
        );
    }
}

fn log_message<T: serde::Serialize + ?Sized>(
    direction: &'static str,
    message_type: &str,
    method: Option<&str>,
    id: Option<&RequestId>,
    message: &T,
) {
    let id = id.map(ToString::to_string);

    tracing::info!(
        direction,
        message_type = %message_type,
        method = method.unwrap_or("-"),
        id = id.as_deref().unwrap_or("-"),
        "MCP message",
    );

    tracing::trace!(
        direction,
        payload = %serde_json::to_string(message)
            .unwrap_or_else(|err| format!("<serialize error: {err}>")),
        "MCP payload",
    );
}

async fn run_stdio() -> SdkResult<()> {
    // stdout is reserved for MCP messages; never print logs there.
    let transport = StdioTransport::new(TransportOptions::default())?;

    let server = server_runtime::create_server(McpServerOptions {
        server_details: server_details("stdio"),
        transport,
        handler: ToolkitHandler.to_mcp_server_handler(),
        message_observer: None,
    });

    server.start().await
}

async fn run_http(host: &str, port: u16, api_key: Option<String>) -> SdkResult<()> {
    tracing::info!(host, port, "starting MCP HTTP server");

    let auth: Option<Arc<dyn AuthProvider>> = match api_key {
        Some(key) if !key.is_empty() => {
            tracing::info!("API key authentication enabled");
            Some(Arc::new(ApiKeyAuthProvider::new(key)))
        }
        Some(_) => {
            tracing::warn!("MCP_API_KEY is empty; API key authentication disabled");
            None
        }
        None => None,
    };

    let server = create_axum_server(
        server_details("streamable HTTP"),
        ToolkitHandler.to_mcp_server_handler(),
        AxumServerOptions {
            host: host.to_string(),
            port,
            // MCP endpoint is /mcp by default; /health is a plain GET.
            health_endpoint: Some("/health".into()),
            message_observer: Some(Arc::new(TracingServerObserver)),
            auth,
            ..Default::default()
        },
    );

    server.start().await
}
