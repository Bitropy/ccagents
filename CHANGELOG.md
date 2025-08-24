# Changelog

All notable changes to ccagents will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-08-23

### Added
- Initial release of ccagents CLI tool
- Agent configuration management via `.agents.json`
- Commands for agent management:
  - `add` - Add agents from local paths or GitHub URLs
  - `list` - List enabled, disabled, and available agents
  - `enable` - Enable an agent by creating symlinks
  - `disable` - Disable an agent by removing symlinks
  - `sync` - Sync agents based on configuration
  - `version` - Display version information
- Automatic directory creation for `.claude/agents` and `.ccagents`
- GitHub repository downloading with progress indicators
- Colored terminal output for better user experience
- Symlink-based agent management
- Build-time version information including git commit hash
- Comprehensive error handling and validation