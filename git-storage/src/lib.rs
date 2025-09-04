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
    use sea_orm::{EntityTrait, ActiveModelTrait, Set, ColumnTrait, QueryFilter};
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

    #[tokio::test]
    async fn test_migrations_work() {
        // Test that migrations can run successfully
        let db = init_db("sqlite::memory:").await.unwrap();
        run_migrations(&db).await.unwrap();

        // If we get here, the migrations worked
        assert!(true);
    }

    #[tokio::test]
    async fn test_separate_tables_created() {
        use sea_orm::{Statement, ConnectionTrait};
        
        // Test that the separate tables can be created and basic data inserted
        let db = init_db("sqlite::memory:").await.unwrap();
        run_migrations(&db).await.unwrap();

        // Disable foreign key constraints for this test
        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "PRAGMA foreign_keys = OFF".to_string(),
        ))
        .await
        .unwrap();

        // This test doesn't depend on existing repositories/users
        // It just tests that our new tables work independently

        let fake_repo_id = Uuid::new_v4(); // This won't be in the repo table, but that's OK for this test

        // Test commit insertion
        let commit = commit::ActiveModel {
            id: Set("commit123abcdef".to_string()),
            repository_id: Set(fake_repo_id),
            parent_ids: Set(Some("[]".to_string())),
            tree_id: Set("tree123abcdef".to_string()),
            author_name: Set("Test Author".to_string()),
            author_email: Set("author@test.com".to_string()),
            author_date: Set(Utc::now().into()),
            committer_name: Set("Test Committer".to_string()),
            committer_email: Set("committer@test.com".to_string()),
            committer_date: Set(Utc::now().into()),
            message: Set("Initial commit".to_string()),
            content: Set(b"commit content bytes".to_vec()),
            created_at: Set(Utc::now().into()),
        };
        let commit_result = commit.insert(&db).await.unwrap();
        assert_eq!(commit_result.id, "commit123abcdef");

        // Test tree insertion  
        let tree = tree::ActiveModel {
            id: Set("tree123abcdef".to_string()),
            repository_id: Set(fake_repo_id),
            entries: Set(r#"[{"mode": "100644", "name": "README.md", "sha": "blob123"}]"#.to_string()),
            size: Set(1024),
            content: Set(b"tree object content bytes".to_vec()),
            created_at: Set(Utc::now().into()),
        };
        let tree_result = tree.insert(&db).await.unwrap();
        assert_eq!(tree_result.id, "tree123abcdef");

        // Test branch insertion
        let branch = branch::ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(fake_repo_id),
            name: Set("main".to_string()),
            commit_id: Set("commit123abcdef".to_string()),
            is_default: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        let branch_result = branch.insert(&db).await.unwrap();
        assert_eq!(branch_result.name, "main");

        // Test tag insertion
        let tag = tag::ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(fake_repo_id),
            name: Set("v1.0.0".to_string()),
            target_id: Set("commit123abcdef".to_string()),
            target_type: Set("commit".to_string()),
            tag_object_id: Set(Some("tag123abcdef".to_string())),
            tagger_name: Set(Some("Test Tagger".to_string())),
            tagger_email: Set(Some("tagger@test.com".to_string())),
            tagger_date: Set(Some(Utc::now().into())),
            message: Set(Some("Release version 1.0.0".to_string())),
            content: Set(Some(b"tag object content".to_vec())),
            is_lightweight: Set(false),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        let tag_result = tag.insert(&db).await.unwrap();
        assert_eq!(tag_result.name, "v1.0.0");

        // Test querying the data back
        let found_commit = commit::Entity::find_by_id("commit123abcdef").one(&db).await.unwrap().unwrap();
        assert_eq!(found_commit.message, "Initial commit");

        let found_tree = tree::Entity::find_by_id("tree123abcdef").one(&db).await.unwrap().unwrap();
        assert_eq!(found_tree.size, 1024);

        println!("All separate table operations successful!");
    }
}
