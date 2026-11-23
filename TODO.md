# TODO

待开发的工具清单。每项完成后请按 `AGENTS.md` 的流程同步实现、测试和文档。

## 1. 正则工具（regex）

建议放在 `crates/toolkit/src/tools/encoding/`。

- [ ] `regex_match`
  - 递归匹配 `pattern`，返回所有匹配项
  - 参数：`pattern`、`text`，可选 `limit`（默认 100，最大 1000）、`case_insensitive`
  - 返回：JSON 数组，包含匹配文本、起止位置、捕获组
- [ ] `regex_replace`
  - 按正则替换文本
  - 参数：`pattern`、`text`、`replacement`（支持 `$1` / `${name}`），可选 `limit`
  - 返回：替换后的文本和替换次数
- [ ] `regex_test`
  - 判断是否匹配，并返回第一个匹配的捕获组
  - 参数：`pattern`、`text`
  - 返回：`matched: bool`、`groups`

约束与注意：

- 使用 `regex` crate（线性时间，无回溯灾难）
- 限制 `pattern` / `text` 大小，避免超大输入
- 非法正则返回 `isError: true`，消息包含 `regex` crate 的错误
- 建议所有三个工具返回 `structuredContent` + `outputSchema`

## 2. JSON 工具（json）

建议放在 `crates/toolkit/src/tools/encoding/`。

- [ ] `json_format`
  - 格式化 JSON
  - 参数：`json`，可选 `indent`（`2` / `4` / `tab`，默认 `2`）
  - 返回：格式化后的文本
- [ ] `json_minify`
  - 压缩 JSON（移除无意义空白）
  - 参数：`json`
- [ ] `json_validate`
  - 校验 JSON 并返回解析结果摘要
  - 参数：`json`
  - 返回：`valid`、`error`（无效时）、顶层类型
- [ ] 可选：`json_query`
  - 简单路径查询（如 `a.b[0].c`），暂不引入 JSONPath

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
