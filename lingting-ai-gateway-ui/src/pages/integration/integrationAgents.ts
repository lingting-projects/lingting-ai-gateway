import { bizApi } from "@/api/BizApi";

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
  /** 同步配置到该 agent，返回旧配置备份路径，无旧文件时为 null */
  sync: () => Promise<string | null>;
};

/** 当前支持的集成 agent，参数全部留空，由后端取默认目录、地址与环境变量名。 */
export const INTEGRATION_AGENTS: IntegrationAgent[] = [
  {
    key: "pi",
    name: "pi",
    url: "https://pi.dev/",
    exportConfig: () => bizApi.agentPiExportText({}),
    sync: () => bizApi.agentPiSync({}),
  },
];
