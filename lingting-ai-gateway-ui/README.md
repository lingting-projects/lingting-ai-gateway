<div align="center">Lingting AI Gateway UI</div>

## 项目说明

桌面 UI 使用 React、TypeScript、Vite、Ant Design、Ant Design Pro Components 和 Tailwind CSS。

## 技术职责

- `Ant Design` / `Ant Design Pro Components`：业务组件、表单、表格、文本展示、反馈和主题
- `Tailwind CSS`：窗口结构、布局、间距和局部样式

## 目录说明

| 目录                   | 说明                                                               |
|:-----------------------|:-------------------------------------------------------------------|
| `public`               | 使用路径引用的静态资源                                             |
| `src/assets`           | 内嵌静态资源                                                       |
| `src/components`       | 无业务逻辑的布局和公共组件                                         |
| `src/components/Theme` | Ant Design 全局主题与明暗模式                                      |
| `src/pages`            | 业务页面                                                           |
| `src/pages/components` | 包含业务逻辑的页面复合组件                                         |
| `src/api`              | `BizApi` 业务命令、`NativeApi` 原生窗口能力、远程请求、IPC 或 mock |

## 当前约束

- 交互控件优先使用 Ant Design / Pro Components，不直接使用原生 `button`、`input`、`select`、`textarea` 和 `dialog`
- 数据表格必须使用 `ExtTable`
- 业务表单优先使用 `ProForm`
- 弹窗表单优先使用 `ModalForm`
- 图标统一使用 `@ant-design/icons`
- 无业务逻辑的公共组件必须放在 `src/components/<独立子目录>`
- 页面业务组件必须放在 `src/pages/components/<独立子目录>`
- 每个 `.tsx` 文件只允许包含一个主组件
- 后端契约对象不在函数参数中解构字段

## 构建

当前 `package.json` 使用 Volta 固定 `node@22.22.3`。

```bash
pnpm install
pnpm build
```

Debug 开发模式对应 `pnpm dev`。
