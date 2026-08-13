use lib_core::{
    Id, NewRequestLog, ProviderConfig, ProviderModel, RequestCompletion, RequestFailure,
    RequestStatus, ResolvedModel,
};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Clone)]
pub struct Store {
    pool: PgPool,
}

impl Store {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn anonymous_access(&self) -> Result<bool, sqlx::Error> {
        let value: Option<Value> =
            sqlx::query_scalar("SELECT value FROM settings WHERE key = 'anonymous_access'")
                .fetch_optional(&self.pool)
                .await?;
        Ok(value.and_then(|value| value.as_bool()).unwrap_or(false))
    }

    pub async fn authenticate_hash(&self, hash: &str) -> Result<Option<Id>, sqlx::Error> {
        let id: Option<i64> =
            sqlx::query_scalar("SELECT id FROM api_keys WHERE hash = $1 AND enabled")
                .bind(hash)
                .fetch_optional(&self.pool)
                .await?;
        Ok(id.map(Id::from_value))
    }

    async fn default_provider(&self) -> Result<Option<ProviderConfig>, sqlx::Error> {
        let value: Option<Value> =
            sqlx::query_scalar("SELECT value FROM settings WHERE key = 'default_provider_id'")
                .fetch_optional(&self.pool)
                .await?;
        let Some(id) = value.and_then(|value| value.as_i64()) else {
            return Ok(None);
        };
        let row: Option<ProviderRow> = sqlx::query_as("SELECT id, name, base_url, api_key FROM providers WHERE id = $1 AND enabled AND deleted_at = 0").bind(id).fetch_optional(&self.pool).await?;
        Ok(row.map(Into::into))
    }

    pub async fn resolve_model(&self, model: &str) -> Result<Option<ResolvedModel>, sqlx::Error> {
        let Some(provider) = self.default_provider().await? else {
            return Ok(None);
        };
        let row: Option<ModelRow> = sqlx::query_as("SELECT id, provider_id, model_name, upstream_model FROM provider_models WHERE provider_id = $1 AND model_name = $2 AND enabled").bind(provider.id.value()).bind(model).fetch_optional(&self.pool).await?;
        Ok(row.map(|row| ResolvedModel {
            client_model: model.to_owned(),
            provider,
            provider_model: row.into(),
        }))
    }

    pub async fn default_models(&self) -> Result<Option<Vec<String>>, sqlx::Error> {
        let Some(provider) = self.default_provider().await? else {
            return Ok(None);
        };
        let models = sqlx::query_scalar("SELECT model_name FROM provider_models WHERE provider_id = $1 AND enabled ORDER BY model_name").bind(provider.id.value()).fetch_all(&self.pool).await?;
        Ok(Some(models))
    }
    pub async fn default_provider_available(&self) -> Result<bool, sqlx::Error> {
        Ok(self.default_provider().await?.is_some())
    }

    pub async fn logical_delete_provider(&self, id: Id) -> anyhow::Result<bool> {
        let deleted_at = lib_core::current_millis()?;
        let result = sqlx::query("UPDATE providers SET deleted_at = $2, updated_at = $2 WHERE id = $1 AND deleted_at = 0")
            .bind(id.value())
            .bind(deleted_at)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn create_request(&self, request: &NewRequestLog) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO requests (id, api_key_id, client_model, provider_id, provider_model, request_type, status, started_at, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$8)")
            .bind(request.id.value()).bind(request.api_key_id.value()).bind(&request.client_model).bind(request.provider_id.value()).bind(&request.provider_model).bind(request.request_type.as_str()).bind(request.status.as_str()).bind(request.started_at).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn complete_request(
        &self,
        id: Id,
        result: &RequestCompletion,
    ) -> Result<(), sqlx::Error> {
        let usage = &result.usage;
        sqlx::query("UPDATE requests SET status=$2, completed_at=$3, response_model=$4, status_code=$5, input_tokens=$6, output_tokens=$7, total_tokens=$8, cache_read_input_tokens=$9, cache_write_input_tokens=$10, reasoning_tokens=$11, input_audio_tokens=$12, output_audio_tokens=$13, latency_ms=$14 WHERE id=$1")
            .bind(id.value()).bind(RequestStatus::Completed.as_str()).bind(result.completed_at).bind(&result.response_model).bind(result.status_code).bind(usage.input_tokens).bind(usage.output_tokens).bind(usage.total_tokens).bind(usage.cache_read_input_tokens).bind(usage.cache_write_input_tokens).bind(usage.reasoning_tokens).bind(usage.input_audio_tokens).bind(usage.output_audio_tokens).bind(result.latency_ms).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn fail_request(
        &self,
        id: Id,
        status: RequestStatus,
        failure: &RequestFailure,
    ) -> Result<(), sqlx::Error> {
        let usage = &failure.usage;
        sqlx::query("UPDATE requests SET status=$2, completed_at=$3, response_model=$4, status_code=$5, input_tokens=$6, output_tokens=$7, total_tokens=$8, cache_read_input_tokens=$9, cache_write_input_tokens=$10, reasoning_tokens=$11, input_audio_tokens=$12, output_audio_tokens=$13, latency_ms=$14, error_type=$15, error_code=$16, error_message=$17 WHERE id=$1")
            .bind(id.value()).bind(status.as_str()).bind(failure.completed_at).bind(&failure.response_model).bind(failure.status_code).bind(usage.input_tokens).bind(usage.output_tokens).bind(usage.total_tokens).bind(usage.cache_read_input_tokens).bind(usage.cache_write_input_tokens).bind(usage.reasoning_tokens).bind(usage.input_audio_tokens).bind(usage.output_audio_tokens).bind(failure.latency_ms).bind(&failure.error_type).bind(&failure.error_code).bind(&failure.error_message).execute(&self.pool).await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct ProviderRow {
    id: i64,
    name: String,
    base_url: String,
    api_key: String,
}
impl From<ProviderRow> for ProviderConfig {
    fn from(row: ProviderRow) -> Self {
        Self {
            id: Id::from_value(row.id),
            name: row.name,
            base_url: row.base_url,
            api_key: row.api_key,
        }
    }
}
#[derive(sqlx::FromRow)]
struct ModelRow {
    id: i64,
    provider_id: i64,
    model_name: String,
    upstream_model: String,
}
impl From<ModelRow> for ProviderModel {
    fn from(row: ModelRow) -> Self {
        Self {
            id: Id::from_value(row.id),
            provider_id: Id::from_value(row.provider_id),
            model_name: row.model_name,
            upstream_model: row.upstream_model,
        }
    }
}
