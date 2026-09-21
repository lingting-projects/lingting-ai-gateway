import dayjs from "dayjs";
import type { ProviderDetailVO } from "@lingting/ai-gateway-sdk";

import { includesKeyword } from "./textUtils";

/** 供应商搜索：匹配供应商名称、接口地址与模型名称。 */
export function filterProviderDetails(
  details: ProviderDetailVO[],
  keyword: string,
): ProviderDetailVO[] {
  const value = keyword.trim().toLowerCase();
  if (!value) {
    return details;
  }

  return details.filter((detail) => {
    const { models, provider } = detail;
    if (
      includesKeyword(provider.name, value) ||
      includesKeyword(provider.displayName, value) ||
      includesKeyword(provider.baseUrl, value)
    ) {
      return true;
    }
    return models.some(
      (model) => includesKeyword(model.model, value) || includesKeyword(model.displayName, value),
    );
  });
}

/** 毫秒时间戳展示，0 表示未知。 */
export function formatProviderMillis(value: string): string {
  const millis = Number(value);
  if (!Number.isFinite(millis) || millis <= 0) {
    return "-";
  }
  return dayjs(millis).format("YYYY-MM-DD HH:mm:ss");
}

/** token 数量展示：按 K / M 单位缩写，0 表示未知。 */
export function formatTokenCount(value: string): string {
  const count = Number(value);
  if (!Number.isFinite(count) || count <= 0) {
    return "-";
  }
  if (count >= 1_000_000) {
    return `${formatUnitValue(count / 1_000_000)}M`;
  }
  if (count >= 1000) {
    return `${formatUnitValue(count / 1000)}K`;
  }
  return String(count);
}

/** 单位数值保留一位小数并去掉多余的 0。 */
function formatUnitValue(value: number): string {
  return String(Math.round(value * 10) / 10);
}
