import {
  ApiDefinitions,
  BasicAiGatewayApi,
  type PiAgentPO,
  type R,
  RCodeKindMap,
} from "@lingting/ai-gateway-sdk";
import { DocumentUtils } from "@lri";
import { AuthorizationStore } from "@/store/AuthorizationStore";

function buildQueryString(query: unknown): string {
  if (!query || typeof query !== "object") {
    return "";
  }

  const search = new URLSearchParams();
  Object.entries(query as Record<string, unknown>).forEach(([key, value]) => {
    if (value === undefined || value === null || value === "") {
      return;
    }
    search.append(key, String(value));
  });
  return search.toString();
}

function buildRequestUrl(path: string, query?: unknown): string {
  const prefix = import.meta.env.VITE_API_PREFIX?.trim();
  const target = DocumentUtils.buildTarget(prefix, "/api");
  const url = `${target}/${path}`;
  const search = buildQueryString(query);
  return search ? `${url}?${search}` : url;
}

/** 提取错误响应中的提示信息，非统一响应体时回退到状态码提示。 */
async function resolveErrorMessage(response: Response): Promise<string> {
  try {
    const result = (await response.json()) as R<unknown>;
    return result.message || `请求失败：${response.status}`;
  } catch {
    return `请求失败：${response.status}`;
  }
}

/**
 * 业务接口实现：统一拼装请求前缀、管理令牌与响应包装。
 */
export class BizApi extends BasicAiGatewayApi {
  /** pi 模型配置导出：接口直接返回 models.json 文本，不套用网关统一响应体。 */
  agentPiExportText(params: PiAgentPO): Promise<string> {
    return this.callText("POST", ApiDefinitions.agentPiExport.path, params);
  }

  /** 请求原始文本响应：接口直接返回内容体，不套用网关统一响应体。 */
  protected async callText(
    method: string,
    path: string,
    body?: unknown,
    query?: unknown,
  ): Promise<string> {
    const response = await this.request(method, path, body, query);
    if (!response.ok) {
      throw new Error(await resolveErrorMessage(response));
    }
    return await response.text();
  }

  protected async call<T>(
    method: string,
    path: string,
    body?: unknown,
    query?: unknown,
  ): Promise<T> {
    const response = await this.request(method, path, body, query);
    const contentType = response.headers.get("content-type") ?? "";
    if (!contentType.includes("application/json")) {
      if (!response.ok) {
        throw new Error(`请求失败：${response.status}`);
      }
      return (await response.blob()) as T;
    }

    const result = (await response.json()) as R<T>;
    if (result.code !== RCodeKindMap.SUCCESS.code) {
      throw new Error(result.message || `请求失败：${result.code}`);
    }
    return result.data as T;
  }

  /** 发起请求：统一拼装请求地址、请求头与管理令牌，并处理未授权响应。 */
  private async request(
    method: string,
    path: string,
    body?: unknown,
    query?: unknown,
  ): Promise<Response> {
    const upperMethod = method.toUpperCase();
    const hasBody =
      body !== undefined && body !== null && upperMethod !== "GET" && upperMethod !== "HEAD";
    const headers: Record<string, string> = { Accept: "application/json" };
    const token = AuthorizationStore.get();
    if (token) {
      headers.Authorization = `Bearer ${token}`;
    }
    if (hasBody) {
      headers["Content-Type"] = "application/json";
    }

    const response = await fetch(buildRequestUrl(path, query), {
      body: hasBody ? JSON.stringify(body) : undefined,
      headers,
      method: upperMethod,
    });

    if (response.status === 401) {
      AuthorizationStore.renderInput();
      throw new Error("管理令牌无效，请重新设置");
    }

    return response;
  }
}

export const bizApi = new BizApi();
