use git_storage::{RepositoryService, UserService};
use git_protocol::{GitProtocol, ProtocolHandler};
use git_protocol::pack::PackParser;
use russh::server::{Auth, Msg, Session, Handle, Server};
use russh::{Channel, ChannelId, CryptoVec};
use russh_keys::key;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, debug, error, warn};
use tokio::sync::Mutex;

/// SSH Git server implementation
pub struct GitSshServer {
    repository_service: Arc<RepositoryService>,
    user_service: Arc<UserService>,
    protocol_handler: ProtocolHandler,
    sessions: Arc<Mutex<HashMap<usize, GitSshSession>>>,
}

/// Individual SSH session for Git operations
pub struct GitSshSession {
    session_id: usize,
    authenticated_user: Option<String>,
    current_command: Option<String>,
    repository_service: Arc<RepositoryService>,
    protocol_handler: ProtocolHandler,
}

impl GitSshServer {
    pub fn new(repository_service: Arc<RepositoryService>, user_service: Arc<UserService>) -> Self {
        Self {
            repository_service,
            user_service,
            protocol_handler: ProtocolHandler::new(),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl russh::server::Server for GitSshServer {
    type Handler = GitSshSession;

    async fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        let session_id = rand::random::<usize>();
        info!("New SSH client connected with session ID: {}", session_id);

        GitSshSession {
            session_id,
            authenticated_user: None,
            current_command: None,
            repository_service: Arc::clone(&self.repository_service),
            protocol_handler: ProtocolHandler::new(),
        }
    }
}

#[async_trait]
impl russh::server::Handler for GitSshSession {
    type Error = anyhow::Error;

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        debug!("SSH channel opened for session {}", self.session_id);
        Ok(true)
    }

    async fn auth_publickey(
        &mut self,
        user: &str,
        _public_key: &key::PublicKey,
    ) -> Result<Auth, Self::Error> {
        info!("SSH public key authentication attempt for user: {}", user);
        
        // For now, accept any public key - in production you'd verify against stored keys
        self.authenticated_user = Some(user.to_string());
        Ok(Auth::Accept)
    }

    async fn auth_password(
        &mut self,
        user: &str,
        _password: &str,
    ) -> Result<Auth, Self::Error> {
        info!("SSH password authentication attempt for user: {}", user);
        
        // Note: In production, you would not typically allow password auth for Git
        // but we'll support it for development purposes
        warn!("Password authentication is not recommended for Git SSH access");
        
        self.authenticated_user = Some(user.to_string());
        Ok(Auth::Accept)
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let command = String::from_utf8_lossy(data);
        info!("SSH exec request: {}", command);
        
        self.current_command = Some(command.to_string());

        // Parse Git commands
        if command.starts_with("git-receive-pack") {
            self.handle_receive_pack(channel, &command, session).await?;
        } else if command.starts_with("git-upload-pack") {
            self.handle_upload_pack(channel, &command, session).await?;
        } else {
            error!("Unsupported command: {}", command);
            session.data(channel, CryptoVec::from_slice(b"Unsupported command\n"));
            session.eof(channel);
            session.close(channel);
        }

        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        debug!("SSH data received: {} bytes", data.len());
        
        // Handle incoming pack data for git-receive-pack
        if let Some(ref command) = self.current_command {
            if command.starts_with("git-receive-pack") {
                self.handle_pack_data(channel, data, session).await?;
            }
        }

        Ok(())
    }
}

impl GitSshSession {
    /// Handle git-receive-pack (push) operations
    async fn handle_receive_pack(
        &mut self,
        channel: ChannelId,
        command: &str,
        session: &mut Session,
    ) -> Result<(), anyhow::Error> {
        info!("Handling git-receive-pack: {}", command);
        
        // Extract repository path from command
        let repo_path = self.extract_repo_path(command)?;
        info!("Repository path: {}", repo_path);

        // Send initial reference advertisement
        let refs = vec![
            ("refs/heads/main".to_string(), "0000000000000000000000000000000000000000".to_string()),
        ];
        
        let capabilities = ["report-status", "delete-refs", "ofs-delta", "side-band-64k"];
        let advertisement = self.protocol_handler.create_ref_advertisement(&refs, &capabilities);
        
        session.data(channel, CryptoVec::from_slice(&advertisement));

        Ok(())
    }

    /// Handle git-upload-pack (fetch/pull) operations
    async fn handle_upload_pack(
        &mut self,
        channel: ChannelId,
        command: &str,
        session: &mut Session,
    ) -> Result<(), anyhow::Error> {
        info!("Handling git-upload-pack: {}", command);
        
        // Extract repository path from command
        let repo_path = self.extract_repo_path(command)?;
        info!("Repository path: {}", repo_path);

        // Send reference advertisement
        let refs = vec![
            ("refs/heads/main".to_string(), "1234567890abcdef1234567890abcdef12345678".to_string()),
        ];
        
        let capabilities = ["multi_ack", "ofs-delta", "side-band-64k", "thin-pack"];
        let advertisement = self.protocol_handler.create_ref_advertisement(&refs, &capabilities);
        
        session.data(channel, CryptoVec::from_slice(&advertisement));

        Ok(())
    }

    /// Handle incoming pack data
    async fn handle_pack_data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), anyhow::Error> {
        debug!("Processing pack data: {} bytes", data.len());
        
        // Parse pkt-line format
        match self.protocol_handler.parse_pkt_line(data) {
            Ok(lines) => {
                for line in lines {
                    debug!("Pack line: {}", line);
                    // Process Git protocol messages
                    if line.starts_with("want") || line.starts_with("have") {
                        // Handle want/have negotiation
                        debug!("Negotiation: {}", line);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to parse pkt-line data: {}", e);
            }
        }

        Ok(())
    }

    /// Extract repository path from Git command
    fn extract_repo_path(&self, command: &str) -> Result<String, anyhow::Error> {
        // Commands are like: "git-upload-pack '/path/to/repo.git'"
        if let Some(start) = command.find('\'') {
            if let Some(end) = command.rfind('\'') {
                if end > start {
                    return Ok(command[start + 1..end].to_string());
                }
            }
        }
        
        // Fallback: split on whitespace and take last part
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.len() >= 2 {
            Ok(parts[1].trim_matches('\'').to_string())
        } else {
            Err(anyhow::anyhow!("Could not extract repository path from command: {}", command))
        }
    }
}

/// Start the SSH server for Git operations
pub async fn start_ssh_server(
    repository_service: Arc<RepositoryService>,
    user_service: Arc<UserService>,
) -> anyhow::Result<()> {
    let bind_address = std::env::var("SSH_BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:2222".to_string());

    info!("Starting SSH Git server on {}", bind_address);

    // Generate or load server keys
    let server_key = russh_keys::key::KeyPair::generate_ed25519()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate server key"))?;

    // Create SSH server configuration
    let config = russh::server::Config {
        keys: vec![server_key],
        ..Default::default()
    };

    // Create the SSH server
    let server = GitSshServer::new(repository_service, user_service);

    // Start listening
    info!("SSH server listening on {}", bind_address);
    
    let mut handle = russh::server::run(config, &bind_address, server);
    handle.await?;

    Ok(())
}