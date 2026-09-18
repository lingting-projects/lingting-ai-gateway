pub mod dto;
pub mod entity;

pub use dto::*;
pub use entity::*;

/// 默认模型数据使用的 provider_id；该数据仅作为模型基础配置，不参与路由与展示。
pub const DEFAULT_PROVIDER_ID: i64 = -20260918;
