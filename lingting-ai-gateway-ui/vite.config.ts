import { resolve } from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite-plus";

export default defineConfig(({ mode }) => {
  const isDesktopBuild = mode === "desktop";

  return {
    plugins: [react(), tailwindcss()],
    resolve: {
      // LRI 以源码符号链接引入，其第三方依赖只会从 lri/node_modules 解析。pnpm 会为 LRI 的 peerDependencies 自动安装独立副本，
      // 于是同一个包在本项目与 LRI 下各存在一份实例。
      // dev 下 Vite 预打包会把裸模块说明符统一改写为同一 chunk，问题被掩盖；构建（rolldown）按
      // importer 各自解析，会产出两份实现，React Context 与模块级单例无法跨实例共享。曾表现为
      // ModalForm 内 ExFormDictSelect 读不到宿主 Form 的 layout，退化为 antd 默认 horizontal，
      // 渲染成 ant-form-item-horizontal。
      //
      // 维护规则：本列表必须与 lri/package.json 的 peerDependencies 包名保持一致，
      // LRI 新增 peer 依赖时必须同步补齐。匹配基于包名，深层导入（如 react/jsx-runtime、
      // antd/es/form）已由包名覆盖，无需单列。
      dedupe: [
        "@ant-design/icons",
        "@ant-design/pro-components",
        "@marsidev/react-turnstile",
        "@tanstack/react-query",
        "@tanstack/react-router",
        "@tanstack/react-store",
        "antd",
        "clsx",
        "dayjs",
        "react",
        "react-dom",
      ],
      alias: {
        "@": resolve(import.meta.dirname, "src"),
        "@lri": resolve(import.meta.dirname, "lri"),
      },
    },
    fmt: {
      ignorePatterns: [".agents", "node_modules", "dist", "*.yaml", "lri", "*.md"],
    },
    server: {
      host: "127.0.0.1",
      port: 26381,
      proxy: {
        // 必须带尾部斜杠：vite 的代理 key 按前缀匹配，写成 "/api" 会把前端路由 /api-key 也转发给后端
        "^/api/": {
          target: "http://localhost:26380",
          changeOrigin: true,
          rewrite: (path) => path.replace(/^\/api/, ""),
        },
      },
    },
    build: {
      cssCodeSplit: !isDesktopBuild,
      rolldownOptions: {
        output: {
          codeSplitting: !isDesktopBuild,
        },
      },
    },
  };
});
