import { Checkbox, type CheckboxChangeEvent, Flex, InputNumber } from "antd";
import { useCallback } from "react";

import {
  MIN_AUTO_REFRESH_SECONDS,
  normalizeAutoRefreshSeconds,
  type RequestAutoRefreshSetting,
} from "./requestAutoRefreshUtils";

type RequestAutoRefreshProps = {
  onChange: (setting: RequestAutoRefreshSetting) => void;
  setting: RequestAutoRefreshSetting;
};

/**
 * 自动刷新开关：设置表格轮询间隔，设置本身由调用方持久化。
 *
 * 勾选后间隔输入框禁用，避免轮询过程中改动间隔。
 */
export function RequestAutoRefresh({ onChange, setting }: RequestAutoRefreshProps) {
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
