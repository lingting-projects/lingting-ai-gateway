import { AppHolder } from "@lri";
import { Flex } from "antd";
import { useCallback, useMemo, useState } from "react";

import { IntegrationAgentList } from "./IntegrationAgentList";
import { IntegrationConfigPanel } from "./IntegrationConfigPanel";
import { INTEGRATION_AGENTS, type IntegrationAgent } from "./integrationAgents";

import "./integration.css";

/**
 * 集成：左侧 agent 列表，右侧展示选中 agent 的配置预览与同步入口。
 */
export function IntegrationMain() {
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [config, setConfig] = useState("");
  const [loading, setLoading] = useState(false);
  const [syncing, setSyncing] = useState(false);

  const selected = useMemo(
    () => INTEGRATION_AGENTS.find((agent) => agent.key === selectedKey) ?? null,
    [selectedKey],
  );

  const handleSelect = useCallback(async (agent: IntegrationAgent) => {
    setSelectedKey(agent.key);
    setLoading(true);
    try {
      setConfig(await agent.exportConfig());
    } catch (error) {
      setConfig("");
      void AppHolder.message.error(error instanceof Error ? error.message : "获取配置失败");
    } finally {
      setLoading(false);
    }
  }, []);

  const handleSync = useCallback(async () => {
    if (!selected) {
      return;
    }

    setSyncing(true);
    try {
      const backup = await selected.sync();
      void AppHolder.message.success(backup ? `同步成功，原配置已备份至 ${backup}` : "同步成功");
    } catch (error) {
      void AppHolder.message.error(error instanceof Error ? error.message : "同步失败");
    } finally {
      setSyncing(false);
    }
  }, [selected]);

  return (
    <Flex className="integration-main" gap="middle">
      <IntegrationAgentList
        agents={INTEGRATION_AGENTS}
        onSelect={handleSelect}
        selectedKey={selectedKey}
      />
      <IntegrationConfigPanel
        agent={selected}
        config={config}
        loading={loading}
        onSync={handleSync}
        syncing={syncing}
      />
    </Flex>
  );
}
