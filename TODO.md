# TODO

待开发的工具清单。每项完成后请按 `AGENTS.md` 的流程同步实现、测试和文档。

## 1. 正则工具（regex）

放在 `crates/toolkit/src/tools/regex/`。

- [x] `regex_match`（已实现）
  - 返回所有匹配项
  - 参数：`pattern`、`text`、可选 `limit`、`case_insensitive`
  - 返回：`count`、`truncated`、`matches`（文本、字节偏移、捕获组、命名捕获组）
- [x] `regex_replace`（已实现）
  - 按正则替换文本
  - 参数：`pattern`、`text`、`replacement`、可选 `limit`、`case_insensitive`
  - 返回：`result`、`replacements`
- [x] `regex_test`（已实现）
  - 判断是否匹配，并返回第一个匹配的详情
  - 参数：`pattern`、`text`、可选 `case_insensitive`
  - 返回：`matched`、`match`

约束与注意：

- 使用 `regex` crate（线性时间，无回溯灾难）
- 限制 `pattern` / `text` 大小，避免超大输入
- 非法正则返回 `isError: true`，消息包含 `regex` crate 的错误
- 建议所有三个工具返回 `structuredContent` + `outputSchema`

## 2. JSON 工具（json）

放在 `crates/toolkit/src/tools/json/`。

- [x] `json_pretty`（已实现）
  - 格式化 JSON，保留键顺序
  - 参数：`json`、可选 `indent`（`0`–`8`，默认 `2`）、`unicode`（`characters` / `escapes`）
  - 支持 `\uXXXX` 解码为真实字符，也支持转义输出与代理对
- [x] `json_minify`（已实现）
  - 压缩 JSON，保留键顺序
  - 参数：`json`、可选 `unicode`（`characters` / `escapes`）
- [x] `json_validate`（已实现）
  - 校验 JSON 语法
  - 参数：`json`
  - 返回：`valid`、`line`、`column`、`error`
- [x] `json_schema_validate`（已实现）
  - 按 JSON Schema 校验 JSON
  - 参数：`json`、`json_schema`
  - 返回：`valid`、`schema_errors`
- [x] `json_query`（已实现）
  - 使用 JSONPath（RFC 9535）查询
  - 参数：`json`、`path`、可选 `limit`
  - 返回：`count`、`truncated`、`matches`

约束与注意：

- 复用现有 `serde_json` 依赖
- 限制输入大小（例如 1 MiB）
- 无效 JSON 返回 `isError: true`，包含行列位置（`serde_json::Error` 的 `line()` / `column()`）
- 注意：`serde_json::Map` 默认按键排序；如需保留原始顺序，需要启用 `preserve_order` feature，文档中明确说明
- 建议 `json_validate` 返回 `structuredContent`

## 3. Cron 表达式工具（cron）

建议放在 `crates/toolkit/src/tools/time/`。

- [ ] `cron_validate`
  - 校验 cron 表达式
  - 参数：`expression`
  - 返回：`valid`、字段数量、`error`（无效时）
- [ ] `cron_next`
  - 计算接下来 N 次执行时间
  - 参数：`expression`、可选 `from`（RFC 3339，默认当前时间）、可选 `count`（默认 5，最大 50）、可选 `timezone`
  - 返回：RFC 3339 时间数组
- 可选：`cron_prev` / `cron_describe`

约束与注意：

- 依赖候选：`croner`（支持 5/6/7 字段）或 `cron`
- 明确支持的字段数：
  - 5 字段：分 时 日 月 周
  - 6 字段（可选）：秒 分 时 日 月 周
- 时区：
  - 初期可只支持 `utc` 和 `asia_shanghai` 两个枚举值（避免引入完整 tzdb）
  - 如需任意 IANA 时区，再评估 `chrono-tz`
- DST 处理需要在文档中明确说明
- 非法表达式或超出范围的时间返回 `isError: true`

## 完成标准

每个工具都需要：

- [ ] 在对应分类 `mod.rs` 中注册
- [ ] 加入 `tool_box!` 和 `handler.rs` dispatch
- [ ] 更新 `server_details` instructions
- [ ] 在 `crates/toolkit/tests/<category>.rs` 增加成功和错误用例
- [ ] 更新 `docs/tools.md` 和 `docs/tools.zh-CN.md`
- [ ] 更新 `README.md` 对应分类表
- [ ] 如有 `structuredContent`，实现 `output_schema()` / provider 并加入 `OUTPUT_SCHEMA_PROVIDERS`
- [ ] `cargo fmt --all`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] stdio 冒烟测试

## 其他建议（暂不实现）

- 显式 `base64_encode` / `base64_decode`（`standard` / `url_safe`）
- 输入大小上限与远程模式限流
- `digest` / `hmac` 增加 `output: hex | base64`
- 统一各工具的结构化输出
