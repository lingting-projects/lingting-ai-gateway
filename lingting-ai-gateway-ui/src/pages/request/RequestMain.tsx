import type { RequestMainDetailVO, RequestMainQO, RequestMainVO } from "@lingting/ai-gateway-sdk";
import { ExTable, type PaginationParams } from "@lri";
import { useCallback, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";

import { RequestDetailModal } from "./RequestDetailModal";
import { RequestSubTable } from "./RequestSubTable";
import {
  buildRequestMainSections,
  cleanRequestQuery,
  createRequestMainColumns,
  createRequestMainDetailColumns,
} from "./requestFields";

import "./request.css";

const MAIN_DETAIL_COLUMNS = createRequestMainDetailColumns();

/**
 * 请求日志：主请求日志分页表格，行可展开查看该主请求下的全部子请求日志。
 */
export function RequestMain() {
  const [detail, setDetail] = useState<RequestMainVO | null>(null);
  const columns = useMemo(() => createRequestMainColumns(setDetail), []);
  const sections = useMemo(() => (detail ? buildRequestMainSections(detail) : []), [detail]);

  const request = useCallback(
    (pagination: PaginationParams, query: RequestMainQO) =>
      bizApi.requestMainInfoPage(pagination, cleanRequestQuery(query)),
    [],
  );

  const expandable = useMemo(
    () => ({
      expandedRowRender: (record: RequestMainDetailVO) => (
        <RequestSubTable subs={record.children}/>
      ),
      rowExpandable: (record: RequestMainDetailVO) => record.children.length > 0,
    }),
    [],
  );

  const handleOpenChange = useCallback((open: boolean) => {
    if (!open) {
      setDetail(null);
    }
  }, []);

  return (
    <>
      <ExTable<RequestMainDetailVO, RequestMainQO>
        columns={columns}
        expandable={expandable}
        request={request}
      />
      <RequestDetailModal<RequestMainVO>
        columns={MAIN_DETAIL_COLUMNS}
        data={detail ?? undefined}
        onOpenChange={handleOpenChange}
        open={Boolean(detail)}
        sections={sections}
        title="主请求详情"
      />
    </>
  );
}
