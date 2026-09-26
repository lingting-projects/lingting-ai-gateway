<div align="center">Lingting AI Gateway UI</div>

## 项目说明

桌面 UI 使用 React、TypeScript、Vite、Ant Design、Ant Design Pro Components 和 Tailwind CSS。

## 开发说明

1. `lri` 文件夹挂载：`lri` 是组件库源码目录的挂载（默认 `D:\code\lingting\lingting-react-ui\src`），
   由本目录下的 `mout-ui.ps1` 创建；`@lri` 别名在 `vite.config.ts` 与 `tsconfig.json` 中声明，挂载缺失时无法启动。
2. 版本要求：node 由 Volta 固定为 `22.22.3`，pnpm 使用 v11（当前 11.24.0）。
3. 依赖导出的 ts sdk：`@lingting/ai-gateway-sdk` 由 `pnpm-workspace.yaml` 的 `packages: ['../packages/*']`
   从 `packages/sdk` 引入，需先导出 ts sdk（见根目录 README 的「本地编译说明」），未导出时启动与构建都会失败。

## 本地启动说明

```bash
pnpm dev
```

开发服务器监听 `127.0.0.1:26381`，并把 `^/api/` 代理到后端 `http://localhost:26384`（见 `vite.config.ts`）。
