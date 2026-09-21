import { ModalForm, ProFormText, ProFormTextArea } from "@ant-design/pro-components";
import type { ApiKeyVO } from "@lingting/ai-gateway-sdk";
import { AppHolder } from "@lri";
import type { ReactElement } from "react";

import { bizApi } from "@/api/BizApi";

type ApiKeyValues = {
  name: string;
  remark: string;
};

type ApiKeyEditModalProps = {
  /** 传入表示编辑该密钥，不传表示新增。 */
  record?: ApiKeyVO;
  /** 新增成功回调，参数为服务端返回的明文 key。 */
  onCreated: (key: string) => void;
  onOpenChange?: (open: boolean) => void;
  onSuccess: () => void;
  open?: boolean;
  trigger?: ReactElement;
};

/**
 * API Key 弹窗：新增与编辑共用，编辑时启用状态沿用记录当前值（列表提供快捷切换）。
 */
export function ApiKeyEditModal({
                                  onCreated,
                                  onOpenChange,
                                  onSuccess,
                                  open,
                                  record,
                                  trigger,
                                }: ApiKeyEditModalProps) {
  const editing = Boolean(record);

  return (
    <ModalForm<ApiKeyValues>
      initialValues={editing ? { name: record?.name, remark: record?.remark } : undefined}
      modalProps={{ destroyOnHidden: true }}
      onFinish={async (values) => {
        try {
          if (record) {
            await bizApi.apiKeyUpdate({ ...values, enabled: record.enabled, id: record.id });
            AppHolder.message.success("保存成功");
            onSuccess();
          } else {
            const key = await bizApi.apiKeyCreate(values);
            onSuccess();
            onCreated(key);
          }
          return true;
        } catch (error) {
          AppHolder.message.error(error instanceof Error ? error.message : "操作失败");
          return false;
        }
      }}
      onOpenChange={onOpenChange}
      open={open}
      title={editing ? "编辑 API Key" : "新增 API Key"}
      trigger={trigger}
    >
      <ProFormText
        label="名称"
        name="name"
        placeholder="请输入名称"
        rules={[{ message: "请输入名称", required: true }]}
      />
      <ProFormTextArea
        label="备注"
        name="remark"
        placeholder="请输入备注"
        rules={[{ message: "请输入备注", required: true }]}
      />
    </ModalForm>
  );
}
