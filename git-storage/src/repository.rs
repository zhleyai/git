use crate::entities::{git_object, git_ref, repository};
use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub struct RepositoryService {
    db: DatabaseConnection,
    blob_storage_path: PathBuf,
}

impl RepositoryService {
    pub fn new(db: DatabaseConnection, blob_storage_path: Option<PathBuf>) -> Self {
        let blob_storage_path = blob_storage_path
            .unwrap_or_else(|| PathBuf::from("./blob_storage"));
        
        // Create blob storage directory if it doesn't exist
        if !blob_storage_path.exists() {
            std::fs::create_dir_all(&blob_storage_path).ok();
        }

        Self { db, blob_storage_path }
    }

    /// Create a new repository
    pub async fn create_repository(
        &self,
        name: String,
        description: Option<String>,
        default_branch: String,
        owner_id: Uuid,
        is_private: bool,
    ) -> Result<repository::Model> {
        let repo = repository::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name),
            description: Set(description),
            default_branch: Set(default_branch),
            owner_id: Set(owner_id),
            is_private: Set(is_private),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let result = repo.insert(&self.db).await?;
        Ok(result)
    }

    /// Get repository by name and owner
    pub async fn get_repository_by_name_and_owner(
        &self, 
        name: &str, 
        owner_id: Uuid
    ) -> Result<Option<repository::Model>> {
        let repo = repository::Entity::find()
            .filter(repository::Column::Name.eq(name))
            .filter(repository::Column::OwnerId.eq(owner_id))
            .one(&self.db)
            .await?;
        Ok(repo)
    }

    /// Get repository by name (for backwards compatibility)
    pub async fn get_repository_by_name(&self, name: &str) -> Result<Option<repository::Model>> {
        let repo = repository::Entity::find()
            .filter(repository::Column::Name.eq(name))
            .one(&self.db)
            .await?;
        Ok(repo)
    }

    /// Get repository by ID
    pub async fn get_repository_by_id(&self, id: Uuid) -> Result<Option<repository::Model>> {
        let repo = repository::Entity::find_by_id(id).one(&self.db).await?;
        Ok(repo)
    }

    /// List repositories by owner
    pub async fn list_repositories_by_owner(&self, owner_id: Uuid) -> Result<Vec<repository::Model>> {
        let repos = repository::Entity::find()
            .filter(repository::Column::OwnerId.eq(owner_id))
            .all(&self.db)
            .await?;
        Ok(repos)
    }

    /// List all repositories
    pub async fn list_repositories(&self) -> Result<Vec<repository::Model>> {
        let repos = repository::Entity::find().all(&self.db).await?;
        Ok(repos)
    }

    /// Delete repository
    pub async fn delete_repository(&self, id: Uuid) -> Result<()> {
        repository::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// Store a Git object (handles different storage for blobs vs other objects)
    pub async fn store_object(
        &self,
        repository_id: Uuid,
        object_id: String,
        object_type: String,
        size: i64,
        content: Vec<u8>,
    ) -> Result<git_object::Model> {
        let (db_content, blob_path) = if object_type == "blob" {
            // Store blob in filesystem
            let blob_path = self.get_blob_path(&object_id);
            
            // Create directory structure if it doesn't exist
            if let Some(parent) = blob_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Write blob content to file
            fs::write(&blob_path, &content)?;
            
            // Store only metadata in database
            (None, Some(blob_path.to_string_lossy().to_string()))
        } else {
            // Store commit, tree, tag objects in database
            (Some(content), None)
        };

        let obj = git_object::ActiveModel {
            id: Set(object_id),
            repository_id: Set(repository_id),
            object_type: Set(object_type),
            size: Set(size),
            content: Set(db_content),
            blob_path: Set(blob_path),
            created_at: Set(Utc::now().into()),
        };

        let result = obj.insert(&self.db).await?;
        Ok(result)
    }

    /// Get a Git object (handles reading from filesystem for blobs)
    pub async fn get_object(&self, object_id: &str) -> Result<Option<GitObjectWithContent>> {
        let obj = git_object::Entity::find_by_id(object_id)
            .one(&self.db)
            .await?;
        
        if let Some(obj) = obj {
            let content = if obj.object_type == "blob" && obj.blob_path.is_some() {
                // Read blob content from filesystem
                let blob_path = obj.blob_path.as_ref().unwrap();
                match fs::read(blob_path) {
                    Ok(content) => content,
                    Err(_) => {
                        return Err(anyhow!("Failed to read blob file: {}", blob_path));
                    }
                }
            } else if let Some(content) = obj.content.clone() {
                content
            } else {
                return Err(anyhow!("Object content not found"));
            };

            Ok(Some(GitObjectWithContent {
                id: obj.id,
                repository_id: obj.repository_id,
                object_type: obj.object_type,
                size: obj.size,
                content,
                created_at: obj.created_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get blob path for storage
    fn get_blob_path(&self, object_id: &str) -> PathBuf {
        // Use git-like directory structure: first 2 chars as directory, rest as filename
        let (dir, filename) = object_id.split_at(2);
        self.blob_storage_path.join(dir).join(filename)
    }

    /// Get objects by repository
    pub async fn get_objects_by_repository(
        &self,
        repository_id: Uuid,
    ) -> Result<Vec<git_object::Model>> {
        let objects = git_object::Entity::find()
            .filter(git_object::Column::RepositoryId.eq(repository_id))
            .all(&self.db)
            .await?;
        Ok(objects)
    }

    /// Store or update a Git reference
    pub async fn store_ref(
        &self,
        repository_id: Uuid,
        name: String,
        target: String,
        is_symbolic: bool,
    ) -> Result<git_ref::Model> {
        // Check if ref already exists
        if let Some(existing_ref) = git_ref::Entity::find()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .filter(git_ref::Column::Name.eq(&name))
            .one(&self.db)
            .await?
        {
            // Update existing ref
            let mut ref_active: git_ref::ActiveModel = existing_ref.into();
            ref_active.target = Set(target);
            ref_active.is_symbolic = Set(is_symbolic);
            ref_active.updated_at = Set(Utc::now().into());
            let result = ref_active.update(&self.db).await?;
            Ok(result)
        } else {
            // Create new ref
            let git_ref = git_ref::ActiveModel {
                id: Set(Uuid::new_v4()),
                repository_id: Set(repository_id),
                name: Set(name),
                target: Set(target),
                is_symbolic: Set(is_symbolic),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };
            let result = git_ref.insert(&self.db).await?;
            Ok(result)
        }
    }

    /// Get references by repository
    pub async fn get_refs_by_repository(
        &self,
        repository_id: Uuid,
    ) -> Result<Vec<git_ref::Model>> {
        let refs = git_ref::Entity::find()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .all(&self.db)
            .await?;
        Ok(refs)
    }

    /// Get a specific reference
    pub async fn get_ref(
        &self,
        repository_id: Uuid,
        name: &str,
    ) -> Result<Option<git_ref::Model>> {
        let git_ref = git_ref::Entity::find()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .filter(git_ref::Column::Name.eq(name))
            .one(&self.db)
            .await?;
        Ok(git_ref)
    }

    /// Delete a reference
    pub async fn delete_ref(&self, repository_id: Uuid, name: &str) -> Result<()> {
        git_ref::Entity::delete_many()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .filter(git_ref::Column::Name.eq(name))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// Check if object exists
    pub async fn object_exists(&self, object_id: &str) -> Result<bool> {
        let count = git_object::Entity::find_by_id(object_id)
            .count(&self.db)
            .await?;
        Ok(count > 0)
    }

    /// Get repository statistics
    pub async fn get_repository_stats(&self, repository_id: Uuid) -> Result<RepositoryStats> {
        let object_count = git_object::Entity::find()
            .filter(git_object::Column::RepositoryId.eq(repository_id))
            .count(&self.db)
            .await?;

        let ref_count = git_ref::Entity::find()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .count(&self.db)
            .await?;

        Ok(RepositoryStats {
            object_count,
            ref_count,
        })
    }
}

#[derive(Debug)]
pub struct RepositoryStats {
    pub object_count: u64,
    pub ref_count: u64,
}

#[derive(Debug, Clone)]
pub struct GitObjectWithContent {
    pub id: String,
    pub repository_id: Uuid,
    pub object_type: String,
    pub size: i64,
    pub content: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}