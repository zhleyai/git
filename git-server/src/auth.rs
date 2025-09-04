use crate::AppState;
use actix_web::{get, post, web, HttpResponse, Result};
use actix_session::Session;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub user: Option<UserResponse>,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub user: Option<UserResponse>,
    pub message: String,
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

/// User login endpoint
#[post("/login")]
pub async fn login(
    body: web::Json<LoginRequest>,
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let req = body.into_inner();

    match state
        .user_service
        .authenticate(&req.username_or_email, &req.password)
        .await
    {
        Ok(Some(user)) => {
            if !user.is_active {
                return Ok(HttpResponse::Forbidden().json(LoginResponse {
                    success: false,
                    user: None,
                    message: "Account is not active".to_string(),
                }));
            }

            // Store user session
            if let Err(_) = session.insert("user_id", user.id.to_string()) {
                return Ok(HttpResponse::InternalServerError().json(LoginResponse {
                    success: false,
                    user: None,
                    message: "Failed to create session".to_string(),
                }));
            }

            let user_response = UserResponse {
                id: user.id.to_string(),
                username: user.username,
                email: user.email,
                full_name: user.full_name,
                is_active: user.is_active,
                is_admin: user.is_admin,
                created_at: user.created_at.to_string(),
            };

            Ok(HttpResponse::Ok().json(LoginResponse {
                success: true,
                user: Some(user_response),
                message: "Login successful".to_string(),
            }))
        }
        Ok(None) => Ok(HttpResponse::Unauthorized().json(LoginResponse {
            success: false,
            user: None,
            message: "Invalid credentials".to_string(),
        })),
        Err(_) => Ok(HttpResponse::InternalServerError().json(LoginResponse {
            success: false,
            user: None,
            message: "Login failed due to server error".to_string(),
        })),
    }
}

/// User registration endpoint
#[post("/register")]
pub async fn register(
    body: web::Json<RegisterRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let req = body.into_inner();

    // Validate input
    if req.username.trim().is_empty() {
        return Ok(HttpResponse::BadRequest().json(RegisterResponse {
            success: false,
            user: None,
            message: "Username cannot be empty".to_string(),
        }));
    }

    if req.email.trim().is_empty() || !req.email.contains('@') {
        return Ok(HttpResponse::BadRequest().json(RegisterResponse {
            success: false,
            user: None,
            message: "Valid email is required".to_string(),
        }));
    }

    if req.password.len() < 6 {
        return Ok(HttpResponse::BadRequest().json(RegisterResponse {
            success: false,
            user: None,
            message: "Password must be at least 6 characters".to_string(),
        }));
    }

    // Check if username or email already exists
    if let Ok(true) = state.user_service.username_exists(&req.username).await {
        return Ok(HttpResponse::Conflict().json(RegisterResponse {
            success: false,
            user: None,
            message: "Username already exists".to_string(),
        }));
    }

    if let Ok(true) = state.user_service.email_exists(&req.email).await {
        return Ok(HttpResponse::Conflict().json(RegisterResponse {
            success: false,
            user: None,
            message: "Email already exists".to_string(),
        }));
    }

    // Hash password
    let password_hash = match state.user_service.hash_password(&req.password) {
        Ok(hash) => hash,
        Err(_) => {
            return Ok(HttpResponse::InternalServerError().json(RegisterResponse {
                success: false,
                user: None,
                message: "Failed to process password".to_string(),
            }))
        }
    };

    // Create user
    match state
        .user_service
        .create_user(
            req.username,
            req.email,
            password_hash,
            req.full_name,
            false, // New users are not admin by default
        )
        .await
    {
        Ok(user) => {
            let user_response = UserResponse {
                id: user.id.to_string(),
                username: user.username,
                email: user.email,
                full_name: user.full_name,
                is_active: user.is_active,
                is_admin: user.is_admin,
                created_at: user.created_at.to_string(),
            };

            Ok(HttpResponse::Created().json(RegisterResponse {
                success: true,
                user: Some(user_response),
                message: "Registration successful".to_string(),
            }))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json(RegisterResponse {
            success: false,
            user: None,
            message: "Registration failed due to server error".to_string(),
        })),
    }
}

/// User logout endpoint
#[post("/logout")]
pub async fn logout(session: Session) -> Result<HttpResponse> {
    session.purge();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

/// Get current user endpoint (requires authentication)
#[get("/me")]
pub async fn get_current_user(
    session: Session,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    match session.get::<String>("user_id") {
        Ok(Some(user_id_str)) => {
            let user_id = match uuid::Uuid::parse_str(&user_id_str) {
                Ok(id) => id,
                Err(_) => {
                    return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                        "success": false,
                        "message": "Invalid session"
                    })));
                }
            };

            match state.user_service.get_user_by_id(user_id).await {
                Ok(Some(user)) => {
                    let user_response = UserResponse {
                        id: user.id.to_string(),
                        username: user.username,
                        email: user.email,
                        full_name: user.full_name,
                        is_active: user.is_active,
                        is_admin: user.is_admin,
                        created_at: user.created_at.to_string(),
                    };

                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "success": true,
                        "user": user_response
                    })))
                }
                Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "success": false,
                    "message": "User not found"
                }))),
                Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "message": "Database error"
                }))),
            }
        }
        Ok(None) => Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "success": false,
            "message": "Not authenticated"
        }))),
        Err(_) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "message": "Session error"
        }))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git_storage::{init_db, run_migrations, RepositoryService};
    use actix_web::{test, web, App, middleware};
    use std::sync::Arc;

    async fn create_test_app() -> Arc<git_storage::UserService> {
        // Create in-memory database for testing
        let db = init_db("sqlite::memory:").await.unwrap();
        run_migrations(&db).await.unwrap();
        
        Arc::new(git_storage::UserService::new(db))
    }

    #[tokio::test]
    async fn test_user_service_authentication() {
        let user_service = create_test_app().await;

        // Test user creation
        let user = user_service
            .create_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                user_service.hash_password("password123").unwrap(),
                Some("Test User".to_string()),
                false,
            )
            .await
            .unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
        assert!(!user.is_admin);

        // Test authentication with username
        let auth_result = user_service
            .authenticate("testuser", "password123")
            .await
            .unwrap();
        assert!(auth_result.is_some());
        assert_eq!(auth_result.unwrap().id, user.id);

        // Test authentication with email
        let auth_result = user_service
            .authenticate("test@example.com", "password123")
            .await
            .unwrap();
        assert!(auth_result.is_some());

        // Test authentication with wrong password
        let auth_result = user_service
            .authenticate("testuser", "wrongpassword")
            .await
            .unwrap();
        assert!(auth_result.is_none());

        // Test authentication with non-existent user
        let auth_result = user_service
            .authenticate("nonexistent", "password123")
            .await
            .unwrap();
        assert!(auth_result.is_none());
    }

    #[tokio::test]
    async fn test_password_hashing() {
        let user_service = create_test_app().await;

        let password = "testpassword123";
        let hashed = user_service.hash_password(password).unwrap();
        
        // Verify the password hashing is working
        assert!(user_service.verify_password(password, &hashed).unwrap());
        assert!(!user_service.verify_password("wrongpassword", &hashed).unwrap());
    }

    #[tokio::test]
    async fn test_user_exists_checks() {
        let user_service = create_test_app().await;

        // Initially no users exist
        assert!(!user_service.username_exists("testuser").await.unwrap());
        assert!(!user_service.email_exists("test@example.com").await.unwrap());

        // Create a user
        user_service
            .create_user(
                "testuser".to_string(),
                "test@example.com".to_string(),
                user_service.hash_password("password123").unwrap(),
                None,
                false,
            )
            .await
            .unwrap();

        // Now they should exist
        assert!(user_service.username_exists("testuser").await.unwrap());
        assert!(user_service.email_exists("test@example.com").await.unwrap());
    }
}