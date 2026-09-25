import { CodeHighlighter } from "@ant-design/x";
import { ProDescriptions, type ProDescriptionsColumn } from "@ant-design/pro-components";
import { Collapse, Flex, Modal } from "antd";

import type { RequestDetailSection } from "./requestFields";

type RequestDetailModalProps<T extends Record<string, unknown>> = {
  columns: ProDescriptionsColumn<T>[];
  data?: T;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  sections: RequestDetailSection[];
  title: string;
};

/**
 * 请求日志详情弹窗：全字段展示，复杂字段以默认折叠的代码块展示。
 */
export function RequestDetailModal<T extends Record<string, unknown>>({
                                                                        columns,
                                                                        data,
                                                                        onOpenChange,
                                                                        open,
                                                                        sections,
                                                                        title,
                                                                      }: RequestDetailModalProps<T>) {
  return (
    <Modal
      footer={null}
      onCancel={() => onOpenChange(false)}
      open={open}
      styles={{ body: { maxHeight: "60vh", overflowY: "auto" } }}
      title={title}
      width={1200}
    >
      <Flex gap="middle" vertical>
        <ProDescriptions<T> column={2} columns={columns} dataSource={data}/>
        {sections.length > 0 && (
          <Collapse
            items={sections.map((section) => ({
              children: <CodeHighlighter lang="json">{section.content}</CodeHighlighter>,
              key: section.key,
              label: section.label,
            }))}
          />
        )}
      </Flex>
    </Modal>
  );
}
