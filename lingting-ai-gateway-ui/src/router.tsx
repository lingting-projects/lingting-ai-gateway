import {
  CloudServerOutlined,
  DashboardOutlined,
  DeploymentUnitOutlined,
  FileTextOutlined,
  KeyOutlined,
  SettingOutlined,
} from "@ant-design/icons";

import { type MenuRouteDefinition, PlaceholderPage, type StandaloneRouteDefinition } from "@lri";
import { ApiKeyMain } from "@/pages/api-key/ApiKeyMain";
import { DashboardMain } from "@/pages/dashboard/DashboardMain";
import { ModelMain } from "@/pages/model/ModelMain";
import { ProviderMain } from "@/pages/provider/ProviderMain";
import { RequestMain } from "@/pages/request/RequestMain";

export const menuRoutes: readonly MenuRouteDefinition[] = [
  {
    component: DashboardMain,
    icon: <DashboardOutlined/>,
    path: "dashboard",
    title: "仪表盘",
  },
  {
    component: ProviderMain,
    icon: <CloudServerOutlined/>,
    path: "provider",
    title: "供应商",
  },
  {
    component: ModelMain,
    icon: <DeploymentUnitOutlined/>,
    path: "models",
    title: "模型配置",
  },
  {
    component: RequestMain,
    icon: <FileTextOutlined/>,
    path: "request",
    title: "请求日志",
  },
  {
    component: ApiKeyMain,
    icon: <KeyOutlined/>,
    path: "api-key",
    title: "API Key",
  },
  {
    component: PlaceholderPage,
    icon: <SettingOutlined/>,
    path: "settings",
    title: "软件设置",
  },
];

export const standaloneRoutes: readonly StandaloneRouteDefinition[] = [];
