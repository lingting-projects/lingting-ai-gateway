import type { ProviderModelVO } from "@lingting/ai-gateway-sdk";
import { DictTag } from "@lri";
import { Flex, Tooltip, Typography } from "antd";
import clsx from "clsx";

import { formatProviderMillis, formatTokenCount } from "@/utils/providerUtils";

import "./ModelItem.css";

const ENABLED_DICT = [
  { value: true, label: "启用", tagColor: "success" },
  { value: false, label: "禁用", tagColor: "default" },
];

const REASONING_DICT = [
  { value: true, label: "支持推理", tagColor: "processing" },
  { value: false, label: "不支持推理", tagColor: "default" },
];

export type ModelItemProps = {
  model: ProviderModelVO;
  onClick?: () => void;
  selected?: boolean;
};

/**
 * 模型卡片：展示模型名称与展示名称、推理与启用状态、默认推理等级、上下文与时间信息。
 *
 * 默认推理等级悬浮时展示该模型支持的全部推理等级。
 */
export function ModelItem({ model, onClick, selected }: ModelItemProps) {
  const levelDefault = model.levelDefault || "-";

  return (
    <Flex
      className={clsx(
        "model-item",
        Boolean(onClick) && "model-item-clickable",
        selected && "model-item-selected",
      )}
      gap="small"
      justify="space-between"
      onClick={onClick}
      vertical
    >
      <Flex align="center" gap="small" justify="space-between">
        <Flex align="center" className="model-item-title" gap="small">
          <Typography.Text className="model-item-name" ellipsis strong>
            {model.model}
          </Typography.Text>
          <Typography.Text className="model-item-name" ellipsis type="secondary">
            {model.displayName}
          </Typography.Text>
        </Flex>
        <Flex align="center" className="model-item-tags" gap="small">
          <DictTag dict={REASONING_DICT} value={model.reasoning}/>
          <DictTag dict={ENABLED_DICT} value={model.enabled}/>
        </Flex>
      </Flex>
      <Flex align="center" gap="small" justify="space-between">
        <Tooltip
          title={model.levels.length > 0 ? `全部推理等级：${model.levels.join("、")}` : undefined}
        >
          <Typography.Text type="secondary">默认推理等级 {levelDefault}</Typography.Text>
        </Tooltip>
        <Typography.Text type="secondary">
          上下文 {formatTokenCount(model.contextWindow)}
        </Typography.Text>
      </Flex>
      <Flex align="center" gap="small" justify="space-between">
        <Typography.Text type="secondary">
          知识截止 {formatProviderMillis(model.knowledgeCutoff)}
        </Typography.Text>
        <Typography.Text type="secondary">
          发布日期 {formatProviderMillis(model.releaseDate)}
        </Typography.Text>
      </Flex>
    </Flex>
  );
}
