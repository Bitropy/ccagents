# ccagents

[![Crates.io](https://img.shields.io/crates/v/ccagents.svg)](https://crates.io/crates/ccagents)
[![Documentation](https://docs.rs/ccagents/badge.svg)](https://docs.rs/ccagents)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/Bitropy/ccagents/actions/workflows/ci.yml/badge.svg)](https://github.com/Bitropy/ccagents/actions/workflows/ci.yml)

> Manage Claude Code agents in your local projects with ease

`ccagents` is a Rust CLI tool that helps you manage [Claude Code](https://claude.ai/code) agents by creating symlinks, handling configuration, and downloading agents from GitHub. It ensures your agents are portable across different environments and team members. 

## Features

- 🔗 **Symlink-based management** - Enable/disable agents without copying files
- 📦 **Portable configuration** - Uses relative paths in `.agents.json`
- 🌐 **GitHub integration** - Download individual agent files directly from GitHub
- 🔍 **Smart diagnostics** - Detect and fix broken symlinks, missing sources, and orphaned agents
- 🎨 **Beautiful CLI** - Colored output with clear status indicators
- ⚡ **Fast & lightweight** - Written in Rust for easy portability

## Installation

### From source (requires Rust)

```bash
# Clone the repository
git clone https://github.com/yourusername/ccagents.git
cd ccagents

# Build and install
cargo install --path .
```

### From crates.io

```bash
cargo install ccagents
```

### Pre-built binaries

Download the latest release for your platform from the [releases page](https://github.com/yourusername/ccagents/releases).

## Quick Start

```bash
# Add an agent from a local file
ccagents add ./my-agent.md

# Add an agent from GitHub (single file only)
ccagents add https://github.com/user/repo/blob/main/agents/backend.md

# List all agents
ccagents list

# Enable/disable agents
ccagents enable my-agent.md
ccagents disable my-agent.md

# Sync agents (create/remove symlinks based on config)
ccagents sync

# Clean up orphaned agents
ccagents clean

# Run diagnostics
ccagents doctor
```

## Usage

### Adding Agents

Add agents from local files or GitHub:

```bash
# Local file
ccagents add ~/Documents/my-agent.md

# GitHub file (must be a direct file link)
ccagents add https://github.com/user/repo/blob/main/agent.md
```

Files are copied to `.ccagents/` directory and symlinked to `.claude/agents/`. You can commit `.ccagents` to Git and make it portable between team mates. 

### Managing Agents

```bash
# List all agents with their status
ccagents list

# Output:
# Enabled agents:
#   ● backend-developer.md - ✓ linked
#   ● code-reviewer.md - ⚠ source missing
# 
# Disabled agents:
#   ○ test-agent.md

# Enable an agent
ccagents enable backend-developer.md

# Disable an agent
ccagents disable code-reviewer.md
```

### Syncing Configuration

Sync creates/removes symlinks based on your `.agents.json`:

```bash
# Basic sync
ccagents sync

# Sync and remove orphaned entries
ccagents sync --prune
```

### Importing Unmanaged Agents

If you've added agents directly to `.claude/agents/`, import them:

```bash
# Import a specific agent
ccagents import my-agent.md

# Import all unmanaged agents
ccagents import --all
```

### Diagnostics & Cleanup

```bash
# Check for issues
ccagents doctor

# Fix issues automatically
ccagents doctor --fix

# Remove orphaned agents from config
ccagents clean

# Force cleanup without confirmation
ccagents clean --force
```

## Configuration

The `.agents.json` file stores your agent configuration:

```json
{
  "agents": [
    {
      "name": "backend-developer.md",
      "source": {
        "type": "Local",
        "value": ".ccagents/backend-developer.md"
      },
      "enabled": true
    },
    {
      "name": "code-reviewer.md",
      "source": {
        "type": "GitHub",
        "value": "https://github.com/user/repo/blob/main/agent.md"
      },
      "enabled": false
    }
  ]
}
```

## Directory Structure

```
your-project/
├── .agents.json          # Agent configuration
├── .ccagents/           # Agent storage (managed)
│   ├── backend.md
│   └── frontend.md
└── .claude/
    └── agents/          # Active agents (symlinks)
        ├── backend.md -> ../../.ccagents/backend.md
        └── frontend.md -> ../../.ccagents/frontend.md
```

## Status Indicators

- `✓ linked` - Agent is working correctly
- `⚠ source missing` - Source file has been deleted
- `⚠ not linked` - Symlink is missing
- `⚠ link broken` - Symlink points to non-existent file
- `● enabled` - Agent is enabled
- `○ disabled` - Agent is disabled

## Roadmap

- Simple sub-gent registry 
- Agents via local socket 
- Provide a use case for sockat/remote agents hosted via API 



## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Related

- [Claude Code](https://claude.ai/code) - The AI coding assistant
- [Awesome Claude Agents](https://github.com/vijaythecoder/awesome-claude-agents) - Community collection of Claude agents
