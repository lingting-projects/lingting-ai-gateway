import { ProCard, Statistic } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import type { DashboardQO, DashboardRequestVO, DashboardTokenVO } from "@lingting/ai-gateway-sdk";
import { Col, Row } from "antd";

import { bizApi } from "@/api/BizApi";
import { formatCachePercent, formatTokenCount } from "@/utils/tokenUtils";

import { DASHBOARD_QUERY_ROOT } from "./dashboardQueryKeys";
import "./dashboard.css";

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

/** Token 统计卡片：总数、缓存（含占比）、读、写。 */
function TokenStatCard({ data, loading, title }: TokenStatCardProps) {
  const total = Number(data?.total ?? 0);
  const cache = cacheTokens(data);

  return (
    <ProCard className="dashboard-stat-card" loading={loading} title={title}>
      <Row gutter={[16, 16]}>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="总数" value={formatTokenCount(total)}/>
        </Col>
        <Col span={STAT_COL_SPAN}>
          <Statistic
            suffix={`(${formatCachePercent(cache, total)})`}
            title="缓存"
            value={formatTokenCount(cache)}
          />
        </Col>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="读" value={formatTokenCount(data?.input)}/>
        </Col>
        <Col span={STAT_COL_SPAN}>
          <Statistic title="写" value={formatTokenCount(data?.output)}/>
        </Col>
      </Row>
    </ProCard>
  );
}

type DashboardStatCardsProps = {
  /** 统计时间范围，未选择时跳过查询 */
  rangeTime: [number, number] | null;
};

/**
 * 全局统计卡片：主请求与子请求的请求数量、Token 用量各一张卡片，统计范围由顶部操作行决定。
 */
export function DashboardStatCards({ rangeTime }: DashboardStatCardsProps) {
  const query = useMemo<DashboardQO>(
    () => ({
      endTime: rangeTime ? String(rangeTime[1]) : null,
      startTime: rangeTime ? String(rangeTime[0]) : null,
      withModel: false,
      withProvider: false,
    }),
    [rangeTime],
  );
  const enabled = rangeTime !== null;

  const { data: requestMain, isLoading: requestMainLoading } = useQuery({
    enabled,
    queryFn: () => bizApi.dashboardRequestMain(query),
    queryKey: [...DASHBOARD_QUERY_ROOT, "request-main", query],
    retry: false,
  });
  const { data: requestSub, isLoading: requestSubLoading } = useQuery({
    enabled,
    queryFn: () => bizApi.dashboardRequestSub(query),
    queryKey: [...DASHBOARD_QUERY_ROOT, "request-sub", query],
    retry: false,
  });
  const { data: tokenMain, isLoading: tokenMainLoading } = useQuery({
    enabled,
    queryFn: () => bizApi.dashboardTokenMain(query),
    queryKey: [...DASHBOARD_QUERY_ROOT, "token-main", query],
    retry: false,
  });
  const { data: tokenSub, isLoading: tokenSubLoading } = useQuery({
    enabled,
    queryFn: () => bizApi.dashboardTokenSub(query),
    queryKey: [...DASHBOARD_QUERY_ROOT, "token-sub", query],
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
