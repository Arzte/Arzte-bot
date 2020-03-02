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

pub struct FancyPool {
    pub pooler: Pool<PgConnection>,
}

pub fn new_pool(runtime: Arc<Mutex<Runtime>>) -> Arc<FancyPool> {
    let pool = runtime
        .try_lock()
        .expect("Unable to get runtime lock to start database pool")
        .block_on(async {
            PgPool::new(
                &std::env::var("DATABASE_URL").expect("DATABASE_URL enviroment variable not set"),
            )
            .await
            .expect("unable to connect to db")
        });
    Arc::new(FancyPool { pooler: pool })
}
