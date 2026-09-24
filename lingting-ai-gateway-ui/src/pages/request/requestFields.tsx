import type { ProDescriptionsColumn } from "@ant-design/pro-components";
import { type ExDictProps, type ExTableColumn, formatTimestamp, LinkButton } from "@lri";
import {
  type RequestMainDetailVO,
  type RequestMainQO,
  type RequestMainVO,
  RequestStatusAll,
  RequestStatusMap,
  type RequestSubVO,
} from "@lingting/ai-gateway-sdk";
import { Flex, Typography } from "antd";
import dayjs from "dayjs";

import { formatCachePercent, formatTokenCount } from "@/utils/tokenUtils";

/** 请求状态字典，展示与搜索共用。 */
const REQUEST_STATUS_ITEMS = RequestStatusAll.map((value) => ({
  color: RequestStatusMap[value].color,
  label: RequestStatusMap[value].label,
  value,
}));

/** 表格内展示的状态字典配置。 */
const REQUEST_STATUS_TABLE_DICT: ExDictProps = { dict: REQUEST_STATUS_ITEMS };

/** 搜索表单使用的状态字典配置。 */
const REQUEST_STATUS_SEARCH_DICT: ExDictProps = { dict: REQUEST_STATUS_ITEMS, search: "select" };

/** Token 明细字段，主请求与子请求结构一致。 */
type RequestTokenFields = {
  inputTokens: string;
  outputTokens: string;
  cacheReadTokens: string;
  cacheWriteTokens: string;
  inferenceTokens: string;
  totalTokens: string;
};

const TOKEN_DETAILS: { key: keyof RequestTokenFields; label: string }[] = [
  { key: "inputTokens", label: "输入" },
  { key: "outputTokens", label: "输出" },
  { key: "cacheReadTokens", label: "缓存读" },
  { key: "cacheWriteTokens", label: "缓存写" },
  { key: "inferenceTokens", label: "推理" },
];

const TOKEN_DETAIL_FIELDS: { key: keyof RequestTokenFields; title: string }[] = [
  { key: "inputTokens", title: "输入 Token" },
  { key: "outputTokens", title: "输出 Token" },
  { key: "cacheReadTokens", title: "缓存读 Token" },
  { key: "cacheWriteTokens", title: "缓存写 Token" },
  { key: "inferenceTokens", title: "推理 Token" },
  { key: "totalTokens", title: "总 Token" },
];

/** 时间展示：毫秒时间戳。 */
export function formatRequestTime(value?: string | null) {
  if (value === null || value === undefined || value === "") {
    return "-";
  }
  return formatTimestamp(value, "milliseconds");
}

/** 耗时展示：超过 1 秒时换算为秒。 */
export function formatRequestDuration(value?: string | null) {
  if (value === null || value === undefined || value === "") {
    return "-";
  }
  const millis = Number(value);
  if (!Number.isFinite(millis)) {
    return "-";
  }
  return millis >= 1000 ? `${(millis / 1000).toFixed(2)} s` : `${millis} ms`;
}

/** Token 列：第一行总 / 缓存 / 缓存占比，第二行明细只保留非零项。 */
export function renderTokenSummary(record?: RequestTokenFields) {
  if (!record) {
    return <>-</>;
  }

  const total = Number(record.totalTokens);
  const cache = Number(record.cacheReadTokens) + Number(record.cacheWriteTokens);
  const summary = `总 ${formatTokenCount(total)} · 缓存 ${formatTokenCount(cache)} · ${formatCachePercent(cache, total)}`;
  const details = TOKEN_DETAILS.filter((item) => Number(record[item.key]) > 0).map(
    (item) => `${item.label} ${formatTokenCount(record[item.key])}`,
  );

  return (
    <Flex className="request-token" gap={2} vertical>
      <Typography.Text>{summary}</Typography.Text>
      {details.length > 0 && (
        <Typography.Text type="secondary">{details.join(" · ")}</Typography.Text>
      )}
    </Flex>
  );
}

/** 详情弹窗中的复杂字段：请求参数、请求头、响应头与响应内容。 */
export type RequestDetailSection = {
  content: string;
  key: string;
  label: string;
};

function toSection(key: string, label: string, value: unknown): RequestDetailSection | null {
  if (value === null || value === undefined || value === "") {
    return null;
  }
  const content = typeof value === "string" ? value : JSON.stringify(value, null, 2);
  return { content, key, label };
}

/** 主请求的复杂字段集合，无值时不展示。 */
export function buildRequestMainSections(record: RequestMainVO): RequestDetailSection[] {
  return [
    toSection("requestParams", "请求参数", record.requestParams),
    toSection("requestHeaders", "请求头", record.requestHeaders),
    toSection("responseHeaders", "响应头", record.responseHeaders),
    toSection("responseContent", "响应内容", record.responseContent),
  ].filter((section) => section !== null);
}

/** 子请求的复杂字段集合，无值时不展示。 */
export function buildRequestSubSections(record: RequestSubVO): RequestDetailSection[] {
  return [
    toSection("requestParams", "请求参数", record.requestParams),
    toSection("requestHeaders", "请求头", record.requestHeaders),
    toSection("responseHeaders", "响应头", record.responseHeaders),
    toSection("responseContent", "响应内容", record.responseContent),
  ].filter((section) => section !== null);
}

/** 搜索参数清理：去掉未填写与空字符串条件。 */
export function cleanRequestQuery(query: RequestMainQO): RequestMainQO {
  const entries = Object.entries(query).filter(
    ([, value]) => value !== undefined && value !== null && value !== "",
  );
  return Object.fromEntries(entries) as RequestMainQO;
}

function createOptionColumn<T extends Record<string, unknown>>(
  onDetail: (record: T) => void,
): ExTableColumn<T> {
  return {
    dataIndex: "option",
    search: false,
    render: (_dom, record) => <LinkButton onClick={() => onDetail(record)} text="详情" />,
    title: "操作",
    valueType: "option",
  };
}

/** 主请求日志表格列。 */
export function createRequestMainColumns(
  onDetail: (record: RequestMainVO) => void,
): ExTableColumn<RequestMainDetailVO>[] {
  return [
    {
      dataIndex: ["request", "status"],
      dict: REQUEST_STATUS_TABLE_DICT,
      search: false,
      title: "状态",
    },
    { dataIndex: ["request", "returnModel"], search: false, title: "返回模型" },
    {
      dataIndex: ["request", "totalTokens"],
      render: (_dom, record) => renderTokenSummary(record.request),
      search: false,
      title: "Token",
    },
    {
      dataIndex: ["request", "durationMs"],
      render: (_dom, record) => formatRequestDuration(record.request?.durationMs),
      search: false,
      title: "耗时",
    },
    {
      dataIndex: ["request", "startTime"],
      render: (_dom, record) => formatRequestTime(record.request?.startTime),
      search: false,
      title: "开始时间",
    },
    {
      dataIndex: ["request", "firstChunkTime"],
      render: (_dom, record) => formatRequestTime(record.request?.firstChunkTime),
      search: false,
      title: "首包时间",
    },
    { dataIndex: ["request", "sessionId"], search: false, title: "会话ID" },
    { dataIndex: ["request", "errorType"], search: false, title: "错误类型" },
    { dataIndex: ["request", "errorMessage"], ellipsis: true, search: false, title: "错误信息" },
    { dataIndex: "id", hideInTable: true, title: "ID" },
    { dataIndex: "clientRequestId", hideInTable: true, title: "客户端请求ID" },
    { dataIndex: "sessionId", hideInTable: true, title: "会话ID" },
    { dataIndex: "model", hideInTable: true, title: "模型" },
    { dataIndex: "credentialId", hideInTable: true, title: "凭证ID" },
    { dataIndex: "clientIp", hideInTable: true, title: "客户端IP" },
    {
      dataIndex: "status",
      dict: REQUEST_STATUS_SEARCH_DICT,
      hideInTable: true,
      title: "状态",
    },
    {
      dataIndex: "startTimeRange",
      hideInTable: true,
      search: {
        transform: (value: [string, string]) => ({
          startTimeBegin: dayjs(value[0]).valueOf(),
          startTimeEnd: dayjs(value[1]).valueOf(),
        }),
      },
      title: "开始时间",
      valueType: "dateTimeRange",
    },
    createOptionColumn<RequestMainDetailVO>((record) => onDetail(record.request)),
  ];
}

/** 子请求日志表格列。 */
export function createRequestSubColumns(
  onDetail: (record: RequestSubVO) => void,
): ExTableColumn<RequestSubVO>[] {
  return [
    { dataIndex: "providerName", search: false, title: "供应商" },
    { dataIndex: "status", dict: REQUEST_STATUS_TABLE_DICT, search: false, title: "状态" },
    { dataIndex: "returnModel", search: false, title: "返回模型" },
    {
      dataIndex: "totalTokens",
      render: (_dom, record) => renderTokenSummary(record),
      search: false,
      title: "Token",
    },
    {
      dataIndex: "durationMs",
      render: (_dom, record) => formatRequestDuration(record.durationMs),
      search: false,
      title: "耗时",
    },
    {
      dataIndex: "startTime",
      render: (_dom, record) => formatRequestTime(record.startTime),
      search: false,
      title: "开始时间",
    },
    {
      dataIndex: "firstChunkTime",
      render: (_dom, record) => formatRequestTime(record.firstChunkTime),
      search: false,
      title: "首包时间",
    },
    { dataIndex: "errorMessage", ellipsis: true, search: false, title: "错误信息" },
    createOptionColumn<RequestSubVO>(onDetail),
  ];
}

/** 主请求详情弹窗字段，展示全部字段。 */
export function createRequestMainDetailColumns(): ProDescriptionsColumn<RequestMainVO>[] {
  return [
    { dataIndex: "id", title: "ID" },
    { dataIndex: "traceId", title: "链路ID" },
    { dataIndex: "clientRequestId", title: "客户端请求ID" },
    { dataIndex: "sessionId", title: "会话ID" },
    { dataIndex: "clientIp", title: "客户端IP" },
    { dataIndex: "userAgent", title: "User-Agent" },
    { dataIndex: "credentialId", title: "凭证ID" },
    { dataIndex: "model", title: "模型" },
    { dataIndex: "stream", title: "流式", valueType: "switch" },
    { dataIndex: "method", title: "请求方法" },
    { dataIndex: "path", title: "请求路径" },
    {
      dataIndex: "status",
      render: (_dom, record) => record.statusLabel || record.status,
      title: "状态",
    },
    { dataIndex: "errorType", title: "错误类型" },
    { dataIndex: "errorCode", title: "错误码" },
    { dataIndex: "errorMessage", title: "错误信息" },
    { dataIndex: "returnModel", title: "返回模型" },
    ...TOKEN_DETAIL_FIELDS.map((field) => ({ dataIndex: field.key, title: field.title })),
    {
      dataIndex: "httpStatus",
      render: (_dom, record) => record.httpStatus ?? "-",
      title: "HTTP状态",
    },
    { dataIndex: "finishReason", title: "结束原因" },
    { dataIndex: "providerRequestId", title: "供应商请求ID" },
    {
      dataIndex: "startTime",
      render: (_dom, record) => formatRequestTime(record.startTime),
      title: "开始时间",
    },
    {
      dataIndex: "firstChunkTime",
      render: (_dom, record) => formatRequestTime(record.firstChunkTime),
      title: "首包时间",
    },
    {
      dataIndex: "endTime",
      render: (_dom, record) => formatRequestTime(record.endTime),
      title: "结束时间",
    },
    {
      dataIndex: "durationMs",
      render: (_dom, record) => formatRequestDuration(record.durationMs),
      title: "耗时",
    },
    {
      dataIndex: "createTime",
      render: (_dom, record) => formatRequestTime(record.createTime),
      title: "创建时间",
    },
  ];
}

/** 子请求详情弹窗字段，展示全部字段。 */
export function createRequestSubDetailColumns(): ProDescriptionsColumn<RequestSubVO>[] {
  return [
    { dataIndex: "id", title: "ID" },
    { dataIndex: "mainRequestId", title: "主请求ID" },
    { dataIndex: "providerId", title: "供应商ID" },
    { dataIndex: "providerName", title: "供应商" },
    { dataIndex: "providerUrl", title: "供应商地址" },
    { dataIndex: "model", title: "模型" },
    {
      dataIndex: "status",
      render: (_dom, record) => record.statusLabel || record.status,
      title: "状态",
    },
    { dataIndex: "errorType", title: "错误类型" },
    { dataIndex: "errorCode", title: "错误码" },
    { dataIndex: "errorMessage", title: "错误信息" },
    { dataIndex: "returnModel", title: "返回模型" },
    ...TOKEN_DETAIL_FIELDS.map((field) => ({ dataIndex: field.key, title: field.title })),
    {
      dataIndex: "httpStatus",
      render: (_dom, record) => record.httpStatus ?? "-",
      title: "HTTP状态",
    },
    { dataIndex: "finishReason", title: "结束原因" },
    { dataIndex: "providerRequestId", title: "供应商请求ID" },
    {
      dataIndex: "startTime",
      render: (_dom, record) => formatRequestTime(record.startTime),
      title: "开始时间",
    },
    {
      dataIndex: "firstChunkTime",
      render: (_dom, record) => formatRequestTime(record.firstChunkTime),
      title: "首包时间",
    },
    {
      dataIndex: "endTime",
      render: (_dom, record) => formatRequestTime(record.endTime),
      title: "结束时间",
    },
    {
      dataIndex: "durationMs",
      render: (_dom, record) => formatRequestDuration(record.durationMs),
      title: "耗时",
    },
    {
      dataIndex: "createTime",
      render: (_dom, record) => formatRequestTime(record.createTime),
      title: "创建时间",
    },
  ];
}
