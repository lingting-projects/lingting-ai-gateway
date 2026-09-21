use std::future::Future;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use sqlx::PgPool;

use crate::{PgTransaction, with};

/// 请求级数据库上下文，持有连接池。
pub struct DbContext {
    pool: PgPool,
}

impl DbContext {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// 连接池。
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// 在事务中执行操作，出错或 panic 时自动回滚。
    pub async fn with_transaction<F, T>(&self, operation: F) -> Result<T>
    where
        F: AsyncFnOnce(&mut PgTransaction<'_>) -> Result<T>,
    {
        with(&self.pool, operation).await
    }
}

tokio::task_local! {
    static DB_CONTEXT: Arc<DbContext>;
}

/// 在当前任务内注入数据库上下文。
pub async fn scope_db<F>(context: Arc<DbContext>, future: F) -> F::Output
where
    F: Future,
{
    DB_CONTEXT.scope(context, future).await
}

/// 取当前任务的数据库上下文。
pub fn use_db() -> Result<Arc<DbContext>> {
    DB_CONTEXT
        .try_with(Arc::clone)
        .map_err(|error| anyhow!("当前调用不在数据库上下文作用域内：{error:#}"))
}

/// 取当前任务的数据库连接池。
pub fn use_pool() -> Result<PgPool> {
    Ok(use_db()?.pool().clone())
}
