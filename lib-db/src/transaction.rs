use std::panic::AssertUnwindSafe;

use anyhow::{Context, Result, anyhow};
use futures_util::FutureExt;
use sqlx::{PgPool, Postgres, Transaction};

/// PostgreSQL 事务。
pub type PgTransaction<'connection> = Transaction<'connection, Postgres>;

#[allow(async_fn_in_trait)]
pub trait PgPoolExt {
    /// 在事务中执行操作。
    async fn with_transaction<F, T>(&self, operation: F) -> Result<T>
    where
        F: AsyncFnOnce(&mut PgTransaction<'_>) -> Result<T>;
}

impl PgPoolExt for PgPool {
    async fn with_transaction<F, T>(&self, operation: F) -> Result<T>
    where
        F: AsyncFnOnce(&mut PgTransaction<'_>) -> Result<T>,
    {
        with(self, operation).await
    }
}

/// 在事务中执行操作：正常返回则提交，出错或 panic 则回滚。
pub async fn with<F, T>(pool: &PgPool, operation: F) -> Result<T>
where
    F: AsyncFnOnce(&mut PgTransaction<'_>) -> Result<T>,
{
    let mut transaction = pool.begin().await.context("开启数据库事务失败")?;
    let result = AssertUnwindSafe(async { operation(&mut transaction).await })
        .catch_unwind()
        .await;

    match result {
        Ok(Ok(value)) => {
            transaction.commit().await.context("提交数据库事务失败")?;
            Ok(value)
        }
        Ok(Err(error)) => {
            transaction.rollback().await.context("回滚数据库事务失败")?;
            Err(error)
        }
        Err(_) => {
            transaction.rollback().await.context("回滚数据库事务失败")?;
            Err(anyhow!("事务执行发生 panic"))
        }
    }
}
