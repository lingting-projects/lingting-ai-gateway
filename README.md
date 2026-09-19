# lingting-ai-gateway

个人使用的轻量AI路由。

## 环境要求

- Rust 1.93+（edition 2024）

## 开发

启动服务，调试模式监听 `127.0.0.1:26380`：

```bash
cargo run -p bin-server
```

## 导出 TypeScript SDK

```bash
cargo run -p lib-web --example build_ts --features ts-export
```

元数据由 `web_api_*`、`auto_type`、`auto_enum` 在编译期注册，必须带上 `ts-export` feature，否则导出的内容为空。

产物写入 `packages/sdk`，包含 `package.json` 与 `dist/` 下的 `api`、`types`、`enums`、`index` 模块。
接口、DTO 或枚举发生变化后需重新执行该命令。

该包由 `lingting-ai-gateway-ui` 消费：`pnpm-workspace.yaml` 的 `packages: ['../packages/*']` 将其纳入工作区，
依赖声明为 `"@lingting/ai-gateway-sdk": "workspace:"`。
