# lingting-ai-gateway

个人使用的轻量 AI 路由网关，主要功能：

- **模型路由**：客户端按原始模型名调用（例如 `gpt-5.6-luna`），网关找出提供该模型的全部供应商，按优先级选一个转发；
  模型名不做别名映射。
- **协议兼容**：对外提供 `[OI]` 兼容的 `/v1/chat/completions`、`/v1/responses` 与 `/v1/models`，同时支持普通响应与流式响应；
  请求体原样透传，仅把鉴权头替换为供应商的 `api_key`。
- **供应商与模型管理**：维护供应商、供应商模型（上下文窗口、最大输出、推理级别、视觉与缓存能力等）与模型名映射，
  并支持从供应商同步可用模型列表。
- **请求日志**：记录主请求与其子请求，包含实际命中的供应商、状态码、错误信息、token 用量、首字时间与耗时，控制台可按条件查询与统计。
- **API Key 鉴权**：库中只保存 key 的 sha1，支持启用/禁用与逻辑删除，并可在多个实例之间同步；
  管理接口另有管理员令牌与匿名访问开关。
- **内嵌控制台**：供应商、模型、API Key、请求日志与仪表盘等 React 页面，由 `bin-server` 在 release 打包时内嵌进二进制，无需单独部署。
- **本机运行**：数据存放在本机 pglite，日志落在本机目录；可注册为系统服务（Windows 计划程序 / Linux systemd / macOS launchd），
  提供 `install`、`uninstall`、`reinstall`、`restart`、`stop` 指令。
- **pi agent 适配**：按当前可用模型生成 pi 的 `models.json`，可直接导出，或备份后同步到指定目录。

## 开发说明

1. 入口为 `bin-server`：`cargo run -p bin-server` 启动服务（调试模式固定监听 `127.0.0.1:26384`）。
2. 前端在 `lingting-ai-gateway-ui` 中。
3. 前端启动见 `lingting-ai-gateway-ui/README.md`。
4. 前端依赖导出的 ts sdk，导出方式见「本地编译说明」。

## 本地编译说明

1. 执行 `scripts/build_current.sh`；脚本自动识别系统与架构，以 release 打包 `bin-server`
   （Linux 构建 musl 静态二进制，必要时自动 `rustup target add`）。
2. 脚本会输出编译结果所在位置，例如 `target/release/bin-server`（Windows 为
   `target/release/bin-server.exe`，Linux musl 为 `target/<target>/release/bin-server`）。
3. 编译前要求：必须导出 ts sdk。前端依赖 `packages/sdk`，release 打包时会自动执行
   `pnpm build:desktop`，未导出时前端构建会失败：

   ```bash
   cargo run -p lib-web --example build_ts --features ts-export
   ```
