import { ReloadOutlined, SearchOutlined } from "@ant-design/icons";
import type { ProviderModelVO } from "@lingting/ai-gateway-sdk";
import { Button, ListCard } from "@lri";
import { Flex, Input, Typography } from "antd";
import { useCallback, useState } from "react";

import { ModelItem } from "@/components/model";
import { includesKeyword } from "@/utils/textUtils";

/** 模型卡片高度，与卡片内容高度保持一致。 */
const MODEL_ITEM_HEIGHT = 104;

type ModelDefaultListProps = {
  models: ProviderModelVO[];
  loading?: boolean;
  onRefresh: () => void;
  onSelect: (model: ProviderModelVO) => void;
  refreshing?: boolean;
  selected?: ProviderModelVO | null;
  /** 右侧选中的映射目标，存在时忽略搜索仅展示指向该目标的模型。 */
  targetModel?: string;
};

/**
 * 默认模型配置列表：标题右侧刷新入口，搜索框筛选模型名称与展示名称。
 */
export function ModelDefaultList({
                                   loading,
                                   models,
                                   onRefresh,
                                   onSelect,
                                   refreshing,
                                   selected,
                                   targetModel,
                                 }: ModelDefaultListProps) {
  const [keyword, setKeyword] = useState("");

  const filter = useCallback(
    (items: ProviderModelVO[]) => {
      if (targetModel) {
        return items.filter((item) => item.model === targetModel);
      }

      const value = keyword.trim().toLowerCase();
      if (!value) {
        return items;
      }
      return items.filter(
        (item) => includesKeyword(item.model, value) || includesKeyword(item.displayName, value),
      );
    },
    [keyword, targetModel],
  );

  const renderItem = useCallback(
    (model: ProviderModelVO) => (
      <ModelItem
        model={model}
        onClick={() => onSelect(model)}
        selected={model.id === selected?.id}
      />
    ),
    [onSelect, selected],
  );

  const header = (
    <Flex className="model-panel-header" gap="small" vertical>
      <Flex align="center" gap="small" justify="space-between">
        <Typography.Text strong>默认模型配置</Typography.Text>
        <Button
          icon={<ReloadOutlined/>}
          loading={refreshing}
          onClick={onRefresh}
          tooltip="刷新默认模型配置"
        />
      </Flex>
      <Input
        allowClear
        onChange={(event) => setKeyword(event.target.value)}
        placeholder="搜索模型"
        prefix={<SearchOutlined/>}
        value={keyword}
      />
    </Flex>
  );

  return (
    <Flex className="model-panel" vertical>
      <ListCard
        data={models}
        filter={filter}
        header={header}
        itemHeight={MODEL_ITEM_HEIGHT}
        itemRender={renderItem}
        loading={loading}
        props={{ ghost: true }}
      />
    </Flex>
  );
}
