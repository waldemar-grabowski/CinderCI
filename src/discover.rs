use std::path::{Path, PathBuf};

use crate::error::ErrorType;

#[derive(Debug)]
pub struct SearchResult {
    pub found_path: PathBuf,
    pub levels_up: u8,
}

pub fn find_file(root_path: &Path) -> Result<SearchResult, ErrorType> {
    let path = root_path.join("cinder.yml");
    if path.is_file() {
        Ok(SearchResult {
            found_path: path.to_path_buf(),
            levels_up: 0,
        })
    } else {
        Err(ErrorType::NotFound(vec![root_path.to_path_buf()]))
    }
}

#[cfg(test)]
mod test {

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
}
