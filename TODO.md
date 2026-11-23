# TODO

当前计划内的工具（Regex、JSON、Cron）均已完成。以下为后续可选改进；
新增或修改工具的流程见 `AGENTS.md`。

## 质量与安全

- [ ] 输入大小上限（例如文本/二进制 1 MiB）
- [ ] 远程模式限流
- [ ] `digest` / `hmac` 增加 `output: hex | base64`
- [ ] 统一各工具的结构化输出

## 已明确不做

- 显式 `base64_encode` / `base64_decode`：JSON Schema 无法直接传递字节数组，
  二进制字段统一使用 base64 + `format: "binary"`，由
  `string_to_binary` / `binary_to_string` 覆盖。
