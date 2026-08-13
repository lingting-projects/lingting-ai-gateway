# lingting-ai-gateway 持久化结构

## 项目目标
个人使用的轻量 OpenAI-compatible AI Gateway。客户端只提交 client model，Gateway 通过数据库的 default provider 和 provider model mapping 解析 upstream model。

## Workspace 结构
- `lib-core`: 纯领域模型、ID、错误、路由和值对象；不依赖 HTTP/SQL。
- `lib-api`: Axum 路由、OpenAI DTO、认证、SSE 和错误映射。
- `lib-provider`: Provider trait 与 OpenAI-compatible HTTP 实现。
- `lib-store`: Settings、API Key、Provider、Provider Model、Request repositories；所有业务 SQL 在此。
- `lib-db`: pglite-oxide 生命周期、SQLx Pool、migration。
- `bin-server`: tracing、pglite、Pool、migration、Application、Axum 的唯一启动入口。

## 数据模型
业务表固定为 `settings`、`api_keys`、`providers`、`provider_models`、`requests`。所有实体 ID 为 BIGINT Snowflake；`lib-core` 从 `framework_core` 导出 `Snowflake`/`next_id`，并从 `framework_datetime` 导出统一时间 API。内部 ID、Unix 毫秒时间戳和毫秒时长统一使用 `i64`，数据库映射统一使用 `BIGINT`；禁止 `DateTime`、`TIMESTAMPTZ`、字符串时间和时区转换。requests 是一次客户端请求唯一记录，stream chunk 不落库。

`requests.api_key_id = 0` 表示 Anonymous；正常值关联 `api_keys.id`。Anonymous 开启时认证短路并无条件按匿名处理，即使请求携带有效 API Key。`providers.deleted_at = 0` 表示有效，正数为删除时间 Unix 毫秒。历史 requests 允许关联已删除 provider。

## 路由决策
只读取 `settings.default_provider_id` 指向的 enabled 且未删除 Provider，再读取该 Provider 的 enabled model mapping。缺失默认 Provider 返回 `default_provider_not_available`；模型缺失返回 `model_not_available`；禁止跨 Provider failover。

## 配置决策
settings 初始为空；`anonymous_access` 缺失默认 false；`default_provider_id` 缺失表示没有默认 Provider。所有业务配置来自数据库，不使用配置文件承载业务配置。

## 启动决策
bin-server 使用 pglite-oxide 启动嵌入式 PostgreSQL，创建 SQLx Pool，执行 migrations 后启动 Axum。pglite-oxide 的 TCP proxy 由 lib-db 暴露连接 URI。

## 按需读写声明
`AGENTS.md` 要求在修改已有模块、进行架构决策或需要恢复上下文时读取本文件；结构、职责和已确认决策发生变化时更新本文件。
