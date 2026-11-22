[English](tools.md) | **简体中文**

# 工具参考

所有工具都通过两种传输方式提供：`local`（stdio）和 `remote`（HTTP）。

## 约定

- **二进制数据**始终是 base64 字符串，在 JSON Schema 中标记为
  `format: "binary"`。
- **时间戳**是以 `Z` 结尾的 ISO 8601 UTC 字符串。
- **可选参数**在 schema 中带有默认值；省略时使用默认值。
- **错误**以 `isError: true` 的工具结果返回，并附带可读消息（例如无效输入、
  不支持的枚举值）。
- 受限取值使用枚举，客户端应只发送列出的值。

## 通过 HTTP 调用工具

```bash
curl -sS -X POST http://127.0.0.1:8080/mcp \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'MCP-Protocol-Version: 2026-07-28' \
  -H 'Mcp-Method: tools/call' \
  -H 'Mcp-Name: <tool-name>' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "_meta": {
        "io.modelcontextprotocol/clientCapabilities": {},
        "io.modelcontextprotocol/protocolVersion": "2026-07-28"
      },
      "name": "<tool-name>",
      "arguments": { }
    }
  }'
```

响应是 SSE 帧：`data: {"jsonrpc":"2.0", ...}`。

- `Mcp-Name` 必须与 `params.name` 一致，且 `tools/call` 必须提供该头。
- `Accept` 必须同时包含 `application/json` 和 `text/event-stream`。
- `tools/list` 使用 `Mcp-Method: tools/list`，不需要 `Mcp-Name`。


## 时间（Time）

### current_time

返回当前 UTC 时间，ISO 8601 格式，毫秒精度。

无参数。输出示例：`2026-11-22T04:20:23.384Z`。

---

### unix_to_iso8601

把 Unix 时间戳转换为 ISO 8601 UTC 字符串。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `timestamp` | integer | 是 | — | Unix 时间戳；可以为负数 |
| `unit` | string (`seconds`, `milliseconds`) | 否 | `seconds` | `timestamp` 的单位 |

输出精度跟随 `unit`：秒 → `2023-11-14T22:13:20Z`，
毫秒 → `2023-11-14T22:13:20.123Z`。

时间戳超出范围时返回 `isError: true`。

---

### iso8601_to_unix

把 ISO 8601 日期时间转换为 Unix 时间戳。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `datetime` | string (`format: date-time`) | 是 | — | RFC 3339 日期时间，**必须带时区偏移** |
| `unit` | string (`seconds`, `milliseconds`) | 否 | `seconds` | 返回时间戳的单位 |

接受：`2023-11-14T22:13:20Z`、`2023-11-14T22:13:20.123Z`、
`2023-11-15T06:13:20+08:00`。

拒绝：不带时区偏移的 naive 日期时间（例如 `2023-11-14T22:13:20`）以及
仅日期字符串。

## 编码（Encoding）

### url_encode

对字符串进行百分号编码。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `input` | string | 是 | 原始字符串 |
| `standard` | string (`query`, `form`) | 是 | 编码规则 |

- `query`（RFC 3986）：空格 → `%20`；保留 `~`。
- `form`（`application/x-www-form-urlencoded`）：空格 → `+`；`~` → `%7E`。

示例：`"a b&c=d/é"` → query 为 `a%20b%26c%3Dd%2F%C3%A9`，form 为
`a+b%26c%3Dd%2F%C3%A9`。

---

### url_decode

对字符串进行百分号解码。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `input` | string | 是 | 已编码字符串 |
| `standard` | string (`query`, `form`) | 是 | 解码规则 |

- `query`：解码 `%XX`；`+` 保持为 `+`。
- `form`：先把 `+` 变成空格，再进行百分号解码。

格式错误的百分号转义（例如 `%2`、`%G1`）以及非 UTF-8 结果返回
`isError: true`。

---

### binary_to_hex

把二进制数据转换为十六进制字符串。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `data` | string (`format: binary`) | 是 | — | base64 编码的二进制 |
| `uppercase` | boolean | 否 | `false` | 返回大写十六进制 |

示例：`"aGVsbG8="` → `68656c6c6f`；`uppercase: true` 时为 `68656C6C6F`。

---

### hex_to_binary

把十六进制字符串转换为二进制数据。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `hex` | string | 是 | 十六进制字符串；不区分大小写，忽略空白字符 |

输出：base64 编码的二进制（JSON Schema `binary` 格式）。

示例：`68656c6c6f` → `aGVsbG8=`；`E4BDA0 E5A5BD` → `5L2g5aW9`。

包含非法字符或位数为奇数时返回 `isError: true`。

---

### binary_to_string

把二进制数据解码为文本。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `data` | string (`format: binary`) | 是 | — | base64 编码的二进制 |
| `encoding` | string | 否 | `utf-8` | `utf-8`、`ascii`、`utf-16`、`utf-32`、`gbk`、`gb18030` |

编码行为：

- `utf-8`：严格校验；会剥离开头的 UTF-8 BOM。
- `ascii`：拒绝大于 `0x7F` 的字节。
- `utf-16`：识别 BOM（`FF FE` / `FE FF`），无 BOM 时按小端处理；
  字节数为奇数或代理对无效时返回错误。
- `utf-32`：识别 BOM（`FF FE 00 00` / `00 00 FE FF`），无 BOM 时按小端处理；
  长度必须是 4 的倍数，并且码点必须有效。
- `gbk` / `gb18030`：使用 `encoding_rs` 解码；非法序列返回错误。

---

### string_to_binary

把文本编码为二进制数据。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `text` | string | 是 | — | 要编码的文本 |
| `encoding` | string | 否 | `utf-8` | `utf-8`、`ascii`、`utf-16`、`utf-32`、`gbk`、`gb18030` |

输出：base64 编码的二进制（JSON Schema `binary` 格式）。

- `utf-8`：原始 UTF-8 字节。
- `ascii`：包含非 ASCII 文本时返回错误。
- `utf-16`：带 BOM 的小端（`FF FE`）。
- `utf-32`：带 BOM 的小端（`FF FE 00 00`）。
- `gbk` / `gb18030`：目标编码无法表示的字符会返回错误（GB18030 覆盖的
  Unicode 范围比 GBK 更大，例如 emoji）。

六种编码都可以与 `binary_to_string` 相互往返转换。

## 摘要（Digest）

### digest

计算二进制数据的哈希摘要。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | base64 编码的二进制 |
| `algorithm` | string (`md5`, `sha1`, `sha256`, `sha384`, `sha512`) | 是 | 哈希算法 |

输出：摘要的 base64 编码二进制（JSON Schema `binary` 格式）。

摘要长度：MD5 = 16 字节，SHA-1 = 20 字节，SHA-256 = 32 字节，
SHA-384 = 48 字节，SHA-512 = 64 字节。

ASCII 输入 `abc`（`data` = `YWJj`）的参考值：

| 算法 | 十六进制摘要 |
|---|---|
| `md5` | `900150983cd24fb0d6963f7d28e17f72` |
| `sha1` | `a9993e364706816aba3e25717850c26c9cd0d89d` |
| `sha256` | `ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad` |
| `sha384` | `cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7` |
| `sha512` | `ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f` |

参数示例：

```json
{ "data": "YWJj", "algorithm": "sha256" }
```

---

### hmac

计算带密钥的哈希消息认证码。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | 消息，base64 编码的二进制 |
| `key` | string (`format: binary`) | 是 | 密钥，base64 编码的二进制；长度不限 |
| `algorithm` | string (`md5`, `sha1`, `sha256`, `sha384`, `sha512`) | 是 | 哈希算法 |

输出：MAC 的 base64 编码二进制（JSON Schema `binary` 格式）。
MAC 长度与摘要长度一致：16 / 20 / 32 / 48 / 64 字节。

`key` = 二十个 `0x0b` 字节（`CwsLCwsLCwsLCwsLCwsLCwsLCws=`）、
`data` = `Hi There`（`SGkgVGhlcmU=`）时的参考值：

| 算法 | 十六进制 MAC |
|---|---|
| `md5` | `5ccec34ea9656392457fa1ac27f08fbc` |
| `sha1` | `b617318655057264e28bc0b6fb378c8ef146be00` |
| `sha256` | `b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7` |
| `sha384` | `afd03944d84895626b0825f4ab46907f15f9dadbe4101ec682aa034c7cebc59cfaea9ea9076ede7f4af152e8b2fa9cb6` |
| `sha512` | `87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cdedaa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854` |

参数示例：

```json
{
  "data": "SGkgVGhlcmU=",
  "key": "CwsLCwsLCwsLCwsLCwsLCwsLCws=",
  "algorithm": "sha256"
}
```

本工具覆盖与 `digest` 相同哈希算法的 HMAC 系列。
如果密钥或消息是文本，请先用 `string_to_binary` 编码。

## 对称加密（Symmetric）

### aes_generate_key

使用操作系统 RNG 生成随机 AES 密钥。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `bits` | integer (`128`, `192`, `256`) | 否 | `256` | 密钥长度（位） |

输出：base64 编码的二进制密钥。

```json
{ "bits": 256 }
```

---

### aes_generate_iv

使用操作系统 RNG 生成随机 AES IV/nonce。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `mode` | string (`cbc`, `ctr`, `cfb`, `ofb`, `gcm`) | 是 | IV 将用于的 AES 模式 |

输出：base64 编码的二进制。

- `cbc`、`ctr`、`cfb`、`ofb` 为 16 字节
- `gcm` 为 12 字节

```json
{ "mode": "gcm" }
```

---

### aes_encrypt

使用 AES 加密二进制数据。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | 明文 |
| `key` | string (`format: binary`) | 是 | AES 密钥：16、24 或 32 字节 |
| `mode` | string (`ecb`, `cbc`, `ctr`, `cfb`, `ofb`, `gcm`) | 是 | AES 模式 |
| `padding` | string (`pkcs7`, `none`) | 是 | 填充模式 |
| `iv` | string (`format: binary`) | 否 | cbc/ctr/cfb/ofb 的 16 字节 IV，gcm 的 12 字节 nonce；ecb 忽略 |

输出：密文的 base64 二进制。

- 密钥长度决定 AES-128（16 字节）、AES-192（24 字节）或 AES-256（32 字节）。
- `pkcs7` 仅支持 `ecb` 和 `cbc`；其他模式必须使用 `none`。
- 使用 `none` 填充时，`ecb`/`cbc` 的输入长度必须是 16 字节的倍数。
- GCM 输出为 `ciphertext || tag`（末尾追加 16 字节 tag）。

```json
{
  "data": "aGVsbG8=",
  "key": "MDEyMzQ1Njc4OWFiY2RlZg==",
  "mode": "gcm",
  "padding": "none",
  "iv": "MDEyMzQ1Njc4OWFi"
}
```

---

### aes_decrypt

解密 AES 密文。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | 密文 |
| `key` | string (`format: binary`) | 是 | AES 密钥：16、24 或 32 字节 |
| `mode` | string (`ecb`, `cbc`, `ctr`, `cfb`, `ofb`, `gcm`) | 是 | AES 模式 |
| `padding` | string (`pkcs7`, `none`) | 是 | 填充模式 |
| `iv` | string (`format: binary`) | 否 | IV/nonce（见 `aes_encrypt`） |

输出：明文的 base64 二进制。

对于 GCM，`data` 必须是 `ciphertext || tag`；认证失败返回
`isError: true`。PKCS#7 填充无效同样返回 `isError: true`。

---

### des_generate_key

使用操作系统 RNG 生成随机 DES 或三重 DES 密钥。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `key_size` | string (`des`, `3des-2key`, `3des-3key`) | 否 | `3des-3key` | 密钥类型 |

输出：base64 编码的二进制密钥。

- `des` → 8 字节
- `3des-2key` → 16 字节
- `3des-3key` → 24 字节

```json
{ "key_size": "3des-3key" }
```

---

### des_generate_iv

使用操作系统 RNG 生成随机 8 字节 DES/3DES IV。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `mode` | string (`cbc`, `ctr`, `cfb`, `ofb`) | 是 | IV 将用于的 DES 模式 |

输出：base64 编码的二进制（8 字节）。

```json
{ "mode": "cbc" }
```

---

### des_encrypt

使用 DES 或三重 DES 加密二进制数据。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | 明文 |
| `key` | string (`format: binary`) | 是 | 8 字节（DES）、16 字节（2-key 3DES）或 24 字节（3-key 3DES） |
| `mode` | string (`ecb`, `cbc`, `ctr`, `cfb`, `ofb`) | 是 | DES 模式 |
| `padding` | string (`pkcs7`, `none`) | 是 | 填充模式 |
| `iv` | string (`format: binary`) | 否 | cbc/ctr/cfb/ofb 的 8 字节 IV；ecb 忽略 |

输出：密文的 base64 二进制。

- 密钥长度决定 DES（8 字节）、2-key 3DES（16 字节）或 3-key 3DES（24 字节）。
- `pkcs7` 仅支持 `ecb` 和 `cbc`；其他模式必须使用 `none`。
- 使用 `none` 填充时，`ecb`/`cbc` 的输入长度必须是 8 字节的倍数。

> **安全警告：** DES（56 位）在密码学上已被攻破，3DES 属于遗留算法。
> 新设计请使用 AES。

```json
{
  "data": "aGVsbG8=",
  "key": "MTIzNDU2Nzg=",
  "mode": "cbc",
  "padding": "pkcs7",
  "iv": "MDEyMzQ1Njc="
}
```

---

### des_decrypt

解密 DES 或三重 DES 密文。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | 密文 |
| `key` | string (`format: binary`) | 是 | 8、16 或 24 字节 |
| `mode` | string (`ecb`, `cbc`, `ctr`, `cfb`, `ofb`) | 是 | DES 模式 |
| `padding` | string (`pkcs7`, `none`) | 是 | 填充模式 |
| `iv` | string (`format: binary`) | 否 | IV（见 `des_encrypt`） |

输出：明文的 base64 二进制。PKCS#7 填充无效时返回 `isError: true`。

## RSA

### RSA 工具

通用约定：

- **私钥**：未加密的 PEM，PKCS#8（`BEGIN PRIVATE KEY`）或 PKCS#1
  （`BEGIN RSA PRIVATE KEY`）。
- **公钥**：未加密的 PEM，SPKI（`BEGIN PUBLIC KEY`）或 PKCS#1
  （`BEGIN RSA PUBLIC KEY`）。
- **二进制值**（`data`、`signature`）是带 `format: "binary"` 的 base64 字符串。
- 建议使用 2048 位或更长的密钥。
- OAEP 和 PSS 使用 **SHA-256**；PKCS#1 v1.5 签名也使用 SHA-256。
- **原始模式**（`raw`）用于调试和互操作。它执行无填充的 RSA，要求输入块长度
  正好为 `k` 字节（模数长度）。原始 RSA 是确定性的且可延展的；未经审计的
  方案不要用于真实的加密或签名。

---

### rsa_encrypt

使用 RSA 公钥加密二进制数据。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | 明文 |
| `public_key` | string | 是 | RSA 公钥 PEM |
| `padding` | string (`pkcs1v15`, `oaep-sha256`, `raw`) | 是 | 填充方案 |

输出：密文的 base64 二进制。

优先使用 `oaep-sha256`。明文最大长度取决于密钥长度和填充方案
（PKCS#1 v1.5 为 `k - 11`；使用 2048 位密钥时，OAEP-SHA256 为
`k - 2*32 - 2`）。

`raw` 直接执行 `m^e mod n`。`data` 必须正好为 `k` 字节（模数长度，大端），
并且数值小于模数；输出正好为 `k` 字节。

```json
{ "data": "aGVsbG8=", "public_key": "-----BEGIN PUBLIC KEY-----\n...", "padding": "oaep-sha256" }
```

---

### rsa_decrypt

使用私钥解密 RSA 密文。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | 密文 |
| `private_key` | string | 是 | RSA 私钥 PEM |
| `padding` | string (`pkcs1v15`, `oaep-sha256`, `raw`) | 是 | 填充方案 |

输出：明文的 base64 二进制。

`raw` 直接执行 `c^d mod n` 并返回完整的 `k` 字节块；密文必须正好为 `k` 字节。

> **PKCS#1 v1.5 说明：** 解密使用隐式拒绝。填充无效时返回确定性的合成消息
> 而不是错误；上层协议必须对结果进行认证。OAEP 会把无效密文报告为
> `isError: true`。

---

### rsa_sign

使用 RSA 私钥对二进制数据签名。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | 消息 |
| `private_key` | string | 是 | RSA 私钥 PEM |
| `padding` | string (`pkcs1v15-sha256`, `pss-sha256`, `raw`) | 是 | 签名方案 |

输出：签名的 base64 二进制。优先使用 `pss-sha256`。

`raw` 直接对 `data` 执行 RSA 私钥运算（`RSASP1`）。
`data` 必须是完整编码后的 EMSA 块，正好为 `k` 字节且小于模数。
调用方负责构建并校验编码（例如 DigestInfo 或 EMSA-PSS）。

```json
{ "data": "aGVsbG8=", "private_key": "-----BEGIN PRIVATE KEY-----\n...", "padding": "pss-sha256" }
```

---

### rsa_verify

验证 RSA 签名。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `data` | string (`format: binary`) | 是 | 消息 |
| `signature` | string (`format: binary`) | 是 | 签名 |
| `public_key` | string | 是 | RSA 公钥 PEM |
| `padding` | string (`pkcs1v15-sha256`, `pss-sha256`, `raw`) | 是 | 签名方案 |

输出：文本 `"true"` 或 `"false"`。

签名无效时返回 `"false"`；密钥格式错误、base64 错误或其他运行错误返回
`isError: true`。

`raw` 从 `signature` 计算 `s^e mod n` 并与 `data` 比较。
`data` 和 `signature` 都必须正好为 `k` 字节；签名超出范围时返回 `"false"`。

---

### rsa_generate_key

生成新的 RSA 私钥/公钥对。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `bits` | integer | 否 | `2048` | 模数位数（1024–8192） |
| `format` | string (`pkcs1`, `pkcs8`) | 否 | `pkcs8` | 私钥输出格式 |

结果会以两种方式返回：

- `content`：美化后的 JSON 文本
- `structuredContent`：同一对象

该工具还会在 `tools/list` 中声明与此对象匹配的 `outputSchema`
（`bits`、`format`、`private_key`、`public_key` 均为必填）。

```json
{
  "bits": 2048,
  "format": "pkcs8",
  "private_key": "-----BEGIN PRIVATE KEY-----\n...",
  "public_key": "-----BEGIN PUBLIC KEY-----\n..."
}
```

- `format`：所选的私钥格式
- `private_key`：该格式的私钥（未加密 PEM）
  - `pkcs8` → `BEGIN PRIVATE KEY`
  - `pkcs1` → `BEGIN RSA PRIVATE KEY`
- `public_key`：**SPKI PEM**（`BEGIN PUBLIC KEY`）

为了更高的安全性，建议使用 3072 或 4096 位。生成 4096 位以上的密钥可能
明显更慢。`rsa_private_to_public` 可以从已有私钥推导公钥，
`rsa_convert_private_key` 可以在两种私钥格式之间转换。

```json
{ "bits": 3072 }
```

---

### rsa_private_to_public

从私钥推导 RSA 公钥。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `private_key` | string | 是 | 未加密的私钥 PEM（PKCS#8 或 PKCS#1） |

输出：**SPKI 公钥 PEM**（`BEGIN PUBLIC KEY`）。

```json
{ "private_key": "-----BEGIN PRIVATE KEY-----\n..." }
```

---

### rsa_convert_private_key

在 PEM 格式之间转换 RSA 私钥。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `private_key` | string | 是 | 未加密的私钥 PEM（PKCS#1 或 PKCS#8） |
| `format` | string (`pkcs1`, `pkcs8`) | 是 | 目标格式 |

输出：转换后的私钥 PEM。

- `pkcs8` → `BEGIN PRIVATE KEY`
- `pkcs1` → `BEGIN RSA PRIVATE KEY`

密钥材料不变；只是重新编码容器格式。

```json
{ "private_key": "-----BEGIN PRIVATE KEY-----\n...", "format": "pkcs1" }
```

---

### rsa_key_components

提取 RSA 密钥的数值分量。

| 参数 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `key` | string | 是 | 未加密的私钥或公钥 PEM |

结果会同时以美化后的 JSON 文本和 `structuredContent` 返回。
该工具在 `tools/list` 中声明 `outputSchema`：
`key_type`、`bits`、`n`、`e` 为必填；私有分量在 schema 中是可选的。

字段：

| 字段 | 出现于 | 含义 |
|---|---|---|
| `key_type` | 始终 | `"private"` 或 `"public"` |
| `bits` | 始终 | 模数位长 |
| `n` | 始终 | 模数 |
| `e` | 始终 | 公钥指数 |
| `d` | 私钥 | 私钥指数 |
| `p`, `q` | 私钥 | 素因子 |
| `dp` | 私钥 | `d mod (p-1)`（OpenSSL `dmp1`） |
| `dq` | 私钥 | `d mod (q-1)`（OpenSSL `dmq1`） |
| `qinv` | 私钥 | `q^-1 mod p`（OpenSSL `iqmp`） |

所有值均为小写十六进制，采用最短大端表示（无前导零字节）。

示例：

```json
{
  "key_type": "private",
  "bits": 2048,
  "n": "c3a1...",
  "e": "010001",
  "d": "1f2e...",
  "p": "...",
  "q": "...",
  "dp": "...",
  "dq": "...",
  "qinv": "..."
}
```

> **安全警告：** 对于私钥，`d`、`p`、`q`、`dp`、`dq` 和 `qinv` 属于秘密
> 密钥材料。请勿记录或传输它们。

## 其他（Misc）

### generate_uuid

生成 UUID/GUID。

| 参数 | 类型 | 必填 | 默认值 | 说明 |
|---|---|---|---|---|
| `version` | integer (`4`, `7`) | 是 | — | 4 = 随机，7 = 按时间排序 |
| `uppercase` | boolean | 否 | `false` | 返回大写 |
| `no_hyphens` | boolean | 否 | `false` | `true` 移除 `-`；`false` 保留 |

参数示例：

```json
{ "version": 7, "uppercase": true, "no_hyphens": false }
```

输出：UUID 文本，例如 `01A20175-C53C-7128-A588-7FE1080D1C8D`。
