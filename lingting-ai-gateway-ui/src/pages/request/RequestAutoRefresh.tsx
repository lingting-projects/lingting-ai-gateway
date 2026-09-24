import { Checkbox, type CheckboxChangeEvent, Flex, InputNumber } from "antd";
import { useCallback, useEffect, useState } from "react";

import {
  MIN_AUTO_REFRESH_SECONDS,
  normalizeAutoRefreshSeconds,
  readAutoRefreshSetting,
  saveAutoRefreshSetting,
} from "./requestAutoRefreshUtils";

type RequestAutoRefreshProps = {
  /** 到达设定间隔时刷新表格数据 */
  onRefresh: () => void;
};

/**
 * 自动刷新开关：勾选后按设定秒数定时刷新表格，设置持久化在本地存储。
 *
 * 勾选后间隔输入框禁用，避免刷新过程中改动间隔。
 */
export function RequestAutoRefresh({ onRefresh }: RequestAutoRefreshProps) {
  const [setting, setSetting] = useState(readAutoRefreshSetting);

  useEffect(() => {
    saveAutoRefreshSetting(setting);
  }, [setting]);

  useEffect(() => {
    if (!setting.enabled) {
      return;
    }

    const timer = window.setInterval(onRefresh, setting.seconds * 1000);
    return () => window.clearInterval(timer);
  }, [onRefresh, setting]);

  const handleEnabledChange = useCallback((event: CheckboxChangeEvent) => {
    setSetting((current) => ({ ...current, enabled: event.target.checked }));
  }, []);

  const handleSecondsChange = useCallback((value: number | string | null) => {
    setSetting((current) => ({ ...current, seconds: normalizeAutoRefreshSeconds(value) }));
  }, []);

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
