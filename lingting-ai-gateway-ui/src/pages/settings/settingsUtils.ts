import type { KvConfigKey, KvConfigUpdatePO, KvConfigVO } from "@lingting/ai-gateway-sdk";

import type { SettingsField } from "./settingsFields";

/** 服务端配置值：配置键到配置值。 */
export type SettingsValues = Record<string, string>;

/** 用户编辑覆盖：只记录被修改过的配置键，未覆盖的字段始终跟随服务端值。 */
export type SettingsOverrides = Partial<Record<KvConfigKey, string>>;

/** 把分页结果整理成配置键到配置值的映射，值去掉首尾空白。 */
export function toSettingsValues(records: KvConfigVO[]): SettingsValues {
  return Object.fromEntries(records.map((record) => [record.configKey, record.configValue.trim()]));
}

/** 布尔配置值判断，与后端 string_is_true 保持一致。 */
export function isTrueValue(value: string): boolean {
  return ["t", "true", "y", "yes", "ok", "1"].includes(value.trim().toLowerCase());
}

/** 字段当前生效值：优先用户覆盖值，其次服务端值。 */
export function fieldValue(
  values: SettingsValues,
  overrides: SettingsOverrides,
  key: KvConfigKey,
): string {
  return overrides[key] ?? values[key] ?? "";
}

/** 字段是否已变更：被覆盖且与服务端值不同才算变更。 */
export function isChanged(
  values: SettingsValues,
  overrides: SettingsOverrides,
  key: KvConfigKey,
): boolean {
  const override = overrides[key];
  return override !== undefined && override !== (values[key] ?? "");
}

/** 取出已变更的字段。 */
export function changedFields(
  fields: SettingsField[],
  values: SettingsValues,
  overrides: SettingsOverrides,
): SettingsField[] {
  return fields.filter((field) => isChanged(values, overrides, field.key));
}

/** 构造批量更新参数，取字段当前生效值。 */
export function toUpdateParams(
  fields: SettingsField[],
  values: SettingsValues,
  overrides: SettingsOverrides,
): KvConfigUpdatePO[] {
  return fields.map((field) => ({
    configKey: field.key,
    configValue: fieldValue(values, overrides, field.key),
  }));
}

/** 移除已保存字段的覆盖值，使其回到服务端值。 */
export function clearOverrides(
  overrides: SettingsOverrides,
  keys: KvConfigKey[],
): SettingsOverrides {
  const next = { ...overrides };
  keys.forEach((key) => delete next[key]);
  return next;
}
