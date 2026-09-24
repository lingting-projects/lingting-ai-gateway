import { SyncOutlined } from "@ant-design/icons";
import { CodeHighlighter } from "@ant-design/x";
import { ProCard } from "@ant-design/pro-components";
import { Button } from "@lri";
import { Flex, Typography } from "antd";

import type { IntegrationAgent } from "./integrationAgents";

type IntegrationConfigPanelProps = {
  agent: IntegrationAgent | null;
  config: string;
  loading?: boolean;
  onSync: () => void;
  syncing?: boolean;
};

/**
 * 集成配置面板：头部展示选中 agent 名称与同步入口，底部以代码块预览导出的配置。
 */
export function IntegrationConfigPanel({
                                         agent,
                                         config,
                                         loading,
                                         onSync,
                                         syncing,
                                       }: IntegrationConfigPanelProps) {
  return (
    <ProCard
      className="integration-config-panel"
      extra={
        agent && <Button icon={<SyncOutlined/>} loading={syncing} onClick={onSync} text="同步"/>
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
