# lingting-ai-gateway 项目规则

- 所有输出、注释和文档使用简体中文。
- 这是个人使用的 Rust V1 OpenAI-compatible Gateway；保持实现轻量，禁止投机性扩展。
- 使用 Cargo Workspace；内部非入口 crate 必须以 `lib-` 开头，入口 crate 必须以 `bin-` 开头。
- 业务配置全部来自 PostgreSQL `settings`、`api_keys`、`providers`、`provider_models`；不得放入 toml/yaml/json/env。
- `lib-core` 不依赖 Axum、SQLx、PostgreSQL；业务代码不直接写 SQL，SQL 只在 `lib-store`。
- 业务 ID 使用 `lib-core` 从 `framework_core` 导出的 Snowflake，统一为 `i64`/数据库 `BIGINT`；禁止自实现、UUID、自增 ID。
- 内部时间点统一为 Unix 毫秒 `i64`，时长统一为毫秒 `i64`，数据库统一为 `BIGINT`；禁止 DateTime、TIMESTAMPTZ、字符串时间和时区转换。
- Anonymous 使用 `requests.api_key_id = 0`，`providers.deleted_at = 0` 表示未删除；不得用 NULL 表达这两个状态。Anonymous 开启时无条件优先，所有请求均按匿名处理，即使携带有效 API Key。
- Provider API Key V1 明文存储，但绝不能进入日志、错误、panic、HTTP header 日志或响应。
- V1 只有五张业务表，不创建 Attempt、Execution、Chunk 等子表；stream chunk 只在内存累计。
- V1 不实现 UI、Failover、Retry、Circuit Breaker、Load Balance、健康检查、成本/智能路由和 Provider Key 加密。
- 修改或扩展现有模块前，按需读取 `workerspace.md`；完成架构决策后同步更新该文件。
- 新功能不自动新增测试文件；以 cargo fmt/check 和人工 HTTP 流程验证为主。
- 写完代码后必须进行一次代码审查。
