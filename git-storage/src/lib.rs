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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{branch, commit, tag, tree};
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, Set};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_entities_compile() {
        // This test just verifies that the entity models compile correctly
        // without actually creating database tables

        let repo_id = Uuid::new_v4();

        // Test commit entity creation (not insertion)
        let _commit = commit::ActiveModel {
            id: Set("commit123".to_string()),
            repository_id: Set(repo_id),
            parent_ids: Set(Some("[]".to_string())),
            tree_id: Set("tree123".to_string()),
            author_name: Set("Test Author".to_string()),
            author_email: Set("author@test.com".to_string()),
            author_date: Set(Utc::now().into()),
            committer_name: Set("Test Committer".to_string()),
            committer_email: Set("committer@test.com".to_string()),
            committer_date: Set(Utc::now().into()),
            message: Set("Test commit message".to_string()),
            content: Set(b"commit content".to_vec()),
            created_at: Set(Utc::now().into()),
        };

        // Test branch entity creation
        let _branch = branch::ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(repo_id),
            name: Set("main".to_string()),
            commit_id: Set("commit123".to_string()),
            is_default: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        // Test tag entity creation
        let _tag = tag::ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(repo_id),
            name: Set("v1.0.0".to_string()),
            target_id: Set("commit123".to_string()),
            target_type: Set("commit".to_string()),
            tag_object_id: Set(None),
            tagger_name: Set(Some("Test Tagger".to_string())),
            tagger_email: Set(Some("tagger@test.com".to_string())),
            tagger_date: Set(Some(Utc::now().into())),
            message: Set(Some("Version 1.0.0".to_string())),
            content: Set(Some(b"tag content".to_vec())),
            is_lightweight: Set(false),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        // Test tree entity creation
        let _tree = tree::ActiveModel {
            id: Set("tree123".to_string()),
            repository_id: Set(repo_id),
            entries: Set("[]".to_string()),
            size: Set(100),
            content: Set(b"tree content".to_vec()),
            created_at: Set(Utc::now().into()),
        };

        // If we get here without compiler errors, the entities are correctly defined
        assert!(true);
    }
}
