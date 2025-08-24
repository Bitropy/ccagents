use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod agent;
mod commands;
mod config;
mod downloader;
mod linker;
mod version;

use commands::{add, clean, disable, doctor, enable, import, list, sync};

#[derive(Parser)]
#[command(name = "ccagents")]
#[command(version, about = "Manage Claude Code agents in your project", long_about = None)]
#[command(author = "Darek")]
#[command(long_version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new agent from a local path or GitHub URL
    Add {
        /// Path or URL to the agent
        source: String,
    },
    /// List all agents (enabled, disabled, and available)
    List,
    /// Enable an agent by creating a symlink in .claude/agents
    Enable {
        /// Name of the agent to enable
        name: String,
    },
    /// Disable an agent by removing its symlink from .claude/agents
    Disable {
        /// Name of the agent to disable
        name: String,
    },
    /// Sync agents based on .agents.json configuration
    Sync {
        /// Remove orphaned entries during sync
        #[arg(short, long)]
        prune: bool,
    },
    /// Remove orphaned agents from configuration
    Clean {
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Diagnose and fix issues with agent configuration
    Doctor {
        /// Automatically fix issues
        #[arg(short, long)]
        fix: bool,
    },
    /// Import unmanaged files from .claude/agents
    Import {
        /// Name of specific file to import
        name: Option<String>,
        /// Import all unmanaged files without confirmation
        #[arg(short, long)]
        all: bool,
    },
    /// Display version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Add { source }) => add::execute(&source).await,
        Some(Commands::List) => list::execute(),
        Some(Commands::Enable { name }) => enable::execute(&name),
        Some(Commands::Disable { name }) => disable::execute(&name),
        Some(Commands::Sync { prune }) => sync::execute(prune),
        None => sync::execute(false),
        Some(Commands::Clean { force }) => clean::execute(force),
        Some(Commands::Doctor { fix }) => doctor::execute(fix),
        Some(Commands::Import { name, all }) => import::execute(name, all),
        Some(Commands::Version) => {
            version::print_version_info();
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}
