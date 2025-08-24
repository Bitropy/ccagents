use crate::agent::AgentSource;
use crate::config::{ensure_ccagents_dir, ensure_claude_agents_dir, get_project_root, AgentsConfig};
use crate::downloader::download_from_github;
use crate::linker::{create_symlink, remove_symlink};
use anyhow::Result;
use colored::*;
use std::fs;

pub fn execute(prune: bool) -> Result<()> {
    let project_root = get_project_root()?;
    let mut config = AgentsConfig::load(&project_root)?;
    
    if config.agents.is_empty() {
        println!("{}", "No agents configured in .agents.json".yellow());
        println!("Use 'ccagents add <source>' to add agents");
        return Ok(());
    }
    
    let claude_agents_dir = ensure_claude_agents_dir(&project_root)?;
    let ccagents_dir = ensure_ccagents_dir(&project_root)?;
    
    // Handle pruning if requested
    if prune {
        let mut orphaned_count = 0;
        
        config.agents.retain(|agent| {
            let local_path = agent.get_local_path(&project_root);
            if !local_path.exists() {
                orphaned_count += 1;
                println!("  {} Pruning orphaned agent: {}", "✗".red(), agent.name);
                // Also remove orphaned symlink if it exists
                let link_path = agent.get_link_path(&project_root);
                if link_path.exists() || link_path.is_symlink() {
                    remove_symlink(&link_path).ok();
                }
                false
            } else {
                true
            }
        });
        
        if orphaned_count > 0 {
            config.save(&project_root)?;
            println!("{} Pruned {} orphaned agent{}\n", 
                "→".yellow(), 
                orphaned_count, 
                if orphaned_count == 1 { "" } else { "s" });
        }
    }
    
    println!("{}", "Syncing agents...".cyan().bold());
    
    // First, check for unmanaged files and remove symlinks
    let mut unmanaged_files = Vec::new();
    if claude_agents_dir.exists() {
        for entry in fs::read_dir(&claude_agents_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_symlink() {
                remove_symlink(&path).ok();
            } else if path.is_file() {
                // Regular file - not managed by ccagents
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                unmanaged_files.push(name);
            }
        }
    }
    
    // Warn about unmanaged files
    if !unmanaged_files.is_empty() {
        println!("\n{} Found {} unmanaged file{} in .claude/agents/:",
            "⚠".yellow().bold(),
            unmanaged_files.len(),
            if unmanaged_files.len() == 1 { "" } else { "s" });
        
        for name in &unmanaged_files {
            println!("  {} {}", "◆".blue(), name);
        }
        
        println!("\n  {} Run 'ccagents import' to convert these to managed agents", "→".cyan());
        println!();
    }
    
    // Sync enabled agents
    for agent in config.enabled_agents() {
        print!("  {} {}", "→".cyan(), agent.name);
        
        let local_path = agent.get_local_path(&project_root);
        let link_path = agent.get_link_path(&project_root);
        
        // Ensure the source exists
        if !local_path.exists() {
            match &agent.source {
                AgentSource::GitHub(url) => {
                    println!(" - {}", "downloading from GitHub...".yellow());
                    tokio::runtime::Runtime::new()?.block_on(async {
                        download_from_github(url, &ccagents_dir).await
                    })?;
                }
                AgentSource::Local(_) => {
                    println!(" - {}", "source not found, skipping".red());
                    continue;
                }
            }
        }
        
        // Create symlink
        create_symlink(&local_path, &link_path)?;
        println!(" - {}", "enabled".green());
    }
    
    // Report disabled agents
    let disabled = config.disabled_agents();
    if !disabled.is_empty() {
        println!("\n{}", "Disabled agents:".yellow());
        for agent in disabled {
            println!("  {} {} - {}", "○".yellow(), agent.name, "disabled".dimmed());
        }
    }
    
    println!("\n{} Sync complete!", "✓".green().bold());
    
    Ok(())
}