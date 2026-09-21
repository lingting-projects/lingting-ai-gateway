import { ProCard } from "@ant-design/pro-components";
import type { ProviderDetailVO } from "@lingting/ai-gateway-sdk";
import { Tabs } from "antd";
import { useMemo } from "react";

import { ProviderBasicInfo } from "./ProviderBasicInfo";
import { ProviderModelList } from "./ProviderModelList";
import { ProviderOverview } from "./ProviderOverview";

type ProviderDetailProps = {
  detail?: ProviderDetailVO;
  enabledCount: number;
  onChanged: () => void;
  total: number;
};

/**
 * 供应商详情：未选中时展示供应商统计，选中时展示基础信息与模型列表。
 */
export function ProviderDetail({ detail, enabledCount, onChanged, total }: ProviderDetailProps) {
  const items = useMemo(() => {
    if (!detail) {
      return [];
    }
    return [
      {
        children: <ProviderBasicInfo detail={detail} key={detail.id} onChanged={onChanged}/>,
        key: "basic",
        label: "基础信息",
      },
      {
        children: <ProviderModelList detail={detail} key={detail.id}/>,
        key: "model",
        label: "模型列表",
      },
    ];
  }, [detail, onChanged]);

  return (
    <>
      {!detail && <ProviderOverview enabledCount={enabledCount} total={total}/>}
      {detail && (
        <ProCard className="provider-detail-card" ghost={true}>
          <Tabs
            className="provider-detail-tabs"
            classNames={{
              body: "provider-detail-tabs-body",
              content: "provider-detail-tabs-content",
            }}
            items={items}
          />
        </ProCard>
      )}
    </>
  );
}
