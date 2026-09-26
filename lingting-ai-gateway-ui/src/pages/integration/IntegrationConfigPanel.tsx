import { CodeHighlighter } from "@ant-design/x";
import { ProCard } from "@ant-design/pro-components";
import { Flex, Typography } from "antd";

import type { IntegrationAgent, IntegrationSyncTarget } from "./integrationAgents";
import { IntegrationSyncButton } from "./IntegrationSyncButton";

type IntegrationConfigPanelProps = {
  agent: IntegrationAgent | null;
  config: string;
  loading?: boolean;
  /** 服务端提供的同步目标 */
  serverTargets: IntegrationSyncTarget[];
};

/**
 * 集成配置面板：头部展示选中 agent 名称与同步入口，底部以代码块预览导出的配置。
 */
export function IntegrationConfigPanel({
  agent,
  config,
  loading,
  serverTargets,
}: IntegrationConfigPanelProps) {
  return (
    <ProCard
      className="integration-config-panel"
      extra={
        agent && (
          <IntegrationSyncButton
            agent={agent}
            key={agent.key}
            serverTargets={serverTargets}
          />
        )
      }
      loading={loading}
      title={agent?.name}
    >
      {!agent && (
        <Flex align="center" className="integration-config-empty" justify="center">
          <Typography.Text type="secondary">请选择左侧 Agent 查看配置</Typography.Text>
        </Flex>
      )}
      {agent && <CodeHighlighter lang="json">{config}</CodeHighlighter>}
    </ProCard>
  );
}
