use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub http_bind_address: String,
    pub ssh_bind_address: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "sqlite:./git_server.db".to_string(),
            http_bind_address: "127.0.0.1:8080".to_string(),
            ssh_bind_address: "127.0.0.1:2222".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./git_server.db".to_string()),
            http_bind_address: std::env::var("HTTP_BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            ssh_bind_address: std::env::var("SSH_BIND_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:2222".to_string()),
        }
    }
}