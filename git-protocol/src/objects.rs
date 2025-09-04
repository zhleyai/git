use crate::{GitObject, ObjectType};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sha1::{Digest, Sha1};

/// Git commit object
#[derive(Debug, Clone)]
pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub committer: String,
    pub message: String,
    pub author_date: DateTime<Utc>,
    pub commit_date: DateTime<Utc>,
}

/// Git tree entry
#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: String,    // e.g., "100644", "040000"
    pub name: String,
    pub hash: String,
}

/// Git tree object
#[derive(Debug, Clone)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

/// Git blob object
#[derive(Debug, Clone)]
pub struct Blob {
    pub content: Vec<u8>,
}

/// Git tag object
#[derive(Debug, Clone)]
pub struct Tag {
    pub object: String,
    pub obj_type: String,
    pub tag_name: String,
    pub tagger: String,
    pub message: String,
    pub tagger_date: DateTime<Utc>,
}

/// Object parser and serializer
pub struct ObjectHandler;

impl ObjectHandler {
    pub fn new() -> Self {
        Self
    }

    /// Parse a Git object from its raw content
    pub fn parse_object(&self, obj_type: ObjectType, content: &[u8]) -> Result<GitObject> {
        let _content_str = String::from_utf8_lossy(content);
        let id = self.calculate_hash(obj_type.clone(), content)?;

        Ok(GitObject {
            id,
            obj_type,
            size: content.len(),
            content: content.to_vec(),
        })
    }

    /// Parse a commit object
    pub fn parse_commit(&self, content: &[u8]) -> Result<Commit> {
        let content_str = String::from_utf8_lossy(content);
        let lines: Vec<&str> = content_str.lines().collect();
        
        let mut tree = String::new();
        let mut parents = Vec::new();
        let mut author = String::new();
        let mut committer = String::new();
        let mut author_date = Utc::now();
        let mut commit_date = Utc::now();
        let mut message_start = 0;

        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("tree ") {
                tree = line[5..].to_string();
            } else if line.starts_with("parent ") {
                parents.push(line[7..].to_string());
            } else if line.starts_with("author ") {
                author = line[7..].to_string();
                // Parse date from author line (simplified)
                author_date = Utc::now(); // Should parse actual timestamp
            } else if line.starts_with("committer ") {
                committer = line[10..].to_string();
                // Parse date from committer line (simplified)
                commit_date = Utc::now(); // Should parse actual timestamp
            } else if line.is_empty() {
                message_start = i + 1;
                break;
            }
        }

        let message = lines[message_start..].join("\n");

        Ok(Commit {
            tree,
            parents,
            author,
            committer,
            message,
            author_date,
            commit_date,
        })
    }

    /// Parse a tree object
    pub fn parse_tree(&self, content: &[u8]) -> Result<Tree> {
        let mut entries = Vec::new();
        let mut pos = 0;

        while pos < content.len() {
            // Find the space after mode
            let space_pos = content[pos..]
                .iter()
                .position(|&b| b == b' ')
                .ok_or_else(|| anyhow!("Invalid tree format: no space after mode"))?;
            
            let mode = String::from_utf8_lossy(&content[pos..pos + space_pos]).to_string();
            pos += space_pos + 1;

            // Find the null byte after filename
            let null_pos = content[pos..]
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| anyhow!("Invalid tree format: no null after filename"))?;
            
            let name = String::from_utf8_lossy(&content[pos..pos + null_pos]).to_string();
            pos += null_pos + 1;

            // Read 20-byte SHA-1 hash
            if pos + 20 > content.len() {
                return Err(anyhow!("Invalid tree format: incomplete hash"));
            }
            
            let hash = hex::encode(&content[pos..pos + 20]);
            pos += 20;

            entries.push(TreeEntry { mode, name, hash });
        }

        Ok(Tree { entries })
    }

    /// Parse a blob object
    pub fn parse_blob(&self, content: &[u8]) -> Result<Blob> {
        Ok(Blob {
            content: content.to_vec(),
        })
    }

    /// Serialize a commit object
    pub fn serialize_commit(&self, commit: &Commit) -> Vec<u8> {
        let mut content = Vec::new();
        
        content.extend_from_slice(format!("tree {}\n", commit.tree).as_bytes());
        
        for parent in &commit.parents {
            content.extend_from_slice(format!("parent {}\n", parent).as_bytes());
        }
        
        content.extend_from_slice(format!("author {}\n", commit.author).as_bytes());
        content.extend_from_slice(format!("committer {}\n", commit.committer).as_bytes());
        content.extend_from_slice(b"\n");
        content.extend_from_slice(commit.message.as_bytes());
        
        content
    }

    /// Serialize a tree object
    pub fn serialize_tree(&self, tree: &Tree) -> Vec<u8> {
        let mut content = Vec::new();
        
        for entry in &tree.entries {
            content.extend_from_slice(entry.mode.as_bytes());
            content.push(b' ');
            content.extend_from_slice(entry.name.as_bytes());
            content.push(0);
            content.extend_from_slice(&hex::decode(&entry.hash).unwrap_or_default());
        }
        
        content
    }

    /// Calculate SHA-1 hash for an object
    pub fn calculate_hash(&self, obj_type: ObjectType, content: &[u8]) -> Result<String> {
        let type_str = match obj_type {
            ObjectType::Commit => "commit",
            ObjectType::Tree => "tree",
            ObjectType::Blob => "blob",
            ObjectType::Tag => "tag",
        };

        let header = format!("{} {}\0", type_str, content.len());
        let mut hasher = Sha1::new();
        hasher.update(header.as_bytes());
        hasher.update(content);
        
        Ok(hex::encode(hasher.finalize()))
    }

    /// Create a new blob object
    pub fn create_blob(&self, content: &[u8]) -> Result<GitObject> {
        let id = self.calculate_hash(ObjectType::Blob, content)?;
        Ok(GitObject {
            id,
            obj_type: ObjectType::Blob,
            size: content.len(),
            content: content.to_vec(),
        })
    }

    /// Create a new commit object
    pub fn create_commit(&self, commit: &Commit) -> Result<GitObject> {
        let content = self.serialize_commit(commit);
        let id = self.calculate_hash(ObjectType::Commit, &content)?;
        Ok(GitObject {
            id,
            obj_type: ObjectType::Commit,
            size: content.len(),
            content,
        })
    }

    /// Create a new tree object
    pub fn create_tree(&self, tree: &Tree) -> Result<GitObject> {
        let content = self.serialize_tree(tree);
        let id = self.calculate_hash(ObjectType::Tree, &content)?;
        Ok(GitObject {
            id,
            obj_type: ObjectType::Tree,
            size: content.len(),
            content,
        })
    }
}

impl Default for ObjectHandler {
    fn default() -> Self {
        Self::new()
    }
}