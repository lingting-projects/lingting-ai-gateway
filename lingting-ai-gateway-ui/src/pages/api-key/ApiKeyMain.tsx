import { PlusOutlined } from "@ant-design/icons";
import type { ActionType, ProColumns } from "@ant-design/pro-components";
import type { ApiKeyQO, ApiKeyVO } from "@lingting/ai-gateway-sdk";
import { AppHolder, Button, DictTag, ExTable, formatTimestamp, LinkButton } from "@lri";
import { Flex } from "antd";
import { useCallback, useMemo, useRef, useState } from "react";

import { bizApi } from "@/api/BizApi";

import { ApiKeyEditModal } from "./ApiKeyEditModal";
import { ApiKeySecretModal } from "./ApiKeySecretModal";

const ENABLED_DICT = [
  { value: true, label: "启用", tagColor: "success" },
  { value: false, label: "禁用", tagColor: "default" },
];

/**
 * API Key 管理：分页列表、新增、编辑与删除，启用状态支持列表内快捷切换。
 */
export function ApiKeyMain() {
  const actionRef = useRef<ActionType>(undefined);
  const [editing, setEditing] = useState<ApiKeyVO | null>(null);
  const [secret, setSecret] = useState<string | undefined>(undefined);

  const reload = useCallback(() => void actionRef.current?.reload(), []);

  const handleToggleEnabled = useCallback(
    async (record: ApiKeyVO) => {
      try {
        await bizApi.apiKeyUpdate({
          enabled: !record.enabled,
          id: record.id,
          name: record.name,
          remark: record.remark,
        });
        void AppHolder.message.success("操作成功");
        reload();
      } catch (error) {
        void AppHolder.message.error(error instanceof Error ? error.message : "操作失败");
      }
    },
    [reload],
  );

  const handleDelete = useCallback(
    async (record: ApiKeyVO) => {
      try {
        await bizApi.apiKeyDelete({ id: record.id });
        void AppHolder.message.success("删除成功");
        reload();
      } catch (error) {
        void AppHolder.message.error(error instanceof Error ? error.message : "删除失败");
      }
    },
    [reload],
  );

  const columns = useMemo<ProColumns<ApiKeyVO>[]>(
    () => [
      { dataIndex: "name", title: "名称" },
      {
        dataIndex: "enabled",
        render: (_dom, record) => (
          <Flex align="center" gap="small">
            <DictTag dict={ENABLED_DICT} value={record.enabled}/>
            <LinkButton
              hidden={record.enabled}
              onClick={() => handleToggleEnabled(record)}
              text="启用"
            />
            <LinkButton
              hidden={!record.enabled}
              onClick={() => handleToggleEnabled(record)}
              text="禁用"
            />
          </Flex>
        ),
        title: "启用状态",
      },
      { dataIndex: "remark", ellipsis: true, title: "备注" },
      {
        dataIndex: "createTime",
        render: (_dom, record) => formatTimestamp(record.createTime, "milliseconds"),
        title: "创建时间",
      },
      {
        dataIndex: "updateTime",
        render: (_dom, record) => formatTimestamp(record.updateTime, "milliseconds"),
        title: "更新时间",
      },
      {
        dataIndex: "option",
        render: (_dom, record) => (
          <Flex gap="small">
            <LinkButton onClick={() => setEditing(record)} text="编辑"/>
            <LinkButton
              confirm={{
                description: "删除后使用该密钥的请求会立即失败。",
                title: "确认删除该密钥？",
              }}
              danger
              onConfirm={() => handleDelete(record)}
              text="删除"
            />
          </Flex>
        ),
        title: "操作",
        valueType: "option",
      },
    ],
    [handleDelete, handleToggleEnabled],
  );

  const request = useCallback(
    (pagination: Parameters<typeof bizApi.apiKeyPage>[0], query: ApiKeyQO) =>
      bizApi.apiKeyPage(pagination, query),
    [],
  );

  return (
    <>
      <ExTable<ApiKeyVO, ApiKeyQO>
        actionRef={actionRef}
        columns={columns}
        request={request}
        search={false}
        toolBarRender={() => [
          <ApiKeyEditModal
            key="create"
            onCreated={setSecret}
            onSuccess={reload}
            trigger={<Button icon={<PlusOutlined/>} text="新增" type="primary"/>}
          />,
        ]}
      />
      <ApiKeyEditModal
        onCreated={setSecret}
        onOpenChange={(open) => !open && setEditing(null)}
        onSuccess={reload}
        open={Boolean(editing)}
        record={editing ?? undefined}
      />
      <ApiKeySecretModal onClose={() => setSecret(undefined)} secret={secret}/>
    </>
  );
}
