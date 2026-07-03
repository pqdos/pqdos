//! Storage Backend Traits for pqdos
//!
//! This module provides abstract traits for storage backends, enabling
//! different implementations (GitHub, local filesystem, S3, etc.)
//! to store and retrieve memory blocks.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use std::result::Result;

/// Error type for storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Storage not found")]
    NotFound,

    #[error("Storage already exists")]
    AlreadyExists,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for StorageError {
    fn from(error: std::io::Error) -> Self {
        StorageError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(error: serde_json::Error) -> Self {
        StorageError::SerializationError(error.to_string())
    }
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Trait for block data stored in a storage backend
pub trait StoredBlock: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// Return the unique identifier of this block
    fn id(&self) -> &str;

    /// Return the previous block ID (for blockchain)
    fn previous_id(&self) -> Option<&str>;

    /// Return the base64-encoded data
    fn data(&self) -> &str;

    /// Return the owner identifier
    fn owner_id(&self) -> &str;

    /// Return the block type (e.g., "data", "system", "executable")
    fn block_type(&self) -> &str;

    /// Return the timestamp when this block was created
    fn timestamp(&self) -> i64;

    /// Return the signature (if any)
    fn signature(&self) -> Option<&str>;

    /// Return metadata associated with this block
    fn metadata(&self) -> &std::collections::HashMap<String, String>;

    /// Check if this block is a genesis block (no previous block)
    fn is_genesis(&self) -> bool {
        self.previous_id().is_none()
    }
}

/// Trait for blockchain data stored in a storage backend
pub trait StoredBlockchain: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// Return the name of this blockchain
    fn name(&self) -> &str;

    /// Return the genesis block ID
    fn genesis_block(&self) -> &str;

    /// Return the head block ID
    fn head_block(&self) -> &str;

    /// Return all block IDs in this blockchain
    fn blocks(&self) -> &[String];

    /// Return the creation timestamp
    fn created_at(&self) -> i64;

    /// Return the last update timestamp
    fn updated_at(&self) -> i64;

    /// Return optional description
    fn description(&self) -> Option<&str>;
}

/// Trait for a storage backend
///
/// Storage backends provide persistent storage for memory blocks and blockchains.
/// Each user can have multiple storage backends configured.
pub trait StorageBackend: Send + Sync {
    /// Type of block stored by this backend
    type Block: StoredBlock + Clone;
    /// Type of blockchain stored by this backend
    type Blockchain: StoredBlockchain + Clone;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Return a unique identifier for this storage backend
    fn id(&self) -> &str;

    /// Return the type of this storage backend (e.g., "github", "local", "s3")
    fn backend_type(&self) -> &str;

    /// Return the owner/user identifier that this storage belongs to
    fn owner_id(&self) -> &str;

    /// Return the storage configuration as a string representation
    fn config(&self) -> String;

    /// Initialize the storage backend
    fn initialize(&mut self) -> StorageResult<()>;

    /// Check if the storage is accessible
    fn is_accessible(&self) -> StorageResult<bool>;

    /// Store a block
    fn store_block(&self, block: &Self::Block) -> StorageResult<()>;

    /// Retrieve a block by ID
    fn get_block(&self, block_id: &str) -> StorageResult<Option<Self::Block>>;

    /// Check if a block exists
    fn has_block(&self, block_id: &str) -> StorageResult<bool>;

    /// Delete a block by ID
    fn delete_block(&self, block_id: &str) -> StorageResult<()>;

    /// List all block IDs in this storage
    fn list_block_ids(&self) -> StorageResult<Vec<String>>;

    /// List all blocks owned by a specific user
    fn list_blocks_by_owner(&self, owner_id: &str) -> StorageResult<Vec<Self::Block>>;

    /// Store a blockchain
    fn store_blockchain(&self, chain: &Self::Blockchain) -> StorageResult<()>;

    /// Retrieve a blockchain by name
    fn get_blockchain(&self, name: &str) -> StorageResult<Option<Self::Blockchain>>;

    /// List all blockchain names
    fn list_blockchain_names(&self) -> StorageResult<Vec<String>>;

    /// Create a new block with the given parameters
    fn create_block(
        &self,
        data: &[u8],
        owner_id: &str,
        block_type: &str,
        previous_id: Option<&str>,
        metadata: std::collections::HashMap<String, String>,
    ) -> StorageResult<Self::Block>;

    /// Get the genesis block for a blockchain
    fn get_genesis_block(&self, chain_name: &str) -> StorageResult<Option<Self::Block>> {
        let chain = self.get_blockchain(chain_name)?;
        match chain {
            Some(chain) => self.get_block(chain.genesis_block()),
            None => Ok(None),
        }
    }

    /// Get the head block for a blockchain
    fn get_head_block(&self, chain_name: &str) -> StorageResult<Option<Self::Block>> {
        let chain = self.get_blockchain(chain_name)?;
        match chain {
            Some(chain) => self.get_block(chain.head_block()),
            None => Ok(None),
        }
    }
}

/// Trait for creating storage backends
pub trait StorageBackendFactory: Send + Sync {
    /// The type of storage backend to create
    type Backend: StorageBackend;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new storage backend with the given configuration
    fn create(&self, config: &str, owner_id: &str) -> StorageResult<Self::Backend>;

    /// Create a storage backend from environment variables
    fn from_env(&self, owner_id: &str) -> StorageResult<Option<Self::Backend>>;
}

/// Trait for a storage registry that manages multiple storage backends per user
pub trait StorageRegistry: Send + Sync {
    /// Type of storage backend
    type Backend: StorageBackend;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Add a storage backend for a user
    fn add_storage(&mut self, user_id: &str, backend: Self::Backend) -> StorageResult<()>;

    /// Get all storage backends for a user
    fn get_storages(&self, user_id: &str) -> StorageResult<Vec<&Self::Backend>>;

    /// Get the default storage for a user
    fn get_default_storage(&self, user_id: &str) -> StorageResult<Option<&Self::Backend>>;

    /// Set the default storage for a user
    fn set_default_storage(&mut self, user_id: &str, backend_id: &str) -> StorageResult<()>;

    /// Remove a storage backend for a user
    fn remove_storage(&mut self, user_id: &str, backend_id: &str) -> StorageResult<()>;

    /// Check if a user has a specific storage backend
    fn has_storage(&self, user_id: &str, backend_id: &str) -> bool;
}

/// Configuration for a storage backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Unique identifier for this storage configuration
    pub id: String,
    /// Type of storage (e.g., "github", "local", "s3")
    pub backend_type: String,
    /// Owner/user identifier
    pub owner_id: String,
    /// Configuration parameters (key-value pairs)
    pub parameters: std::collections::HashMap<String, String>,
    /// Whether this is the default storage for the owner
    pub is_default: bool,
}

impl StorageConfig {
    /// Create a new storage configuration
    pub fn new(id: impl Into<String>, backend_type: impl Into<String>, owner_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            backend_type: backend_type.into(),
            owner_id: owner_id.into(),
            parameters: std::collections::HashMap::new(),
            is_default: false,
        }
    }

    /// Set a configuration parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Mark as default storage
    pub fn as_default(mut self) -> Self {
        self.is_default = true;
        self
    }
}
