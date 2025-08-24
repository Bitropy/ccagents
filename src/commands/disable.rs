use crate::config::{get_project_root, AgentsConfig};
use crate::linker::remove_symlink;
use anyhow::Result;
use colored::*;

pub fn execute(name: &str) -> Result<()> {
    let project_root = get_project_root()?;
    let mut config = AgentsConfig::load(&project_root)?;

    // Find the agent
    let agent = config
        .get_agent_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found in .agents.json", name))?;

    if !agent.enabled {
        println!("{} Agent '{}' is already disabled", "ℹ".blue(), name);
        return Ok(());
    }

    // Disable the agent
    agent.enabled = false;

    // Remove symlink
    let link_path = agent.get_link_path(&project_root);

    if link_path.exists() || link_path.is_symlink() {
        remove_symlink(&link_path)?;
        println!("  {} Removed symlink from .claude/agents/", "→".cyan());
    }

    // Save config
    config.save(&project_root)?;

    println!("{} Agent '{}' has been disabled", "✓".green().bold(), name);

    Ok(())
}
