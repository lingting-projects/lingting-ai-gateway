import { ReloadOutlined } from "@ant-design/icons";
import { Button } from "@lri";
import { DatePicker, Flex, Radio } from "antd";
import type { Dayjs } from "dayjs";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";

import {
  AutoRefreshControl,
  type AutoRefreshSetting,
  DEFAULT_AUTO_REFRESH_SECONDS,
} from "@/components/auto-refresh";

import { DashboardStatCards } from "./DashboardStatCards";
import { DashboardTokenChart } from "./DashboardTokenChart";
import { DASHBOARD_QUERY_ROOT } from "./dashboardQueryKeys";
import {
  DASHBOARD_RANGE_OPTIONS,
  type DashboardRangeKey,
  DEFAULT_DASHBOARD_RANGE,
  resolveRangeTime,
} from "./dashboardRangeUtils";

import "./dashboard.css";

const { RangePicker } = DatePicker;

/** 自定义时间范围取值。 */
type CustomRange = [Dayjs | null, Dayjs | null] | null;

/** 仪表盘默认关闭自动刷新。 */
const DEFAULT_AUTO_REFRESH: AutoRefreshSetting = {
  enabled: false,
  seconds: DEFAULT_AUTO_REFRESH_SECONDS,
};

/**
 * 仪表盘：顶部操作行控制时间范围与自动刷新，下方依次为全局统计卡片与 Token 统计折线图。
 */
export function DashboardMain() {
  const queryClient = useQueryClient();
  const [range, setRange] = useState<DashboardRangeKey>(DEFAULT_DASHBOARD_RANGE);
  const [customRange, setCustomRange] = useState<[Dayjs, Dayjs] | null>(null);
  const [autoRefresh, setAutoRefresh] = useState<AutoRefreshSetting>(DEFAULT_AUTO_REFRESH);

  const rangeTime = useMemo(() => resolveRangeTime(range, customRange), [customRange, range]);

  const handleRefresh = useCallback(() => {
    void queryClient.invalidateQueries({ queryKey: DASHBOARD_QUERY_ROOT });
  }, [queryClient]);

  useEffect(() => {
    if (!autoRefresh.enabled) {
      return;
    }

    const timer = window.setInterval(handleRefresh, autoRefresh.seconds * 1000);
    return () => window.clearInterval(timer);
  }, [autoRefresh, handleRefresh]);

  const handleCustomRangeChange = useCallback((value: CustomRange) => {
    if (value?.[0] && value[1]) {
      setCustomRange([value[0], value[1]]);
      return;
    }

    setCustomRange(null);
  }, []);

  return (
    <Flex className="dashboard-main" gap="middle" vertical>
      <Flex align="center" className="dashboard-actions" gap="middle" wrap>
        <Button
          disabled={autoRefresh.enabled}
          icon={<ReloadOutlined />}
          onClick={handleRefresh}
          tooltip="刷新仪表盘数据"
        />
        <AutoRefreshControl onChange={setAutoRefresh} setting={autoRefresh} />
        <Radio.Group
          onChange={(event) => setRange(event.target.value)}
          optionType="button"
          options={DASHBOARD_RANGE_OPTIONS}
          value={range}
        />
        {range === "custom" && (
          <RangePicker onChange={handleCustomRangeChange} value={customRange} />
        )}
      </Flex>
      <DashboardStatCards rangeTime={rangeTime} />
      <DashboardTokenChart rangeTime={rangeTime} />
    </Flex>
  );
}
