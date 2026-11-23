# AGENTS.md

Guidelines for coding agents and contributors working on `devkit-mcp`.

## Project

Rust workspace with two crates:

- `crates/toolkit` — tool implementations plus the shared `ToolkitHandler`
  and server metadata (`server_details`).
- `crates/cli` — the server binary. `local` = stdio, `remote` = Streamable
  HTTP, with `--host` / `--port`.

Tools live in `crates/toolkit/src/tools/<category>/`, one file per tool.
Categories:

- `time` — current time, timestamp, lunar/solar date conversion
- `cron` — cron expression validation, scheduling, parsing, and generation
- `encoding` — URL / binary / text encodings, case conversion, JWT decoding
- `network` — IP / CIDR utilities
- `regex` — regular expression matching and replacement
- `json` — JSON formatting, querying, and validation
- `digest` — hashes and HMAC
- `symmetric` — symmetric encryption (AES, DES/3DES)
- `rsa` — RSA keys, encryption, signatures
- `misc` — other generators (UUID, random values, random primes)

## Common commands

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build

cargo run -p cli -- local
cargo run -p cli -- remote --host 127.0.0.1 --port 8080
RUST_LOG=trace cargo run -p cli -- remote   # HTTP request/payload tracing

./scripts/build-local.sh                    # release binary
./scripts/docker-build.sh                   # Docker image (HTTP mode)
```

## Testing

Every tool **must** have tests in `crates/toolkit/tests/<category>.rs`.
A tool change is not complete until `cargo test --workspace` passes and
`cargo clippy --workspace --all-targets -- -D warnings` is clean.

Test layout:

```
crates/toolkit/tests/
├── common/mod.rs   # shared helpers (text, base64, RSA test keys)
├── time.rs
├── cron.rs
├── encoding.rs
├── network.rs
├── regex.rs
├── json.rs
├── digest.rs
├── symmetric.rs
├── rsa.rs
└── misc.rs
```

Conventions:

- Call each tool's `call_tool()` directly; no transport needed.
- Cover at least one success case per tool, plus error/edge cases where
  applicable (invalid input, wrong key size, bad padding, failed verification).
- Prefer deterministic test vectors (known hashes, AES/NIST vectors) where
  possible; use round-trips for randomized schemes.
- RSA tests should share one 1024-bit key via
  `common::rsa_test_keys()`; key generation is slow.
- Add new test files under the matching category name.

## Linting

`cargo clippy --workspace --all-targets -- -D warnings` must finish with no
warnings. Fix warnings rather than suppressing them with `#[allow(...)]`
unless there is a documented reason.

Quick stdio smoke test (optional manual check):

```bash
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/clientCapabilities":{},"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}' \
  | ./target/debug/devkit-mcp local
```

## Adding, changing, or removing a tool

A tool change is **not complete** until the docs are updated.

1. Add or edit `crates/toolkit/src/tools/<category>/<tool_name>.rs` using the
   closest category (`time`, `encoding`, `digest`, `rsa`, `misc`).
2. Register it in `crates/toolkit/src/tools/<category>/mod.rs`:
   - `mod <tool_name>;`
   - `pub use <tool_name>::<ToolType>;`
3. Add the tool type to `tool_box!(ToolkitTools, [...])` in
   `crates/toolkit/src/tools/mod.rs`.
4. Add the dispatch arm in `crates/toolkit/src/handler.rs`.
5. Update the tool list in the `server_details` instructions in
   `crates/toolkit/src/lib.rs`.
6. **Add tests (required):** extend
   `crates/toolkit/tests/<category>.rs` with at least one success case and
   relevant error/edge cases for the tool.
7. **Update documentation (required):**
   - add/update the tool section under the matching category in
     `docs/tools.md` (parameters, defaults, behaviour, errors, examples);
   - update the matching category table in `README.md`.
8. If the tool returns `structuredContent`, implement
   `output_schema()` + `output_schema_provider()` in the tool file,
   re-export the provider from the category's `mod.rs`, and add it to
   `OUTPUT_SCHEMA_PROVIDERS` in `crates/toolkit/src/tools/mod.rs`.
9. Run `cargo fmt --all`, `cargo check --workspace`,
   `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, and a smoke test. **All checks must pass.**

When removing a tool, do the inverse and delete its tests, docs section, and
README row.

## Tool conventions

- Put each tool in its own file under `src/tools/<category>/`.
- Every tool must have tests under `tests/<category>.rs`.
- Use `#[mcp_tool(name, title, description)]` and derive
  `Deserialize`, `Serialize`, `JsonSchema`.
- Constrain acceptable values with enums; do not accept free-form strings for
  a fixed set of options.
- For string enums, implement `json_schema()` manually with
  `crate::tools::string_enum_schema(&[...])` so clients see a single `enum`
  array instead of `oneOf`.
- Optional parameters: use `Option<T>` with
  `#[json_schema(default = ...)]`, and apply the same default with
  `unwrap_or(...)` in code.
- Binary payloads: use `crate::BinaryData` for tool fields. It wraps
  `Vec<u8>`, serializes/deserializes as a base64 string, and its JSON schema is
  `{"type":"string","format":"binary"}`. Methods should use the bytes directly
  (`self.field.as_slice()` or deref); do not base64-decode inside the method.
  When returning binary data as text, encode it with `STANDARD.encode(...)`.
- Datetimes: use `crate::UtcDateTime` for tool fields. It deserializes RFC 3339
  strings (timezone offset required) into UTC, serializes as RFC 3339 `Z`, and
  its JSON schema is `{"type":"string","format":"date-time"}`. Methods use it
  directly; do not parse datetimes inside the method.
- CPU-heavy blocking work (e.g., RSA key generation) must not run directly on
  the async executor. Use an async wrapper with
  `tokio::task::spawn_blocking` and call it from `handler.rs`; keep the
  synchronous `call_tool()` for tests.
- Return user-facing failures with `CallToolError::from_message(...)`.
- Keep descriptions accurate: they are what MCP clients display.

## MCP protocol notes

- `rust-mcp-sdk` 2.0 implements **MCP 2026-07-28 only**.
- **Stateless: yes** — no `initialize` handshake, no session state, and no
  `Mcp-Session-Id` header; every POST is independent. Every request needs
  `params._meta` with `io.modelcontextprotocol/protocolVersion` and
  `io.modelcontextprotocol/clientCapabilities`.
- Optional API-key auth for `remote` mode: configure `--api-key` or
  `MCP_API_KEY`. When set, requests must send
  `Authorization: Bearer <key>`; when unset or empty, auth is disabled.
- HTTP requires `Mcp-Method` (and `Mcp-Name` for `tools/call`) headers,
  `Accept: application/json, text/event-stream`, and POST-only endpoints.
- Older 2025 clients that require `initialize` cannot connect.
