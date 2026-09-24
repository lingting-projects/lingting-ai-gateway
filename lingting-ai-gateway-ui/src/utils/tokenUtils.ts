/** Token 数量展示：按 K / M 单位缩写，避免数值过长。 */
export function formatTokenCount(value: string | number | null | undefined): string {
  const count = Number(value ?? 0);
  if (!Number.isFinite(count)) {
    return "-";
  }
  if (Math.abs(count) >= 1_000_000) {
    return `${formatUnitValue(count / 1_000_000)}M`;
  }
  if (Math.abs(count) >= 1000) {
    return `${formatUnitValue(count / 1000)}K`;
  }
  return String(count);
}

/** 缓存占比：缓存(读+写)占总 Token 的百分比，最多保留 2 位小数。 */
export function formatCachePercent(
  cache: string | number | null | undefined,
  total: string | number | null | undefined,
): string {
  const totalCount = Number(total ?? 0);
  if (!Number.isFinite(totalCount) || totalCount <= 0) {
    return "0%";
  }

  const cacheCount = Number(cache ?? 0);
  if (!Number.isFinite(cacheCount)) {
    return "0%";
  }
  return `${Number(((cacheCount / totalCount) * 100).toFixed(2))}%`;
}

/** 单位数值保留一位小数并去掉多余的 0。 */
function formatUnitValue(value: number): string {
  return String(Math.round(value * 10) / 10);
}
