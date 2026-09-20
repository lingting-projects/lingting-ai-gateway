import { AppHolder } from "@lri";
import { Flex, Input, Typography } from "antd";
import { type ComponentProps, useCallback, useState } from "react";

const STORAGE_KEY = "lingting-ai-gateway:authorization-token";

let cachedToken: string | null = null;
let inputVisible = false;

function readStorage(): string | null {
  try {
    return window.localStorage.getItem(STORAGE_KEY);
  } catch {
    return null;
  }
}

function writeStorage(token: string) {
  try {
    window.localStorage.setItem(STORAGE_KEY, token);
  } catch {
    // 本地存储不可用时仅保留内存缓存
  }
}

/** 读取管理令牌：从本地缓存读取并同步内存缓存。 */
function get(): string {
  const stored = readStorage();
  if (stored !== null) {
    cachedToken = stored;
  }
  return cachedToken ?? "";
}

/** 写入管理令牌：先落本地缓存，再更新内存缓存。 */
function set(token: string) {
  writeStorage(token);
  cachedToken = token;
}

type TokenInputProps = {
  defaultValue: string;
  onChange: (value: string) => void;
};

function TokenInput({ defaultValue, onChange }: TokenInputProps) {
  const [value, setValue] = useState(defaultValue);
  const handleChange = useCallback<NonNullable<ComponentProps<typeof Input.Password>["onChange"]>>(
    (event) => {
      setValue(event.target.value);
      onChange(event.target.value);
    },
    [onChange],
  );

  return (
    <Flex gap="small" vertical>
      <Typography.Text type="secondary">
        管理令牌用于访问管理接口，保存后缓存在本地。
      </Typography.Text>
      <Input.Password
        autoFocus
        onChange={handleChange}
        placeholder="请输入管理令牌"
        value={value}
      />
    </Flex>
  );
}

/** 弹出不可关闭的管理令牌输入框，保存后关闭。 */
function renderInput() {
  if (inputVisible) {
    return;
  }
  inputVisible = true;

  let token = get();
  AppHolder.modal.confirm({
    afterClose: () => {
      inputVisible = false;
    },
    cancelButtonProps: { style: { display: "none" } },
    closable: false,
    content: (
      <TokenInput
        defaultValue={token}
        onChange={(value) => {
          token = value;
        }}
      />
    ),
    icon: null,
    keyboard: false,
    mask: { closable: false },
    okText: "保存",
    onOk: () => {
      set(token.trim());
    },
    title: "设置管理令牌",
  });
}

export const AuthorizationStore = { get, renderInput, set };
