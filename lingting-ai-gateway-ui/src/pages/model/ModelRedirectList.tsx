import {
  ArrowRightOutlined,
  DeleteOutlined,
  EditOutlined,
  PlusOutlined,
  ReloadOutlined,
  SearchOutlined,
} from "@ant-design/icons";
import type { ProviderModelRedirectVO } from "@lingting/ai-gateway-sdk";
import { AppHolder, Button, LinkButton, ListCard } from "@lri";
import { Flex, Input, Typography } from "antd";
import clsx from "clsx";
import { useCallback, useState } from "react";

import { bizApi } from "@/api/BizApi";
import { includesKeyword } from "@/utils/textUtils";

import { ModelRedirectModal } from "./ModelRedirectModal";

/** 映射卡片高度，与卡片内容高度保持一致。 */
const REDIRECT_ITEM_HEIGHT = 64;

type ModelRedirectListProps = {
  redirects: ProviderModelRedirectVO[];
  loading?: boolean;
  onRefresh: () => void;
  onSelect: (redirect: ProviderModelRedirectVO) => void;
  refreshing?: boolean;
  selected?: ProviderModelRedirectVO | null;
  /** 映射目标候选项，来自默认模型配置。 */
  targetOptions: { label: string; value: string }[];
  /** 左侧选中的模型名，存在时忽略搜索仅展示指向该模型的映射。 */
  targetModel?: string;
};

/**
 * 模型映射列表：标题右侧刷新与新增入口，搜索框筛选映射源与映射目标，卡片尾部提供编辑与删除。
 */
export function ModelRedirectList({
                                    loading,
                                    onRefresh,
                                    onSelect,
                                    redirects,
                                    refreshing,
                                    selected,
                                    targetOptions,
                                    targetModel,
                                  }: ModelRedirectListProps) {
  const [keyword, setKeyword] = useState("");
  const [editing, setEditing] = useState<ProviderModelRedirectVO | null>(null);

  const handleDelete = useCallback(
    async (redirect: ProviderModelRedirectVO) => {
      try {
        await bizApi.providerModelRedirectDelete({ id: redirect.id });
        AppHolder.message.success("删除成功");
        onRefresh();
      } catch (error) {
        AppHolder.message.error(error instanceof Error ? error.message : "删除失败");
      }
    },
    [onRefresh],
  );

  const filter = useCallback(
    (items: ProviderModelRedirectVO[]) => {
      if (targetModel) {
        return items.filter((item) => item.target === targetModel);
      }

      const value = keyword.trim().toLowerCase();
      if (!value) {
        return items;
      }
      return items.filter(
        (item) => includesKeyword(item.source, value) || includesKeyword(item.target, value),
      );
    },
    [keyword, targetModel],
  );

  const renderItem = useCallback(
    (redirect: ProviderModelRedirectVO) => (
      <Flex
        align="center"
        className={clsx(
          "model-redirect-item",
          redirect.id === selected?.id && "model-redirect-item-selected",
        )}
        gap="small"
        onClick={() => onSelect(redirect)}
      >
        <Flex align="center" className="model-redirect-content" gap="small">
          <Typography.Text ellipsis>{redirect.source}</Typography.Text>
          <ArrowRightOutlined className="model-redirect-arrow"/>
          <Typography.Text ellipsis strong>
            {redirect.target}
          </Typography.Text>
        </Flex>
        <Flex
          className="model-redirect-actions"
          gap="small"
          onClick={(event) => event.stopPropagation()}
          vertical
        >
          <LinkButton
            icon={<EditOutlined/>}
            onClick={() => setEditing(redirect)}
            text="编辑"
            tooltip="编辑模型映射"
          />
          <LinkButton
            confirm={{ description: "删除后该映射立即失效。", title: "确认删除该映射？" }}
            danger
            icon={<DeleteOutlined/>}
            onConfirm={() => handleDelete(redirect)}
            text="删除"
            tooltip="删除模型映射"
          />
        </Flex>
      </Flex>
    ),
    [handleDelete, onSelect, selected],
  );

  const header = (
    <Flex className="model-panel-header" gap="small" vertical>
      <Flex align="center" gap="small" justify="space-between">
        <Typography.Text strong>模型映射</Typography.Text>
        <Flex gap="small">
          <Button
            icon={<ReloadOutlined/>}
            loading={refreshing}
            onClick={onRefresh}
            tooltip="刷新模型映射"
          />
          <ModelRedirectModal
            onSuccess={onRefresh}
            targetOptions={targetOptions}
            trigger={<Button icon={<PlusOutlined/>} tooltip="新增模型映射"/>}
          />
        </Flex>
      </Flex>
      <Input
        allowClear
        onChange={(event) => setKeyword(event.target.value)}
        placeholder="搜索映射源或映射目标"
        prefix={<SearchOutlined/>}
        value={keyword}
      />
    </Flex>
  );

  return (
    <Flex className="model-panel" vertical>
      <ListCard
        data={redirects}
        filter={filter}
        header={header}
        itemHeight={REDIRECT_ITEM_HEIGHT}
        itemRender={renderItem}
        loading={loading}
        props={{ ghost: true }}
      />
      <ModelRedirectModal
        onOpenChange={(open) => !open && setEditing(null)}
        onSuccess={onRefresh}
        open={Boolean(editing)}
        redirect={editing ?? undefined}
        targetOptions={targetOptions}
      />
    </Flex>
  );
}
