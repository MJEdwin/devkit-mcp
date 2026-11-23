[English](README.md) | **简体中文**

# devkit-mcp

一个小型 [MCP](https://modelcontextprotocol.io/) 服务器，通过 **stdio** 和
**Streamable HTTP** 提供共享的 Rust 工具集。

## 工作区结构

```text
.
├── Cargo.toml                 # workspace 根配置（共享依赖）
├── crates/
│   ├── toolkit/               # 工具库：工具 + 共享 MCP handler
│   │   └── src/
│   │       ├── lib.rs         # 服务器元数据 / instructions
│   │       ├── handler.rs     # ToolkitHandler（两种传输方式共用）
│   │       └── tools/         # 按分类组织的工具模块
│   │           ├── time/
│   │           ├── cron/
│   │           ├── encoding/
│   │           ├── regex/
│   │           ├── json/
│   │           ├── digest/
│   │           ├── symmetric/
│   │           ├── rsa/
│   │           └── misc/
│   └── cli/                   # MCP 服务器二进制
│       └── src/main.rs        # `local`（stdio）/ `remote`（HTTP）
├── docs/tools.md              # 完整工具参考
├── LICENSE                    # Apache License 2.0
└── AGENTS.md                  # 贡献者 / 编码代理指南
```

## 快速开始

```bash
# stdio 传输（由 MCP 客户端启动进程）
cargo run -p cli -- local

# Streamable HTTP 传输（默认：127.0.0.1:8080）
cargo run -p cli -- remote
cargo run -p cli -- remote --host 0.0.0.0 --port 9000

# 通过环境变量设置
MCP_HOST=0.0.0.0 MCP_PORT=9000 cargo run -p cli -- remote
```

`--host` / `--port` 的优先级高于 `MCP_HOST` / `MCP_PORT`。

HTTP 端点：

| 端点 | 说明 |
|---|---|
| `POST /mcp` | MCP JSON-RPC 端点 |
| `GET /health` | 健康检查 |

### API Key 认证

`remote` 模式可以要求静态 API Key：

```bash
cargo run -p cli -- remote --api-key secret
# 通过环境变量设置
MCP_API_KEY=secret cargo run -p cli -- remote
```

配置 Key 后，MCP 请求必须携带：

```http
Authorization: Bearer secret
```

如果 `--api-key` / `MCP_API_KEY` 未设置或为空，则关闭认证。
`GET /health` 始终公开。Key 不会写入日志。

开启 HTTP 请求日志：

```bash
RUST_LOG=info  cargo run -p cli -- remote          # 请求摘要
RUST_LOG=trace cargo run -p cli -- remote          # 完整 JSON payload
```

## 连接本地客户端

先构建二进制，然后在客户端配置中使用它的**绝对路径**：

```bash
cargo build --release
# 或：./scripts/build-local.sh
# 二进制：/absolute/path/to/devkit-mcp/target/release/devkit-mcp
```

### stdio（本地客户端推荐）

客户端启动服务器，并通过 stdin/stdout 通信。

Claude Code：

```bash
claude mcp add devkit-mcp -- \
  /absolute/path/to/devkit-mcp/target/release/devkit-mcp local
claude mcp list
```

项目 `.mcp.json`（Claude Code）：

```json
{
  "mcpServers": {
    "devkit-mcp": {
      "type": "stdio",
      "command": "/absolute/path/to/devkit-mcp/target/release/devkit-mcp",
      "args": ["local"]
    }
  }
}
```

Claude Desktop（`claude_desktop_config.json`）：

- macOS：`~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows：`%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "devkit-mcp": {
      "command": "/absolute/path/to/devkit-mcp/target/release/devkit-mcp",
      "args": ["local"]
    }
  }
}
```

Windows 上请使用 `...\target\release\devkit-mcp.exe`，并在 JSON 中转义反斜杠
（例如 `C:\\tools\\devkit-mcp.exe`）。其他 stdio 客户端也可以使用同样的
`command` / `args` 配置。

开发时也可以让客户端直接运行 Cargo（需要客户端支持 `cwd`）：

```json
{
  "mcpServers": {
    "devkit-mcp-dev": {
      "type": "stdio",
      "command": "cargo",
      "args": ["run", "-p", "cli", "--", "local"],
      "cwd": "/absolute/path/to/devkit-mcp"
    }
  }
}
```

### Streamable HTTP（本地服务器）

在终端启动服务器：

```bash
./target/release/devkit-mcp remote --host 127.0.0.1 --port 8080
```

Claude Code：

```bash
claude mcp add --transport http devkit-mcp http://127.0.0.1:8080/mcp
claude mcp list
```

项目 `.mcp.json`：

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

如果设置了 `--api-key` / `MCP_API_KEY`，需要添加认证头：

```json
{
  "mcpServers": {
    "devkit-mcp": {
      "type": "http",
      "url": "http://127.0.0.1:8080/mcp",
      "headers": {
        "Authorization": "Bearer <your-key>"
      }
    }
  }
}
```

> 客户端必须支持 **MCP 2026-07-28**。服务器是无状态的，没有 `initialize`
> 握手；只支持 2025 流程的客户端无法连接。

### 协议说明

本服务器使用 `rust-mcp-sdk` 2.0，仅实现 **MCP 2026-07-28**：

- **无状态**：没有 `initialize` 握手、没有会话状态、不使用
  `Mcp-Session-Id` 头；每个 POST 请求独立处理。
- 每个请求都携带 `params._meta`，包含
  `io.modelcontextprotocol/protocolVersion` 和
  `io.modelcontextprotocol/clientCapabilities`。
- HTTP 请求还必须发送 `Mcp-Method`（`tools/call` 还需要 `Mcp-Name`），以及
  `Accept: application/json, text/event-stream`。
- HTTP 端点仅支持 POST；GET/DELETE 返回 `405`。

只支持 2025 `initialize` 流程的客户端无法连接。

## 工具

### 时间

| 工具 | 说明 |
|---|---|
| `current_time` | 当前 UTC 时间（ISO 8601） |
| `unix_to_iso8601` | Unix 时间戳 → ISO 8601 UTC |
| `iso8601_to_unix` | ISO 8601 时间（需带偏移）→ Unix 时间戳 |
| `solar_to_lunar_date` | 阳历日期 → 农历日期、干支年、生肖 |
| `lunar_to_solar_date` | 农历日期（年/月/日、闰月标记）→ 阳历日期 |

### Cron

| 工具 | 说明 |
|---|---|
| `cron_validate` | 校验 cron 表达式并描述调度 |
| `cron_next` | 用固定 UTC 偏移计算接下来 N 次执行时间 |
| `cron_prev` | 用固定 UTC 偏移计算之前 N 次执行时间 |
| `cron_generate` | 根据各字段值生成 cron 表达式 |
| `cron_parse` | 把 cron 表达式解析为字段和描述 |

### 编码

| 工具 | 说明 |
|---|---|
| `url_encode` | 按 query 或 form 规则进行百分号编码 |
| `url_decode` | 按 query 或 form 规则进行百分号解码 |
| `binary_to_hex` | base64 二进制 → 十六进制字符串（可选大写） |
| `hex_to_binary` | 十六进制字符串 → base64 二进制 |
| `binary_to_string` | base64 二进制 → 文本（utf-8 / ascii / utf-16 / utf-32 / gbk / gb18030） |
| `string_to_binary` | 文本 → base64 二进制（同上编码） |
| `change_case` | 小写 / 大写 / 标题格式转换 |
| `naming_case` | PascalCase / camelCase / lower_snake / UPPER_SNAKE 转换 |

### 正则

| 工具 | 说明 |
|---|---|
| `regex_match` | 查找正则匹配，返回捕获组和字节偏移 |
| `regex_replace` | 替换正则匹配，支持 `$1` / `${name}` |
| `regex_test` | 测试正则并返回第一个匹配 |

### JSON

| 工具 | 说明 |
|---|---|
| `json_pretty` | 格式化 JSON，支持缩进和 `\uXXXX` 解码/转义 |
| `json_minify` | 压缩 JSON，可选转义非 ASCII 字符 |
| `json_query` | 使用 JSONPath（RFC 9535）查询 JSON |
| `json_validate` | 校验 JSON 语法并返回行列和原因 |
| `json_schema_validate` | 按 JSON Schema 校验 JSON |

### 摘要

| 工具 | 说明 |
|---|---|
| `digest` | 计算二进制数据的哈希（MD5 / SHA-1 / SHA-256 / SHA-384 / SHA-512） |
| `hmac` | 计算二进制数据的 HMAC（同上算法） |

### 对称加密

| 工具 | 说明 |
|---|---|
| `aes_generate_key` | 生成随机 AES 密钥（128 / 192 / 256 位） |
| `aes_generate_iv` | 为所选模式生成随机 AES IV/nonce |
| `aes_encrypt` | AES 加密（ECB/CBC/CTR/CFB/OFB/GCM；PKCS#7 或无填充） |
| `aes_decrypt` | AES 解密（同上模式和填充选项） |
| `des_generate_key` | 生成随机 DES/3DES 密钥（8 / 16 / 24 字节） |
| `des_generate_iv` | 生成随机 8 字节 DES/3DES IV |
| `des_encrypt` | DES/3DES 加密（ECB/CBC/CTR/CFB/OFB；PKCS#7 或无填充） |
| `des_decrypt` | DES/3DES 解密（同上模式和填充选项） |

### RSA

| 工具 | 说明 |
|---|---|
| `rsa_encrypt` | 使用 RSA 加密二进制数据（PKCS#1 v1.5 / OAEP-SHA256 / raw） |
| `rsa_decrypt` | 使用 RSA 解密二进制数据 |
| `rsa_sign` | 使用 RSA 对二进制数据签名（PKCS#1 v1.5 / PSS-SHA256 / raw） |
| `rsa_verify` | 验证 RSA 签名 |
| `rsa_generate_key` | 生成 RSA 密钥对（PKCS#1 或 PKCS#8 私钥，SPKI 公钥） |
| `rsa_private_to_public` | 从私钥推导 RSA 公钥 |
| `rsa_convert_private_key` | 在 PKCS#1 和 PKCS#8 私钥格式之间转换 |
| `rsa_key_components` | 提取 RSA 密钥分量（n, e, d, p, q, dp, dq, qinv）为十六进制 |

### 其他

| 工具 | 说明 |
|---|---|
| `generate_uuid` | 生成 UUID v4 或 v7，可选大写 / 移除连字符 |
| `random_number` | 生成随机整数（`decimals: 0`）或浮点数 |
| `random_string` | 按字符集生成随机字符串 |
| `random_prime` | 在闭区间内生成随机质数 |

完整参数表和示例：[`docs/tools.md`](docs/tools.md)（英文）/
[`docs/tools.zh-CN.md`](docs/tools.zh-CN.md)（简体中文）。

## 预编译二进制

推送 `v*` tag 会触发 **Release** workflow，构建发布二进制并附加到 GitHub
Release：

| 平台 | 架构 |
|---|---|
| Linux | x86_64, arm64 |
| macOS | arm64, x86_64 |
| Windows | x86_64, arm64 |

```bash
git tag v0.0.1
git push origin v0.0.1
```

每个压缩包包含 `devkit-mcp` 二进制、`README.md` 和 `LICENSE`。
`SHA256SUMS.txt` 会与压缩包一起发布。也可以手动运行 workflow
（`workflow_dispatch`），只构建产物而不创建 Release。

构建前，workflow 会把 `[workspace.package]` 版本重写为 tag 版本（手动运行时为
`0.0.0-<sha>`），并刷新 `Cargo.lock`，因此二进制上报的版本与 Release tag 一致。
仓库本身不会被修改。

## 构建脚本

```bash
./scripts/build-local.sh
# -> target/release/devkit-mcp

./scripts/docker-build.sh          # 构建镜像
./scripts/docker-build.sh --run    # 构建并运行容器
```

覆盖默认值：

```bash
IMAGE=my-registry/devkit-mcp:1.0 PORT=9000 ./scripts/docker-build.sh
```

Docker 镜像会以 HTTP MCP 模式启动服务器，并读取 `MCP_HOST` / `MCP_PORT`
（默认 `0.0.0.0:8080`）。脚本中的 `PORT` 同时用于容器端口和发布到宿主机的
端口：

```bash
docker run --rm -e MCP_PORT=9000 -p 9000:9000 devkit-mcp:latest
# 使用 API Key 认证
docker run --rm -e MCP_PORT=9000 -e MCP_API_KEY=secret -p 9000:9000 devkit-mcp:latest
```

端点：

- `POST http://127.0.0.1:<PORT>/mcp`
- `GET  http://127.0.0.1:<PORT>/health`

## 开发

```bash
cargo fmt --all
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build
```

新增或修改工具时，请遵循 [`AGENTS.md`](AGENTS.md) —— 特别是：在认为改动完成
之前，**先更新 `docs/tools.md` 和上面的工具表**。

## 许可证

本项目基于 [Apache License, Version 2.0](LICENSE) 授权。
