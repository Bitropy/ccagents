#[cfg(test)]
pub mod test_helpers {
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    pub struct TestEnvironment {
        pub temp_dir: TempDir,
        pub project_root: PathBuf,
    }

    impl TestEnvironment {
        pub fn new() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let project_root = temp_dir.path().to_path_buf();
            
            Self {
                temp_dir,
                project_root,
            }
        }

        pub fn create_file(&self, path: &str, content: &str) -> PathBuf {
            let full_path = self.project_root.join(path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&full_path, content).unwrap();
            full_path
        }

        pub fn create_dir(&self, path: &str) -> PathBuf {
            let full_path = self.project_root.join(path);
            fs::create_dir_all(&full_path).unwrap();
            full_path
        }

        pub fn file_exists(&self, path: &str) -> bool {
            self.project_root.join(path).exists()
        }

        pub fn read_file(&self, path: &str) -> String {
            fs::read_to_string(self.project_root.join(path)).unwrap()
        }

        pub fn is_symlink(&self, path: &str) -> bool {
            self.project_root.join(path).is_symlink()
        }

        pub fn symlink_target(&self, path: &str) -> Option<PathBuf> {
            fs::read_link(self.project_root.join(path)).ok()
        }
    }
}