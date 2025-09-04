use crate::entities::{git_object, git_ref, repository};
use anyhow::Result;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use uuid::Uuid;

pub struct RepositoryService {
    db: DatabaseConnection,
}

impl RepositoryService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new repository
    pub async fn create_repository(
        &self,
        name: String,
        description: Option<String>,
        default_branch: String,
    ) -> Result<repository::Model> {
        let repo = repository::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(name),
            description: Set(description),
            default_branch: Set(default_branch),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        let result = repo.insert(&self.db).await?;
        Ok(result)
    }

    /// Get repository by name
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

    /// Store a Git object
    pub async fn store_object(
        &self,
        repository_id: Uuid,
        object_id: String,
        object_type: String,
        size: i64,
        content: Vec<u8>,
    ) -> Result<git_object::Model> {
        let obj = git_object::ActiveModel {
            id: Set(object_id),
            repository_id: Set(repository_id),
            object_type: Set(object_type),
            size: Set(size),
            content: Set(content),
            created_at: Set(Utc::now().into()),
        };

        let result = obj.insert(&self.db).await?;
        Ok(result)
    }

    /// Get a Git object
    pub async fn get_object(&self, object_id: &str) -> Result<Option<git_object::Model>> {
        let obj = git_object::Entity::find_by_id(object_id)
            .one(&self.db)
            .await?;
        Ok(obj)
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