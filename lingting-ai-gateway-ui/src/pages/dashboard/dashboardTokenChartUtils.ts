import type { DashboardProviderVO, DashboardTokenFilterVO } from "@lingting/ai-gateway-sdk";
import dayjs from "dayjs";

/** 折线图数据点：同一天同一系列一个点。 */
export type DashboardTokenPoint = {
  day: string;
  series: string;
  value: number;
};

/** 折线指标：总、缓存、读、写。 */
const TOKEN_METRICS: {
  label: string;
  resolve: (row: DashboardTokenFilterVO) => number;
}[] = [
  { label: "总", resolve: (row) => Number(row.total) },
  { label: "缓存", resolve: (row) => Number(row.cacheRead) + Number(row.cacheWrite) },
  { label: "读", resolve: (row) => Number(row.input) },
  { label: "写", resolve: (row) => Number(row.output) },
];

/** 供应商展示名：优先展示名，其次名称，均为空时回退主键。 */
function providerLabel(provider: DashboardProviderVO | null | undefined): string {
  if (!provider) {
    return "";
  }
  return provider.displayName || provider.name || `#${provider.id}`;
}

/**
 * 规范化日期。
 *
 * 服务端一般返回 YYYY-MM-DD。
 * 如果服务端返回的是时间戳或带时间的字符串，也统一转换成 YYYY-MM-DD，
 * 保证同一天的数据可以正确归并。
 */
function normalizeDay(value: unknown): string {
  if (typeof value === "number") {
    return dayjs(value).format("YYYY-MM-DD");
  }

  if (typeof value === "string") {
    const parsed = dayjs(value);

    if (parsed.isValid()) {
      return parsed.format("YYYY-MM-DD");
    }

    return value;
  }

  return "";
}

/**
 * 根据查询时间范围生成完整的日期轴。
 *
 * 例如：
 *
 * 2026-09-20 ~ 2026-09-23
 *
 * 返回：
 *
 * [
 *   "2026-09-20",
 *   "2026-09-21",
 *   "2026-09-22",
 *   "2026-09-23",
 * ]
 */
function buildDays(rangeTime: [number, number]): string[] {
  const [startTime, endTime] = rangeTime;

  let current = dayjs(startTime).startOf("day");
  const end = dayjs(endTime).startOf("day");

  const days: string[] = [];

  while (current.isBefore(end) || current.isSame(end, "day")) {
    days.push(current.format("YYYY-MM-DD"));
    current = current.add(1, "day");
  }

  return days;
}

/**
 * 生成当前行对应的分组名称。
 *
 * 分组组合：
 *
 * 无分组：
 *   ""
 *
 * 供应商：
 *   "OpenAI"
 *
 * 模型：
 *   "gpt-5"
 *
 * 供应商 + 模型：
 *   "OpenAI · gpt-5"
 */
function resolveGroup(
  row: DashboardTokenFilterVO,
  withProvider: boolean,
  withModel: boolean,
): string {
  return [withProvider ? providerLabel(row.provider) : "", withModel ? (row.model ?? "") : ""]
    .filter(Boolean)
    .join(" · ");
}

/**
 * 根据分组名称和指标名称生成折线系列名称。
 *
 * 无分组：
 *   "总"
 *   "缓存"
 *   "读"
 *   "写"
 *
 * 供应商分组：
 *   "OpenAI · 总"
 *   "OpenAI · 缓存"
 *
 * 供应商 + 模型：
 *   "OpenAI · gpt-5 · 总"
 */
function resolveSeries(
  row: DashboardTokenFilterVO,
  withProvider: boolean,
  withModel: boolean,
  metricLabel: string,
): string {
  const group = resolveGroup(row, withProvider, withModel);
  return group ? `${group} · ${metricLabel}` : metricLabel;
}

/**
 * 把分组统计结果摊平成折线数据点，并补齐时间范围内缺失的日期。
 *
 * 核心规则：
 *
 * 1. 日期轴以 rangeTime 为准，而不是以后端返回的数据为准。
 * 2. 每个「日期 + 系列」只生成一个点。
 * 3. 后端没有返回的「日期 + 系列」自动补 0。
 * 4. 同一个「日期 + 系列」存在多条数据时进行累加。
 *
 * 例如服务端返回：
 *
 * [
 *   { day: "2026-09-20", provider: "A", total: 100 },
 *   { day: "2026-09-22", provider: "A", total: 300 },
 * ]
 *
 * rangeTime：
 *
 * 2026-09-20 ~ 2026-09-22
 *
 * 最终：
 *
 * [
 *   { day: "2026-09-20", series: "A · 总", value: 100 },
 *   { day: "2026-09-21", series: "A · 总", value: 0 },
 *   { day: "2026-09-22", series: "A · 总", value: 300 },
 * ]
 */
export function buildTokenPoints(
  rows: DashboardTokenFilterVO[],
  withProvider: boolean,
  withModel: boolean,
  rangeTime: [number, number] | null,
): DashboardTokenPoint[] {
  if (!rangeTime) {
    return [];
  }

  /**
   * 1. 生成完整日期轴。
   *
   * 这里不依赖 rows，因此即使服务端某一天完全没有返回数据，
   * 该日期仍然会出现在折线图中。
   */
  const days = buildDays(rangeTime);

  /**
   * 2. 建立所有实际存在的 series。
   *
   * series 仍然以服务端返回的数据为准。
   *
   * 例如后端只有 provider A：
   *
   *   A · 总
   *   A · 缓存
   *   A · 读
   *   A · 写
   *
   * 那么就只生成这一组 series。
   */
  const seriesSet = new Set<string>();

  /**
   * 3. 建立「day + series -> value」索引。
   *
   * Map key：
   *
   *   2026-09-20::OpenAI · 总
   *
   * value：
   *
   *   100
   */
  const valueMap = new Map<string, number>();

  for (const row of rows) {
    const day = normalizeDay(row.day);

    if (!day) {
      continue;
    }

    for (const metric of TOKEN_METRICS) {
      const series = resolveSeries(row, withProvider, withModel, metric.label);

      seriesSet.add(series);

      const key = `${day}::${series}`;
      const value = metric.resolve(row);

      /**
       * 如果同一天同一系列有多条记录，则累加。
       *
       * 这样可以避免最终生成多个相同 day + series 的点，
       * 同时也兼容服务端按更细粒度返回数据的情况。
       */
      valueMap.set(key, (valueMap.get(key) ?? 0) + value);
    }
  }

  /**
   * 没有任何服务端数据时，没有办法知道有哪些分组。
   *
   * 这种情况下：
   * - 如果没有分组，仍然可以生成「总 / 缓存 / 读 / 写」四条 0 线。
   * - 如果启用了供应商/模型分组，则无法凭空知道有哪些分组，
   *   因此不生成不存在的 series。
   */
  if (seriesSet.size === 0 && !withProvider && !withModel) {
    for (const metric of TOKEN_METRICS) {
      seriesSet.add(metric.label);
    }
  }

  /**
   * 4. 根据「完整日期轴 × 所有 series」生成最终数据。
   *
   * 这是整个补 0 逻辑的核心。
   *
   * 例如：
   *
   * days:
   *   20, 21, 22
   *
   * series:
   *   A · 总
   *   A · 缓存
   *
   * 最终一定生成：
   *
   *   20 A·总
   *   20 A·缓存
   *   21 A·总
   *   21 A·缓存
   *   22 A·总
   *   22 A·缓存
   *
   * 不存在的数据通过 ?? 0 补齐。
   */
  return days.flatMap((day) =>
    [...seriesSet].map((series) => ({
      day,
      series,
      value: valueMap.get(`${day}::${series}`) ?? 0,
    })),
  );
}
