import { SyncOutlined } from "@ant-design/icons";
import type { ProviderDetailVO } from "@lingting/ai-gateway-sdk";
import { AppHolder, Button, ListCard } from "@lri";
import { Flex, Typography } from "antd";
import { useCallback, useState } from "react";

import { bizApi } from "@/api/BizApi";
import { ModelItem } from "@/components/model";

/** 模型卡片高度，与卡片内容高度保持一致。 */
const MODEL_ITEM_HEIGHT = 104;

type ProviderModelListProps = {
  detail: ProviderDetailVO;
};

/**
 * 供应商模型列表：标题展示数量与同步入口，卡片复用 ModelItem 展示模型信息。
 */
export function ProviderModelList({ detail }: ProviderModelListProps) {
  const models = detail.models;
  const [syncing, setSyncing] = useState(false);

  const handleSync = useCallback(async () => {
    setSyncing(true);
    try {
      await bizApi.providerUpdateModel({ id: detail.provider.id });
      AppHolder.message.success("同步任务已启动，请稍后刷新供应商列表");
    } catch (error) {
      AppHolder.message.error(error instanceof Error ? error.message : "同步失败");
    } finally {
      setSyncing(false);
    }
  }, [detail.provider.id]);

  const renderItem = useCallback(
    (model: (typeof models)[number]) => <ModelItem model={model}/>,
    [],
  );

  const header = (
    <Flex align="center" className="provider-model-header" gap="middle">
      <Typography.Text type="secondary">共 {models.length} 个模型</Typography.Text>
      <Button
        icon={<SyncOutlined/>}
        loading={syncing}
        onClick={handleSync}
        tooltip="同步供应商模型"
      />
    </Flex>
  );

  return (
    <Flex className="provider-model-list" vertical>
      <ListCard
        data={models}
        header={header}
        itemHeight={MODEL_ITEM_HEIGHT}
        itemRender={renderItem}
      />
    </Flex>
  );
}
