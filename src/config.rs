use crate::agent::Agent;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    pub agents: Vec<Agent>,
}

impl AgentsConfig {
    pub fn load(project_root: &Path) -> Result<Self> {
        let config_path = project_root.join(".agents.json");

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {:?}", config_path))?;

        serde_json::from_str(&content).with_context(|| format!("Failed to parse {:?}", config_path))
    }

    pub fn save(&self, project_root: &Path) -> Result<()> {
        let config_path = project_root.join(".agents.json");
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize agents config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write {:?}", config_path))?;

        Ok(())
    }

    pub fn add_agent(&mut self, agent: Agent) -> Result<()> {
        // Check for duplicates
        if self.agents.iter().any(|a| a.name == agent.name) {
            return Err(anyhow::anyhow!("Agent '{}' already exists", agent.name));
        }

        self.agents.push(agent);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_agent(&mut self, name: &str) -> Result<()> {
        let initial_len = self.agents.len();
        self.agents.retain(|a| a.name != name);

        if self.agents.len() == initial_len {
            return Err(anyhow::anyhow!("Agent '{}' not found", name));
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_agent(&self, name: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.name == name)
    }

    pub fn get_agent_mut(&mut self, name: &str) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.name == name)
    }

    pub fn enabled_agents(&self) -> Vec<&Agent> {
        self.agents.iter().filter(|a| a.enabled).collect()
    }

    pub fn disabled_agents(&self) -> Vec<&Agent> {
        self.agents.iter().filter(|a| !a.enabled).collect()
    }
}

pub fn get_project_root() -> Result<PathBuf> {
    std::env::current_dir().context("Failed to get current directory")
}

pub fn ensure_claude_agents_dir(project_root: &Path) -> Result<PathBuf> {
    let claude_agents_dir = project_root.join(".claude").join("agents");

    if !claude_agents_dir.exists() {
        fs::create_dir_all(&claude_agents_dir)
            .with_context(|| format!("Failed to create {:?}", claude_agents_dir))?;
    }

    Ok(claude_agents_dir)
}

pub fn ensure_ccagents_dir(project_root: &Path) -> Result<PathBuf> {
    let ccagents_dir = project_root.join(".ccagents");

    if !ccagents_dir.exists() {
        fs::create_dir_all(&ccagents_dir)
            .with_context(|| format!("Failed to create {:?}", ccagents_dir))?;
    }

    Ok(ccagents_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{Agent, AgentSource};
    use tempfile::TempDir;

    #[test]
    fn test_agents_config_default() {
        let config = AgentsConfig::default();
        assert!(config.agents.is_empty());
    }

    #[test]
    fn test_agents_config_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = AgentsConfig::load(temp_dir.path()).unwrap();
        assert!(config.agents.is_empty());
    }

    #[test]
    fn test_agents_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = AgentsConfig::default();

        let agent = Agent::new(
            "test-agent".to_string(),
            AgentSource::Local(PathBuf::from(".ccagents/test-agent")),
        );
        config.agents.push(agent);

        config.save(temp_dir.path()).unwrap();

        let loaded_config = AgentsConfig::load(temp_dir.path()).unwrap();
        assert_eq!(loaded_config.agents.len(), 1);
        assert_eq!(loaded_config.agents[0].name, "test-agent");
    }

    #[test]
    fn test_add_agent() {
        let mut config = AgentsConfig::default();
        let agent = Agent::new(
            "test".to_string(),
            AgentSource::Local(PathBuf::from("path")),
        );

        config.add_agent(agent.clone()).unwrap();
        assert_eq!(config.agents.len(), 1);

        // Test duplicate detection
        let result = config.add_agent(agent);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_agent() {
        let mut config = AgentsConfig::default();
        let agent = Agent::new(
            "test".to_string(),
            AgentSource::Local(PathBuf::from("path")),
        );

        config.agents.push(agent);
        config.remove_agent("test").unwrap();
        assert!(config.agents.is_empty());

        // Test removing non-existent
        let result = config.remove_agent("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_agent() {
        let mut config = AgentsConfig::default();
        let agent = Agent::new(
            "test".to_string(),
            AgentSource::Local(PathBuf::from("path")),
        );

        config.agents.push(agent);

        let found = config.get_agent("test");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test");

        let not_found = config.get_agent("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_agent_mut() {
        let mut config = AgentsConfig::default();
        let agent = Agent::new(
            "test".to_string(),
            AgentSource::Local(PathBuf::from("path")),
        );

        config.agents.push(agent);

        if let Some(agent) = config.get_agent_mut("test") {
            agent.enabled = false;
        }

        assert!(!config.agents[0].enabled);
    }

    #[test]
    fn test_enabled_disabled_agents() {
        let mut config = AgentsConfig::default();

        let mut agent1 = Agent::new(
            "enabled".to_string(),
            AgentSource::Local(PathBuf::from("path1")),
        );
        agent1.enabled = true;

        let mut agent2 = Agent::new(
            "disabled".to_string(),
            AgentSource::Local(PathBuf::from("path2")),
        );
        agent2.enabled = false;

        config.agents.push(agent1);
        config.agents.push(agent2);

        let enabled = config.enabled_agents();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "enabled");

        let disabled = config.disabled_agents();
        assert_eq!(disabled.len(), 1);
        assert_eq!(disabled[0].name, "disabled");
    }

    #[test]
    fn test_ensure_claude_agents_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = ensure_claude_agents_dir(temp_dir.path()).unwrap();

        assert!(result.exists());
        assert!(result.is_dir());
        assert_eq!(result, temp_dir.path().join(".claude").join("agents"));
    }

    #[test]
    fn test_ensure_ccagents_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = ensure_ccagents_dir(temp_dir.path()).unwrap();

        assert!(result.exists());
        assert!(result.is_dir());
        assert_eq!(result, temp_dir.path().join(".ccagents"));
    }

    #[test]
    fn test_config_json_format() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = AgentsConfig::default();

        let agent = Agent::new(
            "test".to_string(),
            AgentSource::GitHub("https://github.com/user/repo".to_string()),
        );
        config.agents.push(agent);

        config.save(temp_dir.path()).unwrap();

        let json_content = fs::read_to_string(temp_dir.path().join(".agents.json")).unwrap();
        assert!(json_content.contains("\"name\": \"test\""));
        assert!(json_content.contains("\"type\": \"GitHub\""));
        assert!(json_content.contains("\"enabled\": true"));
    }
}
