import type {
  RequestMainDetailVO,
  RequestMainQO,
  RequestMainVO,
  RequestSubVO,
} from "@lingting/ai-gateway-sdk";
import { ExTable, ExTableProps, type PaginationParams } from "@lri";
import { useCallback, useEffect, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";

import { RequestAutoRefresh } from "./RequestAutoRefresh";
import { RequestDetailModal } from "./RequestDetailModal";
import { RequestSubTable } from "./RequestSubTable";
import { readAutoRefreshSetting, saveAutoRefreshSetting } from "./requestAutoRefreshUtils";
import {
  buildRequestMainSections,
  buildRequestSubSections,
  cleanRequestQuery,
  createRequestMainColumns,
  createRequestMainDetailColumns,
  createRequestSubDetailColumns,
} from "./requestFields";

import "./request.css";

const MAIN_DETAIL_COLUMNS = createRequestMainDetailColumns();
const SUB_DETAIL_COLUMNS = createRequestSubDetailColumns();

/**
 * 请求日志：主请求日志分页表格，行可展开查看该主请求下的全部子请求日志。
 */
export function RequestMain() {
  const [detail, setDetail] = useState<RequestMainVO | null>(null);
  const [subDetail, setSubDetail] = useState<RequestSubVO | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(readAutoRefreshSetting);
  const columns = useMemo(() => createRequestMainColumns(setDetail), []);
  const sections = useMemo(() => (detail ? buildRequestMainSections(detail) : []), [detail]);
  const subSections = useMemo(
    () => (subDetail ? buildRequestSubSections(subDetail) : []),
    [subDetail],
  );

  const request = useCallback(
    (pagination: PaginationParams, query: RequestMainQO) =>
      bizApi.requestMainInfoPage(pagination, cleanRequestQuery(query)),
    [],
  );

  useEffect(() => {
    saveAutoRefreshSetting(autoRefresh);
  }, [autoRefresh]);

  /** 自动刷新轮询间隔，未勾选时不轮询。 */
  const polling = autoRefresh.enabled ? autoRefresh.seconds * 1000 : undefined;

  const handleSubDetail = useCallback((record: RequestSubVO) => setSubDetail(record), []);

  const expandable = useMemo(() => {
    const v: ExTableProps<RequestMainDetailVO>["expandable"] = {
      childrenColumnName: "________children",
      expandedRowRender: (record) => (
        <RequestSubTable onDetail={handleSubDetail} subs={record?.children || []}/>
      ),
      rowExpandable: (record) => !!record?.children?.length,
    };
    return v;
  }, [handleSubDetail]);

  const handleOpenChange = useCallback((open: boolean) => {
    if (!open) {
      setDetail(null);
    }
  }, []);

  const handleSubOpenChange = useCallback((open: boolean) => {
    if (!open) {
      setSubDetail(null);
    }
  }, []);

  return (
    <>
      <ExTable<RequestMainDetailVO, RequestMainQO>
        columns={columns}
        expandable={expandable}
        request={request}
        polling={polling}
        search={{ filterType: "light" }}
        toolBarRender={() => [
          <RequestAutoRefresh key="auto-refresh" onChange={setAutoRefresh} setting={autoRefresh}/>,
        ]}
      />
      <RequestDetailModal<RequestMainVO>
        columns={MAIN_DETAIL_COLUMNS}
        data={detail ?? undefined}
        onOpenChange={handleOpenChange}
        open={Boolean(detail)}
        sections={sections}
        title="主请求详情"
      />
      <RequestDetailModal<RequestSubVO>
        columns={SUB_DETAIL_COLUMNS}
        data={subDetail ?? undefined}
        onOpenChange={handleSubOpenChange}
        open={Boolean(subDetail)}
        sections={subSections}
        title="子请求详情"
      />
    </>
  );
}
