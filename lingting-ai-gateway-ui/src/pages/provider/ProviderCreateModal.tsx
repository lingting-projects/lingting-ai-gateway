import { PlusOutlined } from "@ant-design/icons";
import { ModalForm, ProForm, ProFormDigit, ProFormText } from "@ant-design/pro-components";
import type { ProviderCreatePO } from "@lingting/ai-gateway-sdk";
import { AppHolder, Button } from "@lri";
import { Checkbox } from "antd";

import { bizApi } from "@/api/BizApi";

type ProviderCreateModalProps = {
  onCreated: () => void;
};

/**
 * 添加供应商弹窗：启用状态使用复选框且默认启用，提交成功后回调刷新列表。
 */
export function ProviderCreateModal({ onCreated }: ProviderCreateModalProps) {
  return (
    <ModalForm<ProviderCreatePO>
      modalProps={{ destroyOnHidden: true }}
      onFinish={async (values) => {
        try {
          await bizApi.providerCreate(values);
          AppHolder.message.success("添加成功");
          onCreated();
          return true;
        } catch (error) {
          AppHolder.message.error(error instanceof Error ? error.message : "添加失败");
          return false;
        }
      }}
      title="添加供应商"
      trigger={<Button icon={<PlusOutlined/>} type={"primary"} text="添加" tooltip="添加供应商"/>}
    >
      <ProFormText
        label="标识"
        name="name"
        placeholder="请输入供应商标识"
        rules={[{ message: "请输入供应商标识", required: true }]}
      />
      <ProFormText
        label="展示名称"
        name="displayName"
        placeholder="请输入展示名称"
        rules={[{ message: "请输入展示名称", required: true }]}
      />
      <ProFormText
        label="接口地址"
        name="baseUrl"
        placeholder="请输入接口地址"
        rules={[{ message: "请输入接口地址", required: true }]}
      />
      <ProFormText.Password
        label="API Key"
        name="apiKey"
        placeholder="请输入 API Key"
        rules={[{ message: "请输入 API Key", required: true }]}
      />
      <ProFormDigit
        initialValue={0}
        label="优先级"
        name="priority"
        rules={[{ message: "请输入优先级", required: true }]}
        width="sm"
      />
      <ProForm.Item initialValue={true} name="enabled" valuePropName="checked">
        <Checkbox>启用</Checkbox>
      </ProForm.Item>
    </ModalForm>
  );
}
