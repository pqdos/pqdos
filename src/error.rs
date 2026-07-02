//! Error types for the PQC Distributed OS

use thiserror::Error;

/// Result type for the entire crate
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type
#[derive(Error, Debug)]
pub enum Error {
    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Consensus error: {0}")]
    ConsensusError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}
