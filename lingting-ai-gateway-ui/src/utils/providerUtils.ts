import dayjs from "dayjs";
import type { ProviderDetailVO } from "@lingting/ai-gateway-sdk";

import { includesKeyword } from "./textUtils";
import { formatTokenCount as formatTokenAmount } from "./tokenUtils";

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
  return formatTokenAmount(count);
}
