import { bizApi } from "@/api/BizApi";

/** 集成 agent 的同步目标：展示名称与实际文件夹。 */
export type IntegrationSyncTarget = {
  /** 实际文件夹，作为 sync 的 dir 参数传入 */
  home: string;
  /** 展示名称；手动输入的文件夹与 home 一致 */
  name: string;
};

/** 集成 agent：名称、官网地址与接口调用方式全部内置在前端。 */
export type IntegrationAgent = {
  /** 唯一标识，同时作为列表 key */
  key: string;
  /** 展示名称 */
  name: string;
  /** 官网地址，点击后在新标签打开 */
  url: string;
  /** 拉取该 agent 的配置预览文本 */
  exportConfig: () => Promise<string>;
  /** 拉取该 agent 可选的同步目标（服务端账户） */
  loadSyncTargets: () => Promise<IntegrationSyncTarget[]>;
  /** 同步配置到该 agent，dir 为空时由后端取默认目录，返回旧配置备份路径 */
  sync: (dir?: string) => Promise<string | null>;
};

/** 当前支持的集成 agent，其余参数留空由后端取默认值。 */
export const INTEGRATION_AGENTS: IntegrationAgent[] = [
  {
    key: "pi",
    name: "pi",
    url: "https://pi.dev/",
    exportConfig: () => bizApi.agentPiExportText({}),
    loadSyncTargets: async () => {
      const users = await bizApi.agentPiUsers();
      return users.map((user) => ({ home: user.home, name: user.name }));
    },
    sync: (dir) => bizApi.agentPiSync(dir ? { dir } : {}),
  },
];
