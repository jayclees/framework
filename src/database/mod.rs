use crate::env::Env;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::env;
use std::error::Error;
use std::time::Duration;

pub(crate) async fn db(env: &Env) -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let mut opt = ConnectOptions::new(env::var("DATABASE_URL")?);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false) // disable SQLx logging
        .sqlx_logging_level(log::LevelFilter::Info);
    let db = Database::connect(opt).await?;
    db.get_schema_registry("app::entity::*").sync(&db).await?;
    Ok(db)
}
