use crate::AppState;
use actix_web::{
    get, post, web, HttpResponse, Result,
};
use git_protocol::{GitProtocol, ProtocolHandler};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_private: Option<bool>,
    pub owner_id: Option<String>, // UUID as string
}

#[derive(Serialize, Deserialize)]
pub struct RepositoryResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub owner_id: String,
    pub is_private: bool,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
    pub is_admin: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    pub created_at: String,
}

/// Handle Git info/refs request
#[get("/{repo}/info/refs")]
pub async fn info_refs(
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let repo_name = path.into_inner();
    let service = query.get("service").cloned();

    // Get repository from database
    let repository = match state.repository_service.get_repository_by_name(&repo_name).await {
        Ok(Some(repo)) => repo,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json("Repository not found"));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json("Database error"));
        }
    };

    // Get references
    let refs = match state.repository_service.get_refs_by_repository(repository.id).await {
        Ok(refs) => refs,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json("Failed to get references"));
        }
    };

    let protocol = ProtocolHandler::new();
    let ref_pairs: Vec<(String, String)> = refs
        .into_iter()
        .map(|r| (r.name, r.target))
        .collect();

    let capabilities = match service.as_deref() {
        Some("git-upload-pack") => vec!["multi_ack", "side-band-64k", "ofs-delta"],
        Some("git-receive-pack") => vec!["report-status", "delete-refs", "ofs-delta"],
        _ => vec![],
    };

    let response_data = protocol.create_ref_advertisement(&ref_pairs, &capabilities);

    let content_type = match service.as_deref() {
        Some("git-upload-pack") => "application/x-git-upload-pack-advertisement",
        Some("git-receive-pack") => "application/x-git-receive-pack-advertisement",
        _ => "text/plain",
    };

    Ok(HttpResponse::Ok()
        .content_type(content_type)
        .body(response_data))
}

/// Handle Git upload-pack request
#[post("/{repo}/git-upload-pack")]
pub async fn upload_pack(
    path: web::Path<String>,
    body: web::Bytes,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let repo_name = path.into_inner();
    
    // Get repository from database
    let _repository = match state.repository_service.get_repository_by_name(&repo_name).await {
        Ok(Some(repo)) => repo,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json("Repository not found"));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json("Database error"));
        }
    };

    let protocol = ProtocolHandler::new();
    
    // Parse the request
    let pkt_lines = match protocol.parse_pkt_line(&body) {
        Ok(lines) => lines,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json("Invalid pkt-line format"));
        }
    };

    let (_wants, _haves) = match protocol.parse_want_have(&pkt_lines) {
        Ok(wh) => wh,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json("Invalid want/have format"));
        }
    };

    // For now, just return NAK (no objects to send)
    // In a full implementation, we would:
    // 1. Calculate which objects the client needs
    // 2. Create a pack file with those objects
    // 3. Send the pack file back
    let nak_response = protocol.create_nak();

    Ok(HttpResponse::Ok()
        .content_type("application/x-git-upload-pack-result")
        .body(nak_response))
}

/// Handle Git receive-pack request
#[post("/{repo}/git-receive-pack")]
pub async fn receive_pack(
    path: web::Path<String>,
    _body: web::Bytes,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let repo_name = path.into_inner();
    
    // Get repository from database
    let _repository = match state.repository_service.get_repository_by_name(&repo_name).await {
        Ok(Some(repo)) => repo,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json("Repository not found"));
        }
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json("Database error"));
        }
    };

    // For now, just accept the push
    // In a full implementation, we would:
    // 1. Parse the pack file
    // 2. Store the objects in the database
    // 3. Update the references
    // 4. Return appropriate status

    Ok(HttpResponse::Ok()
        .content_type("application/x-git-receive-pack-result")
        .body("unpack ok\n"))
}

/// List all repositories
#[get("/repositories")]
pub async fn list_repositories(state: web::Data<AppState>) -> Result<HttpResponse> {
    match state.repository_service.list_repositories().await {
        Ok(repos) => {
            let response: Vec<RepositoryResponse> = repos
                .into_iter()
                .map(|repo| RepositoryResponse {
                    id: repo.id.to_string(),
                    name: repo.name,
                    description: repo.description,
                    default_branch: repo.default_branch,
                    owner_id: repo.owner_id.to_string(),
                    is_private: repo.is_private,
                    created_at: repo.created_at.to_string(),
                })
                .collect();
            Ok(HttpResponse::Ok().json(response))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Database error")),
    }
}

/// Get a specific repository
#[get("/repositories/{name}")]
pub async fn get_repository(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let repo_name = path.into_inner();
    
    match state.repository_service.get_repository_by_name(&repo_name).await {
        Ok(Some(repo)) => {
            let response = RepositoryResponse {
                id: repo.id.to_string(),
                name: repo.name,
                description: repo.description,
                default_branch: repo.default_branch,
                owner_id: repo.owner_id.to_string(),
                is_private: repo.is_private,
                created_at: repo.created_at.to_string(),
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json("Repository not found")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Database error")),
    }
}

/// Create a new repository
#[post("/repositories")]
pub async fn create_repository(
    body: web::Json<CreateRepositoryRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let req = body.into_inner();
    
    // Parse owner_id if provided, otherwise use a default admin user (for demo)
    let owner_id = if let Some(owner_id_str) = req.owner_id {
        match uuid::Uuid::parse_str(&owner_id_str) {
            Ok(id) => id,
            Err(_) => return Ok(HttpResponse::BadRequest().json("Invalid owner_id format")),
        }
    } else {
        // For demo purposes, create a default admin user if none exists
        // In production, you'd want proper authentication
        match state.user_service.get_user_by_username("admin").await {
            Ok(Some(user)) => user.id,
            Ok(None) => {
                // Create default admin user
                match state
                    .user_service
                    .create_user(
                        "admin".to_string(),
                        "admin@example.com".to_string(),
                        "password_hash".to_string(), // In production, use proper password hashing
                        Some("Administrator".to_string()),
                        true,
                    )
                    .await
                {
                    Ok(admin_user) => admin_user.id,
                    Err(_) => return Ok(HttpResponse::InternalServerError().json("Failed to create default admin user")),
                }
            }
            Err(_) => return Ok(HttpResponse::InternalServerError().json("Database error")),
        }
    };
    
    match state
        .repository_service
        .create_repository(
            req.name,
            req.description,
            "main".to_string(),
            owner_id,
            req.is_private.unwrap_or(false),
        )
        .await
    {
        Ok(repo) => {
            let response = RepositoryResponse {
                id: repo.id.to_string(),
                name: repo.name,
                description: repo.description,
                default_branch: repo.default_branch,
                owner_id: repo.owner_id.to_string(),
                is_private: repo.is_private,
                created_at: repo.created_at.to_string(),
            };
            Ok(HttpResponse::Created().json(response))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to create repository")),
    }
}

// User Management API Endpoints

/// Create a new user
#[post("/users")]
pub async fn create_user(
    body: web::Json<CreateUserRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let req = body.into_inner();
    
    // Check if username or email already exists
    if let Ok(true) = state.user_service.username_exists(&req.username).await {
        return Ok(HttpResponse::Conflict().json("Username already exists"));
    }
    
    if let Ok(true) = state.user_service.email_exists(&req.email).await {
        return Ok(HttpResponse::Conflict().json("Email already exists"));
    }
    
    // In production, hash the password properly
    let password_hash = format!("hashed_{}", req.password); // Placeholder
    
    match state
        .user_service
        .create_user(
            req.username,
            req.email,
            password_hash,
            req.full_name,
            req.is_admin.unwrap_or(false),
        )
        .await
    {
        Ok(user) => {
            let response = UserResponse {
                id: user.id.to_string(),
                username: user.username,
                email: user.email,
                full_name: user.full_name,
                is_active: user.is_active,
                is_admin: user.is_admin,
                created_at: user.created_at.to_string(),
            };
            Ok(HttpResponse::Created().json(response))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to create user")),
    }
}

/// List all users
#[get("/users")]
pub async fn list_users(state: web::Data<AppState>) -> Result<HttpResponse> {
    match state.user_service.list_users().await {
        Ok(users) => {
            let response: Vec<UserResponse> = users
                .into_iter()
                .map(|user| UserResponse {
                    id: user.id.to_string(),
                    username: user.username,
                    email: user.email,
                    full_name: user.full_name,
                    is_active: user.is_active,
                    is_admin: user.is_admin,
                    created_at: user.created_at.to_string(),
                })
                .collect();
            Ok(HttpResponse::Ok().json(response))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Database error")),
    }
}

/// Get a specific user by username
#[get("/users/{username}")]
pub async fn get_user(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let username = path.into_inner();
    
    match state.user_service.get_user_by_username(&username).await {
        Ok(Some(user)) => {
            let response = UserResponse {
                id: user.id.to_string(),
                username: user.username,
                email: user.email,
                full_name: user.full_name,
                is_active: user.is_active,
                is_admin: user.is_admin,
                created_at: user.created_at.to_string(),
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json("User not found")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Database error")),
    }
}

/// Get repositories by user
#[get("/users/{username}/repositories")]
pub async fn get_user_repositories(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let username = path.into_inner();
    
    // Get user first
    let user = match state.user_service.get_user_by_username(&username).await {
        Ok(Some(user)) => user,
        Ok(None) => return Ok(HttpResponse::NotFound().json("User not found")),
        Err(_) => return Ok(HttpResponse::InternalServerError().json("Database error")),
    };
    
    // Get user's repositories
    match state.repository_service.list_repositories_by_owner(user.id).await {
        Ok(repos) => {
            let response: Vec<RepositoryResponse> = repos
                .into_iter()
                .map(|repo| RepositoryResponse {
                    id: repo.id.to_string(),
                    name: repo.name,
                    description: repo.description,
                    default_branch: repo.default_branch,
                    owner_id: repo.owner_id.to_string(),
                    is_private: repo.is_private,
                    created_at: repo.created_at.to_string(),
                })
                .collect();
            Ok(HttpResponse::Ok().json(response))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Database error")),
    }
}