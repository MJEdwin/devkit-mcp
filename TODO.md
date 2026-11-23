# TODO

当前计划内的工具（Regex、JSON、Cron）均已完成。以下为后续可选改进；
新增或修改工具的流程见 `AGENTS.md`。

## 可选工具

- [ ] `cron_prev`：计算上一次执行时间
- [ ] `cron_describe`：单独返回 cron 的英文描述（目前由 `cron_validate` 返回）
- [ ] 显式 `base64_encode` / `base64_decode`（`standard` / `url_safe`）

## 质量与安全

- [ ] 输入大小上限（例如文本/二进制 1 MiB）
- [ ] 远程模式限流
- [ ] `digest` / `hmac` 增加 `output: hex | base64`
- [ ] 统一各工具的结构化输出
