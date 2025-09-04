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
}

#[derive(Serialize, Deserialize)]
pub struct RepositoryResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
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
    let repository = match state.repository_service.get_repository_by_name(&repo_name).await {
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
    body: web::Bytes,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let repo_name = path.into_inner();
    
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
    
    match state
        .repository_service
        .create_repository(req.name, req.description, "main".to_string())
        .await
    {
        Ok(repo) => {
            let response = RepositoryResponse {
                id: repo.id.to_string(),
                name: repo.name,
                description: repo.description,
                default_branch: repo.default_branch,
                created_at: repo.created_at.to_string(),
            };
            Ok(HttpResponse::Created().json(response))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to create repository")),
    }
}