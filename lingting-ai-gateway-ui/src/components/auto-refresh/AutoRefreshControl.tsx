import { Checkbox, type CheckboxChangeEvent, Flex, InputNumber } from "antd";
import { useCallback } from "react";

import {
  type AutoRefreshSetting,
  MIN_AUTO_REFRESH_SECONDS,
  normalizeAutoRefreshSeconds,
} from "./autoRefreshSetting";

export type AutoRefreshControlProps = {
  onChange: (setting: AutoRefreshSetting) => void;
  setting: AutoRefreshSetting;
};

/**
 * 自动刷新开关：勾选后按设定秒数定时刷新，设置本身由调用方持有。
 *
 * 勾选后间隔输入框禁用，避免运行中改动间隔。
 */
export function AutoRefreshControl({ onChange, setting }: AutoRefreshControlProps) {
  const handleEnabledChange = useCallback(
    (event: CheckboxChangeEvent) => onChange({ ...setting, enabled: event.target.checked }),
    [onChange, setting],
  );

  const handleSecondsChange = useCallback(
    (value: number | string | null) =>
      onChange({ ...setting, seconds: normalizeAutoRefreshSeconds(value) }),
    [onChange, setting],
  );

  return (
    <Flex align="center" gap="small">
      <Checkbox checked={setting.enabled} onChange={handleEnabledChange}>
        自动刷新
      </Checkbox>
      <InputNumber
        disabled={setting.enabled}
        min={MIN_AUTO_REFRESH_SECONDS}
        onChange={handleSecondsChange}
        precision={0}
        suffix="秒"
        value={setting.seconds}
      />
    </Flex>
  );
}
