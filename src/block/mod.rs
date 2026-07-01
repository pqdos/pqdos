//! Block storage module for Post-Quantum Secure OS
//!
//! This module provides abstract block storage traits for
//! content-addressed block storage with Git-like hashing.

pub mod traits;

pub use traits::{
    Block,
    BlockBuilder,
    BlockHasher,
    BlockId,
    BlockIterator,
    BlockQuery,
    BlockStorage,
    BlockVerifier,
    ContentAddressedStorage,
    EncryptedBlock,
    EncryptedBlockBuilder,
    StorageStats,
};
