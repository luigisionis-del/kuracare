/// Errors of the Kuracare core.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("wrong passphrase")]
    WrongPassphrase,
    #[error("the vault is locked")]
    Locked,
    #[error("a vault already exists")]
    AlreadyExists,
    #[error("no vault on this device")]
    NoVault,
    #[error("record {0} is corrupted or was tampered with")]
    Corrupt(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("migration: {0}")]
    Migration(String),
    #[error("key derivation: {0}")]
    Kdf(String),
    #[error("storage: {0}")]
    Store(#[from] rusqlite::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
