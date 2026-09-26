# lingting-ai-gateway

个人使用的轻量 AI 路由网关。

客户端按原始模型名调用（例如 `gpt-5.6-luna`），网关找出提供该模型的所有供应商并按优先级选一个转发：
请求体原样透传，鉴权头替换为供应商的 `api_key`，同时记录主请求与子请求日志。对外提供 `[OI]` 兼容的
`/v1/chat/completions`、`/v1/responses`、`/v1/models`（支持流式），数据存放在本机 pglite，
控制台前端由 `bin-server` 内嵌进二进制，release 打包时一并产出，可注册为系统服务
（Windows 计划程序 / Linux systemd / macOS launchd）。

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
