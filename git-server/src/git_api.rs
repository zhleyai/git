use crate::AppState;
use actix_web::{web, HttpResponse, Result, get, post, delete};
use actix_session::Session;
use serde::{Deserialize, Serialize};
use git_storage::{GitOperations, CreateCommitRequest, MergeRequest};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct CreateBranchRequest {
    pub name: String,
    pub start_commit: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub target_commit: String,
}

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

/// List branches in a repository
#[get("/repositories/{repo_id}/branches")]
pub async fn list_branches(
    path: web::Path<String>,
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    // Check authentication
    let user_id = match get_authenticated_user(&session) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Authentication required".to_string(),
            }));
        }
    };

    let repo_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Invalid repository ID".to_string(),
            }));
        }
    };

    // Check repository access (simplified - in production, check permissions)
    match state.repository_service.get_repository(repo_id).await {
        Ok(Some(_)) => {
            let git_ops = GitOperations::new((*state.repository_service).clone());
            match git_ops.list_branches(repo_id).await {
                Ok(branches) => Ok(HttpResponse::Ok().json(ApiResponse {
                    success: true,
                    data: Some(branches),
                    message: "Branches retrieved successfully".to_string(),
                })),
                Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
                    success: false,
                    data: None,
                    message: format!("Failed to list branches: {}", e),
                })),
            }
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: "Repository not found".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: format!("Database error: {}", e),
        })),
    }
}

/// Create a new branch
#[post("/repositories/{repo_id}/branches")]
pub async fn create_branch(
    path: web::Path<String>,
    body: web::Json<CreateBranchRequest>,
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let user_id = match get_authenticated_user(&session) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Authentication required".to_string(),
            }));
        }
    };

    let repo_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Invalid repository ID".to_string(),
            }));
        }
    };

    let req = body.into_inner();

    // Validate branch name
    if req.name.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: "Branch name cannot be empty".to_string(),
        }));
    }

    let git_ops = GitOperations::new((*state.repository_service).clone());
    match git_ops.create_branch(repo_id, req.name, req.start_commit).await {
        Ok(branch_info) => Ok(HttpResponse::Created().json(ApiResponse {
            success: true,
            data: Some(branch_info),
            message: "Branch created successfully".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: format!("Failed to create branch: {}", e),
        })),
    }
}

/// Delete a branch
#[delete("/repositories/{repo_id}/branches/{branch_name}")]
pub async fn delete_branch(
    path: web::Path<(String, String)>,
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let user_id = match get_authenticated_user(&session) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Authentication required".to_string(),
            }));
        }
    };

    let (repo_id_str, branch_name) = path.into_inner();
    let repo_id = match Uuid::parse_str(&repo_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Invalid repository ID".to_string(),
            }));
        }
    };

    let git_ops = GitOperations::new((*state.repository_service).clone());
    match git_ops.delete_branch(repo_id, branch_name).await {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::<()> {
            success: true,
            data: None,
            message: "Branch deleted successfully".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: format!("Failed to delete branch: {}", e),
        })),
    }
}

/// List tags in a repository
#[get("/repositories/{repo_id}/tags")]
pub async fn list_tags(
    path: web::Path<String>,
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let user_id = match get_authenticated_user(&session) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Authentication required".to_string(),
            }));
        }
    };

    let repo_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Invalid repository ID".to_string(),
            }));
        }
    };

    let git_ops = GitOperations::new((*state.repository_service).clone());
    match git_ops.list_tags(repo_id).await {
        Ok(tags) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(tags),
            message: "Tags retrieved successfully".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: format!("Failed to list tags: {}", e),
        })),
    }
}

/// Create a new tag
#[post("/repositories/{repo_id}/tags")]
pub async fn create_tag(
    path: web::Path<String>,
    body: web::Json<CreateTagRequest>,
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let user_id = match get_authenticated_user(&session) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Authentication required".to_string(),
            }));
        }
    };

    let repo_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Invalid repository ID".to_string(),
            }));
        }
    };

    let req = body.into_inner();

    if req.name.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: "Tag name cannot be empty".to_string(),
        }));
    }

    let git_ops = GitOperations::new((*state.repository_service).clone());
    match git_ops.create_lightweight_tag(repo_id, req.name, req.target_commit).await {
        Ok(tag_info) => Ok(HttpResponse::Created().json(ApiResponse {
            success: true,
            data: Some(tag_info),
            message: "Tag created successfully".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: format!("Failed to create tag: {}", e),
        })),
    }
}

/// Create a new commit
#[post("/repositories/{repo_id}/commits")]
pub async fn create_commit(
    path: web::Path<String>,
    body: web::Json<CreateCommitRequest>,
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let user_id = match get_authenticated_user(&session) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Authentication required".to_string(),
            }));
        }
    };

    let repo_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Invalid repository ID".to_string(),
            }));
        }
    };

    let git_ops = GitOperations::new((*state.repository_service).clone());
    match git_ops.create_commit(repo_id, body.into_inner()).await {
        Ok(commit_hash) => Ok(HttpResponse::Created().json(ApiResponse {
            success: true,
            data: Some(commit_hash),
            message: "Commit created successfully".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: format!("Failed to create commit: {}", e),
        })),
    }
}

/// Merge branches
#[post("/repositories/{repo_id}/merge")]
pub async fn merge_branches(
    path: web::Path<String>,
    body: web::Json<MergeRequest>,
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let user_id = match get_authenticated_user(&session) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Authentication required".to_string(),
            }));
        }
    };

    let repo_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Invalid repository ID".to_string(),
            }));
        }
    };

    let git_ops = GitOperations::new((*state.repository_service).clone());
    match git_ops.merge_branch(repo_id, body.into_inner()).await {
        Ok(merge_commit) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(merge_commit),
            message: "Branches merged successfully".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: format!("Failed to merge branches: {}", e),
        })),
    }
}

/// Get commit history for a branch
#[get("/repositories/{repo_id}/branches/{branch_name}/commits")]
pub async fn get_commit_history(
    path: web::Path<(String, String)>,
    query: web::Query<CommitHistoryQuery>,
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let user_id = match get_authenticated_user(&session) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Authentication required".to_string(),
            }));
        }
    };

    let (repo_id_str, branch_name) = path.into_inner();
    let repo_id = match Uuid::parse_str(&repo_id_str) {
        Ok(id) => id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(ApiResponse::<()> {
                success: false,
                data: None,
                message: "Invalid repository ID".to_string(),
            }));
        }
    };

    let git_ops = GitOperations::new((*state.repository_service).clone());
    match git_ops.get_commit_history(repo_id, branch_name, query.limit).await {
        Ok(commits) => Ok(HttpResponse::Ok().json(ApiResponse {
            success: true,
            data: Some(commits),
            message: "Commit history retrieved successfully".to_string(),
        })),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()> {
            success: false,
            data: None,
            message: format!("Failed to get commit history: {}", e),
        })),
    }
}

#[derive(Deserialize)]
pub struct CommitHistoryQuery {
    pub limit: Option<usize>,
}

/// Helper function to get authenticated user ID from session
fn get_authenticated_user(session: &Session) -> Option<Uuid> {
    session
        .get::<String>("user_id")
        .ok()
        .flatten()
        .and_then(|user_id_str| Uuid::parse_str(&user_id_str).ok())
}