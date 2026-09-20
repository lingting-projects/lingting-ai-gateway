import { useQuery } from "@tanstack/react-query";
import { Flex } from "antd";
import { AppHolder } from "@lri";
import { useCallback, useEffect, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";

import { ProviderDetail } from "./ProviderDetail";
import { ProviderList } from "./ProviderList";

import "./provider.css";

/**
 * 供应商管理：左侧供应商列表，右侧选中供应商详情或供应商统计。
 */
export function ProviderMain() {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const { data, error, isFetching, isLoading, refetch } = useQuery({
    queryKey: ["provider-list"],
    queryFn: () => bizApi.providerList(),
    retry: false,
  });

  const details = useMemo(() => data ?? [], [data]);
  const selected = useMemo(
    () => details.find((detail) => detail.id === selectedId),
    [details, selectedId],
  );
  const enabledCount = useMemo(
    () => details.filter((detail) => detail.provider.enabled).length,
    [details],
  );

  const handleRefresh = useCallback(() => void refetch(), [refetch]);
  const handleSelect = useCallback((id: string) => setSelectedId(id), []);

  useEffect(() => {
    if (error) {
      AppHolder.message.error(error.message);
    }
  }, [error]);

  return (
    <Flex className="provider-main" gap="middle">
      <ProviderList
        details={details}
        loading={isLoading}
        onRefresh={handleRefresh}
        onSelect={handleSelect}
        refreshing={isFetching}
        selectedId={selectedId}
      />
      <ProviderDetail
        detail={selected}
        enabledCount={enabledCount}
        onChanged={handleRefresh}
        total={details.length}
      />
    </Flex>
  );
}
