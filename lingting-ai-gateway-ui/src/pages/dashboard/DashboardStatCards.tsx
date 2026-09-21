import { ProCard, Statistic } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import type { DashboardQO, DashboardRequestVO, DashboardTokenVO } from "@lingting/ai-gateway-sdk";
import { Col, Row } from "antd";

import { bizApi } from "@/api/BizApi";

import "./dashboard.css";

/** 全局统计不附加筛选条件，也不做分组。 */
const GLOBAL_QUERY: DashboardQO = { withModel: false, withProvider: false };

/** 卡片内的统计项两列排布。 */
const STAT_COL_SPAN = 12;

/** 成功率：未失败且未取消的请求占总请求的比例。 */
function successRate(vo: DashboardRequestVO | undefined): number {
  const total = Number(vo?.total ?? 0);
  if (total <= 0) {
    return 0;
  }
  return (total - Number(vo?.failed ?? 0) - Number(vo?.cancelled ?? 0)) / total;
}

/** 缓存 Token：缓存读与缓存写之和。 */
function cacheTokens(vo: DashboardTokenVO | undefined): number {
  return Number(vo?.cacheRead ?? 0) + Number(vo?.cacheWrite ?? 0);
}

type StatCardProps = {
  loading: boolean;
  title: string;
};

type RequestStatCardProps = StatCardProps & { data: DashboardRequestVO | undefined };

type TokenStatCardProps = StatCardProps & { data: DashboardTokenVO | undefined };

/** 请求统计卡片：总数、异常、取消、成功率。 */
function RequestStatCard({ data, loading, title }: RequestStatCardProps) {
  return (
    <ProCard className="dashboard-stat-card" loading={loading} title={title}>
      <Row gutter={[16, 16]}>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="总数" value={data?.total ?? 0}/>
        </Col>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="异常" value={data?.failed ?? 0}/>
        </Col>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="取消" value={data?.cancelled ?? 0}/>
        </Col>
        <Col span={STAT_COL_SPAN}>
          <Statistic precision={2} suffix="%" title="成功率" value={successRate(data) * 100}/>
        </Col>
      </Row>
    </ProCard>
  );
}

/** Token 统计卡片：总数、缓存、读、写。 */
function TokenStatCard({ data, loading, title }: TokenStatCardProps) {
  return (
    <ProCard className="dashboard-stat-card" loading={loading} title={title}>
      <Row gutter={[16, 16]}>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="总数" value={data?.total ?? 0}/>
        </Col>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="缓存" value={cacheTokens(data)}/>
        </Col>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="读" value={data?.input ?? 0}/>
        </Col>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="写" value={data?.output ?? 0}/>
        </Col>
      </Row>
    </ProCard>
  );
}

/**
 * 全局统计卡片：主请求与子请求的请求数量、Token 用量各一张卡片。
 */
export function DashboardStatCards() {
  const { data: requestMain, isLoading: requestMainLoading } = useQuery({
    queryFn: () => bizApi.dashboardRequestMain(GLOBAL_QUERY),
    queryKey: ["dashboard-request-main"],
    retry: false,
  });
  const { data: requestSub, isLoading: requestSubLoading } = useQuery({
    queryFn: () => bizApi.dashboardRequestSub(GLOBAL_QUERY),
    queryKey: ["dashboard-request-sub"],
    retry: false,
  });
  const { data: tokenMain, isLoading: tokenMainLoading } = useQuery({
    queryFn: () => bizApi.dashboardTokenMain(GLOBAL_QUERY),
    queryKey: ["dashboard-token-main"],
    retry: false,
  });
  const { data: tokenSub, isLoading: tokenSubLoading } = useQuery({
    queryFn: () => bizApi.dashboardTokenSub(GLOBAL_QUERY),
    queryKey: ["dashboard-token-sub"],
    retry: false,
  });

  return (
    <Row gutter={[16, 16]}>
      <Col lg={12} xl={6} xs={24}>
        <RequestStatCard data={requestMain} loading={requestMainLoading} title="请求统计"/>
      </Col>
      <Col lg={12} xl={6} xs={24}>
        <TokenStatCard data={tokenMain} loading={tokenMainLoading} title="Token统计"/>
      </Col>
      <Col lg={12} xl={6} xs={24}>
        <RequestStatCard data={requestSub} loading={requestSubLoading} title="子请求统计"/>
      </Col>
      <Col lg={12} xl={6} xs={24}>
        <TokenStatCard data={tokenSub} loading={tokenSubLoading} title="子请求Token统计"/>
      </Col>
    </Row>
  );
}
