use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ErrorType {
    #[error("cinder.yml not found")]
    NotFound(Vec<PathBuf>),
    #[error("file '{0}' is empty")]
    Empty(PathBuf),
    #[error("cannot read '{path}'")]
    NotReadable {
        path: PathBuf,
        source: std::io::Error,
    },
}
