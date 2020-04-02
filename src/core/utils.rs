use sqlx::{
    PgConnection,
    PgPool,
    Pool,
};
use std::sync::{
    Arc,
    Mutex,
};
use tokio::runtime::Runtime;

// Struct to hold the database pool so it can be wrapped
// in an Arc.
// sqlx's pool type get's weird errors trying to send it
// across threads on it's own, however it won't accept
// pool wrapped in an Arc, so we put it in this struct,
// and wrap the struct in an arc
pub struct FancyPool {
    pub postgres_pool: Pool<PgConnection>,
}

impl FancyPool {
    /// Returns a new instance of FancyPool, using the tokio runtime
    pub fn new(runtime: Arc<Mutex<Runtime>>) -> Self {
        let pool = runtime
            .try_lock()
            .expect("Unable to get runtime lock to start database pool")
            .block_on(async {
                PgPool::new(
                    &std::env::var("DATABASE_URL")
                        .expect("DATABASE_URL enviroment variable not set"),
                )
                .await
                .expect("unable to connect to db")
            });
        FancyPool {
            postgres_pool: pool,
        }
    }
    /// Returns a reference to the database connection pool.
    /// This is a convenience function
    pub fn pool(&self) -> &Pool<PgConnection> {
        &self.postgres_pool
    }
}
