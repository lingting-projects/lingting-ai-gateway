import {
  type AutoRefreshSetting,
  DEFAULT_AUTO_REFRESH_SECONDS,
  normalizeAutoRefreshSeconds,
} from "@/components/auto-refresh";

/** 自动刷新设置的本地存储键。 */
const STORAGE_KEY = "lingting-ai-gateway:request-auto-refresh";

/** 请求日志页默认开启自动刷新。 */
const DEFAULT_ENABLED = true;

const DEFAULT_SETTING: AutoRefreshSetting = {
  enabled: DEFAULT_ENABLED,
  seconds: DEFAULT_AUTO_REFRESH_SECONDS,
};

/** 读取自动刷新设置：缺失、损坏或非法时逐项回落到默认值。 */
export function readAutoRefreshSetting(): AutoRefreshSetting {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return DEFAULT_SETTING;
    }

    const stored = JSON.parse(raw) as Partial<AutoRefreshSetting>;
    return {
      enabled: typeof stored.enabled === "boolean" ? stored.enabled : DEFAULT_ENABLED,
      seconds: normalizeAutoRefreshSeconds(stored.seconds),
    };
  } catch {
    return DEFAULT_SETTING;
  }
}

/** 写入自动刷新设置：本地存储不可用时忽略。 */
export function saveAutoRefreshSetting(setting: AutoRefreshSetting): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(setting));
  } catch {
    // 本地存储不可用时仅保留内存状态
  }
}
