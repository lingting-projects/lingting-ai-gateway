import { AppHolder, Button, copyText } from "@lri";
import { Flex, Modal, Typography } from "antd";
import { useCallback } from "react";

import "./apiKey.css";

type ApiKeySecretModalProps = {
  onClose: () => void;
  /** 服务端仅在创建时返回一次的明文 key。 */
  secret?: string;
};

/**
 * 明文密钥弹窗：仅创建成功后展示一次，遮罩与 Esc 可关闭，不提供关闭图标。
 */
export function ApiKeySecretModal({ onClose, secret }: ApiKeySecretModalProps) {
  const handleCopy = useCallback(async () => {
    if (!secret) {
      return;
    }
    const result = await copyText(secret);
    if (result.success) {
      void AppHolder.message.success("已复制到剪贴板");
    } else {
      void AppHolder.message.error("复制失败，请手动复制");
    }
  }, [secret]);

  return (
    <Modal
      closable={false}
      footer={
        <Flex gap="small" justify="end">
          <Button onClick={handleCopy} text="复制" type="primary"/>
          <Button onClick={onClose} text="关闭"/>
        </Flex>
      }
      keyboard
      maskClosable
      onCancel={onClose}
      open={Boolean(secret)}
      title="API Key 创建成功"
      width={560}
    >
      <Flex className="api-key-secret" gap="small" vertical>
        <Typography.Text type="secondary">
          明文密钥仅在创建时返回一次，请立即复制保存。
        </Typography.Text>
        <Typography.Text className="api-key-secret-value" strong>
          {secret}
        </Typography.Text>
      </Flex>
    </Modal>
  );
}
