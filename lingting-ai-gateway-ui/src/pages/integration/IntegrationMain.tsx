import { useQuery } from "@tanstack/react-query";
import { AppHolder } from "@lri";
import { Flex } from "antd";
import { useCallback, useEffect, useMemo, useState } from "react";

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

  const selected = useMemo(
    () => INTEGRATION_AGENTS.find((agent) => agent.key === selectedKey) ?? null,
    [selectedKey],
  );

  /** 同步目标：选中 agent 后加载服务端账户，未选中时为空列表。 */
  const { data: serverTargets, error } = useQuery({
    enabled: selected !== null,
    queryFn: () => selected?.loadSyncTargets() ?? Promise.resolve([]),
    queryKey: ["integration-sync-targets", selectedKey],
    retry: false,
  });

  useEffect(() => {
    if (error) {
      AppHolder.message.error(error.message);
    }
  }, [error]);

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
        serverTargets={serverTargets ?? []}
      />
    </Flex>
  );
}
