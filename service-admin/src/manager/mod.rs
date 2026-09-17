//! 跨表编排：需要组合多张表数据时放在这里，单表逻辑仍归 service。

pub mod request_manager;

pub use request_manager::RequestManager;
