use crate::agent::{Agent, AgentSource};
use crate::config::{ensure_ccagents_dir, get_project_root, AgentsConfig};
use crate::linker::create_symlink;
use anyhow::Result;
use colored::*;
use std::fs;
use std::io::{self, Write};

pub fn execute(specific_name: Option<String>, all: bool) -> Result<()> {
    let project_root = get_project_root()?;
    let mut config = AgentsConfig::load(&project_root)?;
    let claude_agents_dir = project_root.join(".claude").join("agents");

    if !claude_agents_dir.exists() {
        println!("{}", "No .claude/agents directory found.".yellow());
        return Ok(());
    }

    // Find unmanaged files
    let mut unmanaged_files = Vec::new();

    for entry in fs::read_dir(&claude_agents_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip directories and symlinks
        if !path.is_file() || path.is_symlink() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Check if specific name was requested
        if let Some(ref specific) = specific_name {
            if name != *specific {
                continue;
            }
        }

        // Check if already managed
        if !config.agents.iter().any(|a| a.name == name) {
            unmanaged_files.push((name, path));
        }
    }

    if unmanaged_files.is_empty() {
        if specific_name.is_some() {
            println!("{} No unmanaged file found with that name.", "ℹ".blue());
        } else {
            println!(
                "{} No unmanaged files found in .claude/agents/",
                "✓".green()
            );
        }
        return Ok(());
    }

    // Report findings
    println!(
        "{} Found {} unmanaged file{}:",
        "ℹ".blue().bold(),
        unmanaged_files.len(),
        if unmanaged_files.len() == 1 { "" } else { "s" }
    );

    for (name, _) in &unmanaged_files {
        println!("  {} {}", "◆".blue(), name);
    }

    // Ask for confirmation if not using --all
    let should_import = if all {
        true
    } else {
        println!("\n{}", "Import these files as managed agents?".yellow());
        print!("This will move them to .ccagents/ and create symlinks [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    };

    if !should_import {
        println!("{}", "Import cancelled.".yellow());
        return Ok(());
    }

    // Import each file
    let ccagents_dir = ensure_ccagents_dir(&project_root)?;
    let mut imported_count = 0;

    for (name, source_path) in unmanaged_files {
        println!("\n{} {}", "Importing:".cyan(), name);

        // Copy to .ccagents
        let target_path = ccagents_dir.join(&name);

        // Handle existing file in .ccagents
        if target_path.exists() {
            println!(
                "  {} File already exists in .ccagents/, using existing",
                "⚠".yellow()
            );
        } else {
            fs::copy(&source_path, &target_path)
                .map_err(|e| anyhow::anyhow!("Failed to copy {}: {}", name, e))?;
            println!("  {} Copied to .ccagents/", "→".cyan());
        }

        // Remove original file
        fs::remove_file(&source_path)
            .map_err(|e| anyhow::anyhow!("Failed to remove original {}: {}", name, e))?;
        println!("  {} Removed original file", "→".cyan());

        // Create symlink
        create_symlink(&target_path, &source_path)?;
        println!("  {} Created symlink", "→".cyan());

        // Add to config
        let relative_target = target_path
            .strip_prefix(&project_root)
            .unwrap_or(&target_path)
            .to_path_buf();

        let agent = Agent::new(name.clone(), AgentSource::Local(relative_target));

        config.add_agent(agent)?;
        println!("  {} Added to .agents.json", "→".cyan());

        imported_count += 1;
    }

    // Save config
    config.save(&project_root)?;

    println!(
        "\n{} Successfully imported {} agent{}",
        "✓".green().bold(),
        imported_count,
        if imported_count == 1 { "" } else { "s" }
    );

    Ok(())
}
