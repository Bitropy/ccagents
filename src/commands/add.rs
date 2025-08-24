use crate::agent::{Agent, AgentSource};
use crate::config::{
    ensure_ccagents_dir, ensure_claude_agents_dir, get_project_root, AgentsConfig,
};
use crate::downloader::download_from_github;
use crate::linker::create_symlink;
use anyhow::Result;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

pub async fn execute(source: &str) -> Result<()> {
    let project_root = get_project_root()?;
    let mut config = AgentsConfig::load(&project_root)?;

    println!("{} agent from {}", "Adding".cyan().bold(), source);

    // Determine if source is a URL or local path
    let agent = if source.starts_with("http://") || source.starts_with("https://") {
        // Handle GitHub URL
        if !source.contains("github.com") {
            return Err(anyhow::anyhow!("Only GitHub URLs are currently supported"));
        }

        let agent = Agent::from_url(source)?;

        // Download the agent
        let ccagents_dir = ensure_ccagents_dir(&project_root)?;
        println!("  {} from GitHub...", "Downloading".yellow());
        download_from_github(source, &ccagents_dir).await?;

        agent
    } else {
        // Handle local path
        let path = PathBuf::from(source);
        let absolute_path = if path.is_absolute() {
            path
        } else {
            project_root.join(&path)
        };

        if !absolute_path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {:?}", absolute_path));
        }

        // If the path is outside the project, copy it to .ccagents
        let agent = if !absolute_path.starts_with(&project_root) {
            let ccagents_dir = ensure_ccagents_dir(&project_root)?;
            let agent_name = absolute_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;

            let target_path = ccagents_dir.join(agent_name);

            println!("  {} agent to .ccagents/...", "Copying".yellow());

            // Check if source is a file or directory
            if absolute_path.is_file() {
                fs::copy(&absolute_path, &target_path)?;
            } else if absolute_path.is_dir() {
                copy_dir_all(&absolute_path, &target_path)?;
            } else {
                return Err(anyhow::anyhow!(
                    "Path is neither a file nor a directory: {:?}",
                    absolute_path
                ));
            }

            // Use relative path for portability
            let relative_target = target_path
                .strip_prefix(&project_root)
                .unwrap_or(&target_path)
                .to_path_buf();

            Agent::new(agent_name.to_string(), AgentSource::Local(relative_target))
        } else {
            // Use relative path for agents within the project
            let relative_path = absolute_path
                .strip_prefix(&project_root)
                .unwrap_or(&absolute_path)
                .to_path_buf();

            Agent::from_path(&relative_path)?
        };

        agent
    };

    // Add to config
    config.add_agent(agent.clone())?;
    config.save(&project_root)?;

    // Create symlink if enabled
    if agent.enabled {
        let _claude_agents_dir = ensure_claude_agents_dir(&project_root)?;
        let local_path = agent.get_local_path(&project_root);
        let link_path = agent.get_link_path(&project_root);

        create_symlink(&local_path, &link_path)?;
        println!("  {} symlink in .claude/agents/", "Created".green());
    }

    println!(
        "\n{} Agent '{}' added successfully!",
        "✓".green().bold(),
        agent.name
    );

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
