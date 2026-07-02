//! Abstract memory traits for Post-Quantum Secure OS
//!
//! These traits define interfaces for the unified memory abstraction,
//! treating all storage (RAM, files, etc.) as content-addressed encrypted blocks
//! with blockchain-based history tracking.

use crate::block::traits::{
    Block, BlockId, BlockStorage, ContentAddressedStorage, EncryptedBlock, StorageStats,
};
use crate::crypto::traits::{HashFunction, SignatureScheme, SymmetricEncryption};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use std::path::PathBuf;
use std::result::Result;
use std::sync::Arc;

/// Memory address type
pub type MemoryAddress = Vec<u8>;

/// Trait for a memory region identifier
pub trait MemoryRegionId:
    Clone + Eq + std::hash::Hash + AsRef<[u8]> + Debug + Serialize + DeserializeOwned + Send + Sync
{
    fn from_bytes(bytes: Vec<u8>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}

/// Trait for a memory region
///
/// Represents a contiguous region of memory that can be addressed as a whole.
pub trait MemoryRegion: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// The type of region identifier
    type Id: MemoryRegionId;

    /// Return the unique identifier of this region
    fn id(&self) -> &Self::Id;

    /// Return the base address of this region
    fn base_address(&self) -> &MemoryAddress;

    /// Return the size of this region in bytes
    fn size(&self) -> usize;

    /// Return the type of this region (RAM, file, mmap, etc.)
    fn region_type(&self) -> MemoryRegionType;

    /// Return whether this region is encrypted
    fn is_encrypted(&self) -> bool;

    /// Return the encryption algorithm (if encrypted)
    fn encryption_algorithm(&self) -> Option<&str>;

    /// Return whether this region is writable
    fn is_writable(&self) -> bool;

    /// Return whether this region is executable
    fn is_executable(&self) -> bool;

    /// Return the access permissions for this region
    fn permissions(&self) -> MemoryPermissions;

    /// Return metadata associated with this region
    fn metadata(&self) -> &std::collections::HashMap<String, String>;
}

/// Memory region types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryRegionType {
    /// Regular RAM memory
    Ram,
    /// Memory-mapped file
    Mmap,
    /// Heap-allocated memory
    Heap,
    /// Stack memory
    Stack,
    /// Block storage (file system)
    BlockStorage,
    /// Network storage (distributed)
    Network,
    /// Custom region type
    Custom(u8),
}

/// Memory access permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MemoryPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl MemoryPermissions {
    pub fn new(read: bool, write: bool, execute: bool) -> Self {
        Self {
            read,
            write,
            execute,
        }
    }

    pub fn none() -> Self {
        Self::default()
    }

    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
        }
    }

    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
        }
    }

    pub fn read_execute() -> Self {
        Self {
            read: true,
            write: false,
            execute: true,
        }
    }

    pub fn full() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
        }
    }
}

/// Trait for a memory block
///
/// Represents a fixed-size block of memory that can be addressed by content hash.
pub trait MemoryBlock: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// The type of block this wraps
    type Block: EncryptedBlock;

    /// Return the underlying block
    fn block(&self) -> &Self::Block;

    /// Return the memory address (content hash)
    fn address(&self) -> &MemoryAddress;

    /// Return the data size in bytes
    fn size(&self) -> usize;

    /// Return whether this block is currently loaded in memory
    fn is_loaded(&self) -> bool;

    /// Return the last access time
    fn last_accessed(&self) -> Option<i64>;

    /// Return the last modification time
    fn last_modified(&self) -> Option<i64>;

    /// Return the reference count (number of references to this block)
    fn ref_count(&self) -> usize;
}

/// Trait for a memory manager
///
/// Manages the unified memory abstraction, providing allocation,
/// deallocation, and access to memory blocks.
pub trait MemoryManager: Send + Sync {
    /// The type of memory block
    type Block: MemoryBlock;
    /// The type of memory region identifier
    type RegionId: MemoryRegionId;
    /// The type of memory region
    type Region: MemoryRegion<Id = Self::RegionId>;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Allocate a new memory region of the given size
    fn allocate(
        &mut self,
        size: usize,
        region_type: MemoryRegionType,
    ) -> Result<Self::RegionId, Self::Error>;

    /// Allocate a memory region from existing data
    fn allocate_from_data(&mut self, data: Vec<u8>) -> Result<Self::RegionId, Self::Error>;

    /// Deallocate a memory region
    fn deallocate(&mut self, region_id: Self::RegionId) -> Result<(), Self::Error>;

    /// Read data from a memory region
    fn read(
        &self,
        region_id: &Self::RegionId,
        offset: usize,
        size: usize,
    ) -> Result<Vec<u8>, Self::Error>;

    /// Write data to a memory region
    fn write(
        &mut self,
        region_id: &Self::RegionId,
        offset: usize,
        data: &[u8],
    ) -> Result<(), Self::Error>;

    /// Get a memory block by its content address
    fn get_block(&self, address: &MemoryAddress) -> Result<Self::Block, Self::Error>;

    /// Get a memory region by its identifier
    fn get_region(&self, region_id: &Self::RegionId) -> Result<Self::Region, Self::Error>;

    /// Map a file into memory
    fn map_file(
        &mut self,
        path: &PathBuf,
        offset: usize,
        size: usize,
    ) -> Result<Self::RegionId, Self::Error>;

    /// Unmap a file from memory
    fn unmap_file(&mut self, region_id: Self::RegionId) -> Result<(), Self::Error>;

    /// Create a memory-mapped view of a block storage
    fn map_block_storage<B, E>(
        &mut self,
        storage: Arc<dyn BlockStorage<Block = B, Error = E>>,
    ) -> Result<Self::RegionId, Self::Error>
    where
        B: Block + 'static,
        E: std::error::Error + Send + Sync + 'static;

    /// Get the total allocated memory
    fn total_allocated(&self) -> Result<usize, Self::Error>;

    /// Get the available memory
    fn available_memory(&self) -> Result<usize, Self::Error>;

    /// Get memory statistics
    fn stats(&self) -> Result<MemoryStats, Self::Error>;

    /// Flush all changes to persistent storage
    fn flush(&mut self) -> Result<(), Self::Error>;

    /// Clear the memory cache
    fn clear_cache(&mut self) -> Result<(), Self::Error>;
}

/// Memory statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total_regions: u64,
    pub total_blocks: u64,
    pub total_size: usize,
    pub used_size: usize,
    pub available_size: usize,
    pub loaded_blocks: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// Trait for an address space
///
/// Represents a virtual address space that maps addresses to memory regions.
pub trait AddressSpace: Send + Sync {
    /// The type of memory manager
    type MemoryManager: MemoryManager;
    /// The type of memory region identifier
    type RegionId: MemoryRegionId;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get the memory manager for this address space
    fn memory_manager(&self) -> &Self::MemoryManager;

    /// Allocate memory at a specific address
    fn allocate_at(&mut self, address: MemoryAddress, size: usize) -> Result<(), Self::Error>;

    /// Free memory at a specific address
    fn free_at(&mut self, address: &MemoryAddress) -> Result<(), Self::Error>;

    /// Map a memory region to an address
    fn map_region(
        &mut self,
        region_id: Self::RegionId,
        address: MemoryAddress,
    ) -> Result<(), Self::Error>;

    /// Unmap a memory region from an address
    fn unmap_region(&mut self, address: &MemoryAddress) -> Result<Self::RegionId, Self::Error>;

    /// Resolve an address to a memory region and offset
    fn resolve(&self, address: &MemoryAddress) -> Result<(Self::RegionId, usize), Self::Error>;

    /// Read from an address
    fn read_at(&self, address: &MemoryAddress, size: usize) -> Result<Vec<u8>, Self::Error>;

    /// Write to an address
    fn write_at(&mut self, address: &MemoryAddress, data: &[u8]) -> Result<(), Self::Error>;

    /// Get the base address of a region
    fn get_base_address(&self, region_id: &Self::RegionId) -> Result<MemoryAddress, Self::Error>;

    /// Check if an address is mapped
    fn is_mapped(&self, address: &MemoryAddress) -> bool;

    /// Get all mapped regions
    fn mapped_regions(&self) -> Result<Vec<Self::RegionId>, Self::Error>;
}

/// Trait for a file system based on content-addressed blocks
///
/// Provides a file system interface where files are stored as blocks
/// addressed by their content hash.
pub trait ContentAddressedFileSystem: Send + Sync {
    /// The type of block storage backend
    type Storage: ContentAddressedStorage;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Get the underlying block storage
    fn storage(&self) -> &Self::Storage;

    /// Create a new file from data
    fn create_file(&mut self, path: &str, data: Vec<u8>) -> Result<(), Self::Error>;

    /// Read a file by path
    fn read_file(&self, path: &str) -> Result<Vec<u8>, Self::Error>;

    /// Write data to a file
    fn write_file(&mut self, path: &str, data: Vec<u8>) -> Result<(), Self::Error>;

    /// Append data to a file
    fn append_file(&mut self, path: &str, data: Vec<u8>) -> Result<(), Self::Error>;

    /// Delete a file
    fn delete_file(&mut self, path: &str) -> Result<(), Self::Error>;

    /// Get file metadata
    fn file_metadata(&self, path: &str) -> Result<FileMetadata, Self::Error>;

    /// List files in a directory
    fn list_files(&self, path: &str) -> Result<Vec<String>, Self::Error>;

    /// Create a directory
    fn create_directory(&mut self, path: &str) -> Result<(), Self::Error>;

    /// Get the content address of a file
    fn content_address(&self, path: &str) -> Result<MemoryAddress, Self::Error>;

    /// Check if a file exists
    fn file_exists(&self, path: &str) -> Result<bool, Self::Error>;

    /// Get file size
    fn file_size(&self, path: &str) -> Result<usize, Self::Error>;

    /// Rename a file
    fn rename_file(&mut self, old_path: &str, new_path: &str) -> Result<(), Self::Error>;

    /// Get the file's history (previous versions as content addresses)
    fn file_history(&self, path: &str) -> Result<Vec<MemoryAddress>, Self::Error>;
}

/// File metadata
#[derive(Debug, Clone, Default)]
pub struct FileMetadata {
    pub path: String,
    pub size: usize,
    pub content_address: MemoryAddress,
    pub created_at: i64,
    pub modified_at: i64,
    pub accessed_at: i64,
    pub permissions: FilePermissions,
    pub is_directory: bool,
    pub metadata: std::collections::HashMap<String, String>,
}

/// File permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FilePermissions {
    pub owner_read: bool,
    pub owner_write: bool,
    pub owner_execute: bool,
    pub group_read: bool,
    pub group_write: bool,
    pub group_execute: bool,
    pub other_read: bool,
    pub other_write: bool,
    pub other_execute: bool,
}

/// Trait for a file handle
///
/// Represents an open file in the content-addressed file system.
pub trait FileHandle: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Read data from the file
    fn read(&mut self, size: usize) -> Result<Vec<u8>, Self::Error>;

    /// Write data to the file
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Seek to a position in the file
    fn seek(&mut self, pos: u64) -> Result<(), Self::Error>;

    /// Get the current position in the file
    fn position(&self) -> Result<u64, Self::Error>;

    /// Get the size of the file
    fn size(&self) -> Result<u64, Self::Error>;

    /// Flush changes to storage
    fn flush(&mut self) -> Result<(), Self::Error>;

    /// Close the file handle
    fn close(&mut self) -> Result<(), Self::Error>;
}

/// Trait for a block cache
///
/// Caches frequently accessed blocks in memory for performance.
pub trait BlockCache: Send + Sync {
    /// The type of block to cache
    type Block: Block;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Get a block from cache
    fn get(&self, address: &MemoryAddress) -> Option<Self::Block>;

    /// Insert a block into cache
    fn insert(&mut self, block: Self::Block) -> Result<(), Self::Error>;

    /// Remove a block from cache
    fn remove(&mut self, address: &MemoryAddress) -> Option<Self::Block>;

    /// Clear the cache
    fn clear(&mut self) -> Result<(), Self::Error>;

    /// Get the cache size in bytes
    fn size(&self) -> usize;

    /// Get the maximum cache size in bytes
    fn max_size(&self) -> usize;

    /// Set the maximum cache size
    fn set_max_size(&mut self, size: usize) -> Result<(), Self::Error>;

    /// Get cache statistics
    fn stats(&self) -> CacheStats;
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub entries: u64,
    pub size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

/// Trait for a memory allocator
///
/// Handles allocation and deallocation of memory blocks.
pub trait MemoryAllocator: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Allocate a block of memory
    fn allocate(&mut self, size: usize, alignment: usize) -> Result<MemoryAddress, Self::Error>;

    /// Deallocate a block of memory
    fn deallocate(&mut self, address: MemoryAddress, size: usize) -> Result<(), Self::Error>;

    /// Reallocate a block of memory
    fn reallocate(
        &mut self,
        address: MemoryAddress,
        old_size: usize,
        new_size: usize,
    ) -> Result<MemoryAddress, Self::Error>;

    /// Get allocation statistics
    fn stats(&self) -> AllocationStats;
}

/// Allocation statistics
#[derive(Debug, Clone, Default)]
pub struct AllocationStats {
    pub allocations: u64,
    pub deallocations: u64,
    pub total_allocated: usize,
    pub total_freed: usize,
    pub peak_memory: usize,
    pub current_memory: usize,
}

/// Trait for a memory mapper
///
/// Maps between virtual addresses and physical/storage addresses.
pub trait MemoryMapper: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Map a virtual address to a physical address
    fn map(
        &mut self,
        virtual_addr: MemoryAddress,
        physical_addr: MemoryAddress,
        size: usize,
    ) -> Result<(), Self::Error>;

    /// Unmap a virtual address
    fn unmap(&mut self, virtual_addr: &MemoryAddress) -> Result<MemoryAddress, Self::Error>;

    /// Resolve a virtual address to a physical address
    fn resolve(&self, virtual_addr: &MemoryAddress) -> Result<MemoryAddress, Self::Error>;

    /// Get all mappings
    fn mappings(
        &self,
    ) -> Result<std::collections::HashMap<MemoryAddress, MemoryAddress>, Self::Error>;

    /// Clear all mappings
    fn clear(&mut self) -> Result<(), Self::Error>;
}

/// Trait for a unified storage interface
///
/// Provides a single interface for all types of storage (RAM, file, network).
pub trait UnifiedStorage: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Store data and return its content address
    fn store(&mut self, data: Vec<u8>) -> Result<MemoryAddress, Self::Error>;

    /// Retrieve data by its content address
    fn retrieve(&self, address: &MemoryAddress) -> Result<Vec<u8>, Self::Error>;

    /// Update existing data (creates a new version)
    fn update(
        &mut self,
        old_address: &MemoryAddress,
        new_data: Vec<u8>,
    ) -> Result<MemoryAddress, Self::Error>;

    /// Delete data by its content address
    fn delete(&mut self, address: &MemoryAddress) -> Result<(), Self::Error>;

    /// Check if data exists
    fn exists(&self, address: &MemoryAddress) -> Result<bool, Self::Error>;

    /// Get the size of stored data
    fn size(&self, address: &MemoryAddress) -> Result<usize, Self::Error>;

    /// Get metadata for stored data
    fn metadata(
        &self,
        address: &MemoryAddress,
    ) -> Result<std::collections::HashMap<String, String>, Self::Error>;

    /// List all stored content addresses
    fn list(&self) -> Result<Vec<MemoryAddress>, Self::Error>;

    /// Get storage statistics
    fn stats(&self) -> Result<StorageStats, Self::Error>;
}

/// Trait for a versioned storage
///
/// Tracks multiple versions of data with content addressing.
pub trait VersionedStorage: UnifiedStorage {
    /// Get all versions of data with the same initial content address
    fn get_versions(
        &self,
        initial_address: &MemoryAddress,
    ) -> Result<Vec<MemoryAddress>, Self::Error>;

    /// Get the current version of data
    fn get_current(&self, initial_address: &MemoryAddress) -> Result<MemoryAddress, Self::Error>;

    /// Get a specific version by index
    fn get_version(
        &self,
        initial_address: &MemoryAddress,
        version: u64,
    ) -> Result<MemoryAddress, Self::Error>;

    /// Get the version history (chain of addresses)
    fn version_history(
        &self,
        current_address: &MemoryAddress,
    ) -> Result<Vec<MemoryAddress>, Self::Error>;
}

/// Trait for a memory factory
///
/// Creates memory manager instances with specific configurations.
pub trait MemoryFactory: Send + Sync {
    /// The type of memory manager to create
    type MemoryManager: MemoryManager;
    /// Configuration type
    type Config: MemoryConfig;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new memory manager with the given configuration
    fn create(&self, config: Self::Config) -> Result<Self::MemoryManager, Self::Error>;

    /// Create a new memory manager with default configuration
    fn create_default(&self) -> Result<Self::MemoryManager, Self::Error>;
}

/// Trait for memory configuration
pub trait MemoryConfig: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// Return the maximum memory size
    fn max_memory_size(&self) -> usize;

    /// Return the block size for memory allocation
    fn block_size(&self) -> usize;

    /// Return the cache size
    fn cache_size(&self) -> usize;

    /// Return whether to enable encryption
    fn enable_encryption(&self) -> bool;

    /// Return the encryption algorithm (if enabled)
    fn encryption_algorithm(&self) -> Option<&str>;

    /// Return whether to enable compression
    fn enable_compression(&self) -> bool;

    /// Return the compression algorithm (if enabled)
    fn compression_algorithm(&self) -> Option<&str>;

    /// Return whether to use memory-mapped files
    fn use_memory_mapped_files(&self) -> bool;

    /// Return the hash function for content addressing
    fn hash_function(&self) -> &str;
}
