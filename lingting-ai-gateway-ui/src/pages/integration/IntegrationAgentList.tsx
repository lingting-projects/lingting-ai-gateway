import { ListCard } from "@lri";
import { Flex, Typography } from "antd";
import clsx from "clsx";
import { useCallback } from "react";

import type { IntegrationAgent } from "./integrationAgents";

/** Agent 卡片高度，与卡片内容高度保持一致。 */
const AGENT_ITEM_HEIGHT = 72;

type IntegrationAgentListProps = {
  agents: IntegrationAgent[];
  onSelect: (agent: IntegrationAgent) => void;
  selectedKey?: string | null;
};

/**
 * 集成 agent 列表：展示名称与官网地址，点击地址在新标签打开官网，点击卡片选中该 agent。
 */
export function IntegrationAgentList({ agents, onSelect, selectedKey }: IntegrationAgentListProps) {
  const renderItem = useCallback(
    (agent: IntegrationAgent) => (
      <Flex
        className={clsx(
          "integration-agent-item",
          agent.key === selectedKey && "integration-agent-item-selected",
        )}
        gap="small"
        justify="space-between"
        onClick={() => onSelect(agent)}
        vertical
      >
        <Typography.Text ellipsis strong>
          {agent.name}
        </Typography.Text>
        <Typography.Link
          ellipsis
          href={agent.url}
          onClick={(event) => event.stopPropagation()}
          target="_blank"
        >
          {agent.url}
        </Typography.Link>
      </Flex>
    ),
    [onSelect, selectedKey],
  );

  return (
    <Flex className="integration-agent-list" vertical>
      <ListCard
        data={agents}
        itemHeight={AGENT_ITEM_HEIGHT}
        itemRender={renderItem}
        props={{ ghost: true }}
      />
    </Flex>
  );
}
