import { DeleteOutlined, EditOutlined } from "@ant-design/icons";
import { ProDescriptions, type ProDescriptionsColumn } from "@ant-design/pro-components";
import type { ProviderDetailVO, ProviderUpdatePO, ProviderVO } from "@lingting/ai-gateway-sdk";
import { AppHolder, Button, DictTag, LinkButton } from "@lri";
import { Flex, Form } from "antd";
import { useCallback, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";
import { formatProviderMillis } from "@/utils/providerUtils";

const ENABLED_DICT = [
  { value: true, label: "启用", tagColor: "success" },
  { value: false, label: "禁用", tagColor: "default" },
];

/** 可编辑字段，与供应商更新接口的可修改字段一致。 */
const EDITABLE_FIELDS = ["displayName", "baseUrl", "apiKey", "priority", "enabled"];

type ProviderUpdateValues = {
  apiKey?: string;
  baseUrl: string;
  displayName: string;
  enabled: boolean;
  priority: number;
};

type ProviderBasicInfoProps = {
  detail: ProviderDetailVO;
  onChanged: () => void;
};

function resolveErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "操作失败";
}

/** 组装更新参数：key 留空或仍为脱敏值时表示不修改已保存的 key。 */
function toUpdatePO(provider: ProviderVO, values: Partial<ProviderUpdateValues>): ProviderUpdatePO {
  const apiKey = values.apiKey?.trim() ?? "";
  return {
    apiKey: !apiKey || apiKey === provider.apiKey ? null : apiKey,
    baseUrl: values.baseUrl ?? provider.baseUrl,
    displayName: values.displayName ?? provider.displayName,
    enabled: values.enabled ?? provider.enabled,
    id: provider.id,
    priority: values.priority ?? provider.priority,
  };
}

/**
 * 供应商基础信息：展示全部字段，编辑时仅可编辑字段进入输入态，底部提供取消与提交。
 */
export function ProviderBasicInfo({ detail, onChanged }: ProviderBasicInfoProps) {
  const provider = detail.provider;
  const [editing, setEditing] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [form] = Form.useForm<ProviderUpdateValues>();

  const handleToggleEnabled = useCallback(async () => {
    try {
      await bizApi.providerUpdate(toUpdatePO(provider, { enabled: !provider.enabled }));
      AppHolder.message.success("操作成功");
      onChanged();
    } catch (error) {
      AppHolder.message.error(resolveErrorMessage(error));
    }
  }, [onChanged, provider]);

  const handleDelete = useCallback(async () => {
    try {
      await bizApi.providerDelete({ id: provider.id });
      AppHolder.message.success("删除成功");
      onChanged();
    } catch (error) {
      AppHolder.message.error(resolveErrorMessage(error));
    }
  }, [onChanged, provider.id]);

  const handleEdit = useCallback(() => setEditing(true), []);

  const handleCancel = useCallback(() => {
    setEditing(false);
    form.resetFields();
  }, [form]);

  const handleSubmit = useCallback(async () => {
    let values: ProviderUpdateValues;
    try {
      values = await form.validateFields();
    } catch {
      return;
    }

    setSubmitting(true);
    try {
      await bizApi.providerUpdate(toUpdatePO(provider, values));
      AppHolder.message.success("保存成功");
      setEditing(false);
      onChanged();
    } catch (error) {
      AppHolder.message.error(resolveErrorMessage(error));
    } finally {
      setSubmitting(false);
    }
  }, [form, onChanged, provider]);

  const columns = useMemo<ProDescriptionsColumn<ProviderVO>[]>(
    () => [
      { copyable: true, dataIndex: "id", editable: false, title: "ID" },
      { copyable: true, dataIndex: "name", editable: false, title: "标识" },
      { dataIndex: "displayName", editable: false, title: "展示名称" },
      { copyable: true, dataIndex: "baseUrl", editable: false, title: "接口地址" },
      { copyable: true, dataIndex: "apiKey", editable: false, ellipsis: true, title: "API Key" },
      { dataIndex: "priority", editable: false, title: "优先级", valueType: "digit" },
      {
        dataIndex: "enabled",
        editable: false,
        render: (_dom, record) => (
          <Flex align="center" gap="small">
            <DictTag dict={ENABLED_DICT} value={record.enabled}/>
            <LinkButton
              hidden={editing}
              onClick={handleToggleEnabled}
              text={record.enabled ? "禁用" : "启用"}
            />
          </Flex>
        ),
        title: "启用状态",
        valueType: "switch",
      },
      {
        dataIndex: "createTime",
        editable: false,
        render: (_dom, record) => formatProviderMillis(record.createTime),
        title: "创建时间",
      },
      {
        dataIndex: "updateTime",
        editable: false,
        render: (_dom, record) => formatProviderMillis(record.updateTime),
        title: "更新时间",
      },
    ],
    [editing, handleToggleEnabled],
  );

  return (
    <Flex className="provider-basic-info" gap="middle" vertical>
      <ProDescriptions<ProviderVO>
        column={2}
        columns={columns}
        dataSource={provider}
        editable={{
          actionRender: () => [],
          editableKeys: editing ? [...EDITABLE_FIELDS] : [],
          form,
          type: "multiple",
        }}
        extra={
          <Flex gap="small">
            <Button
              confirm={{
                description: "删除后该供应商的模型配置会一并移除。",
                title: "确认删除该供应商？",
              }}
              danger
              icon={<DeleteOutlined/>}
              onConfirm={handleDelete}
              text="删除"
            />
            <Button hidden={editing} icon={<EditOutlined/>} onClick={handleEdit} text="编辑"/>
          </Flex>
        }
      />
      {editing && (
        <Flex gap="small" justify="end">
          <Button onClick={handleCancel} text="取消"/>
          <Button loading={submitting} onClick={handleSubmit} text="提交" type="primary"/>
        </Flex>
      )}
    </Flex>
  );
}
