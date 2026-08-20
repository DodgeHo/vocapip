use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("invalid data: {0}")]
    InvalidData(String),
    #[error("resource unavailable: {0}")]
    Resource(String),
}

pub type AppResult<T> = Result<T, AppError>;

pub trait CommandResultExt<T> {
    fn command(self) -> Result<T, String>;
}

impl<T> CommandResultExt<T> for AppResult<T> {
    fn command(self) -> Result<T, String> {
        self.map_err(|error| {
            let message = error.to_string();
            eprintln!("[vocapip] command failed: {message}");
            message
        })
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::InvalidData(value)
    }
}
