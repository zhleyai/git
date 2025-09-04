pub mod entities;
pub mod migrations;
pub mod repository;
pub mod user;

use anyhow::Result;
use sea_orm::{Database, DatabaseConnection};

pub use repository::*;
pub use user::*;

/// Initialize the database connection
pub async fn init_db(database_url: &str) -> Result<DatabaseConnection> {
    let db = Database::connect(database_url).await?;
    Ok(db)
}

/// Run database migrations
pub async fn run_migrations(db: &DatabaseConnection) -> Result<()> {
    use migrations::Migrator;
    use sea_orm_migration::MigratorTrait;
    
    Migrator::up(db, None).await?;
    Ok(())
}
