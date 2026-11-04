use std::path::PathBuf;

#[derive(Debug)]
pub enum ErrorType {
    NotFound,
    Empty(PathBuf),
    NotReadable {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Empty(path) => write!(f, "file '{}' is empty", path.display()),
            Self::NotFound => write!(f, "cinder.yml not found"),
            Self::NotReadable { path: p, .. } => write!(f, "cannot read '{}'", p.display()),
        }
    }
}

impl std::error::Error for ErrorType {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::NotReadable { source , .. } => Some(source),
            Self::Empty(..) | Self::NotFound => None,
        }
    }
}
