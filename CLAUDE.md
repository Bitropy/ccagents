
# ccagents - Claude Code Agent Manager

A Rust CLI application that manages Claude Code agents in local projects by enabling/disabling them through symlinks.

## Overview

ccagents manages agents for Claude Code by:
- Maintaining a `.agents.json` configuration file (similar to `.mcp.json`)
- Creating symlinks in `.claude/agents/` to enable agents
- Storing downloaded/copied agents in `.ccagents/` directory
- Using relative paths for portability

## Commands

### `ccagents` or `ccagents sync [--prune]`
- Syncs agents based on `.agents.json` configuration
- Creates/removes symlinks in `.claude/agents/`
- Downloads missing GitHub agents automatically
- `--prune` flag removes orphaned entries from configuration

### `ccagents add <source>`
- Adds agent from local path or GitHub URL
- Creates `.agents.json` if not present
- Copies external files/directories to `.ccagents/`
- Automatically creates symlink if agent is enabled
- Supports both files and directories

### `ccagents list`
- Shows enabled agents with detailed status:
  - `✓ linked` - Working correctly
  - `⚠ source missing` - Source file deleted
  - `⚠ not linked` - Missing symlink
  - `⚠ link broken` - Invalid symlink
- Shows disabled agents from configuration
- Lists available agents in `.ccagents/` not in config

### `ccagents enable <name>` / `ccagents disable <name>`
- Enables/disables agents by managing symlinks
- Updates `.agents.json` enabled status
- Preserves agent source in configuration

### `ccagents clean [--force]`
- Removes orphaned agents (missing sources) from configuration
- Interactive confirmation (bypass with `--force`)
- Cleans up broken symlinks

### `ccagents doctor [--fix]`
- Comprehensive diagnostics for:
  - Missing sources
  - Broken/missing symlinks
  - Duplicate agents
  - Orphaned symlinks
- `--fix` automatically repairs fixable issues

### `ccagents version`
- Displays version information
- Shows build timestamp and git info (when available)

## Directory Structure

```
project/
├── .agents.json          # Agent configuration (uses relative paths)
├── .ccagents/           # Storage for downloaded/copied agents
│   └── agent-name/
└── .claude/
    └── agents/          # Active agents via symlinks
        └── agent-name -> ../../.ccagents/agent-name
```

## Configuration Format (.agents.json)

```json
{
  "agents": [
    {
      "name": "agent-name",
      "source": {
        "type": "Local",
        "value": ".ccagents/agent-name"
      },
      "enabled": true
    },
    {
      "name": "github-agent",
      "source": {
        "type": "GitHub",
        "value": "https://github.com/user/repo"
      },
      "enabled": false
    }
  ]
}
```

## Edge Case Handling

- **Missing sources**: Detected in list, can be cleaned with `clean` command
- **Broken symlinks**: Shown in status, fixable with `doctor --fix`
- **Orphaned symlinks**: Removed during sync or by doctor
- **Duplicate agents**: Detected and fixed by doctor
- **Portability**: Uses relative paths in `.agents.json`

## Version

Current version: 0.1.0

See CHANGELOG.md for version history.


