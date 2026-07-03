//! Storage module for pqdos
//!
//! This module provides implementations for storing and retrieving
//! blocks and blockchains using various backends (GitHub, local filesystem, etc.).

pub mod github;
pub mod local;
pub mod traits;

pub use github::{AsyncStorageBackend, GitHubBlock, GitHubBlockchain, GitHubConfig, GitHubStorage};
pub use local::{create_pqdos_system_storage, LocalBlock, LocalBlockchain, LocalStorage, LocalStorageFactory};
pub use traits::{
    StorageBackend, StorageBackendFactory, StorageConfig, StorageError, StorageRegistry,
    StorageResult, StoredBlock, StoredBlockchain,
};
