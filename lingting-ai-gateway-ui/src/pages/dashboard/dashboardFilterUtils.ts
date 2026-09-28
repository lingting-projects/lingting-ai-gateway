import type { Dayjs } from "dayjs";

import { DEFAULT_DASHBOARD_RANGE, type DashboardRangeKey } from "./dashboardRangeUtils";

/** 自定义时间范围取值。 */
export type CustomRange = [Dayjs | null, Dayjs | null] | null;

/**
 * 仪表盘筛选条件：条件变更只写入草稿，点击「应用」后生效并触发查询。
 */
export type DashboardFilterDraft = {
  /** 快捷时间范围 */
  range: DashboardRangeKey;
  /** 自定义时间范围，仅 `range` 为 `custom` 时使用 */
  customRange: [Dayjs, Dayjs] | null;
  providerIds: string[];
  models: string[];
  /** 会话 ID，精确匹配 */
  sessionId: string;
  /** 凭证 ID，精确匹配 */
  credentialId: string;
};

/** 仪表盘默认筛选条件。 */
export const DEFAULT_DASHBOARD_FILTER: DashboardFilterDraft = {
  customRange: null,
  credentialId: "",
  models: [],
  providerIds: [],
  range: DEFAULT_DASHBOARD_RANGE,
  sessionId: "",
};

/**
 * 条件签名：逐字段压缩为字符串，用于判断草稿是否与已生效条件一致。
 *
 * 供应商与模型是多选，顺序不影响查询结果，比较前先排序；文本条件忽略首尾空白。
 */
export function dashboardFilterSignature(filter: DashboardFilterDraft): string {
  return JSON.stringify([
    filter.range,
    filter.customRange?.[0].valueOf() ?? null,
    filter.customRange?.[1].valueOf() ?? null,
    [...filter.providerIds].sort(),
    [...filter.models].sort(),
    filter.sessionId.trim(),
    filter.credentialId.trim(),
  ]);
}
