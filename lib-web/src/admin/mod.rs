use anyhow::{Result, anyhow};
use framework_web::{Json, WebError, catch_panic, use_web, web_api_post};
use lib_db::{DbContext, use_db, RequestMainRepository, RequestSubRepository, ProviderRepository, KvConfigRepository};
use lib_web_core::{AppContext, AuthRule, AuthRuleExt, use_app, Authorization, AuthType};
use types_admin::entity::{RequestMain, RequestSub, Provider, KvConfig};
use types_admin::dto::{RequestMainQO, RequestSubQO, ProviderQO, KvConfigQO};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 请求日志管理
pub mod request;
/// API Key 管理
pub mod api_key;
/// 配置管理
pub mod config;
/// 供应商管理
pub mod provider;
