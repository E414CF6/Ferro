#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DbQueryResult {
    pub rows_affected: u64,
}

impl DbQueryResult {
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

macro_rules! impl_cmp {
    ($($t:ty),*) => {
        $(
            impl PartialEq<$t> for DbQueryResult {
                fn eq(&self, other: &$t) -> bool {
                    self.rows_affected == (*other as u64)
                }
            }
            impl PartialOrd<$t> for DbQueryResult {
                fn partial_cmp(&self, other: &$t) -> Option<std::cmp::Ordering> {
                    self.rows_affected.partial_cmp(&(*other as u64))
                }
            }
        )*
    };
}
impl_cmp!(u64, u32, usize, i64, i32, isize);

#[macro_export]
macro_rules! db_fetch_all {
    ($db:expr, $entity:ty, $sql:expr $(, $bind:expr)*) => {
        match $db.backend() {
            $crate::infrastructure::db::postgres::DatabaseBackend::Postgres(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query_as::<sqlx::Postgres, $entity>($sql);
                $(query = query.bind($bind);)*
                query.fetch_all(pool).await
            }
            $crate::infrastructure::db::postgres::DatabaseBackend::Sqlite(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query_as::<sqlx::Sqlite, $entity>($sql);
                $(query = query.bind($bind);)*
                query.fetch_all(pool).await
            }
        }
    };
}

#[macro_export]
macro_rules! db_fetch_optional {
    ($db:expr, $entity:ty, $sql:expr $(, $bind:expr)*) => {
        match $db.backend() {
            $crate::infrastructure::db::postgres::DatabaseBackend::Postgres(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query_as::<sqlx::Postgres, $entity>($sql);
                $(query = query.bind($bind);)*
                query.fetch_optional(pool).await
            }
            $crate::infrastructure::db::postgres::DatabaseBackend::Sqlite(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query_as::<sqlx::Sqlite, $entity>($sql);
                $(query = query.bind($bind);)*
                query.fetch_optional(pool).await
            }
        }
    };
}

#[macro_export]
macro_rules! db_fetch_one {
    ($db:expr, $entity:ty, $sql:expr $(, $bind:expr)*) => {
        match $db.backend() {
            $crate::infrastructure::db::postgres::DatabaseBackend::Postgres(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query_as::<sqlx::Postgres, $entity>($sql);
                $(query = query.bind($bind);)*
                query.fetch_one(pool).await
            }
            $crate::infrastructure::db::postgres::DatabaseBackend::Sqlite(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query_as::<sqlx::Sqlite, $entity>($sql);
                $(query = query.bind($bind);)*
                query.fetch_one(pool).await
            }
        }
    };
}

#[macro_export]
macro_rules! db_execute {
    ($db:expr, $sql:expr $(, $bind:expr)*) => {
        match $db.backend() {
            $crate::infrastructure::db::postgres::DatabaseBackend::Postgres(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query::<sqlx::Postgres>($sql);
                $(query = query.bind($bind);)*
                query.execute(pool).await.map(|r| {
                    $crate::infrastructure::db::macros::DbQueryResult {
                        rows_affected: sqlx::postgres::PgQueryResult::rows_affected(&r),
                    }
                })
            }
            $crate::infrastructure::db::postgres::DatabaseBackend::Sqlite(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query::<sqlx::Sqlite>($sql);
                $(query = query.bind($bind);)*
                query.execute(pool).await.map(|r| {
                    $crate::infrastructure::db::macros::DbQueryResult {
                        rows_affected: sqlx::sqlite::SqliteQueryResult::rows_affected(&r),
                    }
                })
            }
        }
    };
}

#[macro_export]
macro_rules! db_scalar {
    ($db:expr, $ty:ty, $sql:expr $(, $bind:expr)*) => {
        match $db.backend() {
            $crate::infrastructure::db::postgres::DatabaseBackend::Postgres(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query_scalar::<sqlx::Postgres, $ty>($sql);
                $(query = query.bind($bind);)*
                query.fetch_one(pool).await
            }
            $crate::infrastructure::db::postgres::DatabaseBackend::Sqlite(pool) => {
                #[allow(unused_mut)]
                let mut query = sqlx::query_scalar::<sqlx::Sqlite, $ty>($sql);
                $(query = query.bind($bind);)*
                query.fetch_one(pool).await
            }
        }
    };
}
