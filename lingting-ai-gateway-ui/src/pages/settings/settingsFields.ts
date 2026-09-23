import type { KvConfigKey } from "@lingting/ai-gateway-sdk";

/** 配置项控件类型。 */
export type SettingsFieldType = "number" | "secret" | "switch" | "text";

/** 卡片内的单个配置项；名称与说明取自 KvConfigKeyMap。 */
export type SettingsField = {
  key: KvConfigKey;
  type: SettingsFieldType;
  /** 数字控件的最大值。 */
  max?: number;
  /** 数字控件的最小值。 */
  min?: number;
};

/** 服务绑定配置字段。 */
export const BIND_FIELDS: SettingsField[] = [
  { key: "BIND_ADDRESS", type: "text" },
  { key: "BIND_PORT", max: 65535, min: 0, type: "number" },
];

/** 安全配置字段。 */
export const SECURITY_FIELDS: SettingsField[] = [
  { key: "ALLOW_ANONYMOUS", type: "switch" },
  { key: "DEBUG_MODE", type: "switch" },
  { key: "ADMIN_TOKEN", type: "secret" },
];
