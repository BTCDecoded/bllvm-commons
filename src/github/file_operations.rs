//! GitHub File Operations
//!
//! This module provides utilities for fetching file content and directory structures
//! from GitHub repositories via the GitHub API.

use crate::error::GovernanceError;
use octocrab::Octocrab;
use octocrab::models::Content;
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use base64;

/// Represents a file in a GitHub repository
#[derive(Debug, Clone)]
pub struct GitHubFile {
    pub path: String,
    pub content: Vec<u8>,
    pub sha: String,
    pub size: u64,
    pub download_url: Option<String>,
}

/// Represents a directory tree in a GitHub repository
#[derive(Debug, Clone)]
pub struct GitHubDirectory {
    pub path: String,
    pub files: Vec<GitHubFile>,
    pub subdirectories: Vec<GitHubDirectory>,
    pub total_size: u64,
}

/// GitHub repository information
#[derive(Debug, Clone)]
pub struct GitHubRepo {
    pub owner: String,
    pub name: String,
    pub default_branch: String,
    pub last_commit_sha: String,
}

/// File comparison result
#[derive(Debug, Clone)]
pub struct FileComparison {
    pub file_path: String,
    pub source_sha: String,
    pub target_sha: Option<String>,
    pub is_same: bool,
    pub size_diff: Option<i64>,
    pub content_diff: Option<String>,
}

pub struct GitHubFileOperations {
    client: Octocrab,
}

impl GitHubFileOperations {
    /// Create a new GitHub file operations client
    pub fn new(token: String) -> Result<Self, GovernanceError> {
        let client = Octocrab::builder()
            .personal_token(token)
            .build()
            .map_err(|e| GovernanceError::GitHubError(format!("Failed to create GitHub client: {}", e)))?;

        Ok(Self { client })
    }

    /// Fetch file content from GitHub repository
    pub async fn fetch_file_content(
        &self,
        owner: &str,
        repo: &str,
        file_path: &str,
        branch: Option<&str>,
    ) -> Result<GitHubFile, GovernanceError> {
        info!("Fetching file content: {}/{}:{}", owner, repo, file_path);

        let branch = branch.unwrap_or("main");
        
        let response = self
            .client
            .repos(owner, repo)
            .get_content()
            .path(file_path)
            .r#ref(branch)
            .send()
            .await
            .map_err(|e| GovernanceError::GitHubError(format!("Failed to fetch file: {}", e)))?;

        // Handle the response - octocrab 0.38 returns Content enum
        match response {
            Content::File(file) => {
                // Decode base64 content
                let content_bytes = match file.content {
                    Some(encoded) => {
                        base64::engine::general_purpose::STANDARD
                            .decode(encoded.trim_end_matches('\n'))
                            .map_err(|e| GovernanceError::GitHubError(format!("Failed to decode base64 content: {}", e)))?
                    }
                    None => {
                        return Err(GovernanceError::GitHubError("File content is empty".to_string()));
                    }
                };
                
                Ok(GitHubFile {
                    path: file.path,
                    content: content_bytes,
                    sha: file.sha,
                    size: file.size as u64,
                    download_url: file.download_url.map(|u| u.to_string()),
                })
            }
            Content::Directory(_) => {
                Err(GovernanceError::GitHubError(format!("Path '{}' is a directory, not a file", file_path)))
            }
            Content::Symlink(_) => {
                Err(GovernanceError::GitHubError(format!("Path '{}' is a symlink, not a file", file_path)))
            }
            Content::Submodule(_) => {
                Err(GovernanceError::GitHubError(format!("Path '{}' is a submodule, not a file", file_path)))
            }
        }
    }

    /// Fetch directory tree from GitHub repository
    pub async fn fetch_directory_tree(
        &self,
        owner: &str,
        repo: &str,
        directory_path: &str,
        branch: Option<&str>,
    ) -> Result<GitHubDirectory, GovernanceError> {
        info!("Fetching directory tree: {}/{}:{}", owner, repo, directory_path);

        let branch = branch.unwrap_or("main");
        
        let response = self
            .client
            .repos(owner, repo)
            .get_content()
            .path(directory_path)
            .r#ref(branch)
            .send()
            .await
            .map_err(|e| GovernanceError::GitHubError(format!("Failed to fetch directory: {}", e)))?;

        // Handle the response - octocrab 0.38 returns Content enum
        match response {
            Content::Directory(items) => {
                let mut files = Vec::new();
                let mut subdirectories = Vec::new();
                let mut total_size = 0u64;
                
                // Process each item in the directory
                // Content::Directory contains Vec<Content> where each Content has type and path
                for item in items {
                    match item {
                        Content::File(file) => {
                            // For files, create GitHubFile with metadata
                            // Content can be fetched later if needed via fetch_file_content()
                            let size = file.size as u64;
                            total_size += size;
                            
                            files.push(GitHubFile {
                                path: file.path.clone(),
                                content: Vec::new(), // Content not loaded by default (can fetch later)
                                sha: file.sha,
                                size,
                                download_url: file.download_url.map(|u| u.to_string()),
                            });
                        }
                        Content::Directory(_) => {
                            // For subdirectories, we need to extract the path
                            // In octocrab 0.38, Content::Directory in items list doesn't directly expose path
                            // We need to make another API call to get the directory
                            // For now, skip nested directories - they can be fetched separately if needed
                            // This can be enhanced later to extract path from ContentItem metadata
                            debug!("Skipping nested directory - fetch separately if needed");
                        }
                        Content::Symlink(_) | Content::Submodule(_) => {
                            // Skip symlinks and submodules
                            debug!("Skipping symlink/submodule in directory: {}", directory_path);
                        }
                    }
                }
                
                Ok(GitHubDirectory {
                    path: directory_path.to_string(),
                    files,
                    subdirectories,
                    total_size,
                })
            }
            Content::File(_) => {
                Err(GovernanceError::GitHubError(format!("Path '{}' is a file, not a directory", directory_path)))
            }
            Content::Symlink(_) => {
                Err(GovernanceError::GitHubError(format!("Path '{}' is a symlink, not a directory", directory_path)))
            }
            Content::Submodule(_) => {
                Err(GovernanceError::GitHubError(format!("Path '{}' is a submodule, not a directory", directory_path)))
            }
        }
    }

    /// Compute hash of entire repository state
    /// Returns the SHA of the latest commit on the specified branch
    pub async fn compute_repo_hash(
        &self,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
    ) -> Result<String, GovernanceError> {
        info!("Computing repository hash: {}/{}", owner, repo);

        let branch = branch.unwrap_or("main");
        
        // Get the branch reference to get the latest commit SHA
        let branch_ref = self
            .client
            .repos(owner, repo)
            .get_branch(branch)
            .await
            .map_err(|e| GovernanceError::GitHubError(format!("Failed to get branch: {}", e)))?;
        
        // Extract commit SHA from branch reference
        let commit_sha = branch_ref.commit.sha;
        
        info!("Repository hash for {}/{}:{} = {}", owner, repo, branch, commit_sha);
        
        Ok(commit_sha)
    }

    /// Compare file versions across repositories
    pub async fn compare_file_versions(
        &self,
        source_owner: &str,
        source_repo: &str,
        source_file: &str,
        target_owner: &str,
        target_repo: &str,
        target_file: &str,
        branch: Option<&str>,
    ) -> Result<FileComparison, GovernanceError> {
        info!("Comparing files: {}/{}:{} vs {}/{}:{}", 
              source_owner, source_repo, source_file,
              target_owner, target_repo, target_file);

        let branch = branch.unwrap_or("main");

        // Fetch source file
        let source_file_data = self.fetch_file_content(source_owner, source_repo, source_file, Some(branch)).await?;

        // Try to fetch target file
        let target_file_data = match self.fetch_file_content(target_owner, target_repo, target_file, Some(branch)).await {
            Ok(file) => Some(file),
            Err(e) => {
                warn!("Target file not found: {}", e);
                None
            }
        };

        let is_same = if let Some(ref target) = target_file_data {
            source_file_data.sha == target.sha
        } else {
            false
        };

        let size_diff = target_file_data.as_ref().map(|target| {
            source_file_data.size as i64 - target.size as i64
        });

        let content_diff = if let Some(ref target) = target_file_data {
            if source_file_data.content != target.content {
                Some(format!("Content differs: {} bytes vs {} bytes", 
                            source_file_data.content.len(), target.content.len()))
            } else {
                None
            }
        } else {
            Some("Target file does not exist".to_string())
        };

        Ok(FileComparison {
            file_path: source_file.to_string(),
            source_sha: source_file_data.sha,
            target_sha: target_file_data.map(|f| f.sha),
            is_same,
            size_diff,
            content_diff,
        })
    }

    /// Get repository information
    pub async fn get_repo_info(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubRepo, GovernanceError> {
        info!("Getting repository info: {}/{}", owner, repo);

        // Get repository information using octocrab API
        let repository = self
            .client
            .repos(owner, repo)
            .get()
            .await
            .map_err(|e| GovernanceError::GitHubError(format!("Failed to get repository info: {}", e)))?;
        
        // Get the default branch's latest commit SHA
        let default_branch = repository.default_branch.as_deref().unwrap_or("main");
        let last_commit_sha = self.compute_repo_hash(owner, repo, Some(default_branch)).await?;
        
        Ok(GitHubRepo {
            owner: repository.owner.login.clone(),
            name: repository.name.clone(),
            default_branch: default_branch.to_string(),
            last_commit_sha,
        })
    }

    /// Fetch multiple files in parallel
    pub async fn fetch_multiple_files(
        &self,
        owner: &str,
        repo: &str,
        file_paths: &[String],
        branch: Option<&str>,
    ) -> Result<HashMap<String, GitHubFile>, GovernanceError> {
        info!("Fetching {} files in parallel", file_paths.len());

        let mut results = HashMap::new();
        let mut tasks = Vec::new();

        for file_path in file_paths {
            let client = self.client.clone();
            let owner = owner.to_string();
            let repo = repo.to_string();
            let file_path = file_path.clone();
            let branch = branch.map(|s| s.to_string());

            let task = tokio::spawn(async move {
                match Self::fetch_file_content_static(&client, &owner, &repo, &file_path, branch.as_deref()).await {
                    Ok(file) => Some((file_path, file)),
                    Err(e) => {
                        error!("Failed to fetch file {}: {}", file_path, e);
                        None
                    }
                }
            });

            tasks.push(task);
        }

        // Wait for all tasks to complete
        for task in tasks {
            if let Ok(Some((path, file))) = task.await {
                results.insert(path, file);
            }
        }

        Ok(results)
    }

    /// Static method for fetching file content (used in async tasks)
    async fn fetch_file_content_static(
        client: &Octocrab,
        owner: &str,
        repo: &str,
        file_path: &str,
        branch: Option<&str>,
    ) -> Result<GitHubFile, GovernanceError> {
        let branch = branch.unwrap_or("main");
        
        let response = client
            .repos(owner, repo)
            .get_content()
            .path(file_path)
            .r#ref(branch)
            .send()
            .await
            .map_err(|e| GovernanceError::GitHubError(format!("Failed to fetch file: {}", e)))?;

        // Handle the response - octocrab 0.38 returns Content enum
        match response {
            Content::File(file) => {
                // Decode base64 content
                let content_bytes = match file.content {
                    Some(encoded) => {
                        base64::engine::general_purpose::STANDARD
                            .decode(encoded.trim_end_matches('\n'))
                            .map_err(|e| GovernanceError::GitHubError(format!("Failed to decode base64 content: {}", e)))?
                    }
                    None => {
                        return Err(GovernanceError::GitHubError("File content is empty".to_string()));
                    }
                };
                
                Ok(GitHubFile {
                    path: file.path,
                    content: content_bytes,
                    sha: file.sha,
                    size: file.size as u64,
                    download_url: file.download_url.map(|u| u.to_string()),
                })
            }
            Content::Directory(_) => {
                Err(GovernanceError::GitHubError(format!("Path '{}' is a directory, not a file", file_path)))
            }
            Content::Symlink(_) => {
                Err(GovernanceError::GitHubError(format!("Path '{}' is a symlink, not a file", file_path)))
            }
            Content::Submodule(_) => {
                Err(GovernanceError::GitHubError(format!("Path '{}' is a submodule, not a file", file_path)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_github_file_operations_creation() {
        // This test requires a valid GitHub token
        // In a real test environment, you would use a test token
        let result = GitHubFileOperations::new("test_token".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_comparison_creation() {
        let comparison = FileComparison {
            file_path: "test.txt".to_string(),
            source_sha: "abc123".to_string(),
            target_sha: Some("def456".to_string()),
            is_same: false,
            size_diff: Some(100),
            content_diff: Some("Content differs".to_string()),
        };

        assert_eq!(comparison.file_path, "test.txt");
        assert_eq!(comparison.source_sha, "abc123");
        assert_eq!(comparison.target_sha, Some("def456".to_string()));
        assert!(!comparison.is_same);
        assert_eq!(comparison.size_diff, Some(100));
        assert_eq!(comparison.content_diff, Some("Content differs".to_string()));
    }

    #[test]
    fn test_github_repo_creation() {
        let repo = GitHubRepo {
            owner: "test-owner".to_string(),
            name: "test-repo".to_string(),
            default_branch: "main".to_string(),
            last_commit_sha: "abc123def456".to_string(),
        };

        assert_eq!(repo.owner, "test-owner");
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.default_branch, "main");
        assert_eq!(repo.last_commit_sha, "abc123def456");
    }
}
