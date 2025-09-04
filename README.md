# Git Server in Rust

A complete Git server implementation written in Rust that provides both HTTP and SSH access to Git repositories, with a modern React frontend for repository management.

## Features

### Core Functionality
- **Native Git Protocol Parsing**: Direct parsing of Git pack files, objects, and refs without using git CLI
- **HTTP Git Protocol**: Full support for git clone, fetch, and push over HTTP
- **SSH Git Protocol**: SSH server support for Git operations (simplified implementation)
- **Database Storage**: SeaORM with SQLite for persistent storage of repositories, objects, and references
- **Web Interface**: React-based frontend for repository management

### Technical Stack
- **Backend**: Rust with Actix-web, SeaORM, russh
- **Frontend**: React with TypeScript, Vite
- **Database**: SQLite (configurable)
- **Protocols**: HTTP/HTTPS and SSH

## Quick Start

### Prerequisites
- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Node.js 18+ (for frontend development)

### Running the Server

1. **Clone and build the backend**:
   ```bash
   cargo build --release
   ```

2. **Run database migrations**:
   ```bash
   # The server will automatically run migrations on startup
   # Default database: ./git_server.db
   ```

3. **Start the server**:
   ```bash
   cargo run --bin git-server
   ```
   The server will start:
   - HTTP server on `http://localhost:8080`
   - SSH server on `localhost:2222` (simplified implementation)

4. **Build and serve the frontend** (optional for development):
   ```bash
   cd frontend
   npm install
   npm run build
   # The built files are served by the Rust server at http://localhost:8080
   ```

### Using the Git Server

#### Web Interface
Visit `http://localhost:8080` to:
- View all repositories
- Create new repositories
- Get clone URLs

#### Git Operations

**Clone a repository**:
```bash
# HTTP
git clone http://localhost:8080/git/my-repo

# SSH (simplified - may not work with all Git clients)
git clone ssh://git@localhost:2222/my-repo
```

**Create and push to a new repository**:
```bash
# First create the repository via web interface or API
curl -X POST http://localhost:8080/api/repositories \
  -H "Content-Type: application/json" \
  -d '{"name": "my-repo", "description": "My new repository"}'

# Then clone and push
git clone http://localhost:8080/git/my-repo
cd my-repo
echo "# My Repo" > README.md
git add README.md
git commit -m "Initial commit"
git push origin main
```

## API Endpoints

### Repository Management
- `GET /api/repositories` - List all repositories
- `POST /api/repositories` - Create new repository
- `GET /api/repositories/{name}` - Get repository details

### Git Protocol Endpoints
- `GET /git/{repo}/info/refs?service=git-upload-pack` - Reference advertisement for fetch
- `POST /git/{repo}/git-upload-pack` - Upload pack for fetch/pull
- `POST /git/{repo}/git-receive-pack` - Receive pack for push

## Configuration

Set environment variables:

```bash
# Database URL (default: sqlite:./git_server.db)
export DATABASE_URL="sqlite:./git_server.db"

# HTTP server bind address (default: 127.0.0.1:8080)
export BIND_ADDRESS="0.0.0.0:8080"

# SSH server bind address (default: 127.0.0.1:2222)
export SSH_BIND_ADDRESS="0.0.0.0:2222"
```

## Development

### Architecture

The project is organized as a Cargo workspace with four crates:

- **`git-server`**: Main binary with HTTP and SSH servers
- **`git-protocol`**: Git protocol parsing and handling
- **`git-storage`**: Database models and repository operations
- **`frontend`**: React web application (placeholder Rust crate)

### Frontend Development

```bash
cd frontend
npm install
npm run dev  # Development server with hot reload
```

The development server will proxy API calls to the Rust backend.

### Running Tests

```bash
cargo test
```

## Project Structure

```
├── git-server/          # Main server application
├── git-protocol/        # Git protocol implementation  
├── git-storage/         # Database and storage layer
├── frontend/            # React web interface
└── README.md
```

## Implementation Status

✅ **Completed**:
- Cargo workspace setup
- Git protocol parser (pack files, objects, refs)
- Database schema and operations with SeaORM
- HTTP server with Actix-web
- REST API for repository management
- React frontend with TypeScript
- Basic Git HTTP protocol support

🚧 **In Progress**:
- Complete Git pack file parsing
- Full SSH server implementation
- Authentication and authorization
- Advanced Git operations (branching, tagging)

## Contributing

This is a demonstration implementation. For production use, consider:

1. **Security**: Add authentication, authorization, and input validation
2. **Performance**: Optimize pack file handling and database queries  
3. **Compliance**: Ensure full Git protocol compatibility
4. **Features**: Add advanced Git features like hooks, submodules, etc.

## License

This project is created for demonstration purposes.