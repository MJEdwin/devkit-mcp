# syntax=docker/dockerfile:1

FROM rust:1.100-bookworm AS builder

# aws-lc-rs (pulled in by rust-mcp-sdk auth) needs CMake to build.
RUN apt-get update \
    && apt-get install -y --no-install-recommends cmake \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release -p cli

FROM debian:bookworm-slim

COPY --from=builder /app/target/release/devkit-mcp /usr/local/bin/devkit-mcp

ENV RUST_LOG=info
ENV MCP_HOST=0.0.0.0
ENV MCP_PORT=8080

EXPOSE 8080

USER 10001:10001

ENTRYPOINT ["/usr/local/bin/devkit-mcp"]
CMD ["remote"]
