# lingting-ai-gateway 持久化结构

- 本项目属于业务项目

## 项目目标

个人使用的轻量AI路由。客户端提交模型和推理度，路由通过根据配置的供应商优先级来匹配供应商, 模型不需要别名, 就是原始的名称,
如: gpt-5.6-luna, 则直接找出所有有 gpt-5.6-luna模型的供应商然后按照优先级使用。

## Workspace 结构

所有 crate 统一位于 `crates/` 目录下。

- `lib-core`: 日志, 程序文件夹等
- `lib-system-service`: 系统服务注册与卸载。对外提供统一配置与调用入口，按操作系统选择实现：Windows 计划程序、
  Linux systemd、macOS launchd。
- `lib-provider`: 供应商基础类型和trait以及公用方法定义。只放与协议无关的通用能力：`RouteRequest`(网关路由与日志所需字段)、
  `ForwardRequest`、`ForwardCallback` / `ForwardOutcome`、`ChunkSink`、模型同步用的远程模型 `RemoteModel`。具体协议的请求/响应模型不放在这里。
- `lib-provider-openai`: `[OI]` 协议的转发实现。按协议分文件: `chat.rs` / `chat_response.rs` / `chat_stream.rs`、
  `responses.rs` / `responses_response.rs` / `responses_stream.rs`, 另有共用的流式驱动 `stream.rs`、工具 `utils.rs` 与模型列表获取 `models.rs`。
  协议响应模型跟随协议放在本 crate, 不放 `lib-provider`。
- `lib-web`: 定义 API 路由、请求映射、参数解析和响应转换. 其中管理接口路由固定前缀 `___`  而 ai 路由则为正常接口, 用于区分两种方式
- `lib-web-core`: 基于 `framework-web` 提供项目 Web 适配；维护请求级用户授权上下文。
- `lib-db`: 基于 pglite-oxide 数据库的数据库操作模块, 直接提供 `PgPool`、`DbConfig`、初始化方法 (提供数据存放目录)、请求级
  `DbContext` 和事务上下文，不提供通用数据库抽象。
- `types-admin`: 管理接口类型定义
- `service-admin`: 管理接口实现
- `service-provider`: 供应商业务层。按模型路由到具体供应商并转发请求, 同时负责模型同步与请求日志编排。
- `bin-server`: tracing、pglite、Pool、migration、`GatewayApplication`、Axum 的唯一启动入口。

## AI 转发链路

`/v1/chat/completions` 与 `/v1/responses` 的结构完全对称, 各自的协议实现互不干扰:

1. `lib-web/src/openai.rs`: 声明路由, 解析请求体得到 `RouteRequest`。
2. `service-provider/src/openai/{chat,responses}.rs`: 按模型取优先级最高的供应商, 按 `stream` 分派到普通或流式转发。
3. `service-provider/src/openai/forward.rs`: `with_wrapper` 统一写主 / 子请求日志、装配转发回调, 再执行具体转发。
4. `service-provider/src/openai/call_{chat,response}[_stream].rs`: 构造协议请求并执行。
5. `lib-provider-openai/src/{chat,chat_stream,responses,responses_stream}.rs`: 发起转发并把上游响应原样返回。
   流式由 `stream.rs` 的 `forward_stream` 驱动, 协议差异由各自的 `StreamParser` 实现承担。

上游地址为 `{provider.base_url}` 拼接协议后缀(`/chat/completions`、`/responses`); 请求体原样透传, 鉴权头替换为供应商 api_key。
