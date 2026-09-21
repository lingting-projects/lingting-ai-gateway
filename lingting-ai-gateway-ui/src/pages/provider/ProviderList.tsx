import { ReloadOutlined, SearchOutlined } from "@ant-design/icons";
import type { ProviderDetailVO } from "@lingting/ai-gateway-sdk";
import { AppHolder, Button, DictTag, ListCard } from "@lri";
import { Flex, Input, Typography } from "antd";
import clsx from "clsx";
import { useCallback, useState } from "react";

import { bizApi } from "@/api/BizApi";
import { filterProviderDetails } from "@/utils/providerUtils";

import { ProviderCreateModal } from "./ProviderCreateModal";

/** 供应商卡片高度，与卡片内容高度保持一致。 */
const PROVIDER_ITEM_HEIGHT = 72;

const ENABLED_DICT = [
  { value: true, label: "启用", tagColor: "success" },
  { value: false, label: "禁用", tagColor: "default" },
];

type ProviderListProps = {
  details: ProviderDetailVO[];
  loading?: boolean;
  onRefresh: () => void;
  onSelect: (id: string) => void;
  refreshing?: boolean;
  selectedId?: string | null;
};

/**
 * 供应商列表：标题搜索与刷新，卡片展示名称、模型数量、启用状态与优先级。
 */
export function ProviderList({
                               details,
                               loading,
                               onRefresh,
                               onSelect,
                               refreshing,
                               selectedId,
                             }: ProviderListProps) {
  const [keyword, setKeyword] = useState("");
  const [syncingId, setSyncingId] = useState<string | null>(null);

  const handleSyncModel = useCallback(async (id: string) => {
    setSyncingId(id);
    try {
      await bizApi.providerUpdateModel({ id });
      void AppHolder.message.success("同步任务已启动，请稍后刷新供应商列表");
    } catch (error) {
      void AppHolder.message.error(error instanceof Error ? error.message : "同步失败");
    } finally {
      setSyncingId(null);
    }
  }, []);

  const filter = useCallback(
    (items: ProviderDetailVO[]) => filterProviderDetails(items, keyword),
    [keyword],
  );

  const renderItem = useCallback(
    (detail: ProviderDetailVO) => {
      const { provider, models } = detail;
      return (
        <Flex
          className={clsx(
            "provider-list-item",
            detail.id === selectedId && "provider-list-item-selected",
          )}
          gap="small"
          justify="space-between"
          onClick={() => onSelect(detail.id)}
          vertical
        >
          <Flex align="center" gap="small" justify="space-between">
            <Typography.Text ellipsis strong>
              {provider.displayName}
            </Typography.Text>
            <DictTag dict={ENABLED_DICT} value={provider.enabled}/>
          </Flex>
          <Flex align="center" gap="small" justify="space-between">
            <Flex align="center" gap="small">
              <Typography.Text type="secondary">模型 {models.length}</Typography.Text>
              <Button
                icon={<ReloadOutlined/>}
                loading={syncingId === detail.id}
                onClick={(event) => {
                  event.stopPropagation();
                  void handleSyncModel(detail.id);
                }}
                size="small"
                tooltip="同步供应商模型"
                type="text"
              />
            </Flex>
            <Typography.Text type="secondary">优先级 {provider.priority}</Typography.Text>
          </Flex>
        </Flex>
      );
    },
    [handleSyncModel, onSelect, selectedId, syncingId],
  );

  const header = (
    <Flex align="center" className="provider-list-header" gap="small">
      <Input
        allowClear
        className="provider-list-search"
        onChange={(event) => setKeyword(event.target.value)}
        placeholder="搜索供应商或模型"
        prefix={<SearchOutlined/>}
        value={keyword}
      />
      <Button
        icon={<ReloadOutlined/>}
        loading={refreshing}
        onClick={onRefresh}
        tooltip="刷新供应商列表"
      />
      <ProviderCreateModal onCreated={onRefresh}/>
    </Flex>
  );

  return (
    <Flex className="provider-list" vertical>
      <ListCard
        data={details}
        filter={filter}
        header={header}
        itemHeight={PROVIDER_ITEM_HEIGHT}
        itemRender={renderItem}
        loading={loading}
        props={{ ghost: true }}
      />
    </Flex>
  );
}
