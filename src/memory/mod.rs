//! Memory module for Post-Quantum Secure OS
//!
//! This module provides abstract memory traits for
//! the unified memory abstraction, treating all storage as
//! content-addressed encrypted blocks with blockchain history.

pub mod traits;

pub use traits::{
    AddressSpace,
    AllocationStats,
    BlockCache,
    CacheStats,
    ContentAddressedFileSystem,
    FileHandle,
    FileMetadata,
    FilePermissions,
    MemoryAddress,
    MemoryAllocator,
    MemoryBlock,
    MemoryConfig,
    MemoryFactory,
    MemoryManager,
    MemoryMapper,
    MemoryPermissions,
    MemoryRegion,
    MemoryRegionId,
    MemoryRegionType,
    MemoryStats,
    // StorageStats, // Defined in block traits
    UnifiedStorage,
    VersionedStorage,
    // VersionedStorage as StorageVersioned,
};
