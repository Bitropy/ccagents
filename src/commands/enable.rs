use crate::config::{ensure_claude_agents_dir, get_project_root, AgentsConfig};
use crate::linker::create_symlink;
use anyhow::Result;
use colored::*;

pub fn execute(name: &str) -> Result<()> {
    let project_root = get_project_root()?;
    let mut config = AgentsConfig::load(&project_root)?;
    
    // Find the agent
    let agent = config
        .get_agent_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found in .agents.json", name))?;
    
    if agent.enabled {
        println!("{} Agent '{}' is already enabled", "ℹ".blue(), name);
        return Ok(());
    }
    
    // Enable the agent
    agent.enabled = true;
    
    // Create symlink
    let _claude_agents_dir = ensure_claude_agents_dir(&project_root)?;
    let local_path = agent.get_local_path(&project_root);
    let link_path = agent.get_link_path(&project_root);
    
    if !local_path.exists() {
        return Err(anyhow::anyhow!(
            "Agent source does not exist: {:?}. Run 'ccagents sync' to download missing agents.",
            local_path
        ));
    }
    
    create_symlink(&local_path, &link_path)?;
    
    // Save config
    config.save(&project_root)?;
    
    println!(
        "{} Agent '{}' has been enabled",
        "✓".green().bold(),
        name
    );
    println!("  {} Created symlink in .claude/agents/", "→".cyan());
    
    Ok(())
}