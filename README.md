# lingting-ai-gateway

个人使用的轻量 OpenAI-compatible AI Gateway。

## 启动

要求 Rust 1.93+。程序会通过 pglite-oxide 在 `data/postgres-v1` 启动嵌入式 PostgreSQL、执行 migrations，然后监听 `127.0.0.1:8080`。

```bash
cargo run -p bin-server
```

数据库初始不包含任何业务配置。首次使用前需直接写入：

- `settings.default_provider_id`
- 可选的 `settings.anonymous_access`（缺失时为 `false`）
- `api_keys`（只保存 SHA-256 十六进制 hash）
- `providers`
- `provider_models`

## API

- `GET /health`
- `GET /v1/models`
- `POST /v1/chat/completions`

Chat Completions 支持普通 JSON 响应和 SSE streaming。客户端只使用 `provider_models.model_name`；Gateway 仅通过默认 Provider 映射到 `upstream_model`，不会 Failover。

## 数据约定

- 所有业务 ID 使用 `lib_core::next_id()` 生成并保存为 `BIGINT`。
- Anonymous 请求使用 `requests.api_key_id = 0`。
- Provider 逻辑删除使用 `deleted_at = 0` / Unix 毫秒时间戳。
- 一次客户端请求只对应一条 `requests` 记录；stream chunk 不落库。
- Provider API Key 明文保存在数据库，但不会写入日志和错误响应。
