import type { RequestSubVO } from "@lingting/ai-gateway-sdk";
import { ExTable } from "@lri";
import { useCallback, useMemo, useState } from "react";

import { RequestDetailModal } from "./RequestDetailModal";
import {
  buildRequestSubSections,
  createRequestSubColumns,
  createRequestSubDetailColumns,
} from "./requestFields";

const SUB_DETAIL_COLUMNS = createRequestSubDetailColumns();

type RequestSubTableProps = {
  subs: RequestSubVO[];
};

/**
 * 展开区域：静态展示该主请求下的全部子请求日志，并提供子请求详情弹窗。
 */
export function RequestSubTable({ subs }: RequestSubTableProps) {
  const [detail, setDetail] = useState<RequestSubVO | null>(null);
  const columns = useMemo(() => createRequestSubColumns(setDetail), []);
  const sections = useMemo(() => (detail ? buildRequestSubSections(detail) : []), [detail]);

  const handleOpenChange = useCallback((open: boolean) => {
    if (!open) {
      setDetail(null);
    }
  }, []);

  return (
    <>
      <ExTable<RequestSubVO>
        columns={columns}
        dataSource={subs}
        options={false}
        pagination={false}
        search={false}
      />
      <RequestDetailModal<RequestSubVO>
        columns={SUB_DETAIL_COLUMNS}
        data={detail ?? undefined}
        onOpenChange={handleOpenChange}
        open={Boolean(detail)}
        sections={sections}
        title="子请求详情"
      />
    </>
  );
}
