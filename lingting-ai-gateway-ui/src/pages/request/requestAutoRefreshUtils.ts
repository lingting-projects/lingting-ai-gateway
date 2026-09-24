/** 自动刷新设置的本地存储键。 */
const STORAGE_KEY = "lingting-ai-gateway:request-auto-refresh";

/** 自动刷新默认开启。 */
const DEFAULT_ENABLED = true;

/** 自动刷新默认间隔，单位秒。 */
const DEFAULT_SECONDS = 3;

/** 自动刷新允许的最小间隔，单位秒。 */
export const MIN_AUTO_REFRESH_SECONDS = 1;

export type RequestAutoRefreshSetting = {
  /** 是否开启自动刷新 */
  enabled: boolean;
  /** 刷新间隔，单位秒 */
  seconds: number;
};

const DEFAULT_SETTING: RequestAutoRefreshSetting = {
  enabled: DEFAULT_ENABLED,
  seconds: DEFAULT_SECONDS,
};

/** 归一化间隔：非数字或低于下限时回落到默认值。 */
export function normalizeAutoRefreshSeconds(value: unknown): number {
  const seconds = Number(value);
  if (!Number.isFinite(seconds) || seconds < MIN_AUTO_REFRESH_SECONDS) {
    return DEFAULT_SECONDS;
  }
  return seconds;
}

/** 读取自动刷新设置：缺失、损坏或非法时逐项回落到默认值。 */
export function readAutoRefreshSetting(): RequestAutoRefreshSetting {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return DEFAULT_SETTING;
    }

    const stored = JSON.parse(raw) as Partial<RequestAutoRefreshSetting>;
    return {
      enabled: typeof stored.enabled === "boolean" ? stored.enabled : DEFAULT_ENABLED,
      seconds: normalizeAutoRefreshSeconds(stored.seconds),
    };
  } catch {
    return DEFAULT_SETTING;
  }
}

/** 写入自动刷新设置：本地存储不可用时忽略。 */
export function saveAutoRefreshSetting(setting: RequestAutoRefreshSetting): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(setting));
  } catch {
    // 本地存储不可用时仅保留内存状态
  }
}
