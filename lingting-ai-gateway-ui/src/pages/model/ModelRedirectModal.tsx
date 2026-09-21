import { ModalForm, ProFormSelect, ProFormText } from "@ant-design/pro-components";
import type { ProviderModelRedirectVO } from "@lingting/ai-gateway-sdk";
import { AppHolder } from "@lri";
import type { ReactElement } from "react";

import { bizApi } from "@/api/BizApi";

type ModelRedirectValues = {
  source: string;
  target: string;
};

type ModelRedirectModalProps = {
  /** 传入表示编辑该映射，不传表示新增。 */
  redirect?: ProviderModelRedirectVO;
  /** 映射目标候选项，来自默认模型配置。 */
  targetOptions: { label: string; value: string }[];
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  onSuccess: () => void;
  trigger?: ReactElement;
};

/**
 * 模型映射弹窗：新增与编辑共用，提交成功后关闭并回调刷新。
 */
export function ModelRedirectModal({
                                     onOpenChange,
                                     onSuccess,
                                     open,
                                     redirect,
                                     targetOptions,
                                     trigger,
                                   }: ModelRedirectModalProps) {
  const editing = Boolean(redirect);

  return (
    <ModalForm<ModelRedirectValues>
      initialValues={editing ? { source: redirect?.source, target: redirect?.target } : undefined}
      modalProps={{ destroyOnHidden: true }}
      onFinish={async (values) => {
        try {
          if (redirect) {
            await bizApi.providerModelRedirectUpdate({ ...values, id: redirect.id });
            AppHolder.message.success("保存成功");
          } else {
            await bizApi.providerModelRedirectCreate(values);
            AppHolder.message.success("新增成功");
          }
          onSuccess();
          return true;
        } catch (error) {
          AppHolder.message.error(error instanceof Error ? error.message : "操作失败");
          return false;
        }
      }}
      onOpenChange={onOpenChange}
      open={open}
      title={editing ? "编辑模型映射" : "新增模型映射"}
      trigger={trigger}
    >
      <ProFormText
        label="映射源"
        name="source"
        placeholder="请输入非官方模型名"
        rules={[{ message: "请输入映射源", required: true }]}
      />
      <ProFormSelect
        fieldProps={{ optionFilterProp: "label", showSearch: true }}
        label="映射目标"
        name="target"
        options={targetOptions}
        placeholder="请选择官方模型名"
        rules={[{ message: "请选择映射目标", required: true }]}
      />
    </ModalForm>
  );
}
