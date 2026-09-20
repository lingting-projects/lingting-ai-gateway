import { BasicAiGatewayApi, type R, RCodeKindMap } from "@lingting/ai-gateway-sdk";

import { AuthorizationStore } from "@/store/AuthorizationStore";

/** 默认请求前缀，未配置 VITE_API_PREFIX 时使用。 */
const DEFAULT_API_PREFIX = "/api";

/** 请求前缀：由环境变量配置，未配置时使用 /api。 */
function resolveApiPrefix(): string {
  const prefix = import.meta.env.VITE_API_PREFIX?.trim();
  if (!prefix) {
    return DEFAULT_API_PREFIX;
  }
  return prefix.endsWith("/") ? prefix.slice(0, -1) : prefix;
}

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
  const url = `${resolveApiPrefix()}/${path.replace(/^\/+/, "")}`;
  const search = buildQueryString(query);
  return search ? `${url}?${search}` : url;
}

/**
 * 业务接口实现：统一拼装请求前缀、管理令牌与响应包装。
 */
export class BizApi extends BasicAiGatewayApi {
  protected async call<T>(
    method: string,
    path: string,
    body?: unknown,
    query?: unknown,
  ): Promise<T> {
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
}

export const bizApi = new BizApi();
