pub mod pack;
pub mod refs;
pub mod objects;
pub mod protocol;
#[cfg(test)]
mod tests;

pub use protocol::ProtocolHandler;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Git object types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}

/// Git object representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitObject {
    pub id: String,        // SHA-1 hash
    pub obj_type: ObjectType,
    pub size: usize,
    pub content: Vec<u8>,
}

/// Git reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRef {
    pub name: String,      // e.g., "refs/heads/main"
    pub target: String,    // SHA-1 hash or another ref
    pub is_symbolic: bool, // true if it points to another ref
}

/// Repository metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub refs: Vec<GitRef>,
}

/// Git pack entry
#[derive(Debug, Clone)]
pub struct PackEntry {
    pub object_type: ObjectType,
    pub size: usize,
    pub data: Vec<u8>,
}

pub trait GitProtocol {
    /// Parse a Git pack file
    fn parse_pack(&self, data: &[u8]) -> Result<Vec<PackEntry>>;
    
    /// Create a pack file from objects
    fn create_pack(&self, objects: &[GitObject]) -> Result<Vec<u8>>;
    
    /// Parse Git protocol pkt-line format
    fn parse_pkt_line(&self, data: &[u8]) -> Result<Vec<String>>;
    
    /// Create Git protocol pkt-line format
    fn create_pkt_line(&self, lines: &[&str]) -> Vec<u8>;
}
