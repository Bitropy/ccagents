use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub source: AgentSource,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AgentSource {
    Local(PathBuf),
    GitHub(String),
}

impl Agent {
    pub fn new(name: String, source: AgentSource) -> Self {
        Self {
            name,
            source,
            enabled: true,
        }
    }

    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid agent path"))?
            .to_string();

        Ok(Self::new(name, AgentSource::Local(path.to_path_buf())))
    }

    pub fn from_url(url: &str) -> anyhow::Result<Self> {
        let parsed_url = url::Url::parse(url)?;
        
        // Extract agent name from URL
        let name = if parsed_url.host_str() == Some("github.com") {
            let segments: Vec<&str> = parsed_url.path()
                .trim_start_matches('/')
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();
            
            // We only support file URLs (with /blob/)
            if segments.len() >= 5 && segments[2] == "blob" {
                // Use the filename
                segments.last()
                    .ok_or_else(|| anyhow::anyhow!("No filename in URL"))?
                    .to_string()
            } else {
                return Err(anyhow::anyhow!(
                    "Only direct file links are supported. Please provide a URL like:\n\
                     https://github.com/user/repo/blob/main/agent.md"
                ));
            }
        } else {
            // For non-GitHub URLs, use the last segment as filename
            parsed_url
                .path_segments()
                .and_then(|segments| segments.last())
                .ok_or_else(|| anyhow::anyhow!("Invalid URL"))?
                .to_string()
        };

        Ok(Self::new(name, AgentSource::GitHub(url.to_string())))
    }

    pub fn get_local_path(&self, project_root: &Path) -> PathBuf {
        match &self.source {
            AgentSource::Local(path) => {
                if path.is_absolute() {
                    path.clone()
                } else {
                    project_root.join(path)
                }
            }
            AgentSource::GitHub(_) => project_root.join(".ccagents").join(&self.name),
        }
    }

    pub fn get_link_path(&self, project_root: &Path) -> PathBuf {
        project_root.join(".claude").join("agents").join(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_agent_new() {
        let agent = Agent::new(
            "test-agent".to_string(),
            AgentSource::Local(PathBuf::from("path/to/agent")),
        );
        
        assert_eq!(agent.name, "test-agent");
        assert!(agent.enabled);
        matches!(agent.source, AgentSource::Local(_));
    }

    #[test]
    fn test_agent_from_path() {
        let path = Path::new("test-agent.md");
        let agent = Agent::from_path(path).unwrap();
        
        assert_eq!(agent.name, "test-agent.md");
        assert!(agent.enabled);
        
        if let AgentSource::Local(p) = &agent.source {
            assert_eq!(p, Path::new("test-agent.md"));
        } else {
            panic!("Expected Local source");
        }
    }

    #[test]
    fn test_agent_from_github_repo_url_fails() {
        let url = "https://github.com/user/agent-repo";
        let result = Agent::from_url(url);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Only direct file links"));
    }

    #[test]
    fn test_agent_from_github_repo_with_git_suffix_fails() {
        let url = "https://github.com/user/agent-repo.git";
        let result = Agent::from_url(url);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_from_github_file_url() {
        let url = "https://github.com/user/repo/blob/main/agents/backend-developer.md";
        let agent = Agent::from_url(url).unwrap();
        
        assert_eq!(agent.name, "backend-developer.md");
        assert!(agent.enabled);
        
        if let AgentSource::GitHub(u) = &agent.source {
            assert_eq!(u, url);
        } else {
            panic!("Expected GitHub source");
        }
    }

    #[test]
    fn test_agent_from_github_nested_file_url() {
        let url = "https://github.com/vijaythecoder/awesome-claude-agents/blob/main/agents/universal/backend-developer.md";
        let agent = Agent::from_url(url).unwrap();
        
        assert_eq!(agent.name, "backend-developer.md");
    }

    #[test]
    fn test_get_local_path_for_local_relative() {
        let agent = Agent::new(
            "test".to_string(),
            AgentSource::Local(PathBuf::from("relative/path")),
        );
        let project_root = Path::new("/project");
        
        assert_eq!(
            agent.get_local_path(project_root),
            PathBuf::from("/project/relative/path")
        );
    }

    #[test]
    fn test_get_local_path_for_local_absolute() {
        let agent = Agent::new(
            "test".to_string(),
            AgentSource::Local(PathBuf::from("/absolute/path")),
        );
        let project_root = Path::new("/project");
        
        assert_eq!(
            agent.get_local_path(project_root),
            PathBuf::from("/absolute/path")
        );
    }

    #[test]
    fn test_get_local_path_for_github() {
        let agent = Agent::new(
            "repo-name".to_string(),
            AgentSource::GitHub("https://github.com/user/repo".to_string()),
        );
        let project_root = Path::new("/project");
        
        assert_eq!(
            agent.get_local_path(project_root),
            PathBuf::from("/project/.ccagents/repo-name")
        );
    }

    #[test]
    fn test_get_link_path() {
        let agent = Agent::new(
            "test-agent".to_string(),
            AgentSource::Local(PathBuf::from("path")),
        );
        let project_root = Path::new("/project");
        
        assert_eq!(
            agent.get_link_path(project_root),
            PathBuf::from("/project/.claude/agents/test-agent")
        );
    }

    #[test]
    fn test_invalid_url() {
        let result = Agent::from_url("not-a-url");
        assert!(result.is_err());
    }
}