# devkit-mcp

A small [MCP](https://modelcontextprotocol.io/) server exposing a shared Rust
toolkit over **stdio** and **Streamable HTTP**.

## Workspace layout

```
.
├── Cargo.toml                 # workspace root (shared dependencies)
├── crates/
│   ├── toolkit/               # tool library: tools + shared MCP handler
│   │   └── src/
│   │       ├── lib.rs         # server metadata / instructions
│   │       ├── handler.rs     # ToolkitHandler (shared by both transports)
│   │       └── tools/         # tool modules grouped by category
│   │           ├── time/
│   │           ├── encoding/
│   │           ├── digest/
│   │           ├── symmetric/
│   │           ├── rsa/
│   │           └── misc/
│   └── cli/                   # MCP server binary
│       └── src/main.rs        # `local` (stdio) / `remote` (HTTP)
├── docs/tools.md              # full tool reference
├── LICENSE                    # Apache License 2.0
└── AGENTS.md                  # contributor / coding-agent guidelines
```

## Quick start

```bash
# stdio transport (for MCP clients that spawn a process)
cargo run -p cli -- local

# Streamable HTTP transport (default: 127.0.0.1:8080)
cargo run -p cli -- remote
cargo run -p cli -- remote --host 0.0.0.0 --port 9000

# equivalent via environment variables
MCP_HOST=0.0.0.0 MCP_PORT=9000 cargo run -p cli -- remote
```

`--host` / `--port` take precedence over `MCP_HOST` / `MCP_PORT`.

HTTP endpoints:

| Endpoint | Description |
|---|---|
| `POST /mcp` | MCP JSON-RPC endpoint |
| `GET /health` | health check |

### API key authentication

`remote` mode can require a static API key:

```bash
cargo run -p cli -- remote --api-key secret
# equivalent via environment variable
MCP_API_KEY=secret cargo run -p cli -- remote
```

When a key is configured, MCP requests must send:

```http
Authorization: Bearer secret
```

If `--api-key` / `MCP_API_KEY` is unset or empty, authentication is disabled.
`GET /health` stays public. The key is never written to logs.

Enable HTTP request logs with tracing:

```bash
RUST_LOG=info  cargo run -p cli -- remote          # request summaries
RUST_LOG=trace cargo run -p cli -- remote          # full JSON payloads
```

## Connecting a client

Claude Code:

```bash
claude mcp add --transport http devkit-mcp http://127.0.0.1:8080/mcp
claude mcp list
```

`.mcp.json`:

```json
{
  "mcpServers": {
    "devkit-mcp": {
      "type": "http",
      "url": "http://127.0.0.1:8080/mcp"
    }
  }
}
```

For stdio clients, point the command at the built binary:

```bash
cargo build
# command: /path/to/devkit-mcp/target/debug/devkit-mcp
# args:    local
```

### Protocol notes

This server uses `rust-mcp-sdk` 2.0 and implements **MCP 2026-07-28 only**:

- **Stateless: yes** — no `initialize` handshake, no session state, and no
  `Mcp-Session-Id` header. Every POST is handled independently.
- every request carries `params._meta` with
  `io.modelcontextprotocol/protocolVersion` and
  `io.modelcontextprotocol/clientCapabilities`
- HTTP requests must also send `Mcp-Method` (and `Mcp-Name` for `tools/call`)
  plus `Accept: application/json, text/event-stream`
- the HTTP endpoint is POST-only; GET/DELETE return `405`

Clients that only speak the 2025 `initialize` flow cannot connect.

## Tools

### Time

| Tool | Description |
|---|---|
| `current_time` | Current UTC time in ISO 8601 |
| `unix_to_iso8601` | Unix timestamp → ISO 8601 UTC |
| `iso8601_to_unix` | ISO 8601 datetime (offset required) → Unix timestamp |

### Encoding

| Tool | Description |
|---|---|
| `url_encode` | Percent-encode using query or form rules |
| `url_decode` | Percent-decode using query or form rules |
| `binary_to_hex` | Base64 binary → hex string (optional uppercase) |
| `hex_to_binary` | Hex string → base64 binary |
| `binary_to_string` | Base64 binary → text (utf-8 / ascii / utf-16 / utf-32 / gbk / gb18030) |
| `string_to_binary` | Text → base64 binary (same encodings) |

### Digest

| Tool | Description |
|---|---|
| `digest` | Hash binary data with MD5 / SHA-1 / SHA-256 / SHA-384 / SHA-512 |
| `hmac` | Keyed HMAC over binary data (same hash algorithms) |

### Symmetric

| Tool | Description |
|---|---|
| `aes_generate_key` | Generate a random AES key (128 / 192 / 256 bits) |
| `aes_generate_iv` | Generate a random AES IV/nonce for the selected mode |
| `aes_encrypt` | AES encrypt (ECB/CBC/CTR/CFB/OFB/GCM; PKCS#7 or no padding) |
| `aes_decrypt` | AES decrypt (same modes and padding options) |
| `des_generate_key` | Generate a random DES/3DES key (8 / 16 / 24 bytes) |
| `des_generate_iv` | Generate a random 8-byte DES/3DES IV |
| `des_encrypt` | DES/3DES encrypt (ECB/CBC/CTR/CFB/OFB; PKCS#7 or no padding) |
| `des_decrypt` | DES/3DES decrypt (same modes and padding options) |

### RSA

| Tool | Description |
|---|---|
| `rsa_encrypt` | RSA encrypt binary data (PKCS#1 v1.5 / OAEP-SHA256 / raw) |
| `rsa_decrypt` | RSA decrypt binary data |
| `rsa_sign` | RSA sign binary data (PKCS#1 v1.5 / PSS with SHA-256, or raw) |
| `rsa_verify` | RSA verify a signature |
| `rsa_generate_key` | Generate an RSA key pair (PKCS#1 or PKCS#8 private, SPKI public) |
| `rsa_private_to_public` | Derive the RSA public key from a private key |
| `rsa_convert_private_key` | Convert a private key between PKCS#1 and PKCS#8 |
| `rsa_key_components` | Extract RSA key components (n, e, d, p, q, dp, dq, qinv) as hex |

### Misc

| Tool | Description |
|---|---|
| `generate_uuid` | Generate a UUID v4 or v7, with optional uppercase / hyphen control |

Full parameter tables and examples: [`docs/tools.md`](docs/tools.md).

## Build scripts

```bash
./scripts/build-local.sh
# -> target/release/devkit-mcp

./scripts/docker-build.sh          # build image
./scripts/docker-build.sh --run    # build and run the container
```

Overrides:

```bash
IMAGE=my-registry/devkit-mcp:1.0 PORT=9000 ./scripts/docker-build.sh
```

The Docker image starts the server in HTTP MCP mode and reads `MCP_HOST` /
`MCP_PORT` (defaults: `0.0.0.0:8080`). The script's `PORT` is used for both the
container port and the published host port:

```bash
docker run --rm -e MCP_PORT=9000 -p 9000:9000 devkit-mcp:latest
# with API key auth
docker run --rm -e MCP_PORT=9000 -e MCP_API_KEY=secret -p 9000:9000 devkit-mcp:latest
```

with:

- `POST http://127.0.0.1:<PORT>/mcp`
- `GET  http://127.0.0.1:<PORT>/health`

## Development

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build
```

When adding or changing a tool, follow [`AGENTS.md`](AGENTS.md) — in particular,
**update `docs/tools.md` and the tool table above before considering the change
complete**.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
