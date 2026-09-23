import { Line } from "@ant-design/plots";
import { ProCard } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import type { DashboardQO } from "@lingting/ai-gateway-sdk";
import { AppHolder, DictSelect } from "@lri";
import { Checkbox, DatePicker, Flex, Radio, Spin } from "antd";
import type { Dayjs } from "dayjs";
import { useCallback, useEffect, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";

import {
  buildTokenPoints,
  DASHBOARD_RANGE_OPTIONS,
  type DashboardRangeKey,
  DEFAULT_DASHBOARD_RANGE,
  formatTokenValue,
  resolveRangeTime,
} from "./dashboardTokenChartUtils";

import "./dashboard.css";

const { RangePicker } = DatePicker;

/** 折线图高度。 */
const CHART_HEIGHT = 400;

/** 自定义时间范围取值。 */
type CustomRange = [Dayjs | null, Dayjs | null] | null;

/**
 * Token 统计折线图：头部筛选栏控制时间范围、供应商与模型筛选及分组，主体按天展示总、缓存、读、写。
 */
export function DashboardTokenChart() {
  const [range, setRange] = useState<DashboardRangeKey>(DEFAULT_DASHBOARD_RANGE);
  const [customRange, setCustomRange] = useState<[Dayjs, Dayjs] | null>(null);
  const [withProvider, setWithProvider] = useState(false);
  const [withModel, setWithModel] = useState(false);
  const [providerIds, setProviderIds] = useState<string[]>([]);
  const [models, setModels] = useState<string[]>([]);

  const { data: providers } = useQuery({
    queryFn: () => bizApi.providerList(),
    queryKey: ["dashboard-provider-options"],
    retry: false,
  });

  const { data: providerModels } = useQuery({
    queryFn: () => bizApi.providerModelDefault(),
    queryKey: ["dashboard-model-options"],
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
    // 自定义范围未选择时不查询，避免拉取全量数据
    enabled: range !== "custom" || rangeTime !== null,
    queryFn: () => bizApi.dashboardTokenSubFilter(query),
    queryKey: ["dashboard-token-chart", query],
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

  const handleCustomRangeChange = useCallback((value: CustomRange) => {
    if (value?.[0] && value[1]) {
      setCustomRange([value[0], value[1]]);
      return;
    }

    setCustomRange(null);
  }, []);

  return (
    <ProCard className="dashboard-token-chart" title="Token统计">
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

          <Radio.Group
            onChange={(event) => setRange(event.target.value)}
            optionType="button"
            options={DASHBOARD_RANGE_OPTIONS}
            value={range}
          />

          {range === "custom" && (
            <RangePicker onChange={handleCustomRangeChange} value={customRange}/>
          )}
        </Flex>

        <Spin spinning={isFetching}>
          <Line
            axis={{
              x: {
                title: false,
              },
              y: {
                labelFormatter: (value: number) => formatTokenValue(value),
                title: false,
              },
            }}
            colorField="series"
            data={points}
            height={CHART_HEIGHT}
            legend={{
              color: {
                position: "top",
              },
            }}
            xField="day"
            yField="value"
          />
        </Spin>
      </Flex>
    </ProCard>
  );
}
