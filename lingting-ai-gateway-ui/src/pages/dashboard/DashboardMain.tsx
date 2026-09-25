import { ReloadOutlined } from "@ant-design/icons";
import { Button, DictSelect } from "@lri";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type { DashboardQO } from "@lingting/ai-gateway-sdk";
import { DatePicker, Flex, Radio } from "antd";
import type { Dayjs } from "dayjs";
import { useCallback, useEffect, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";
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
 * 仪表盘：顶部操作行控制时间范围、供应商与模型筛选、自动刷新，下方依次为统计卡片与折线图。
 */
export function DashboardMain() {
  const queryClient = useQueryClient();
  const [range, setRange] = useState<DashboardRangeKey>(DEFAULT_DASHBOARD_RANGE);
  const [customRange, setCustomRange] = useState<[Dayjs, Dayjs] | null>(null);
  const [autoRefresh, setAutoRefresh] = useState<AutoRefreshSetting>(DEFAULT_AUTO_REFRESH);
  const [providerIds, setProviderIds] = useState<string[]>([]);
  const [models, setModels] = useState<string[]>([]);

  const { data: providers } = useQuery({
    queryFn: () => bizApi.providerList(),
    queryKey: [...DASHBOARD_QUERY_ROOT, "provider-options"],
    retry: false,
  });

  const { data: providerModels } = useQuery({
    queryFn: () => bizApi.providerModelList(),
    queryKey: [...DASHBOARD_QUERY_ROOT, "model-options"],
    retry: false,
  });

  const providerOptions = useMemo(
    () =>
      (providers ?? []).map((detail) => ({
        label: detail.provider.displayName,
        value: detail.provider.id,
      })),
    [providers],
  );

  const modelOptions = useMemo(
    () =>
      Array.from(new Set((providerModels ?? []).map((model) => model.model))).map((model) => ({
        label: model,
        value: model,
      })),
    [providerModels],
  );

  const rangeTime = useMemo(() => resolveRangeTime(range, customRange), [customRange, range]);

  /** 统计筛选条件：仪表盘所有统计接口共用；分组开关只在折线图内部，由折线图自行追加。 */
  const query = useMemo<DashboardQO>(
    () => ({
      endTime: rangeTime ? String(rangeTime[1]) : null,
      models: models.length > 0 ? models : null,
      providerIds: providerIds.length > 0 ? providerIds : null,
      startTime: rangeTime ? String(rangeTime[0]) : null,
    }),
    [models, providerIds, rangeTime],
  );

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

        <DictSelect
          className="dashboard-filter-select"
          dict={providerOptions}
          mode="multiple"
          onChange={(value) => {
            setProviderIds(Array.isArray(value) ? value.map(String) : []);
          }}
          placeholder="供应商"
          value={providerIds}
        />

        <DictSelect
          className="dashboard-filter-select"
          dict={modelOptions}
          mode="multiple"
          onChange={(value) => {
            setModels(Array.isArray(value) ? value.map(String) : []);
          }}
          placeholder="模型"
          value={models}
        />

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
      <DashboardStatCards query={query} rangeTime={rangeTime}/>
      <DashboardTokenChart query={query} rangeTime={rangeTime}/>
    </Flex>
  );
}
