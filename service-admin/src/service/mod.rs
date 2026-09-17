//! 业务服务层：一张表一个服务，只编排本表的数据访问，不直接编写 SQL。

pub mod api_key_service;
pub mod config_service;
pub mod provider_model_service;
pub mod provider_service;
pub mod request_service;

pub use api_key_service::ApiKeyService;
pub use config_service::ConfigService;
pub use provider_model_service::ProviderModelService;
pub use provider_service::ProviderService;
pub use request_service::RequestService;

use anyhow::Result;
use lib_db::{PgPool, use_db};

/// 取当前请求的数据库连接池。
pub(crate) fn pool() -> Result<PgPool> {
    Ok(use_db()?.pool().clone())
}
