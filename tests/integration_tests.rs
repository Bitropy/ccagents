use ccagents::agent::{Agent, AgentSource};
use ccagents::config::{ensure_claude_agents_dir, ensure_ccagents_dir, AgentsConfig};
use ccagents::linker::{create_symlink, is_symlink_valid};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn test_full_agent_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create a test agent file
    let test_agent_content = "# Test Agent\nThis is a test agent.";
    let test_agent_path = project_root.join("test-agent.md");
    fs::write(&test_agent_path, test_agent_content).unwrap();
    
    // 1. Add agent
    let mut config = AgentsConfig::default();
    let agent = Agent::new(
        "test-agent.md".to_string(),
        AgentSource::Local(PathBuf::from("test-agent.md")),
    );
    config.add_agent(agent.clone()).unwrap();
    config.save(project_root).unwrap();
    
    // Verify agent was added
    let loaded_config = AgentsConfig::load(project_root).unwrap();
    assert_eq!(loaded_config.agents.len(), 1);
    assert_eq!(loaded_config.agents[0].name, "test-agent.md");
    
    // 2. Create symlink (simulate sync)
    let _claude_agents_dir = ensure_claude_agents_dir(project_root).unwrap();
    let link_path = agent.get_link_path(project_root);
    let local_path = agent.get_local_path(project_root);
    
    create_symlink(&local_path, &link_path).unwrap();
    assert!(is_symlink_valid(&link_path));
    
    // 3. Disable agent
    let mut config = AgentsConfig::load(project_root).unwrap();
    if let Some(agent) = config.get_agent_mut("test-agent.md") {
        agent.enabled = false;
    }
    config.save(project_root).unwrap();
    
    // 4. Remove symlink (simulate disable command)
    fs::remove_file(&link_path).ok();
    assert!(!link_path.exists());
    
    // 5. Re-enable agent
    let mut config = AgentsConfig::load(project_root).unwrap();
    if let Some(agent) = config.get_agent_mut("test-agent.md") {
        agent.enabled = true;
    }
    config.save(project_root).unwrap();
    
    // 6. Recreate symlink
    create_symlink(&local_path, &link_path).unwrap();
    assert!(is_symlink_valid(&link_path));
}

#[test]
fn test_orphaned_agent_handling() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create agent pointing to non-existent file
    let mut config = AgentsConfig::default();
    let agent = Agent::new(
        "orphaned.md".to_string(),
        AgentSource::Local(PathBuf::from(".ccagents/orphaned.md")),
    );
    config.add_agent(agent.clone()).unwrap();
    config.save(project_root).unwrap();
    
    // Verify agent exists in config
    let loaded_config = AgentsConfig::load(project_root).unwrap();
    assert_eq!(loaded_config.agents.len(), 1);
    
    // Check that source doesn't exist
    let local_path = agent.get_local_path(project_root);
    assert!(!local_path.exists());
    
    // Simulate clean operation - remove orphaned
    let mut config = AgentsConfig::load(project_root).unwrap();
    config.agents.retain(|a| {
        let path = a.get_local_path(project_root);
        path.exists()
    });
    config.save(project_root).unwrap();
    
    // Verify orphaned agent was removed
    let cleaned_config = AgentsConfig::load(project_root).unwrap();
    assert_eq!(cleaned_config.agents.len(), 0);
}

#[test]
fn test_github_file_agent_storage() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create agent from GitHub file URL
    let url = "https://github.com/user/repo/blob/main/agent.md";
    let agent = Agent::from_url(url).unwrap();
    assert_eq!(agent.name, "agent.md");
    
    // Verify it would be stored in .ccagents
    let expected_path = project_root.join(".ccagents").join("agent.md");
    assert_eq!(agent.get_local_path(project_root), expected_path);
    
    // Save to config
    let mut config = AgentsConfig::default();
    config.add_agent(agent).unwrap();
    config.save(project_root).unwrap();
    
    // Load and verify
    let loaded_config = AgentsConfig::load(project_root).unwrap();
    assert_eq!(loaded_config.agents.len(), 1);
    
    if let AgentSource::GitHub(stored_url) = &loaded_config.agents[0].source {
        assert_eq!(stored_url, url);
    } else {
        panic!("Expected GitHub source");
    }
}

#[test]
fn test_github_repo_url_rejected() {
    // Repository URLs should be rejected
    let repo_url = "https://github.com/user/test-repo";
    let result = Agent::from_url(repo_url);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Only direct file links"));
}

#[test]
fn test_relative_paths_in_config() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create .ccagents directory and agent file
    let ccagents_dir = ensure_ccagents_dir(project_root).unwrap();
    let agent_file = ccagents_dir.join("test.md");
    fs::write(&agent_file, "test content").unwrap();
    
    // Add agent with relative path
    let mut config = AgentsConfig::default();
    let agent = Agent::new(
        "test.md".to_string(),
        AgentSource::Local(PathBuf::from(".ccagents/test.md")),
    );
    config.add_agent(agent.clone()).unwrap();
    config.save(project_root).unwrap();
    
    // Read the JSON directly to verify relative path
    let json_content = fs::read_to_string(project_root.join(".agents.json")).unwrap();
    assert!(json_content.contains("\".ccagents/test.md\""));
    assert!(!json_content.contains(&project_root.to_string_lossy().to_string()));
    
    // Verify agent can still be loaded and path resolved
    let loaded_config = AgentsConfig::load(project_root).unwrap();
    let local_path = loaded_config.agents[0].get_local_path(project_root);
    assert!(local_path.exists());
}

#[test]
fn test_multiple_agents_management() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create multiple test agents
    let agents = vec![
        ("agent1.md", true),
        ("agent2.md", false),
        ("agent3.md", true),
    ];
    
    let mut config = AgentsConfig::default();
    
    for (name, enabled) in &agents {
        let file_path = project_root.join(name);
        fs::write(&file_path, format!("Content for {}", name)).unwrap();
        
        let mut agent = Agent::new(
            name.to_string(),
            AgentSource::Local(PathBuf::from(name)),
        );
        agent.enabled = *enabled;
        config.add_agent(agent).unwrap();
    }
    
    config.save(project_root).unwrap();
    
    // Load and verify
    let loaded_config = AgentsConfig::load(project_root).unwrap();
    assert_eq!(loaded_config.agents.len(), 3);
    
    let enabled = loaded_config.enabled_agents();
    assert_eq!(enabled.len(), 2);
    
    let disabled = loaded_config.disabled_agents();
    assert_eq!(disabled.len(), 1);
    assert_eq!(disabled[0].name, "agent2.md");
}

#[test]
fn test_symlink_management() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create source file and directories
    let source_file = project_root.join("source.md");
    fs::write(&source_file, "source content").unwrap();
    
    let claude_agents_dir = ensure_claude_agents_dir(project_root).unwrap();
    let link_path = claude_agents_dir.join("linked.md");
    
    // Create symlink
    create_symlink(&source_file, &link_path).unwrap();
    assert!(link_path.is_symlink());
    assert!(is_symlink_valid(&link_path));
    
    // Delete source and verify broken symlink detection
    fs::remove_file(&source_file).unwrap();
    assert!(!is_symlink_valid(&link_path));
    
    // Recreate source
    fs::write(&source_file, "new content").unwrap();
    assert!(is_symlink_valid(&link_path));
}

#[test]
fn test_duplicate_agent_prevention() {
    let temp_dir = TempDir::new().unwrap();
    let _project_root = temp_dir.path();
    
    let mut config = AgentsConfig::default();
    let agent = Agent::new(
        "duplicate.md".to_string(),
        AgentSource::Local(PathBuf::from("duplicate.md")),
    );
    
    // Add agent first time - should succeed
    let result1 = config.add_agent(agent.clone());
    assert!(result1.is_ok());
    
    // Add same agent again - should fail
    let result2 = config.add_agent(agent);
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn test_unmanaged_file_detection() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Create .claude/agents directory
    let claude_agents_dir = ensure_claude_agents_dir(project_root).unwrap();
    
    // Create a regular file (unmanaged)
    let unmanaged_file = claude_agents_dir.join("unmanaged.md");
    fs::write(&unmanaged_file, "# Unmanaged Agent").unwrap();
    assert!(unmanaged_file.exists());
    assert!(!unmanaged_file.is_symlink());
    
    // Create a symlink (managed)
    let managed_source = project_root.join("managed.md");
    fs::write(&managed_source, "# Managed Agent").unwrap();
    let managed_link = claude_agents_dir.join("managed.md");
    create_symlink(&managed_source, &managed_link).unwrap();
    assert!(managed_link.is_symlink());
    
    // Check directory contents
    let mut regular_files = 0;
    let mut symlinks = 0;
    
    for entry in fs::read_dir(&claude_agents_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.is_symlink() {
            symlinks += 1;
        } else if path.is_file() {
            regular_files += 1;
        }
    }
    
    assert_eq!(regular_files, 1, "Should have 1 regular file");
    assert_eq!(symlinks, 1, "Should have 1 symlink");
}

#[test]
fn test_import_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    // Setup directories
    let claude_agents_dir = ensure_claude_agents_dir(project_root).unwrap();
    let ccagents_dir = ensure_ccagents_dir(project_root).unwrap();
    
    // Create an unmanaged file in .claude/agents
    let unmanaged_path = claude_agents_dir.join("import-test.md");
    let content = "# Agent to Import";
    fs::write(&unmanaged_path, content).unwrap();
    
    // Simulate import process
    // 1. Copy to .ccagents
    let target_path = ccagents_dir.join("import-test.md");
    fs::copy(&unmanaged_path, &target_path).unwrap();
    
    // 2. Remove original
    fs::remove_file(&unmanaged_path).unwrap();
    
    // 3. Create symlink
    create_symlink(&target_path, &unmanaged_path).unwrap();
    
    // 4. Add to config
    let mut config = AgentsConfig::default();
    let agent = Agent::new(
        "import-test.md".to_string(),
        AgentSource::Local(PathBuf::from(".ccagents/import-test.md")),
    );
    config.add_agent(agent).unwrap();
    config.save(project_root).unwrap();
    
    // Verify results
    assert!(target_path.exists(), "File should exist in .ccagents");
    assert!(unmanaged_path.is_symlink(), "Should be a symlink in .claude/agents");
    assert!(is_symlink_valid(&unmanaged_path), "Symlink should be valid");
    
    let loaded_config = AgentsConfig::load(project_root).unwrap();
    assert_eq!(loaded_config.agents.len(), 1);
    assert_eq!(loaded_config.agents[0].name, "import-test.md");
    
    // Verify content is preserved
    let read_content = fs::read_to_string(&unmanaged_path).unwrap();
    assert_eq!(read_content, content);
}

#[test]
fn test_mixed_agents_directory() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    
    let claude_agents_dir = ensure_claude_agents_dir(project_root).unwrap();
    
    // Create a mix of files, symlinks, and directories
    
    // Regular file (unmanaged)
    let file1 = claude_agents_dir.join("file1.md");
    fs::write(&file1, "content").unwrap();
    
    // Symlink (managed)
    let source = project_root.join("source.md");
    fs::write(&source, "content").unwrap();
    let link1 = claude_agents_dir.join("link1.md");
    create_symlink(&source, &link1).unwrap();
    
    // Directory (should be ignored)
    let dir1 = claude_agents_dir.join("subdir");
    fs::create_dir(&dir1).unwrap();
    
    // Another regular file
    let file2 = claude_agents_dir.join("file2.md");
    fs::write(&file2, "content").unwrap();
    
    // Count each type
    let mut regular_files = Vec::new();
    let mut symlinks = Vec::new();
    let mut directories = Vec::new();
    
    for entry in fs::read_dir(&claude_agents_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        
        if path.is_symlink() {
            symlinks.push(name);
        } else if path.is_file() {
            regular_files.push(name);
        } else if path.is_dir() {
            directories.push(name);
        }
    }
    
    assert_eq!(regular_files.len(), 2, "Should have 2 regular files");
    assert!(regular_files.contains(&"file1.md".to_string()));
    assert!(regular_files.contains(&"file2.md".to_string()));
    
    assert_eq!(symlinks.len(), 1, "Should have 1 symlink");
    assert!(symlinks.contains(&"link1.md".to_string()));
    
    assert_eq!(directories.len(), 1, "Should have 1 directory");
    assert!(directories.contains(&"subdir".to_string()));
}