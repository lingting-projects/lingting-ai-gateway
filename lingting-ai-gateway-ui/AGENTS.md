# 业务控制台项目约束

本项目只承载 Lingting AI Gateway 领域业务应用。基础组件、通用高级组件、通用布局、通用界面区块、通用 Hook、Store、共享类型和组件库工具统一归属
LRI 组件库，并通过文件映射从 `@lri` 加载。

## LRI 组件库映射

- `lri` 是指向组件库源码目录的符号链接；当前映射目标由 `vite.config.ts` 和 `tsconfig.json` 中的 `@lri/*` 配置共同定义。
- LRI 是本项目的外部组件库边界，不得在本项目 `src/` 复制、镜像或二次维护其中的基础组件、高级组件、Block、Layout、Hook、Store、类型、全局样式或通用工具；但领域
  API、授权令牌、业务缓存等业务状态与工具必须留在本项目。发现真正的通用副本时，应先迁入 LRI 并移除业务侧副本。
- 使用 LRI 公开能力时，必须从 `@lri` 根入口导入。不得使用 `@lri/*` 深层路径，不得导入 LRI 的未导出实现文件。
- LRI 以源码符号链接引入，其第三方依赖只从 `lri/node_modules` 解析；pnpm 会为 LRI 的 `peerDependencies`
  自动安装独立副本，因此这些包在本项目与 LRI 下各存在一份实例。`vite.config.ts` 的 `resolve.dedupe` 必须完整镜像 LRI
  `package.json` 中 `peerDependencies` 的全部包名；否则 dev 下 Vite 预打包会掩盖问题，而构建产物会包含同一包的两份实现，React
  Context 与模块级单例无法跨实例共享。LRI 新增 peer 依赖时必须同步补齐该列表。
- 新增或修改 `lri/` 下任何组件、Block、Layout、Hook、Store、类型、工具或导出前，必须先完整阅读组件库项目
  `D:/code/lingting/lingting-react-ui/AGENTS.md`，并加载
  `D:/code/lingting/lingting-react-ui/.agents/skills/lingting-react-ui-skill/SKILL.md` 中与任务匹配的参考资料；其中约束优先适用于
  LRI 文件。涉及 Ant Design 时还必须遵循组件库的 antd skill。
- 不得因本项目业务需求向 LRI 写入领域页面、SDK 模型、API、路由配置、权限数据或业务流程；差异必须通过 Props、泛型、配置、回调或适配接口表达。

## 本项目目录职责与依赖方向

- `src/pages/`：领域页面及其私有业务组件。
- `src/api/`：Lingting AI Gateway 领域 API 客户端、传输和查询封装。
- `src/utils/`：跨领域页面复用的业务工具。
- `src/router.tsx`：本项目业务路由及菜单配置。
- `src/index.css`：本项目应用级样式入口；不得复制 LRI 全局样式或主题能力。

业务页面可以依赖 `@lri`、`src/api/` 和 `src/utils/`。LRI 不得反向依赖本项目 `src/` 的任何业务实现。业务工具不得被 LRI 引用。

### 组件组合优先级

领域页面和内部实现需要使用或组合组件时，必须严格按以下优先级检索与复用：`@lri` → Ant Design Pro Components → Ant Design（
`antd`）。只有确认高优先级来源不存在满足需求的组件时，才可继续检索下一优先级；禁止绕过已有高优先级组件，直接使用低优先级库的组件。
业务组件必须先检索并复用 `src/components/` 中已有的业务组件；仅当不存在可复用业务组件时，才可按 `@lri` → Ant Design Pro
Components → Ant Design（`antd`）的严格优先级组合实现。不得绕过任一更高优先级的组件来源。
禁止在业务页面和组件中直接使用原生 HTML 功能组件或展示元素，包括但不限于 `button`、`input`、`select`、`textarea`、`span`
。必须使用上述优先级中提供的等效组件；例如操作按钮应优先使用 `import { Button } from '@lri'`，不得使用
`import { Button } from 'antd'`；展示文本应使用 `import { Typography } from 'antd'` 与 `<Typography.Text>`，不得使用
`<span>`。
Ant Design 原生组件必须直接从 `antd` 导入，且仅可在 LRI 与 Ant Design Pro Components 均未提供等效组件时使用；图标必须直接从
`@ant-design/icons` 导入。

- 业务组件的布局必须优先使用 Ant Design 提供的 `Flex`、`Grid`、`Space` 等布局组件及其组合能力；禁止通过自定义 CSS
  绘制或替代业务组件布局，包括但不限于使用 `display: flex/grid`、定位、负边距或复杂选择器自行实现布局。只有 Ant Design
  布局组件无法满足的确有必要的视觉细节，才允许使用 CSS 补充，不得以 CSS 取代布局组件。
- 不得仅因组件需要 `onClick` 点击事件就使用 `Button`；只有交互语义被明确声明为“按钮”时才可使用。非按钮式的可点击内容应使用合适的布局组件承载点击事件，例如
  `<Flex onClick={...}>` 内组合 `<Typography.Text>`，不得以 `Button` 配合 `className` 和 CSS `display: flex` 模拟此类布局。

### 条件组件渲染

条件渲染近似操作组件时，不得使用三元表达式或 `if` 分支。组件提供 `hidden` 或同等可见性参数时，优先平铺声明组件并通过该参数控制可见性；没有此类参数时，使用多个
`{条件 && <组件 />}` 表达式。示例：

```tsx
<LinkButton hidden={!enabled} text="启用"/>
<LinkButton hidden={enabled} text="禁用"/>

{
  enabled && <LinkButton text="启用"/>
}
{
  !enabled && <LinkButton text="禁用"/>
}
```

## 业务工具约束

- 新增或修改领域页面、业务组件前，先检查 `src/utils/` 是否已有可复用能力；存在时必须复用或完善。
- 仅服务于一个业务组件且没有跨组件复用条件的工具可放在该组件同目录的 `组件名Utils.ts(x)`；可被两个及以上业务组件复用时必须迁移至
  `src/utils/`。
- `src/utils/` 只能包含领域工具，不得包含可迁移至 LRI 的通用 UI、状态、存储、格式化或交互逻辑。

## Ant Design 与样式约束

- 使用 Ant Design v6 API；编写或分析 antd 代码前，先使用 `.agents/skills/antd/SKILL.md` 规定的 antd MCP 工具查询
  API、文档、示例、语义结构和 token。仅当 MCP 不支持时才使用项目本地 `pnpm antd` CLI。
- 新增页面与业务组件必须符合根目录 `DESIGN.md`。页面表面、文字和状态通过 ConfigProvider、ProConfigProvider 与 ANTD token
  保持主题适配，不建立平行主题体系或硬编码主题色。
- 静态布局、颜色、间距、边框和状态样式使用语义化类名，并在对应 CSS 中以 Tailwind CSS `@apply` 组合；仅运行时动态计算且无法由类名、组件
  API 或 CSS 变量表达的值可以使用内联样式或 token。
- 基础展示能力优先复用 LRI；文本展示优先使用 `Typography`。

## 命名

- `src` 下普通 TypeScript、TSX 与样式文件使用小驼峰；页面和 React 组件文件、导出组件使用 PascalCase。
- 目录使用小写 kebab-case；`main.tsx`、`router.tsx`、各级 `index.ts` 等约定入口保持既有名称。
- 仅在修改或新增文件时执行命名迁移，不批量改动无关文件。

## 页面验证

- 仅当用户明确声明服务已启动并提供地址时，才使用 chrome-devtools 访问目标地址验证；默认地址为 `http://127.0.0.1:26381`
  。未满足此前提时不得启动服务。
- chrome-devtools 打开的浏览器窗口要最大化, 避免未自适应元素异常, 仅在明确声明样式要自适应时才调整不同窗口大小来检查
