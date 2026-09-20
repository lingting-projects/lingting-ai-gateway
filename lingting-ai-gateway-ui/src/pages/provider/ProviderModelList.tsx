import { SyncOutlined } from "@ant-design/icons";
import type { ProviderDetailVO, ProviderModelVO } from "@lingting/ai-gateway-sdk";
import { AppHolder, Button, DictTag, ListCard } from "@lri";
import { Flex, Typography } from "antd";
import { useCallback, useState } from "react";

import { bizApi } from "@/api/BizApi";
import { formatProviderMillis, formatProviderNumber } from "@/utils/providerUtils";

/** 模型卡片高度，与卡片内容高度保持一致。 */
const MODEL_ITEM_HEIGHT = 104;

const ENABLED_DICT = [
  { value: true, label: "启用", tagColor: "success" },
  { value: false, label: "禁用", tagColor: "default" },
];

const REASONING_DICT = [
  { value: true, label: "支持推理", tagColor: "processing" },
  { value: false, label: "不支持推理", tagColor: "default" },
];

type ProviderModelListProps = {
  detail: ProviderDetailVO;
};

/**
 * 供应商模型列表：标题展示数量与同步入口，卡片展示模型基础配置与时间信息。
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
    (model: ProviderModelVO) => (
      <Flex className="provider-model-item" gap="small" justify="space-between" vertical>
        <Flex align="center" gap="small" justify="space-between">
          <Typography.Text ellipsis strong>
            {model.model}
          </Typography.Text>
          <Flex align="center" gap="small">
            <DictTag dict={REASONING_DICT} value={model.reasoning}/>
            <DictTag dict={ENABLED_DICT} value={model.enabled}/>
          </Flex>
        </Flex>
        <Flex align="center" gap="small" justify="space-between">
          <Typography.Text ellipsis type="secondary">
            {model.displayName}
          </Typography.Text>
          <Typography.Text type="secondary">推理等级 {model.levelDefault || "-"}</Typography.Text>
        </Flex>
        <Flex align="center" gap="small" wrap>
          <Typography.Text type="secondary">
            上下文 {formatProviderNumber(model.contextWindow)}
          </Typography.Text>
          <Typography.Text type="secondary">
            知识截止 {formatProviderMillis(model.knowledgeCutoff)}
          </Typography.Text>
          <Typography.Text type="secondary">
            发布日期 {formatProviderMillis(model.releaseDate)}
          </Typography.Text>
        </Flex>
      </Flex>
    ),
    [],
  );

  const header = (
    <Flex align="center" className="provider-model-header" justify="space-between">
      <Typography.Text type="secondary">共 {models.length} 个模型</Typography.Text>
      <Button
        icon={<SyncOutlined/>}
        loading={syncing}
        onClick={handleSync}
        text="同步"
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
