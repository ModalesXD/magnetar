use std::fmt;

/// Errores centralizados para Magnetar
#[derive(Debug, Clone)]
pub enum MagnetarError {
    Compositor(String),
    Ipc(String),
    Window(String),
    Config(String),
    Io(String),
}

impl fmt::Display for MagnetarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compositor(msg) => write!(f, "Compositor error: {}", msg),
            Self::Ipc(msg) => write!(f, "IPC error: {}", msg),
            Self::Window(msg) => write!(f, "Window error: {}", msg),
            Self::Config(msg) => write!(f, "Config error: {}", msg),
            Self::Io(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for MagnetarError {}

impl From<std::io::Error> for MagnetarError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for MagnetarError {
    fn from(err: serde_json::Error) -> Self {
        Self::Config(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MagnetarError>;
