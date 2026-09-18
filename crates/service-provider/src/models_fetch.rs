//! 供应商模型同步：拉取供应商的模型列表，与默认模型配置合并后写入本地。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

use anyhow::{Result, anyhow};
use dashmap::DashMap;
use lib_db::{DbContext, PgPoolExt, scope_db, use_pool};
use lib_provider::build_client;
use serde::Deserialize;
use service_admin::service::{ProviderModelRedirectService, ProviderModelService, ProviderService};
use tokio::sync::{Mutex, OwnedMutexGuard};
use types_admin::dto::ProviderModelCreatePO;
use types_admin::entity::{Provider, ProviderModel};

/// 模型列表接口的路径后缀。
const MODELS_SUFFIX: &str = "/models";

/// 同步中的供应商锁，保证同一供应商同一时刻只有一个同步任务。
static SYNCHRONIZING: LazyLock<DashMap<i64, Arc<Mutex<()>>>> = LazyLock::new(DashMap::new);

/// 启动供应商模型同步：`None` 同步所有启用的供应商，`Some` 只同步指定供应商。
///
/// 同步在独立异步任务中执行，调用方立即返回。
pub async fn async_run(db: Arc<DbContext>, provider: Option<i64>) {
    tokio::spawn(async move {
        scope_db(db, async move {
            if let Err(error) = run(provider).await {
                tracing::warn!("供应商模型同步失败：{error}");
            }
        })
        .await;
    });
}

/// 同步入口：确定本次要同步的供应商，装载默认模型配置与模型映射后逐个同步。
async fn run(provider: Option<i64>) -> Result<()> {
    let providers = match provider {
        // 指定供应商时不论是否启用都同步。
        Some(id) => ProviderService::new()?
            .find_by_id(id)
            .await?
            .into_iter()
            .collect(),
        None => ProviderService::new()?.find_enabled().await?,
    };

    let default_map = default_map().await?;
    let redirect_map = redirect_map().await?;

    for provider in providers {
        if let Err(error) = fetch(provider.id, &default_map, &redirect_map).await {
            tracing::warn!("同步供应商 {} 模型失败：{error}", provider.name);
        }
    }

    Ok(())
}

/// 默认模型配置，键为模型名。
async fn default_map() -> Result<HashMap<String, ProviderModel>> {
    let models = ProviderModelService::new()?.find_default().await?;
    Ok(models
        .into_iter()
        .map(|model| (model.model.clone(), model))
        .collect())
}

/// 模型名称映射，键为非官方模型名。
async fn redirect_map() -> Result<HashMap<String, String>> {
    let redirects = ProviderModelRedirectService::new()?.find_all().await?;
    Ok(redirects
        .into_iter()
        .map(|redirect| (redirect.source, redirect.target))
        .collect())
}

/// 同步单个供应商：取锁、拉取远程模型、合并后在一个事务内更新与禁用。
async fn fetch(
    provider_id: i64,
    default_map: &HashMap<String, ProviderModel>,
    redirect_map: &HashMap<String, String>,
) -> Result<()> {
    let Some(_guard) = try_lock(provider_id) else {
        tracing::warn!("供应商 {provider_id} 正在同步，跳过本次同步");
        return Ok(());
    };

    let provider = ProviderService::new()?
        .find_by_id(provider_id)
        .await?
        .ok_or_else(|| anyhow!("供应商不存在或已删除：{provider_id}"))?;

    let Some(remote) = fetch_remote(&provider).await? else {
        return Ok(());
    };

    let params: Vec<ProviderModelCreatePO> = remote
        .into_iter()
        .map(|model| merge(provider_id, model, default_map, redirect_map))
        .collect();
    let models: Vec<String> = params.iter().map(|param| param.model.clone()).collect();

    // 空列表视为异常响应：不写入任何模型，并禁用该供应商全部已启用模型。
    if params.is_empty() {
        tracing::warn!(
            "供应商 {} 返回空模型列表，将禁用其全部已启用模型",
            provider.name
        );
    }

    let service = ProviderModelService::new()?;
    let removed = missing_models(&service, provider_id, &models).await?;

    use_pool()?
        .with_transaction(async |transaction| {
            if !params.is_empty() {
                service.upsert(&params, transaction).await?;
            }
            service
                .disable_missing(provider_id, &models, transaction)
                .await?;
            Ok(())
        })
        .await?;

    tracing::info!(
        "供应商 {} 模型同步完成：更新 {} 个，禁用 {} 个",
        provider.name,
        models.len(),
        removed.len()
    );

    Ok(())
}

/// 已入库但本次未返回的模型名，即需要禁用的模型。
async fn missing_models(
    service: &ProviderModelService,
    provider_id: i64,
    models: &[String],
) -> Result<Vec<String>> {
    let fetched: HashSet<&str> = models.iter().map(String::as_str).collect();
    let existing = service.find_models(provider_id).await?;
    Ok(existing
        .into_iter()
        .filter(|model| !fetched.contains(model.as_str()))
        .collect())
}

/// 拉取供应商模型列表；请求失败或解析失败只记录日志并返回 `None`。
async fn fetch_remote(provider: &Provider) -> Result<Option<Vec<RemoteModel>>> {
    let url = format!("{}{MODELS_SUFFIX}", provider.base_url.trim_end_matches('/'));
    let response = match build_client(provider)
        .get(url)
        .bearer_auth(&provider.api_key)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!("请求供应商 {} 模型列表失败：{error}", provider.name);
            return Ok(None);
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::warn!(
            "请求供应商 {} 模型列表失败：状态码 {status}，响应体 {body}",
            provider.name
        );
        return Ok(None);
    }

    let body = match response.text().await {
        Ok(body) => body,
        Err(error) => {
            tracing::warn!("读取供应商 {} 模型列表响应失败：{error}", provider.name);
            return Ok(None);
        }
    };

    match serde_json::from_str::<RemoteModelList>(&body) {
        Ok(list) => Ok(Some(list.data)),
        Err(error) => {
            tracing::warn!(
                "解析供应商 {} 模型列表失败：{error}，响应体 {body}",
                provider.name
            );
            Ok(None)
        }
    }
}

/// 尝试占用供应商同步锁，已被占用时返回 `None`。
fn try_lock(provider_id: i64) -> Option<OwnedMutexGuard<()>> {
    let lock = SYNCHRONIZING
        .entry(provider_id)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone();

    lock.try_lock_owned().ok()
}

/// 合并远程模型与默认模型配置：远程返回的值优先，其次取默认模型配置，最后取类型零值。
///
/// 默认模型查询顺序为 `default_map[name]`、`default_map[redirect_map[name]]`。
fn merge(
    provider_id: i64,
    remote: RemoteModel,
    default_map: &HashMap<String, ProviderModel>,
    redirect_map: &HashMap<String, String>,
) -> ProviderModelCreatePO {
    let default = default_map.get(&remote.id).or_else(|| {
        redirect_map
            .get(&remote.id)
            .and_then(|target| default_map.get(target))
    });

    ProviderModelCreatePO {
        provider_id,
        model: remote.id,
        display_name: or_default(remote.display_name, default, |model| {
            model.display_name.clone()
        }),
        reasoning: or_default(remote.reasoning, default, |model| model.reasoning),
        levels: or_default(remote.levels, default, |model| model.levels.clone()),
        level_default: or_default(remote.level_default, default, |model| {
            model.level_default.clone()
        }),
        context_window: or_default(remote.context_window, default, |model| model.context_window),
        max_tokens: or_default(remote.max_tokens, default, |model| model.max_tokens),
        support_tools: or_default(remote.support_tools, default, |model| model.support_tools),
        support_vision: or_default(remote.support_vision, default, |model| model.support_vision),
        support_stream: or_default(remote.support_stream, default, |model| model.support_stream),
        support_json: or_default(remote.support_json, default, |model| model.support_json),
        support_cache: or_default(remote.support_cache, default, |model| model.support_cache),
        knowledge_cutoff: or_default(remote.knowledge_cutoff, default, |model| {
            model.knowledge_cutoff.clone()
        }),
        release_date: or_default(remote.release_date, default, |model| {
            model.release_date.clone()
        }),
    }
}

/// 远程值优先；远程未返回时取默认模型配置的同名字段，无默认模型时取类型零值。
fn or_default<T, F>(remote: Option<T>, default: Option<&ProviderModel>, pick: F) -> T
where
    T: Clone + Default,
    F: FnOnce(&ProviderModel) -> T,
{
    remote.or_else(|| default.map(pick)).unwrap_or_default()
}

/// 供应商模型列表返回的单个模型，未返回的字段为 `None`。
#[derive(Debug, Clone, Deserialize)]
struct RemoteModel {
    id: String,
    display_name: Option<String>,
    reasoning: Option<bool>,
    levels: Option<Vec<String>>,
    level_default: Option<String>,
    context_window: Option<i64>,
    max_tokens: Option<i64>,
    support_tools: Option<bool>,
    support_vision: Option<bool>,
    support_stream: Option<bool>,
    support_json: Option<bool>,
    support_cache: Option<bool>,
    knowledge_cutoff: Option<String>,
    release_date: Option<String>,
}

/// 供应商模型列表响应。
#[derive(Debug, Clone, Deserialize)]
struct RemoteModelList {
    #[serde(default)]
    data: Vec<RemoteModel>,
}
