use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

pub fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    // Remove existing symlink if it exists
    if target.exists() || target.is_symlink() {
        fs::remove_file(target).ok();
    }

    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory for {:?}", target))?;
    }

    // Create symlink
    symlink(source, target)
        .with_context(|| format!("Failed to create symlink from {:?} to {:?}", source, target))?;

    Ok(())
}

pub fn remove_symlink(target: &Path) -> Result<()> {
    if target.is_symlink() {
        fs::remove_file(target)
            .with_context(|| format!("Failed to remove symlink {:?}", target))?;
    } else if target.exists() {
        return Err(anyhow::anyhow!("{:?} is not a symlink", target));
    }

    Ok(())
}

pub fn is_symlink_valid(link_path: &Path) -> bool {
    if !link_path.is_symlink() {
        return false;
    }

    // Check if the symlink points to an existing target
    fs::read_link(link_path)
        .ok()
        .map(|target| target.exists())
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn get_symlink_target(link_path: &Path) -> Result<Option<std::path::PathBuf>> {
    if !link_path.is_symlink() {
        return Ok(None);
    }

    let target = fs::read_link(link_path)
        .with_context(|| format!("Failed to read symlink {:?}", link_path))?;

    Ok(Some(target))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_create_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("link");
        
        fs::write(&source, "test content").unwrap();
        
        create_symlink(&source, &target).unwrap();
        
        assert!(target.is_symlink());
        assert_eq!(fs::read_link(&target).unwrap(), source);
    }

    #[test]
    fn test_create_symlink_with_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("nested").join("dir").join("link");
        
        fs::write(&source, "test content").unwrap();
        
        create_symlink(&source, &target).unwrap();
        
        assert!(target.is_symlink());
        assert!(target.parent().unwrap().exists());
    }

    #[test]
    fn test_create_symlink_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let source1 = temp_dir.path().join("source1.txt");
        let source2 = temp_dir.path().join("source2.txt");
        let target = temp_dir.path().join("link");
        
        fs::write(&source1, "content1").unwrap();
        fs::write(&source2, "content2").unwrap();
        
        // Create first symlink
        create_symlink(&source1, &target).unwrap();
        assert_eq!(fs::read_link(&target).unwrap(), source1);
        
        // Overwrite with second symlink
        create_symlink(&source2, &target).unwrap();
        assert_eq!(fs::read_link(&target).unwrap(), source2);
    }

    #[test]
    fn test_remove_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let target = temp_dir.path().join("link");
        
        fs::write(&source, "test content").unwrap();
        create_symlink(&source, &target).unwrap();
        
        remove_symlink(&target).unwrap();
        
        assert!(!target.exists());
        assert!(source.exists()); // Source should still exist
    }

    #[test]
    fn test_remove_symlink_error_on_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let file = temp_dir.path().join("regular.txt");
        
        fs::write(&file, "content").unwrap();
        
        let result = remove_symlink(&file);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_nonexistent_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("nonexistent");
        
        // Should succeed (no-op)
        let result = remove_symlink(&target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_symlink_valid() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let valid_link = temp_dir.path().join("valid_link");
        let broken_link = temp_dir.path().join("broken_link");
        
        fs::write(&source, "content").unwrap();
        
        // Valid symlink
        create_symlink(&source, &valid_link).unwrap();
        assert!(is_symlink_valid(&valid_link));
        
        // Broken symlink (pointing to non-existent file)
        create_symlink(&temp_dir.path().join("nonexistent"), &broken_link).unwrap();
        assert!(!is_symlink_valid(&broken_link));
        
        // Regular file
        assert!(!is_symlink_valid(&source));
        
        // Non-existent path
        assert!(!is_symlink_valid(&temp_dir.path().join("nonexistent")));
    }

    #[test]
    fn test_get_symlink_target() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let link = temp_dir.path().join("link");
        
        fs::write(&source, "content").unwrap();
        create_symlink(&source, &link).unwrap();
        
        let target = get_symlink_target(&link).unwrap();
        assert_eq!(target, Some(source.clone()));
        
        // Regular file returns None
        let regular_target = get_symlink_target(&source).unwrap();
        assert_eq!(regular_target, None);
        
        // Non-existent returns None
        let nonexistent_target = get_symlink_target(&temp_dir.path().join("nonexistent")).unwrap();
        assert_eq!(nonexistent_target, None);
    }
}