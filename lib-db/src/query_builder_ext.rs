use sqlx::{Encode, Postgres, QueryBuilder, Type};

/// 为 PostgreSQL [`QueryBuilder`] 追加可选筛选条件。
///
/// `column` 必须是调用方控制的可信 SQL 列表达式，值始终通过参数绑定传入。
pub trait QueryBuilderExt<'args> {
    /// 值存在时追加 `AND {column} = $n`。
    fn eq<T>(&mut self, column: &str, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>;

    /// 非空文本存在时追加不区分大小写的包含匹配。
    fn like<T>(&mut self, column: &str, value: Option<T>) -> &mut Self
    where
        T: AsRef<str>;

    /// 值存在时追加 `AND {column} >= $n`。
    fn ge<T>(&mut self, column: &str, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>;

    /// 值存在时追加 `AND {column} <= $n`。
    fn le<T>(&mut self, column: &str, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>;
}

impl<'args> QueryBuilderExt<'args> for QueryBuilder<'args, Postgres> {
    fn eq<T>(&mut self, column: &str, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        if let Some(value) = value {
            self.push(" AND ").push(column).push("=").push_bind(value);
        }
        self
    }

    fn like<T>(&mut self, column: &str, value: Option<T>) -> &mut Self
    where
        T: AsRef<str>,
    {
        if let Some(value) = value.filter(|value| !value.as_ref().is_empty()) {
            self.push(" AND ")
                .push(column)
                .push(" ILIKE ")
                .push_bind(format!("%{}%", value.as_ref()));
        }
        self
    }

    fn ge<T>(&mut self, column: &str, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        if let Some(value) = value {
            self.push(" AND ").push(column).push(">=").push_bind(value);
        }
        self
    }

    fn le<T>(&mut self, column: &str, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        if let Some(value) = value {
            self.push(" AND ").push(column).push("<=").push_bind(value);
        }
        self
    }
}
