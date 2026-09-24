import type { RequestSubVO } from "@lingting/ai-gateway-sdk";
import { ExTable } from "@lri";
import { useMemo } from "react";

import { createRequestSubColumns } from "./requestFields";

type RequestSubTableProps = {
  onDetail: (record: RequestSubVO) => void;
  subs: RequestSubVO[];
};

/**
 * 展开区域：静态展示该主请求下的全部子请求日志。
 *
 * 详情弹窗由 RequestMain 统一渲染，避免每个展开行各渲染一个弹窗。
 */
export function RequestSubTable({ onDetail, subs }: RequestSubTableProps) {
  const columns = useMemo(() => createRequestSubColumns(onDetail), [onDetail]);

  return (
    <ExTable<RequestSubVO>
      columns={columns}
      dataSource={subs}
      options={false}
      pagination={false}
      search={false}
    />
  );
}
