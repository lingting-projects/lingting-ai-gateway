# lingting-ai-gateway 持久化结构

- 本项目属于业务项目

## 项目目标

个人使用的轻量AI路由。客户端提交模型和推理度，路由通过根据配置的供应商优先级来匹配供应商, 模型不需要别名, 就是原始的名称,
如: gpt-5.6-luna, 则直接找出所有有 gpt-5.6-luna模型的供应商然后按照优先级使用。

## Workspace 结构

- `lib-core`: 日志, 程序文件夹等
- `lib-provider`: 供应商基础类型和trait以及公用方法定义
- `lib-web`: 定义 API 路由、请求映射、参数解析和响应转换. 其中管理接口路由固定前缀 `___`  而 ai 路由则为正常接口, 用于区分两种方式
- `lib-web-core`: 基于 `framework-web` 提供项目 Web 适配；维护请求级用户授权上下文。
- `lib-db`: 基于 pglite-oxide 数据库的数据库操作模块, 直接提供 `PgPool`、`DbConfig`、初始化方法 (提供数据存放目录)、请求级
  `DbContext` 和事务上下文，不提供通用数据库抽象。
- `types-admin`: 管理接口类型定义
- `service-admin`: 管理接口实现
- `bin-server`: tracing、pglite、Pool、migration、`GatewayApplication`、Axum 的唯一启动入口。

