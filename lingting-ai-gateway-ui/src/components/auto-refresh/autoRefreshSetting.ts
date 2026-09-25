/** 自动刷新默认间隔，单位秒。 */
export const DEFAULT_AUTO_REFRESH_SECONDS = 3;

/** 自动刷新允许的最小间隔，单位秒。 */
export const MIN_AUTO_REFRESH_SECONDS = 2;

/** 自动刷新设置。 */
export type AutoRefreshSetting = {
  /** 是否开启自动刷新 */
  enabled: boolean;
  /** 刷新间隔，单位秒 */
  seconds: number;
};

/** 归一化间隔：非数字或低于下限时回落到默认间隔。 */
export function normalizeAutoRefreshSeconds(value: unknown): number {
  const seconds = Number(value);
  if (!Number.isFinite(seconds) || seconds < MIN_AUTO_REFRESH_SECONDS) {
    return DEFAULT_AUTO_REFRESH_SECONDS;
  }
  return seconds;
}
