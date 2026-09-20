import { ProCard, Statistic } from "@ant-design/pro-components";
import { Flex } from "antd";

type ProviderOverviewProps = {
  enabledCount: number;
  total: number;
};

/**
 * 未选中供应商时的概览：供应商数量与可用供应商数量。
 */
export function ProviderOverview({ enabledCount, total }: ProviderOverviewProps) {
  return (
    <Flex className="provider-overview" gap="middle">
      <ProCard className="provider-overview-card">
        <Statistic title="供应商数量" value={total}/>
      </ProCard>
      <ProCard className="provider-overview-card">
        <Statistic title="可用供应商数量" value={enabledCount}/>
      </ProCard>
    </Flex>
  );
}
