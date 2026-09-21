import { useQuery } from "@tanstack/react-query";
import type { ProviderModelRedirectVO, ProviderModelVO } from "@lingting/ai-gateway-sdk";
import { AppHolder } from "@lri";
import { Flex } from "antd";
import { useCallback, useEffect, useMemo, useState } from "react";

import { bizApi } from "@/api/BizApi";

import { ModelDefaultList } from "./ModelDefaultList";
import { ModelRedirectList } from "./ModelRedirectList";

import "./model.css";

/**
 * 模型配置：左侧默认模型配置、右侧模型映射，两侧选中项互相过滤对方列表。
 *
 * 选中项按 id 记录，列表刷新后已删除的记录会自动取消选中。
 */
export function ModelMain() {
  const [selectedModelId, setSelectedModelId] = useState<string | null>(null);
  const [selectedRedirectId, setSelectedRedirectId] = useState<string | null>(null);
  const {
    data: models,
    error: modelError,
    isFetching: modelsFetching,
    isLoading: modelsLoading,
    refetch: refetchModels,
  } = useQuery({
    queryKey: ["provider-model-default"],
    queryFn: () => bizApi.providerModelDefault(),
    retry: false,
  });
  const {
    data: redirects,
    error: redirectError,
    isFetching: redirectsFetching,
    isLoading: redirectsLoading,
    refetch: refetchRedirects,
  } = useQuery({
    queryKey: ["provider-model-redirect-list"],
    queryFn: () => bizApi.providerModelRedirectList(),
    retry: false,
  });

  const modelList = useMemo(() => models ?? [], [models]);
  const redirectList = useMemo(() => redirects ?? [], [redirects]);
  const selectedModel = useMemo(
    () => modelList.find((model) => model.id === selectedModelId) ?? null,
    [modelList, selectedModelId],
  );
  const selectedRedirect = useMemo(
    () => redirectList.find((redirect) => redirect.id === selectedRedirectId) ?? null,
    [redirectList, selectedRedirectId],
  );
  // 映射目标候选项来自默认模型配置，按模型名去重
  const targetOptions = useMemo(
    () =>
      Array.from(new Set(modelList.map((model) => model.model))).map((model) => ({
        label: model,
        value: model,
      })),
    [modelList],
  );
  const handleRefreshModels = useCallback(() => void refetchModels(), [refetchModels]);
  const handleRefreshRedirects = useCallback(() => void refetchRedirects(), [refetchRedirects]);
  const handleSelectModel = useCallback((model: ProviderModelVO) => {
    setSelectedModelId((current) => (current === model.id ? null : model.id));
  }, []);
  const handleSelectRedirect = useCallback((redirect: ProviderModelRedirectVO) => {
    setSelectedRedirectId((current) => (current === redirect.id ? null : redirect.id));
  }, []);

  useEffect(() => {
    if (modelError) {
      AppHolder.message.error(modelError.message);
    }
  }, [modelError]);

  useEffect(() => {
    if (redirectError) {
      AppHolder.message.error(redirectError.message);
    }
  }, [redirectError]);

  return (
    <Flex className="model-main" gap="middle">
      <ModelDefaultList
        loading={modelsLoading}
        models={modelList}
        onRefresh={handleRefreshModels}
        onSelect={handleSelectModel}
        refreshing={modelsFetching}
        selected={selectedModel}
        targetModel={selectedRedirect?.target}
      />
      <ModelRedirectList
        loading={redirectsLoading}
        onRefresh={handleRefreshRedirects}
        onSelect={handleSelectRedirect}
        redirects={redirectList}
        refreshing={redirectsFetching}
        selected={selectedRedirect}
        targetOptions={targetOptions}
        targetModel={selectedModel?.model}
      />
    </Flex>
  );
}
