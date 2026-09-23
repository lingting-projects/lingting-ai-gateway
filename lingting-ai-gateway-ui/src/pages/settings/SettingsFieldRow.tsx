import { UndoOutlined } from "@ant-design/icons";
import { KvConfigKeyMap } from "@lingting/ai-gateway-sdk";
import { Button } from "@lri";
import { Flex, Input, InputNumber, Switch, Typography } from "antd";
import { useCallback, useMemo } from "react";

import type { SettingsField } from "./settingsFields";
import { isTrueValue } from "./settingsUtils";

import "./settings.css";

type SettingsFieldRowProps = {
  /** 该字段是否已变更，未变更时重置按钮禁用。 */
  changed: boolean;
  field: SettingsField;
  onChange: (value: string) => void;
  onReset: () => void;
  value: string;
};

/** 单个配置项：左侧名称与说明，右侧编辑控件与重置按钮。 */
export function SettingsFieldRow({
  changed,
  field,
  onChange,
  onReset,
  value,
}: SettingsFieldRowProps) {
  const meta = KvConfigKeyMap[field.key];

  const handleInputChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => onChange(event.target.value),
    [onChange],
  );
  const handleNumberChange = useCallback(
    (next: number | string | null) =>
      onChange(next === null || next === undefined ? "" : String(next)),
    [onChange],
  );
  const handleSwitchChange = useCallback(
    (checked: boolean) => onChange(String(checked)),
    [onChange],
  );

  const control = useMemo(() => {
    if (field.type === "switch") {
      return <Switch checked={isTrueValue(value)} onChange={handleSwitchChange} />;
    }
    if (field.type === "number") {
      return (
        <InputNumber
          className="settings-field-input"
          max={field.max}
          min={field.min}
          onChange={handleNumberChange}
          value={value === "" ? null : Number(value)}
        />
      );
    }
    if (field.type === "secret") {
      return (
        <Input.Password
          className="settings-field-input"
          onChange={handleInputChange}
          value={value}
        />
      );
    }
    return <Input className="settings-field-input" onChange={handleInputChange} value={value} />;
  }, [
    field.max,
    field.min,
    field.type,
    handleInputChange,
    handleNumberChange,
    handleSwitchChange,
    value,
  ]);

  return (
    <Flex align="center" className="settings-field" gap="middle" justify="space-between">
      <Flex className="settings-field-label" gap={2} vertical>
        <Typography.Text strong>{meta.label}</Typography.Text>
        <Typography.Text type="secondary">{meta.description}</Typography.Text>
      </Flex>
      <Flex align="center" className="settings-field-control" gap="small" justify="end">
        {control}
        <Button
          disabled={!changed}
          icon={<UndoOutlined />}
          onClick={onReset}
          tooltip="重置为已保存值"
        />
      </Flex>
    </Flex>
  );
}
