import { Line } from "@ant-design/plots";
import { ProCard } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import type { DashboardQO } from "@lingting/ai-gateway-sdk";
import { AppHolder } from "@lri";
import { Flex, Spin } from "antd";
import { useEffect, useMemo } from "react";

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
  /** 统计筛选条件：与仪表盘其它统计接口一致 */
  query: DashboardQO;
  /** 统计时间范围，未选择时跳过查询 */
  rangeTime: [number, number] | null;
  /** 是否按模型继续分组 */
  withModel: boolean;
  /** 是否按供应商继续分组 */
  withProvider: boolean;
};

/**
 * Token 折线图：主体按天展示总、缓存、读、写，下方再展示缓存读占比与缓存写占比。
 *
 * 筛选条件由仪表盘头部统一控制，两张图共用同一份查询数据。
 */
export function DashboardTokenChart({
                                      query,
                                      rangeTime,
                                      withModel,
                                      withProvider,
                                    }: DashboardTokenChartProps) {
  /** 折线图按天分组，并可按供应商、模型继续分组，因此需要额外带上分组开关。 */
  const chartQuery = useMemo<DashboardQO>(
    () => ({ ...query, withModel, withProvider }),
    [query, withModel, withProvider],
  );

  const { data, error, isFetching } = useQuery({
    enabled: rangeTime !== null,
    queryFn: () => bizApi.dashboardTokenSubFilter(chartQuery),
    queryKey: [...DASHBOARD_QUERY_ROOT, "token-chart", chartQuery],
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

  const ratioPoints = useMemo(
    () => buildCacheRatioPoints(data ?? [], withProvider, withModel, rangeTime),
    [data, rangeTime, withModel, withProvider],
  );

  return (
    <ProCard className="dashboard-token-chart">
      <Spin spinning={isFetching}>
        <Flex gap="middle" vertical>
          <TokenLineChart data={points} formatValue={formatTokenCount}/>
          <TokenLineChart data={ratioPoints} formatValue={formatPercent}/>
        </Flex>
      </Spin>
    </ProCard>
  );
}
