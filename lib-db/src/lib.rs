//! 数据库访问：基于 pglite-oxide 的嵌入式 PostgreSQL 连接池、请求级上下文与事务封装。
//!
//! 本 crate 不提供通用数据库抽象，只暴露 [`PgPool`] 与事务，具体 SQL 由各业务 crate 编写。

mod config;
mod context;
mod query_builder_ext;
mod transaction;

pub use config::{Db, DbConfig, init};
pub use context::{DbContext, scope_db, use_db};
pub use query_builder_ext::QueryBuilderExt;
pub use sqlx::PgPool;
pub use transaction::{PgPoolExt, PgTransaction, with};
