//! Abstract block storage traits for Post-Quantum Secure OS
//!
//! These traits define interfaces for content-addressed block storage,
//! enabling Git-like hashing and blockchain-based history tracking.

use std::sync::Arc;
use std::fmt::Debug;
use std::result::Result;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

/// Trait for a block identifier (content address)
/// 
/// The identifier is typically the hash of the block's content,
/// making it content-addressable like in Git.
pub trait BlockId:
    Clone
    + Eq
    + std::hash::Hash
    + AsRef<[u8]>
    + Debug
    + Serialize
    + DeserializeOwned
    + Send
    + Sync
{
    /// Create a new block ID from raw bytes
    fn from_bytes(bytes: Vec<u8>) -> Self;

    /// Convert to raw bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Return the size in bytes
    fn size(&self) -> usize;
}

/// Trait for a block of data in the content-addressed storage
/// 
/// Blocks are the fundamental units of storage, identified by their content hash.
pub trait Block: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// The type of identifier for this block
    type Id: BlockId;
    /// The type of data stored in this block
    type Data: AsRef<[u8]> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync;

    /// Return the unique identifier of this block (content address)
    fn id(&self) -> &Self::Id;

    /// Return the data stored in this block
    fn data(&self) -> &Self::Data;

    /// Return the identifier of the previous block (for chaining)
    /// Returns None for genesis/root blocks
    fn previous(&self) -> Option<&Self::Id>;

    /// Return the timestamp when this block was created (Unix timestamp)
    fn timestamp(&self) -> i64;

    /// Return the signature of this block (if signed)
    fn signature(&self) -> Option<&[u8]>;

    /// Return the public key that signed this block (if signed)
    fn signer(&self) -> Option<&[u8]>;

    /// Return the size of the data in bytes
    fn data_size(&self) -> usize;

    /// Validate the block's internal consistency
    /// This includes checking hash, signature, and other invariants
    fn is_valid(&self) -> bool;

    /// Return the version of the block format
    fn version(&self) -> u8;

    /// Return metadata associated with this block
    fn metadata(&self) -> &std::collections::HashMap<String, String>;
}

/// Trait for building new blocks
pub trait BlockBuilder: Send + Sync {
    /// The type of block this builder creates
    type Block: Block;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new block with the given data
    /// 
    /// If `previous` is Some, this block will be linked to the previous block.
    /// If `previous` is None, this creates a genesis block.
    fn new_block(
        &mut self,
        data: Vec<u8>,
        previous: Option<<Self::Block as Block>::Id>,
    ) -> Result<<Self::Block as Block>::Id, Self::Error>;

    /// Create a genesis (root) block with the given data
    fn genesis_block(&mut self, data: Vec<u8>) -> Result<<Self::Block as Block>::Id, Self::Error> {
        self.new_block(data, None)
    }

    /// Set metadata for the next block
    fn with_metadata(&mut self, key: String, value: String) -> &mut Self;

    /// Set the timestamp for the next block
    fn with_timestamp(&mut self, timestamp: i64) -> &mut Self;
}

/// Trait for block storage backend
/// 
/// Provides persistent storage for blocks, allowing retrieval by content address.
pub trait BlockStorage: Send + Sync {
    /// The type of block stored
    type Block: Block;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Store a block in the storage
    fn store(&mut self, block: Self::Block) -> Result<(), Self::Error>;

    /// Retrieve a block by its identifier
    fn retrieve(&self, id: &<Self::Block as Block>::Id) -> Result<Self::Block, Self::Error>;

    /// Check if a block exists in storage
    fn exists(&self, id: &<Self::Block as Block>::Id) -> Result<bool, Self::Error>;

    /// Remove a block from storage
    fn remove(&mut self, id: &<Self::Block as Block>::Id) -> Result<(), Self::Error>;

    /// List all block identifiers in storage
    fn list_blocks(&self) -> Result<Vec<<Self::Block as Block>::Id>, Self::Error>;

    /// Get statistics about the storage
    fn stats(&self) -> Result<StorageStats, Self::Error>;

    /// Clear all blocks from storage
    fn clear(&mut self) -> Result<(), Self::Error>;
}

/// Storage statistics
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    pub block_count: u64,
    pub total_size: u64,
    pub average_block_size: f64,
}

/// Trait for encrypted blocks
/// 
/// Blocks that are encrypted at rest using symmetric encryption.
pub trait EncryptedBlock: Block {
    /// Type for the encrypted data (ciphertext)
    type Ciphertext: AsRef<[u8]> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync;

    /// Return the ciphertext (encrypted data)
    fn ciphertext(&self) -> &Self::Ciphertext;

    /// Return the nonce/IV used for encryption
    fn nonce(&self) -> &[u8];

    /// Return the identifier of the key used for encryption
    fn key_id(&self) -> &[u8];

    /// Return the encryption algorithm used
    fn encryption_algorithm(&self) -> &str;
}

/// Trait for a builder that creates encrypted blocks
pub trait EncryptedBlockBuilder: BlockBuilder {
    /// The type of encrypted block this builder creates
    type Block: EncryptedBlock;

    /// Set the encryption scheme to use
    fn with_encryption(&mut self, algorithm: &str) -> &mut Self;

    /// Set the key identifier for encryption
    fn with_key_id(&mut self, key_id: Vec<u8>) -> &mut Self;
}

/// Trait for block verification
/// 
/// Provides methods to verify block integrity, signatures, and chain validity.
pub trait BlockVerifier: Send + Sync {
    /// The type of block to verify
    type Block: Block;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Verify a single block's integrity (hash and signature)
    fn verify_block(&self, block: &Self::Block) -> Result<bool, Self::Error>;

    /// Verify the entire chain from a given block back to genesis
    fn verify_chain(&self, from: &<Self::Block as Block>::Id) -> Result<bool, Self::Error>;

    /// Verify that a block is properly linked to its predecessor
    fn verify_link(
        &self,
        block: &Self::Block,
        previous: &Self::Block,
    ) -> Result<bool, Self::Error>;

    /// Verify the signature on a block
    fn verify_signature(&self, block: &Self::Block) -> Result<bool, Self::Error>;
}

/// Trait for block hashing
/// 
/// Provides content-based addressing for blocks.
pub trait BlockHasher: Send + Sync {
    /// The type of block to hash
    type Block: Block;

    /// Compute the content address (hash) of a block
    fn hash_block(&self, block: &Self::Block) -> <Self::Block as Block>::Id;

    /// Verify that a block's ID matches its content hash
    fn verify_hash(&self, block: &Self::Block) -> bool;
}

/// Trait for block iterator
/// 
/// Allows iterating through blocks in the storage.
pub trait BlockIterator: Send + Sync {
    /// The type of block
    type Block: Block;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Get the next block in iteration
    fn next(&mut self) -> Option<Result<Self::Block, Self::Error>>;
}

/// Trait for block query interface
/// 
/// Provides query capabilities for block storage.
pub trait BlockQuery: Send + Sync {
    /// The type of block
    type Block: Block;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Find blocks by data content hash (not block ID)
    fn find_by_content_hash(&self, hash: &[u8]) -> Result<Vec<<Self::Block as Block>::Id>, Self::Error>;

    /// Find blocks created after a timestamp
    fn find_after(&self, timestamp: i64) -> Result<Vec<<Self::Block as Block>::Id>, Self::Error>;

    /// Find blocks created before a timestamp
    fn find_before(&self, timestamp: i64) -> Result<Vec<<Self::Block as Block>::Id>, Self::Error>;

    /// Find blocks with specific metadata
    fn find_by_metadata(
        &self,
        key: &str,
        value: &str,
    ) -> Result<Vec<<Self::Block as Block>::Id>, Self::Error>;

    /// Get the genesis block
    fn get_genesis(&self) -> Result<Self::Block, Self::Error>;

    /// Get the latest block
    fn get_latest(&self) -> Result<Self::Block, Self::Error>;
}

/// Trait for content-addressed storage
/// 
/// High-level interface for content-addressed block storage,
/// combining storage, retrieval, and content-based addressing.
pub trait ContentAddressedStorage: BlockStorage + BlockQuery + BlockVerifier {
    /// Store data and return its content address
    fn store_data(&mut self, data: Vec<u8>) -> Result<<<Self as BlockStorage>::Block as Block>::Id, <Self as BlockStorage>::Error>;

    /// Retrieve data by its content address
    fn retrieve_data(&self, address: &[u8]) -> Result<Vec<u8>, <Self as BlockStorage>::Error>;

    /// Check if data exists by its content address
    fn exists_data(&self, address: &[u8]) -> Result<bool, <Self as BlockStorage>::Error>;
}
