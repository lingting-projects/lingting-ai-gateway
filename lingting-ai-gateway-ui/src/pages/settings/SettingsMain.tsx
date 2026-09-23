import { useQuery } from "@tanstack/react-query";
import { Flex } from "antd";
import { useCallback, useEffect, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";
import { AppHolder } from "@lri";

import { SettingsCard } from "./SettingsCard";
import { BIND_FIELDS, SECURITY_FIELDS } from "./settingsFields";
import { type SettingsOverrides, toSettingsValues } from "./settingsUtils";

import "./settings.css";

/** 配置项固定 5 条，一页取完。 */
const SETTINGS_PAGE_SIZE = 100;

/** 配置项全量查询条件：不限配置键。 */
const SETTINGS_QUERY = {};

/**
 * 软件设置：服务绑定配置与安全配置各一张卡片，字段可直接编辑，变更后卡片底部出现取消与保存。
 */
export function SettingsMain() {
  const [overrides, setOverrides] = useState<SettingsOverrides>({});

  const { data, error, refetch } = useQuery({
    queryFn: () =>
      bizApi.kvConfigPage({ current: 1, size: SETTINGS_PAGE_SIZE, sorts: [] }, SETTINGS_QUERY),
    queryKey: ["settings-kv-config"],
    retry: false,
  });

  useEffect(() => {
    if (error) {
      AppHolder.message.error(error.message);
    }
  }, [error]);

  const values = useMemo(() => toSettingsValues(data?.records ?? []), [data]);
  // 保存后重新拉取，服务端值追平覆盖值，变更态与底部按钮随之消失，无需清空覆盖值造成回显跳变
  const handleSaved = useCallback(() => void refetch(), [refetch]);

  return (
    <Flex className="settings-main" gap="middle" vertical>
      <SettingsCard
        fields={BIND_FIELDS}
        onOverridesChange={setOverrides}
        onSaved={handleSaved}
        overrides={overrides}
        title="服务绑定配置"
        values={values}
      />
      <SettingsCard
        fields={SECURITY_FIELDS}
        onOverridesChange={setOverrides}
        onSaved={handleSaved}
        overrides={overrides}
        title="安全配置"
        values={values}
      />
    </Flex>
  );
}
