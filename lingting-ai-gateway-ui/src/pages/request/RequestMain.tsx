import type { ActionType } from "@ant-design/pro-components";
import type { RequestMainDetailVO, RequestMainQO, RequestMainVO } from "@lingting/ai-gateway-sdk";
import { ExTable, ExTableProps, type PaginationParams } from "@lri";
import { useCallback, useMemo, useRef, useState } from "react";

import { bizApi } from "@/api/BizApi";

import { RequestAutoRefresh } from "./RequestAutoRefresh";
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
  const actionRef = useRef<ActionType>(undefined);
  const [detail, setDetail] = useState<RequestMainVO | null>(null);
  const columns = useMemo(() => createRequestMainColumns(setDetail), []);
  const sections = useMemo(() => (detail ? buildRequestMainSections(detail) : []), [detail]);

  const request = useCallback(
    (pagination: PaginationParams, query: RequestMainQO) =>
      bizApi.requestMainInfoPage(pagination, cleanRequestQuery(query)),
    [],
  );

  const handleAutoRefresh = useCallback(() => {
    void actionRef.current?.reload();
  }, []);

  const expandable = useMemo(() => {
    const v: ExTableProps<RequestMainDetailVO>["expandable"] = {
      childrenColumnName: "________children",
      expandedRowRender: (record) => <RequestSubTable subs={record?.children || []}/>,
      rowExpandable: (record) => !!record?.children?.length,
    };
    return v;
  }, []);

  const handleOpenChange = useCallback((open: boolean) => {
    if (!open) {
      setDetail(null);
    }
  }, []);

  return (
    <>
      <ExTable<RequestMainDetailVO, RequestMainQO>
        actionRef={actionRef}
        columns={columns}
        expandable={expandable}
        request={request}
        search={{ filterType: "light" }}
        toolBarRender={() => [
          <RequestAutoRefresh key="auto-refresh" onRefresh={handleAutoRefresh}/>,
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
    </>
  );
}
