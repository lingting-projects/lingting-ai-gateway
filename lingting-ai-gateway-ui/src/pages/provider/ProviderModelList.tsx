import type { ProviderDetailVO } from "@lingting/ai-gateway-sdk";
import { ListCard } from "@lri";
import { Flex } from "antd";
import { useCallback } from "react";

import { ModelItem } from "@/components/model";

/** 模型卡片高度，与卡片内容高度保持一致。 */
const MODEL_ITEM_HEIGHT = 104;

type ProviderModelListProps = {
  detail: ProviderDetailVO;
};

/**
 * 供应商模型列表：卡片复用 ModelItem 展示模型信息。
 */
export function ProviderModelList({ detail }: ProviderModelListProps) {
  const models = detail.models;

  const renderItem = useCallback(
    (model: (typeof models)[number]) => <ModelItem model={model}/>,
    [],
  );

  return (
    <Flex className="provider-model-list" vertical>
      <ListCard
        data={models}
        itemHeight={MODEL_ITEM_HEIGHT}
        itemRender={renderItem}
        props={{ ghost: true }}
      />
    </Flex>
  );
}
