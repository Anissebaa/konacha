#[derive(Debug, thiserror::Error)]
pub enum KonachaError {
    #[error("no vault is currently unlocked")]
    Locked,
    #[error("a vault already exists on disk")]
    VaultExists,
    #[error("vault file is corrupt: {0}")]
    VaultCorrupt(&'static str),
    #[error("passphrase verification failed")]
    BadPassphrase,
    #[error("ciphertext integrity check failed")]
    Tampered,
    #[error("malformed ciphertext envelope")]
    Malformed,
    #[error("kdf: {0}")]
    Kdf(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("ai: {0}")]
    Ai(String),
    #[error("script: {0}")]
    Script(String),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, KonachaError>;
