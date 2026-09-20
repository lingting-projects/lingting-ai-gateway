import { DashboardOutlined } from "@ant-design/icons";

import { type MenuRouteDefinition, PlaceholderPage, type StandaloneRouteDefinition } from "@lri";

export const menuRoutes: readonly MenuRouteDefinition[] = [
  {
    component: PlaceholderPage,
    icon: <DashboardOutlined/>,
    path: "dashboard",
    title: "仪表盘",
  },
  {
    component: PlaceholderPage,
    icon: <DashboardOutlined/>,
    path: "provider",
    title: "供应商",
  },
  {
    component: PlaceholderPage,
    icon: <DashboardOutlined/>,
    path: "models",
    title: "模型配置",
  },
  {
    component: PlaceholderPage,
    icon: <DashboardOutlined/>,
    path: "request",
    title: "请求日志",
  },
  {
    component: PlaceholderPage,
    icon: <DashboardOutlined/>,
    path: "settings",
    title: "软件设置",
  },
];

export const standaloneRoutes: readonly StandaloneRouteDefinition[] = [];
