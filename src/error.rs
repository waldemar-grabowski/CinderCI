use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ErrorType {
     #[error("cinder.yml not found")]
    NotFound,
    #[error("file '{}' is empty", 0)]
    Empty(PathBuf),
    #[error("cannot read '{}'", 0)]
    NotReadable {
        path: PathBuf,
        source: std::io::Error,
    },
}
