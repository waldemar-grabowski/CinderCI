use std::path::{Path, PathBuf};

use crate::error::ErrorType;

/// Represents the result of a successful configuration file search.
#[derive(Debug)]
pub struct SearchResult {
    /// Path to the discovered configuration file.
    pub found_path: PathBuf,

    /// Number of parent directories traversed during the search.
    pub levels_up: usize,
}

/// Searches for a CinderCI configuration file in the given directory
/// and its parent directories.
///
/// The search checks `cinder.yml` first, followed by `.cinder.yml`.
/// If a `.git` entry is found, the search stops at the repository root.
///
/// If no configuration file is found, returns `ErrorType::NotFound`
/// containing all directories that were searched.
///
/// # Arguments
///
/// * `root_path` - The directory from which the search begins.
///
/// # Returns
///
/// * `Ok(SearchResult)` - The discovered configuration file and search depth.
/// * `Err(ErrorType::NotFound)` - No configuration file was found.
pub fn find_file(root_path: &Path) -> Result<SearchResult, ErrorType> {
    let mut search_dir = root_path.to_path_buf();
    let mut levels_up = 0;
    let mut searched: Vec<PathBuf> = Vec::new();

    loop {
        // Record the current directory before searching it.
        searched.push(search_dir.to_path_buf());

        // Build configuration file candidates.
        let path = search_dir.join("cinder.yml");
        let hidden_path = search_dir.join(".cinder.yml");

        // Check the primary configuration file first.
        if path.is_file() {
            return Ok(SearchResult {
                found_path: path,
                levels_up,
            });
        }

        // Check the alternative configuration file.
        if hidden_path.is_file() {
            return Ok(SearchResult {
                found_path: hidden_path,
                levels_up,
            });
        }

        // Stop searching when the repository root is reached.
        if search_dir.join(".git").exists() {
            return Err(ErrorType::NotFound(searched));
        }

        // Move to the parent directory.
        match search_dir.parent() {
            Some(parent) => {
                search_dir = parent.to_path_buf();
                levels_up += 1;
            }
            None => {
                return Err(ErrorType::NotFound(searched));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_file() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path();

        std::fs::write(dir_path.join("cinder.yml"), "").unwrap();

        let result = find_file(dir_path).unwrap();

        assert_eq!(result.found_path, dir_path.join("cinder.yml"));
        assert_eq!(result.levels_up, 0);
    }

    #[test]
    fn test_find_hidden_file() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path();

        std::fs::write(dir_path.join(".cinder.yml"), "").unwrap();

        let result = find_file(dir_path).unwrap();

        assert_eq!(result.found_path, dir_path.join(".cinder.yml"));
        assert_eq!(result.levels_up, 0);
    }

    #[test]
    fn test_find_file_in_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path();

        std::fs::write(dir_path.join("cinder.yml"), "").unwrap();

        let nested = dir_path.join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();

        // Simulate a repository root.
        std::fs::create_dir_all(dir_path.join(".git")).unwrap();

        let result = find_file(&nested).unwrap();

        assert_eq!(result.found_path, dir_path.join("cinder.yml"));
        assert_eq!(result.levels_up, 3);
    }

    #[test]
    fn test_stop_at_git_directory() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path();

        std::fs::write(dir_path.join("cinder.yml"), "").unwrap();

        let nested = dir_path.join("a/b");
        std::fs::create_dir_all(&nested).unwrap();

        std::fs::create_dir_all(dir_path.join("a/.git")).unwrap();

        let result = find_file(&nested);

        assert!(matches!(result, Err(ErrorType::NotFound(_))));
    }

    #[test]
    fn test_not_found_returns_searched_directories() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path();

        std::fs::create_dir_all(dir_path.join(".git")).unwrap();

        let result = find_file(dir_path);

        match result {
            Err(ErrorType::NotFound(searched)) => {
                assert_eq!(searched.len(), 1);
                assert_eq!(searched[0], dir_path);
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}
