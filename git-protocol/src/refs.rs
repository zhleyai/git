use crate::GitRef;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Git reference handler
pub struct RefHandler {
    refs: HashMap<String, GitRef>,
}

impl RefHandler {
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
        }
    }

    /// Add a reference
    pub fn add_ref(&mut self, name: String, target: String, is_symbolic: bool) {
        let git_ref = GitRef {
            name: name.clone(),
            target,
            is_symbolic,
        };
        self.refs.insert(name, git_ref);
    }

    /// Get a reference by name
    pub fn get_ref(&self, name: &str) -> Option<&GitRef> {
        self.refs.get(name)
    }

    /// Get all references
    pub fn get_all_refs(&self) -> Vec<&GitRef> {
        self.refs.values().collect()
    }

    /// Get references matching a pattern
    pub fn get_refs_matching(&self, pattern: &str) -> Vec<&GitRef> {
        self.refs
            .values()
            .filter(|r| r.name.contains(pattern))
            .collect()
    }

    /// Update a reference
    pub fn update_ref(&mut self, name: &str, new_target: String) -> Result<()> {
        if let Some(git_ref) = self.refs.get_mut(name) {
            git_ref.target = new_target;
            Ok(())
        } else {
            Err(anyhow!("Reference {} not found", name))
        }
    }

    /// Delete a reference
    pub fn delete_ref(&mut self, name: &str) -> Result<()> {
        if self.refs.remove(name).is_some() {
            Ok(())
        } else {
            Err(anyhow!("Reference {} not found", name))
        }
    }

    /// Resolve a reference to its final target
    pub fn resolve_ref(&self, name: &str) -> Result<String> {
        let mut current = name;
        let mut visited = std::collections::HashSet::new();

        loop {
            if visited.contains(current) {
                return Err(anyhow!("Circular reference detected: {}", current));
            }
            visited.insert(current);

            if let Some(git_ref) = self.refs.get(current) {
                if git_ref.is_symbolic {
                    current = &git_ref.target;
                } else {
                    return Ok(git_ref.target.clone());
                }
            } else {
                return Err(anyhow!("Reference {} not found", current));
            }
        }
    }

    /// Get HEAD reference
    pub fn get_head(&self) -> Option<&GitRef> {
        self.get_ref("HEAD")
    }

    /// Set HEAD reference
    pub fn set_head(&mut self, target: String, is_symbolic: bool) {
        self.add_ref("HEAD".to_string(), target, is_symbolic);
    }

    /// List branches (refs/heads/*)
    pub fn list_branches(&self) -> Vec<&GitRef> {
        self.refs
            .values()
            .filter(|r| r.name.starts_with("refs/heads/"))
            .collect()
    }

    /// List tags (refs/tags/*)
    pub fn list_tags(&self) -> Vec<&GitRef> {
        self.refs
            .values()
            .filter(|r| r.name.starts_with("refs/tags/"))
            .collect()
    }

    /// Create a new branch
    pub fn create_branch(&mut self, name: &str, target: String) -> Result<()> {
        let full_name = format!("refs/heads/{}", name);
        if self.refs.contains_key(&full_name) {
            return Err(anyhow!("Branch {} already exists", name));
        }
        self.add_ref(full_name, target, false);
        Ok(())
    }

    /// Create a new tag
    pub fn create_tag(&mut self, name: &str, target: String) -> Result<()> {
        let full_name = format!("refs/tags/{}", name);
        if self.refs.contains_key(&full_name) {
            return Err(anyhow!("Tag {} already exists", name));
        }
        self.add_ref(full_name, target, false);
        Ok(())
    }

    /// Delete a branch
    pub fn delete_branch(&mut self, name: &str) -> Result<()> {
        let full_name = format!("refs/heads/{}", name);
        self.delete_ref(&full_name)
    }

    /// Delete a tag
    pub fn delete_tag(&mut self, name: &str) -> Result<()> {
        let full_name = format!("refs/tags/{}", name);
        self.delete_ref(&full_name)
    }

    /// Get the default branch (usually main or master)
    pub fn get_default_branch(&self) -> Option<String> {
        // Try to resolve HEAD
        if let Ok(_target) = self.resolve_ref("HEAD") {
            if let Some(git_ref) = self.get_ref("HEAD") {
                if git_ref.is_symbolic && git_ref.target.starts_with("refs/heads/") {
                    return Some(git_ref.target[11..].to_string()); // Remove "refs/heads/"
                }
            }
        }

        // Fallback: look for common default branches
        for branch in ["main", "master", "develop"] {
            let full_name = format!("refs/heads/{}", branch);
            if self.refs.contains_key(&full_name) {
                return Some(branch.to_string());
            }
        }

        None
    }

    /// Import refs from a list of (name, target) tuples
    pub fn import_refs(&mut self, refs: Vec<(String, String)>) {
        for (name, target) in refs {
            let is_symbolic = name == "HEAD" && target.starts_with("refs/");
            self.add_ref(name, target, is_symbolic);
        }
    }

    /// Export refs as a list of (name, target) tuples
    pub fn export_refs(&self) -> Vec<(String, String)> {
        self.refs
            .values()
            .map(|r| (r.name.clone(), r.target.clone()))
            .collect()
    }
}

impl Default for RefHandler {
    fn default() -> Self {
        Self::new()
    }
}