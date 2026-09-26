import { ModalForm, ProFormText, ProFormTextArea } from "@ant-design/pro-components";
import { AppHolder } from "@lri";
import type { ReactElement } from "react";

import { bizApi } from "@/api/BizApi";

type ApiKeySyncValues = {
  keyHash: string;
  name: string;
  remark: string;
};

type ApiKeySyncModalProps = {
  onSuccess: () => void;
  trigger: ReactElement;
};

/**
 * API Key 同步弹窗：把来源实例的 key 摘要写入当前实例。
 *
 * 摘要已存在时服务端会启用原记录并刷新更新时间，不会产生重复行。
 */
export function ApiKeySyncModal({ onSuccess, trigger }: ApiKeySyncModalProps) {
  return (
    <ModalForm<ApiKeySyncValues>
      modalProps={{ destroyOnHidden: true }}
      onFinish={async (values) => {
        try {
          await bizApi.apiKeySync(values);
          AppHolder.message.success("同步成功");
          onSuccess();
          return true;
        } catch (error) {
          AppHolder.message.error(error instanceof Error ? error.message : "同步失败");
          return false;
        }
      }}
      title="同步 API Key"
      trigger={trigger}
    >
      <ProFormText
        label="名称"
        name="name"
        placeholder="请输入名称"
        rules={[{ message: "请输入名称", required: true }]}
      />
      <ProFormText
        label="key 摘要"
        name="keyHash"
        placeholder="请粘贴来源实例的 key 摘要"
        rules={[{ message: "请输入 key 摘要", required: true }]}
        tooltip="来源实例 API Key 列表中的 keyHash（sha1 摘要），可在列表内直接复制"
      />
      <ProFormTextArea label="备注" name="remark" placeholder="请输入备注" />
    </ModalForm>
  );
}
