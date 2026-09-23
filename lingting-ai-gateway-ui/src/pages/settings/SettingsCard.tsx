import { ProCard } from "@ant-design/pro-components";
import type { KvConfigKey } from "@lingting/ai-gateway-sdk";
import { AppHolder, Button } from "@lri";
import { Flex } from "antd";
import { useCallback, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";

import { SettingsFieldRow } from "./SettingsFieldRow";
import type { SettingsField } from "./settingsFields";
import {
  changedFields,
  clearOverrides,
  fieldValue,
  isChanged,
  type SettingsOverrides,
  type SettingsValues,
  toUpdateParams,
} from "./settingsUtils";

import "./settings.css";

/** 管理令牌字段：置空保存等于关闭令牌校验，需要二次确认。 */
const ADMIN_TOKEN_KEY: KvConfigKey = "ADMIN_TOKEN";

/** 令牌置空后的确认提示。 */
const ADMIN_TOKEN_CONFIRM = "管理令牌将置空，保存后管理接口不再校验令牌，任何人都可访问。";

type SettingsCardProps = {
  fields: SettingsField[];
  onOverridesChange: (overrides: SettingsOverrides) => void;
  onSaved: () => void;
  overrides: SettingsOverrides;
  title: string;
  values: SettingsValues;
};

/**
 * 配置卡片：字段直接编辑，底部在有变更时出现取消与保存。
 *
 * 保存只提交本卡片中已变更的字段；取消恢复卡片内全部字段；字段尾部的重置按钮恢复单个字段。
 */
export function SettingsCard({
  fields,
  onOverridesChange,
  onSaved,
  overrides,
  title,
  values,
}: SettingsCardProps) {
  const [saving, setSaving] = useState(false);

  const changed = useMemo(
    () => changedFields(fields, values, overrides),
    [fields, overrides, values],
  );

  const save = useCallback(
    async (params: ReturnType<typeof toUpdateParams>, keys: KvConfigKey[]) => {
      setSaving(true);
      try {
        await bizApi.kvConfigUpsertBatch(params);
        AppHolder.message.success("保存成功");
        // 先清掉已保存字段的覆盖值再重新拉取：令牌等字段服务端存的是摘要，
        // 保留覆盖值会让回显与变更态停留在已提交的明文上。
        onOverridesChange(clearOverrides(overrides, keys));
        onSaved();
      } catch (error) {
        AppHolder.message.error(error instanceof Error ? error.message : "保存失败");
      } finally {
        setSaving(false);
      }
    },
    [onOverridesChange, onSaved, overrides],
  );

  const handleSave = useCallback(() => {
    const params = toUpdateParams(changed, values, overrides);
    const keys = changed.map((field) => field.key);
    const token = changed.find((field) => field.key === ADMIN_TOKEN_KEY);
    if (token && fieldValue(values, overrides, ADMIN_TOKEN_KEY).trim() === "") {
      AppHolder.modal.confirm({
        content: ADMIN_TOKEN_CONFIRM,
        okText: "确认保存",
        onOk: () => save(params, keys),
        title: "确认关闭令牌校验？",
      });
      return;
    }
    void save(params, keys);
  }, [changed, overrides, save, values]);

  const handleCancel = useCallback(() => {
    onOverridesChange(
      clearOverrides(
        overrides,
        fields.map((field) => field.key),
      ),
    );
  }, [fields, onOverridesChange, overrides]);

  const handleChange = useCallback(
    (key: KvConfigKey, value: string) => {
      onOverridesChange({ ...overrides, [key]: value });
    },
    [onOverridesChange, overrides],
  );

  const handleReset = useCallback(
    (key: KvConfigKey) => {
      onOverridesChange(clearOverrides(overrides, [key]));
    },
    [onOverridesChange, overrides],
  );

  return (
    <ProCard className="settings-card" title={title}>
      <Flex gap="middle" vertical>
        {fields.map((field) => (
          <SettingsFieldRow
            changed={isChanged(values, overrides, field.key)}
            field={field}
            key={field.key}
            onChange={(value) => handleChange(field.key, value)}
            onReset={() => handleReset(field.key)}
            value={fieldValue(values, overrides, field.key)}
          />
        ))}
        {changed.length > 0 && (
          <Flex className="settings-card-actions" gap="small" justify="end">
            <Button onClick={handleCancel} text="取消" />
            <Button loading={saving} onClick={handleSave} text="保存" type="primary" />
          </Flex>
        )}
      </Flex>
    </ProCard>
  );
}
