import { Line } from "@ant-design/plots";
import { ProCard } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import type { DashboardQO } from "@lingting/ai-gateway-sdk";
import { AppHolder, DictSelect } from "@lri";
import { Checkbox, Flex, Spin } from "antd";
import { useEffect, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";
import { formatPercent, formatTokenCount } from "@/utils/tokenUtils";

import { DASHBOARD_QUERY_ROOT } from "./dashboardQueryKeys";
import {
  buildCacheRatioPoints,
  buildTokenPoints,
  type DashboardTokenPoint,
} from "./dashboardTokenChartUtils";

import "./dashboard.css";

/** 折线图高度。 */
const CHART_HEIGHT = 400;

type TokenLineChartProps = {
  data: DashboardTokenPoint[];
  /** 数值展示：Token 用量用 K / M 缩写，占比用百分比 */
  formatValue: (value: number) => string;
};

/** 折线图：按天展示各系列，坐标轴与提示共用同一数值格式化。 */
function TokenLineChart({ data, formatValue }: TokenLineChartProps) {
  return (
    <Line
      axis={{ x: { title: false }, y: { labelFormatter: formatValue, title: false } }}
      colorField="series"
      data={data}
      height={CHART_HEIGHT}
      legend={{ color: { position: "top" } }}
      tooltip={{ items: [{ channel: "y", valueFormatter: formatValue }] }}
      xField="day"
      yField="value"
    />
  );
}

type DashboardTokenChartProps = {
  /** 统计时间范围，未选择时跳过查询 */
  rangeTime: [number, number] | null;
};

/**
 * Token 统计折线图：头部筛选栏控制供应商与模型筛选及分组，主体按天展示总、缓存、读、写。
 */
export function DashboardTokenChart({ rangeTime }: DashboardTokenChartProps) {
  const [withProvider, setWithProvider] = useState(false);
  const [withModel, setWithModel] = useState(false);
  const [providerIds, setProviderIds] = useState<string[]>([]);
  const [models, setModels] = useState<string[]>([]);

  const { data: providers } = useQuery({
    queryFn: () => bizApi.providerList(),
    queryKey: [...DASHBOARD_QUERY_ROOT, "provider-options"],
    retry: false,
  });

  const { data: providerModels } = useQuery({
    queryFn: () => bizApi.providerModelDefault(),
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

  const query = useMemo<DashboardQO>(
    () => ({
      endTime: rangeTime ? String(rangeTime[1]) : null,
      models: models.length > 0 ? models : null,
      providerIds: providerIds.length > 0 ? providerIds : null,
      startTime: rangeTime ? String(rangeTime[0]) : null,
      withModel,
      withProvider,
    }),
    [models, providerIds, rangeTime, withModel, withProvider],
  );

  const { data, error, isFetching } = useQuery({
    enabled: rangeTime !== null,
    queryFn: () => bizApi.dashboardTokenSubFilter(query),
    queryKey: [...DASHBOARD_QUERY_ROOT, "token-chart", query],
    retry: false,
  });

  useEffect(() => {
    if (error) {
      AppHolder.message.error(error.message);
    }
  }, [error]);

  const points = useMemo(
    () => buildTokenPoints(data ?? [], withProvider, withModel, rangeTime),
    [data, rangeTime, withModel, withProvider],
  );

  /** 缓存读占比与缓存写占比：与 Token 图共用同一份查询数据。 */
  const ratioPoints = useMemo(
    () => buildCacheRatioPoints(data ?? [], withProvider, withModel, rangeTime),
    [data, rangeTime, withModel, withProvider],
  );

  return (
    <ProCard className="dashboard-token-chart">
      <Flex gap="middle" vertical>
        <Flex align="center" className="dashboard-token-chart-filter" gap="middle" wrap>
          <Checkbox
            checked={withProvider}
            onChange={(event) => setWithProvider(event.target.checked)}
          >
            供应商分组
          </Checkbox>

          <DictSelect
            className="dashboard-token-chart-select"
            dict={providerOptions}
            mode="multiple"
            onChange={(value) => {
              setProviderIds(Array.isArray(value) ? value.map(String) : []);
            }}
            placeholder="供应商"
            value={providerIds}
          />

          <Checkbox checked={withModel} onChange={(event) => setWithModel(event.target.checked)}>
            模型分组
          </Checkbox>

          <DictSelect
            className="dashboard-token-chart-select"
            dict={modelOptions}
            mode="multiple"
            onChange={(value) => {
              setModels(Array.isArray(value) ? value.map(String) : []);
            }}
            placeholder="模型"
            value={models}
          />
        </Flex>

        <Spin spinning={isFetching}>
          <Flex gap="middle" vertical>
            <TokenLineChart data={points} formatValue={formatTokenCount} />
            <TokenLineChart data={ratioPoints} formatValue={formatPercent} />
          </Flex>
        </Spin>
      </Flex>
    </ProCard>
  );
}
