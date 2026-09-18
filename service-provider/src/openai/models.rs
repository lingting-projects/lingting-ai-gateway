//! 模型列表接口。

use anyhow::Result;
use lib_web_core::WebResponse;
use service_admin::service::provider_model_service::ProviderModelService;

use super::OpenaiService;
use super::common::{OpenAiModel, OpenAiModelList};
use super::utils;

impl OpenaiService {
    /// 可用模型列表：取全部已启用且按名称去重的供应商模型，以 [OI] 列表格式返回。
    pub async fn models() -> Result<WebResponse> {
        let models = ProviderModelService::new()?.find_enabled_distinct().await?;

        let data = models
            .into_iter()
            .map(|model| OpenAiModel::new(model.model, model.create_time / 1000))
            .collect::<Vec<_>>();

        utils::json_response(&OpenAiModelList::new(data))
    }
}
