import { ModalForm, ProFormSelect } from "@ant-design/pro-components";
import { Flex, Tag } from "antd";

import type { IntegrationSyncTarget } from "./integrationAgents";

type IntegrationSyncFormValues = {
  target?: string[];
};

type IntegrationSyncModalProps = {
  /** 默认选中的目标文件夹；为空表示不预选 */
  defaultHome?: string;
  onConfirm: (target: IntegrationSyncTarget) => Promise<boolean>;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  targets: IntegrationSyncTarget[];
};

/**
 * 同步目标选项：名称与文件夹不一致时两个值都用 tag 展示，手动输入的文件夹只有一个值。
 */
function renderTargetLabel(target: IntegrationSyncTarget) {
  if (target.name === target.home) {
    return <Tag>{target.home}</Tag>;
  }

  return (
    <Flex align="center" gap={4}>
      <Tag color="blue">{target.name}</Tag>
      <Tag>{target.home}</Tag>
    </Flex>
  );
}

/**
 * 集成同步弹窗：选择或直接输入同步目标后执行同步。
 *
 * 下拉为 tag 模式且只保留一个值；直接输入的文件夹没有对应选项，其名称与文件夹一致。
 */
export function IntegrationSyncModal({
  defaultHome,
  onConfirm,
  onOpenChange,
  open,
  targets,
}: IntegrationSyncModalProps) {
  const options = targets.map((target) => ({
    label: renderTargetLabel(target),
    value: target.home,
  }));

  return (
    <ModalForm<IntegrationSyncFormValues>
      initialValues={{ target: defaultHome ? [defaultHome] : [] }}
      modalProps={{ destroyOnHidden: true }}
      onFinish={async (values) => {
        const home = values.target?.[0]?.trim();
        if (!home) {
          return false;
        }

        const target = targets.find((item) => item.home === home) ?? { home, name: home };
        return await onConfirm(target);
      }}
      onOpenChange={onOpenChange}
      open={open}
      title="同步配置"
    >
      <ProFormSelect<IntegrationSyncFormValues>
        fieldProps={{ maxCount: 1, mode: "tags", options, showSearch: true }}
        label="同步目标"
        name="target"
        rules={[
          {
            validator: (_rule, value: unknown) =>
              Array.isArray(value) && value.length > 0
                ? Promise.resolve()
                : Promise.reject(new Error("请选择同步目标")),
          },
        ]}
        tooltip="下拉为服务端账户与本地历史记录，也可以直接输入文件夹"
      />
    </ModalForm>
  );
}
