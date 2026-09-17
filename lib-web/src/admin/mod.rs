use anyhow::{Result, anyhow};
use framework_web::{Json, use_web, web_api_post};
use lib_db::{KvConfigRepository, ProviderRepository, RequestMainRepository, RequestSubRepository, use_db};
use lib_web_core::{AuthRuleExt, AuthType};
use serde::{Deserialize, Serialize};
use types_admin::dto::{KvConfigQO, ProviderQO, RequestMainQO, RequestSubQO};
use types_admin::entity::KvConfig;

/// 请求日志管理
pub mod request;
/// API Key 管理
pub mod api_key;
/// 配置管理
pub mod config;
/// 供应商管理
pub mod provider;
