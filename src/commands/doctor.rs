use crate::agent::AgentSource;
use crate::config::{ensure_claude_agents_dir, get_project_root, AgentsConfig};
use crate::linker::{create_symlink, is_symlink_valid, remove_symlink};
use anyhow::Result;
use colored::*;
use std::collections::HashSet;
use std::fs;

#[derive(Debug)]
struct Issue {
    agent_name: String,
    issue_type: IssueType,
    description: String,
    fixable: bool,
}

#[derive(Debug)]
enum IssueType {
    MissingSource,
    BrokenSymlink,
    MissingSymlink,
    DuplicateAgent,
    OrphanedSymlink,
    UnmanagedFile,
}

pub fn execute(fix: bool) -> Result<()> {
    let project_root = get_project_root()?;
    let mut config = AgentsConfig::load(&project_root)?;

    println!("{}", "Running diagnostics...".cyan().bold());
    println!();

    let mut issues = Vec::new();
    let mut seen_names = HashSet::new();

    // Check each agent in config
    for agent in &config.agents {
        let local_path = agent.get_local_path(&project_root);
        let link_path = agent.get_link_path(&project_root);

        // Check for missing source
        if !local_path.exists() {
            let fixable = matches!(&agent.source, AgentSource::GitHub(_));
            issues.push(Issue {
                agent_name: agent.name.clone(),
                issue_type: IssueType::MissingSource,
                description: format!("Source file/directory missing: {:?}", local_path),
                fixable,
            });
        } else if agent.enabled {
            // Check symlink status for enabled agents
            if !link_path.exists() && !link_path.is_symlink() {
                issues.push(Issue {
                    agent_name: agent.name.clone(),
                    issue_type: IssueType::MissingSymlink,
                    description: "Agent is enabled but symlink is missing".to_string(),
                    fixable: true,
                });
            } else if !is_symlink_valid(&link_path) {
                issues.push(Issue {
                    agent_name: agent.name.clone(),
                    issue_type: IssueType::BrokenSymlink,
                    description: "Symlink exists but is broken".to_string(),
                    fixable: true,
                });
            }
        }

        // Check for duplicate agents
        if !seen_names.insert(agent.name.clone()) {
            issues.push(Issue {
                agent_name: agent.name.clone(),
                issue_type: IssueType::DuplicateAgent,
                description: "Duplicate agent name in configuration".to_string(),
                fixable: true,
            });
        }
    }

    // Check for orphaned symlinks in .claude/agents
    let claude_agents_dir = project_root.join(".claude").join("agents");
    if claude_agents_dir.exists() {
        for entry in fs::read_dir(&claude_agents_dir)? {
            let entry = entry?;
            let path = entry.path();

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            if path.is_symlink() {
                // Check if this symlink has a corresponding agent in config
                if !config.agents.iter().any(|a| a.name == name && a.enabled) {
                    issues.push(Issue {
                        agent_name: name,
                        issue_type: IssueType::OrphanedSymlink,
                        description: "Symlink exists without corresponding agent in config"
                            .to_string(),
                        fixable: true,
                    });
                }
            } else if path.is_file() {
                // Regular file in .claude/agents - should be managed via symlinks
                issues.push(Issue {
                    agent_name: name,
                    issue_type: IssueType::UnmanagedFile,
                    description: format!(
                        "Regular file in .claude/agents/ should be managed via ccagents"
                    ),
                    fixable: true,
                });
            }
        }
    }

    // Report findings
    if issues.is_empty() {
        println!("{} All checks passed! No issues found.", "✓".green().bold());
        return Ok(());
    }

    println!(
        "{} Found {} issue{}:",
        "⚠".yellow().bold(),
        issues.len(),
        if issues.len() == 1 { "" } else { "s" }
    );
    println!();

    for issue in &issues {
        let icon = match issue.issue_type {
            IssueType::MissingSource => "✗".red(),
            IssueType::BrokenSymlink | IssueType::MissingSymlink => "⚠".yellow(),
            IssueType::DuplicateAgent => "⚠".yellow(),
            IssueType::OrphanedSymlink => "○".yellow(),
            IssueType::UnmanagedFile => "◆".blue(),
        };

        println!(
            "  {} {} - {}",
            icon,
            issue.agent_name.bold(),
            issue.description
        );

        if issue.fixable {
            println!("    {} This issue can be fixed automatically", "→".green());
        } else {
            println!("    {} Manual intervention required", "→".red());
        }
    }

    // Apply fixes if requested
    if fix {
        println!();
        println!("{}", "Applying fixes...".cyan().bold());

        let mut fixed_count = 0;
        let mut config_modified = false;

        for issue in &issues {
            if !issue.fixable {
                continue;
            }

            match issue.issue_type {
                IssueType::MissingSource => {
                    // For GitHub sources, we could re-download, but for now we'll remove
                    config.agents.retain(|a| a.name != issue.agent_name);
                    config_modified = true;
                    println!(
                        "  {} Removed agent with missing source: {}",
                        "✓".green(),
                        issue.agent_name
                    );
                    fixed_count += 1;
                }
                IssueType::BrokenSymlink => {
                    // Remove and recreate the symlink
                    if let Some(agent) = config.agents.iter().find(|a| a.name == issue.agent_name) {
                        let link_path = agent.get_link_path(&project_root);
                        let local_path = agent.get_local_path(&project_root);

                        remove_symlink(&link_path).ok();
                        if local_path.exists() {
                            create_symlink(&local_path, &link_path)?;
                            println!(
                                "  {} Fixed broken symlink: {}",
                                "✓".green(),
                                issue.agent_name
                            );
                            fixed_count += 1;
                        }
                    }
                }
                IssueType::MissingSymlink => {
                    // Create the missing symlink
                    if let Some(agent) = config.agents.iter().find(|a| a.name == issue.agent_name) {
                        let link_path = agent.get_link_path(&project_root);
                        let local_path = agent.get_local_path(&project_root);

                        ensure_claude_agents_dir(&project_root)?;
                        create_symlink(&local_path, &link_path)?;
                        println!(
                            "  {} Created missing symlink: {}",
                            "✓".green(),
                            issue.agent_name
                        );
                        fixed_count += 1;
                    }
                }
                IssueType::DuplicateAgent => {
                    // Remove duplicates, keeping only the first occurrence
                    let mut seen = HashSet::new();
                    config.agents.retain(|a| seen.insert(a.name.clone()));
                    config_modified = true;
                    println!(
                        "  {} Removed duplicate agent: {}",
                        "✓".green(),
                        issue.agent_name
                    );
                    fixed_count += 1;
                }
                IssueType::OrphanedSymlink => {
                    // Remove the orphaned symlink
                    let link_path = claude_agents_dir.join(&issue.agent_name);
                    remove_symlink(&link_path).ok();
                    println!(
                        "  {} Removed orphaned symlink: {}",
                        "✓".green(),
                        issue.agent_name
                    );
                    fixed_count += 1;
                }
                IssueType::UnmanagedFile => {
                    // Import the unmanaged file
                    println!("  {} Unmanaged file '{}' detected - run 'ccagents import' to convert to managed agent", "ℹ".blue(), issue.agent_name);
                    // We don't automatically fix this - require explicit import command
                }
            }
        }

        if config_modified {
            config.save(&project_root)?;
        }

        println!();
        println!(
            "{} Fixed {} of {} issue{}",
            "✓".green().bold(),
            fixed_count,
            issues.len(),
            if issues.len() == 1 { "" } else { "s" }
        );
    } else {
        println!();
        println!(
            "Run {} to automatically fix these issues",
            "ccagents doctor --fix".cyan()
        );
    }

    Ok(())
}
