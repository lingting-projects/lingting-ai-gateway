import { ReloadOutlined } from "@ant-design/icons";
import { Button, DictSelect } from "@lri";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type { DashboardQO } from "@lingting/ai-gateway-sdk";
import { DatePicker, Flex, Input, Radio, Space } from "antd";
import { useCallback, useEffect, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";
import {
  AutoRefreshControl,
  type AutoRefreshSetting,
  DEFAULT_AUTO_REFRESH_SECONDS,
} from "@/components/auto-refresh";

import {
  type CustomRange,
  type DashboardFilterDraft,
  dashboardFilterSignature,
  DEFAULT_DASHBOARD_FILTER,
} from "./dashboardFilterUtils";
import { DashboardStatCards } from "./DashboardStatCards";
import { DashboardTokenChart } from "./DashboardTokenChart";
import { DASHBOARD_QUERY_ROOT } from "./dashboardQueryKeys";
import { DASHBOARD_RANGE_OPTIONS, resolveRangeTime } from "./dashboardRangeUtils";

import "./dashboard.css";

const { RangePicker } = DatePicker;

/** 仪表盘默认关闭自动刷新。 */
const DEFAULT_AUTO_REFRESH: AutoRefreshSetting = {
  enabled: false,
  seconds: DEFAULT_AUTO_REFRESH_SECONDS,
};

/**
 * 仪表盘：顶部操作行控制时间范围、供应商、模型、会话与凭证筛选以及自动刷新，
 * 下方依次为统计卡片与折线图。
 *
 * 条件变更只改草稿，点击「应用」后才生效；自动刷新勾选后条件不可再切换。
 */
export function DashboardMain() {
  const queryClient = useQueryClient();
  const [draft, setDraft] = useState<DashboardFilterDraft>(DEFAULT_DASHBOARD_FILTER);
  const [applied, setApplied] = useState<DashboardFilterDraft>(DEFAULT_DASHBOARD_FILTER);
  const [autoRefresh, setAutoRefresh] = useState<AutoRefreshSetting>(DEFAULT_AUTO_REFRESH);

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

  /** 自动刷新运行期间禁止切换条件，避免轮询与条件变更相互干扰。 */
  const conditionsDisabled = autoRefresh.enabled;

  /** 条件有变更时「应用」才可用。 */
  const conditionsChanged = dashboardFilterSignature(draft) !== dashboardFilterSignature(applied);

  const rangeTime = useMemo(() => resolveRangeTime(applied.range, applied.customRange), [applied]);

  /** 统计筛选条件：仪表盘所有统计接口共用；分组开关只在折线图内部，由折线图自行追加。 */
  const query = useMemo<DashboardQO>(
    () => ({
      credentialId: applied.credentialId.trim() || null,
      endTime: rangeTime ? String(rangeTime[1]) : null,
      models: applied.models.length > 0 ? applied.models : null,
      providerIds: applied.providerIds.length > 0 ? applied.providerIds : null,
      sessionId: applied.sessionId.trim() || null,
      startTime: rangeTime ? String(rangeTime[0]) : null,
    }),
    [applied, rangeTime],
  );

  /** 更新草稿条件：单纯变更不触发查询，等待「应用」。 */
  const updateDraft = useCallback(
    (patch: Partial<DashboardFilterDraft>) => setDraft((previous) => ({ ...previous, ...patch })),
    [],
  );

  const handleApply = useCallback(() => setApplied(draft), [draft]);

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

  /** 自定义时间范围：起止都选齐后才写入草稿，否则视为未选择。 */
  const handleCustomRangeChange = useCallback(
    (value: CustomRange) => {
      if (value?.[0] && value[1]) {
        updateDraft({ customRange: [value[0], value[1]] });
        return;
      }

      updateDraft({ customRange: null });
    },
    [updateDraft],
  );

  return (
    <Flex className="dashboard-main" gap="middle" vertical>
      <Space className="dashboard-actions" size="middle" wrap>
        <Button
          disabled={autoRefresh.enabled}
          icon={<ReloadOutlined />}
          onClick={handleRefresh}
          tooltip="按已应用的条件刷新仪表盘数据"
        />
        <Button disabled={!conditionsChanged} type={"primary"} onClick={handleApply}>
          应用
        </Button>

        <AutoRefreshControl onChange={setAutoRefresh} setting={autoRefresh} />

        <DictSelect
          className="dashboard-filter-select"
          dict={providerOptions}
          disabled={conditionsDisabled}
          mode="multiple"
          onChange={(value) => {
            updateDraft({ providerIds: Array.isArray(value) ? value.map(String) : [] });
          }}
          placeholder="供应商"
          value={draft.providerIds}
        />

        <DictSelect
          className="dashboard-filter-select"
          dict={modelOptions}
          disabled={conditionsDisabled}
          mode="multiple"
          onChange={(value) => {
            updateDraft({ models: Array.isArray(value) ? value.map(String) : [] });
          }}
          placeholder="模型"
          value={draft.models}
        />

        <Input
          allowClear
          className="dashboard-filter-input"
          disabled={conditionsDisabled}
          onChange={(event) => updateDraft({ sessionId: event.target.value })}
          placeholder="会话ID"
          value={draft.sessionId}
        />

        <Input
          allowClear
          className="dashboard-filter-input"
          disabled={conditionsDisabled}
          onChange={(event) => updateDraft({ credentialId: event.target.value })}
          placeholder="凭证ID"
          value={draft.credentialId}
        />

        <Radio.Group
          disabled={conditionsDisabled}
          onChange={(event) => updateDraft({ range: event.target.value })}
          optionType="button"
          options={DASHBOARD_RANGE_OPTIONS}
          value={draft.range}
        />
        {draft.range === "custom" && (
          <RangePicker
            disabled={conditionsDisabled}
            onChange={handleCustomRangeChange}
            value={draft.customRange}
          />
        )}
      </Space>
      <DashboardStatCards query={query} rangeTime={rangeTime} />
      <DashboardTokenChart query={query} rangeTime={rangeTime} />
    </Flex>
  );
}
