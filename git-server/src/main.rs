mod config;
mod http;
mod ssh;

use actix_files::Files;
use actix_web::{web, App, HttpServer};
use anyhow::Context;
use git_storage::{init_db, run_migrations, RepositoryService};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Clone)]
pub struct AppState {
    pub repository_service: Arc<RepositoryService>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Git Server...");

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./git_server.db".to_string());
    
    let db = init_db(&database_url)
        .await
        .context("Failed to initialize database")?;

    // Run migrations
    run_migrations(&db)
        .await
        .context("Failed to run migrations")?;

    // Create repository service
    let repository_service = Arc::new(RepositoryService::new(db));

    let app_state = AppState {
        repository_service: repository_service.clone(),
    };

    // Start SSH server in background
    let ssh_service = repository_service.clone();
    tokio::spawn(async move {
        if let Err(e) = ssh::start_ssh_server(ssh_service).await {
            eprintln!("SSH server error: {}", e);
        }
    });

    // Start HTTP server
    let bind_address = std::env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    info!("Starting HTTP server on {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            // Git HTTP protocol routes
            .service(
                web::scope("/git")
                    .service(http::info_refs)
                    .service(http::upload_pack)
                    .service(http::receive_pack)
            )
            // API routes
            .service(
                web::scope("/api")
                    .service(http::list_repositories)
                    .service(http::get_repository)
                    .service(http::create_repository)
            )
            // Static files for frontend
            .service(Files::new("/", "./frontend/dist").index_file("index.html"))
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}
