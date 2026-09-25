import dayjs, { type Dayjs } from "dayjs";

/** 快捷时间范围。 */
export type DashboardRangeKey = "last7" | "last30" | "week" | "month" | "custom";

/** 快捷时间范围选项。 */
export const DASHBOARD_RANGE_OPTIONS: { label: string; value: DashboardRangeKey }[] = [
  { label: "近7天", value: "last7" },
  { label: "近30天", value: "last30" },
  { label: "本周", value: "week" },
  { label: "本月", value: "month" },
  { label: "自定义", value: "custom" },
];

/** 默认选中的时间范围。 */
export const DEFAULT_DASHBOARD_RANGE: DashboardRangeKey = "last7";

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
