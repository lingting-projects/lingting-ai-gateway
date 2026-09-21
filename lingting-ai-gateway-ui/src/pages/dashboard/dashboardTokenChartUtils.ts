import type { DashboardProviderVO, DashboardTokenFilterVO } from "@lingting/ai-gateway-sdk";
import dayjs, { type Dayjs } from "dayjs";

/** 快捷时间范围。 */
export type DashboardRangeKey = "today" | "last7" | "last30" | "week" | "month" | "custom";

/** 快捷时间范围选项。 */
export const DASHBOARD_RANGE_OPTIONS: { label: string; value: DashboardRangeKey }[] = [
  { label: "当天", value: "today" },
  { label: "近7天", value: "last7" },
  { label: "近30天", value: "last30" },
  { label: "本周", value: "week" },
  { label: "本月", value: "month" },
  { label: "自定义", value: "custom" },
];

/** 默认选中的时间范围。 */
export const DEFAULT_DASHBOARD_RANGE: DashboardRangeKey = "last7";

/** 折线图数据点：同一天同一系列一个点。 */
export type DashboardTokenPoint = { day: string; series: string; value: number };

/** 折线指标：总、缓存、读、写。 */
const TOKEN_METRICS: { label: string; resolve: (row: DashboardTokenFilterVO) => number }[] = [
  { label: "总", resolve: (row) => Number(row.total) },
  { label: "缓存", resolve: (row) => Number(row.cacheRead) + Number(row.cacheWrite) },
  { label: "读", resolve: (row) => Number(row.inputTokens) },
  { label: "写", resolve: (row) => Number(row.outputTokens) },
];

/** 本周起始：按周一为一周起点，dayjs 默认以周日起始，此处手工回退。 */
function startOfWeek(now: Dayjs): Dayjs {
  return now.subtract((now.day() + 6) % 7, "day").startOf("day");
}

/**
 * 计算快捷范围的起止毫秒时间戳；自定义范围未选择时返回 null，调用方据此跳过查询。
 */
export function resolveRangeTime(
  range: DashboardRangeKey,
  customRange: [Dayjs, Dayjs] | null,
): [number, number] | null {
  const now = dayjs();
  if (range === "today") {
    return [now.startOf("day").valueOf(), now.valueOf()];
  }
  if (range === "last7") {
    return [now.subtract(6, "day").startOf("day").valueOf(), now.valueOf()];
  }
  if (range === "last30") {
    return [now.subtract(29, "day").startOf("day").valueOf(), now.valueOf()];
  }
  if (range === "week") {
    return [startOfWeek(now).valueOf(), now.valueOf()];
  }
  if (range === "month") {
    return [now.startOf("month").valueOf(), now.valueOf()];
  }
  if (!customRange) {
    return null;
  }
  return [customRange[0].startOf("day").valueOf(), customRange[1].endOf("day").valueOf()];
}

/** 供应商展示名：优先展示名，其次名称，均为空时回退主键。 */
function providerLabel(provider: DashboardProviderVO | null | undefined): string {
  if (!provider) {
    return "";
  }
  return provider.displayName || provider.name || `#${provider.id}`;
}

/**
 * 把分组统计结果摊平成折线数据点。
 *
 * 勾选分组后，每个指标按分组拆成独立系列，系列名为「分组 · 指标」。
 */
export function buildTokenPoints(
  rows: DashboardTokenFilterVO[],
  withProvider: boolean,
  withModel: boolean,
): DashboardTokenPoint[] {
  return rows.flatMap((row) => {
    const group = [
      withProvider ? providerLabel(row.provider) : "",
      withModel ? (row.model ?? "") : "",
    ]
      .filter(Boolean)
      .join(" · ");
    const prefix = group ? `${group} · ` : "";
    return TOKEN_METRICS.map((metric) => ({
      day: row.day,
      series: `${prefix}${metric.label}`,
      value: metric.resolve(row),
    }));
  });
}

/** 坐标轴与提示的数值展示：千分位。 */
export function formatTokenValue(value: number): string {
  return value.toLocaleString("zh-CN");
}
