use crate::config::{get_project_root, AgentsConfig};
use crate::linker::is_symlink_valid;
use anyhow::Result;
use colored::*;
use std::fs;

pub fn execute() -> Result<()> {
    let project_root = get_project_root()?;
    let config = AgentsConfig::load(&project_root)?;
    
    println!("{}", "Agents Status:".cyan().bold());
    println!();
    
    // List enabled agents
    let enabled = config.enabled_agents();
    if !enabled.is_empty() {
        println!("{}", "Enabled agents:".green().bold());
        for agent in &enabled {
            let link_path = agent.get_link_path(&project_root);
            let local_path = agent.get_local_path(&project_root);
            
            // Determine detailed status
            let status = if !local_path.exists() {
                "⚠ source missing".red().to_string()
            } else if !link_path.exists() && !link_path.is_symlink() {
                "⚠ not linked".yellow().to_string()
            } else if !is_symlink_valid(&link_path) {
                "⚠ link broken".yellow().to_string()
            } else {
                "✓ linked".green().to_string()
            };
            
            println!("  {} {} - {}", "●".green(), agent.name, status);
            
            // Show source
            match &agent.source {
                crate::agent::AgentSource::Local(path) => {
                    println!("    {} {}", "source:".dimmed(), path.display());
                }
                crate::agent::AgentSource::GitHub(url) => {
                    println!("    {} {}", "source:".dimmed(), url);
                }
            }
        }
    } else {
        println!("{}", "No enabled agents".dimmed());
    }
    
    println!();
    
    // List disabled agents from config
    let disabled = config.disabled_agents();
    if !disabled.is_empty() {
        println!("{}", "Disabled agents (in .agents.json):".yellow().bold());
        for agent in &disabled {
            println!("  {} {} - {}", "○".yellow(), agent.name, "disabled".dimmed());
            
            // Show source
            match &agent.source {
                crate::agent::AgentSource::Local(path) => {
                    println!("    {} {}", "source:".dimmed(), path.display());
                }
                crate::agent::AgentSource::GitHub(url) => {
                    println!("    {} {}", "source:".dimmed(), url);
                }
            }
        }
    }
    
    println!();
    
    // List available agents in .ccagents that are not in config
    let ccagents_dir = project_root.join(".ccagents");
    if ccagents_dir.exists() {
        let mut available_agents = Vec::new();
        
        for entry in fs::read_dir(&ccagents_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                
                // Check if this agent is not already in config
                if !config.agents.iter().any(|a| a.name == name) {
                    available_agents.push(name);
                }
            }
        }
        
        if !available_agents.is_empty() {
            println!("{}", "Available agents (in .ccagents but not configured):".blue().bold());
            for name in available_agents {
                println!("  {} {} - {}", "◇".blue(), name, "not configured".dimmed());
                println!("    {} ccagents add .ccagents/{}", "hint:".dimmed(), name);
            }
        }
    }
    
    // Summary
    println!();
    println!(
        "{}: {} enabled, {} disabled",
        "Total".bold(),
        enabled.len(),
        disabled.len()
    );
    
    Ok(())
}