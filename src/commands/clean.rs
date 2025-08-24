use crate::config::{get_project_root, AgentsConfig};
use anyhow::Result;
use colored::*;
use std::io::{self, Write};

pub fn execute(force: bool) -> Result<()> {
    let project_root = get_project_root()?;
    let mut config = AgentsConfig::load(&project_root)?;

    println!("{}", "Checking for orphaned agents...".cyan().bold());

    // Find orphaned agents (source doesn't exist)
    let mut orphaned = Vec::new();
    for agent in &config.agents {
        let local_path = agent.get_local_path(&project_root);
        if !local_path.exists() {
            orphaned.push(agent.clone());
        }
    }

    if orphaned.is_empty() {
        println!("{} No orphaned agents found.", "✓".green().bold());
        return Ok(());
    }

    // Report orphaned agents
    println!("\n{}", "Found orphaned agents:".yellow().bold());
    for agent in &orphaned {
        println!(
            "  {} {} - {}",
            "○".red(),
            agent.name,
            "source missing".red()
        );
        match &agent.source {
            crate::agent::AgentSource::Local(path) => {
                println!("    {} {}", "missing:".dimmed(), path.display());
            }
            crate::agent::AgentSource::GitHub(url) => {
                println!("    {} {} (can be re-downloaded)", "missing:".dimmed(), url);
            }
        }
    }

    // Ask for confirmation or use force flag
    let should_remove = if force {
        true
    } else {
        println!(
            "\n{}",
            "Remove these orphaned entries from .agents.json?".yellow()
        );
        print!("Confirm [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    };

    if should_remove {
        // Remove orphaned agents
        let initial_count = config.agents.len();
        config
            .agents
            .retain(|agent| !orphaned.iter().any(|o| o.name == agent.name));

        let removed_count = initial_count - config.agents.len();

        // Save the cleaned configuration
        config.save(&project_root)?;

        println!(
            "\n{} Removed {} orphaned agent{}",
            "✓".green().bold(),
            removed_count,
            if removed_count == 1 { "" } else { "s" }
        );

        // Also clean up any orphaned symlinks
        let claude_agents_dir = project_root.join(".claude").join("agents");
        if claude_agents_dir.exists() {
            for agent in &orphaned {
                let link_path = agent.get_link_path(&project_root);
                if link_path.exists() || link_path.is_symlink() {
                    std::fs::remove_file(&link_path).ok();
                    println!("  {} Removed orphaned symlink: {}", "→".cyan(), agent.name);
                }
            }
        }
    } else {
        println!("{}", "Clean operation cancelled.".yellow());
    }

    Ok(())
}
