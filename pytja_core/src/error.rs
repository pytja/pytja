use thiserror::Error;

#[derive(Error, Debug)]
pub enum PytjaError {
    #[error("Database connection failed: {0}")]
    DatabaseConnection(String),

    #[error("Database query failed: {0}")]
    DatabaseError(String),
    
    #[error("SQLx Error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Item already exists: {0}")]
    AlreadyExists(String),

    #[error("Quota exceeded. Usage: {current}, Limit: {limit}")]
    QuotaExceeded { current: usize, limit: usize },

    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("System time error")]
    TimeError(#[from] std::time::SystemTimeError),

    #[error("Internal System Error: {0}")]
    System(String),
}

impl From<PytjaError> for std::io::Error {
    fn from(err: PytjaError) -> Self {
        match err {
            // Wenn es bereits ein System/IO-Fehler als String ist, nutzen wir Other
            PytjaError::System(msg) => std::io::Error::new(std::io::ErrorKind::Other, msg),
            // Quota-Fehler werden als 'DiskFull' oder 'PermissionDenied' interpretiert
            PytjaError::QuotaExceeded { .. } => std::io::Error::new(std::io::ErrorKind::StorageFull, err.to_string()),
            // Alle anderen Fehler werden als generische I/O-Fehler weitergegeben
            _ => std::io::Error::new(std::io::ErrorKind::Other, err.to_string()),
        }
    }
}