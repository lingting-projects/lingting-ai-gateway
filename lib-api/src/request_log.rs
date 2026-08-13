use lib_core::{Id, NewRequestLog, RequestStatus, RequestType, ResolvedModel, current_millis};
use lib_store::Store;

pub(super) struct RequestLogContext {
    pub(super) id: Id,
    pub(super) started_at: i64,
}

pub(super) async fn create(
    store: &Store,
    api_key_id: Id,
    resolved: &ResolvedModel,
    request_type: RequestType,
) -> anyhow::Result<RequestLogContext> {
    let started_at = current_millis()?;
    let id = Id::new()?;
    let request = NewRequestLog {
        id,
        api_key_id,
        client_model: resolved.client_model.clone(),
        provider_id: resolved.provider.id,
        provider_model: resolved.provider_model.upstream_model.clone(),
        request_type,
        status: RequestStatus::Connecting,
        started_at,
    };
    store.create_request(&request).await?;

    Ok(RequestLogContext { id, started_at })
}
