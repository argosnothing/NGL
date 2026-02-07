pub mod entities;
use sea_orm::{Database, DatabaseConnection, DbErr};

pub use entities::{example, provider};

use migration::{Migrator, MigratorTrait};

pub async fn establish_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}
