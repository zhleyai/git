use crate::entities::{git_object, git_ref};
use crate::RepositoryService;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use git_protocol::objects::{Commit, ObjectHandler};
use git_protocol::{GitObject, ObjectType};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Advanced Git operations service
pub struct GitOperations {
    repository_service: RepositoryService,
    object_handler: ObjectHandler,
}

/// Branch information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub commit_hash: String,
    pub author: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub is_default: bool,
}

/// Tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    pub name: String,
    pub target_hash: String,
    pub tag_type: TagType,
    pub tagger: Option<String>,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagType {
    Lightweight,
    Annotated,
}

/// Commit creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommitRequest {
    pub tree_hash: String,
    pub parent_hashes: Vec<String>,
    pub author: String,
    pub committer: String,
    pub message: String,
}

/// Merge operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequest {
    pub source_branch: String,
    pub target_branch: String,
    pub author: String,
    pub message: String,
}

impl GitOperations {
    pub fn new(repository_service: RepositoryService) -> Self {
        Self {
            repository_service,
            object_handler: ObjectHandler::new(),
        }
    }

    /// Create a new commit
    pub async fn create_commit(
        &self,
        repository_id: Uuid,
        request: CreateCommitRequest,
    ) -> Result<String> {
        // Create commit object
        let commit = Commit {
            tree: request.tree_hash,
            parents: request.parent_hashes,
            author: request.author.clone(),
            committer: request.committer,
            message: request.message,
            author_date: Utc::now(),
            commit_date: Utc::now(),
        };

        let commit_object = self.object_handler.create_commit(&commit)?;
        let commit_hash = commit_object.id.clone();

        // Store the commit object
        self.store_git_object(repository_id, commit_object).await?;

        Ok(commit_hash)
    }

    /// Create a new branch
    pub async fn create_branch(
        &self,
        repository_id: Uuid,
        branch_name: String,
        start_commit: String,
    ) -> Result<BranchInfo> {
        let full_ref_name = format!("refs/heads/{}", branch_name);

        // Check if branch already exists
        if self.get_ref(repository_id, &full_ref_name).await?.is_some() {
            return Err(anyhow!("Branch '{}' already exists", branch_name));
        }

        // Create the reference
        let git_ref = git_ref::ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(repository_id),
            name: Set(full_ref_name),
            target: Set(start_commit.clone()),
            is_symbolic: Set(false),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        git_ref.insert(self.repository_service.get_db()).await?;

        // Get commit info for the branch
        let commit_info = self.get_commit_info(repository_id, &start_commit).await?;

        Ok(BranchInfo {
            name: branch_name,
            commit_hash: start_commit,
            author: commit_info.author,
            message: commit_info.message,
            created_at: Utc::now(),
            is_default: false,
        })
    }

    /// Delete a branch
    pub async fn delete_branch(&self, repository_id: Uuid, branch_name: String) -> Result<()> {
        let full_ref_name = format!("refs/heads/{}", branch_name);

        // Check if it's the default branch
        let repo = self.repository_service.get_repository_by_id(repository_id).await?
            .ok_or_else(|| anyhow!("Repository not found"))?;

        if repo.default_branch == branch_name {
            return Err(anyhow!("Cannot delete the default branch"));
        }

        // Delete the reference
        git_ref::Entity::delete_many()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .filter(git_ref::Column::Name.eq(full_ref_name))
            .exec(self.repository_service.get_db())
            .await?;

        Ok(())
    }

    /// List branches in a repository
    pub async fn list_branches(&self, repository_id: Uuid) -> Result<Vec<BranchInfo>> {
        let refs = git_ref::Entity::find()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .filter(git_ref::Column::Name.like("refs/heads/%"))
            .all(self.repository_service.get_db())
            .await?;

        let repo = self.repository_service.get_repository_by_id(repository_id).await?
            .ok_or_else(|| anyhow!("Repository not found"))?;

        let mut branches = Vec::new();
        for ref_model in refs {
            let branch_name = ref_model.name[11..].to_string(); // Remove "refs/heads/"
            let commit_info = self.get_commit_info(repository_id, &ref_model.target).await?;

            branches.push(BranchInfo {
                name: branch_name.clone(),
                commit_hash: ref_model.target,
                author: commit_info.author,
                message: commit_info.message,
                created_at: ref_model.created_at.into(),
                is_default: branch_name == repo.default_branch,
            });
        }

        Ok(branches)
    }

    /// Create a lightweight tag
    pub async fn create_lightweight_tag(
        &self,
        repository_id: Uuid,
        tag_name: String,
        target_commit: String,
    ) -> Result<TagInfo> {
        let full_ref_name = format!("refs/tags/{}", tag_name);

        // Check if tag already exists
        if self.get_ref(repository_id, &full_ref_name).await?.is_some() {
            return Err(anyhow!("Tag '{}' already exists", tag_name));
        }

        // Create the reference
        let git_ref = git_ref::ActiveModel {
            id: Set(Uuid::new_v4()),
            repository_id: Set(repository_id),
            name: Set(full_ref_name),
            target: Set(target_commit.clone()),
            is_symbolic: Set(false),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        git_ref.insert(self.repository_service.get_db()).await?;

        Ok(TagInfo {
            name: tag_name,
            target_hash: target_commit,
            tag_type: TagType::Lightweight,
            tagger: None,
            message: None,
            created_at: Utc::now(),
        })
    }

    /// List tags in a repository
    pub async fn list_tags(&self, repository_id: Uuid) -> Result<Vec<TagInfo>> {
        let refs = git_ref::Entity::find()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .filter(git_ref::Column::Name.like("refs/tags/%"))
            .all(self.repository_service.get_db())
            .await?;

        let mut tags = Vec::new();
        for ref_model in refs {
            let tag_name = ref_model.name[10..].to_string(); // Remove "refs/tags/"

            tags.push(TagInfo {
                name: tag_name,
                target_hash: ref_model.target,
                tag_type: TagType::Lightweight, // For now, assume all are lightweight
                tagger: None,
                message: None,
                created_at: ref_model.created_at.into(),
            });
        }

        Ok(tags)
    }

    /// Perform a simple merge (fast-forward only for now)
    pub async fn merge_branch(
        &self,
        repository_id: Uuid,
        request: MergeRequest,
    ) -> Result<String> {
        let source_ref = format!("refs/heads/{}", request.source_branch);
        let target_ref = format!("refs/heads/{}", request.target_branch);

        // Get current commits
        let source_commit = self.get_ref(repository_id, &source_ref).await?
            .ok_or_else(|| anyhow!("Source branch '{}' not found", request.source_branch))?;

        let target_commit = self.get_ref(repository_id, &target_ref).await?
            .ok_or_else(|| anyhow!("Target branch '{}' not found", request.target_branch))?;

        // For now, just do a fast-forward merge (update target to source)
        // In a full implementation, this would check if fast-forward is possible
        // and create a merge commit if necessary
        self.update_ref(repository_id, &target_ref, &source_commit.target).await?;

        Ok(source_commit.target)
    }

    /// Get commit history for a branch
    pub async fn get_commit_history(
        &self,
        repository_id: Uuid,
        branch_name: String,
        limit: Option<usize>,
    ) -> Result<Vec<Commit>> {
        let ref_name = format!("refs/heads/{}", branch_name);
        let branch_ref = self.get_ref(repository_id, &ref_name).await?
            .ok_or_else(|| anyhow!("Branch '{}' not found", branch_name))?;

        // For now, just return the single commit
        // In a full implementation, this would traverse the commit history
        let commit_info = self.get_commit_info(repository_id, &branch_ref.target).await?;
        Ok(vec![commit_info])
    }

    /// Helper: Store a Git object in the database
    async fn store_git_object(&self, repository_id: Uuid, obj: GitObject) -> Result<()> {
        let git_obj = git_object::ActiveModel {
            id: Set(obj.id),
            repository_id: Set(repository_id),
            object_type: Set(match obj.obj_type {
                ObjectType::Commit => "commit".to_string(),
                ObjectType::Tree => "tree".to_string(),
                ObjectType::Blob => "blob".to_string(),
                ObjectType::Tag => "tag".to_string(),
            }),
            size: Set(obj.size as i64),
            content: Set(Some(obj.content)),
            blob_path: Set(None),
            created_at: Set(Utc::now().into()),
        };

        git_obj.insert(self.repository_service.get_db()).await?;
        Ok(())
    }

    /// Helper: Get a reference by name
    async fn get_ref(&self, repository_id: Uuid, ref_name: &str) -> Result<Option<git_ref::Model>> {
        let git_ref = git_ref::Entity::find()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .filter(git_ref::Column::Name.eq(ref_name))
            .one(self.repository_service.get_db())
            .await?;

        Ok(git_ref)
    }

    /// Helper: Update a reference
    async fn update_ref(&self, repository_id: Uuid, ref_name: &str, new_hash: &str) -> Result<()> {
        let git_ref = git_ref::Entity::find()
            .filter(git_ref::Column::RepositoryId.eq(repository_id))
            .filter(git_ref::Column::Name.eq(ref_name))
            .one(self.repository_service.get_db())
            .await?
            .ok_or_else(|| anyhow!("Reference '{}' not found", ref_name))?;

        let mut active_ref: git_ref::ActiveModel = git_ref.into();
        active_ref.target = Set(new_hash.to_string());
        active_ref.updated_at = Set(Utc::now().into());

        active_ref.update(self.repository_service.get_db()).await?;
        Ok(())
    }

    /// Helper: Get commit information
    async fn get_commit_info(&self, repository_id: Uuid, commit_hash: &str) -> Result<Commit> {
        let git_obj = git_object::Entity::find()
            .filter(git_object::Column::RepositoryId.eq(repository_id))
            .filter(git_object::Column::Id.eq(commit_hash))
            .filter(git_object::Column::ObjectType.eq("commit"))
            .one(self.repository_service.get_db())
            .await?
            .ok_or_else(|| anyhow!("Commit '{}' not found", commit_hash))?;

        match &git_obj.content {
            Some(content) => self.object_handler.parse_commit(content),
            None => Err(anyhow!("Commit content is empty")),
        }
    }
}