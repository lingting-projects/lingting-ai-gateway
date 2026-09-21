import { Flex } from "antd";

import { DashboardStatCards } from "./DashboardStatCards";
import { DashboardTokenChart } from "./DashboardTokenChart";

import "./dashboard.css";

/**
 * 仪表盘：自上而下依次为全局统计卡片与 Token 统计折线图。
 */
export function DashboardMain() {
  return (
    <Flex className="dashboard-main" gap="middle" vertical>
      <DashboardStatCards/>
      <DashboardTokenChart/>
    </Flex>
  );
}
