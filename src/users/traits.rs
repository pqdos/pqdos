//! Abstract user management traits for Post-Quantum Secure OS
//!
//! These traits define interfaces for user management, authentication,
//! and block ownership, enabling different implementations (blockchain-based,
//! database-backed, in-memory, etc.) without locking into specific technologies.
//!
//! The key design principle: The genesis user "futuros" owns all system executable blocks,
//! and its private key is NEVER stored or accessible through the system.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use std::result::Result;

/// Name of the genesis user - owner of all system executable blocks
pub const GENESIS_USER_NAME: &str = "futuros";

// ============================================================================
// USER IDENTIFICATION
// ============================================================================

/// Trait for a user identifier
///
/// The identifier is typically derived from the user's public key hash,
/// making it unique and content-addressable.
pub trait UserId:
    Clone + Eq + std::hash::Hash + AsRef<[u8]> + Debug + Serialize + DeserializeOwned + Send + Sync
{
    /// Create a new user ID from raw bytes
    fn from_bytes(bytes: Vec<u8>) -> Self;

    /// Create a user ID from a public key (typically a hash of the public key)
    fn from_public_key(public_key: &[u8]) -> Self;

    /// Convert to raw bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Return the size in bytes
    fn size(&self) -> usize;
}

// ============================================================================
// USER ROLES
// ============================================================================

/// Trait for user roles in the system
pub trait UserRole:
    Clone + Copy + Eq + PartialEq + Debug + Serialize + DeserializeOwned + Send + Sync
{
    /// Check if this is the genesis role
    fn is_genesis(&self) -> bool;

    /// Check if this is an admin role
    fn is_admin(&self) -> bool;

    /// Check if this is a regular user role
    fn is_user(&self) -> bool;

    /// Get the role as a string
    fn as_str(&self) -> &str;
}

/// Trait for user permissions
pub trait UserPermissions:
    Clone + Copy + Eq + PartialEq + Debug + Default + Serialize + DeserializeOwned + Send + Sync
{
    /// Check if user can create blocks
    fn can_create_blocks(&self) -> bool;

    /// Check if user can read all blocks
    fn can_read_all_blocks(&self) -> bool;

    /// Check if user can write all blocks
    fn can_write_all_blocks(&self) -> bool;

    /// Check if user can manage users
    fn can_manage_users(&self) -> bool;

    /// Check if user can manage the system
    fn can_manage_system(&self) -> bool;

    /// Check if user can execute code
    fn can_execute_code(&self) -> bool;

    /// Check if these permissions are full (all true)
    fn is_full(&self) -> bool;

    /// Create full permissions
    fn full() -> Self;
}

// ============================================================================
// USER
// ============================================================================

/// Trait for a user in the system
///
/// A user has an identifier, name, public key, role, and permissions.
/// The private key is NEVER stored in the user struct.
pub trait User: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// The type of user identifier
    type Id: UserId;
    /// The type of user role
    type Role: UserRole;
    /// The type of user permissions
    type Permissions: UserPermissions;

    /// Return the unique identifier of this user
    fn id(&self) -> &Self::Id;

    /// Return the user's name
    fn name(&self) -> &str;

    /// Return the user's public key
    ///
    /// **Security Note**: Only the PUBLIC key is stored. The private key
    /// is NEVER accessible through this system.
    fn public_key(&self) -> &[u8];

    /// Return the user's role
    fn role(&self) -> Self::Role;

    /// Return the user's permissions
    fn permissions(&self) -> Self::Permissions;

    /// Return the timestamp when this user was created (Unix timestamp)
    fn created_at(&self) -> i64;

    /// Check if this is the genesis user
    fn is_genesis(&self) -> bool;

    /// Check if this user has the given role
    fn has_role(&self, role: Self::Role) -> bool;

    /// Check if this user has the given permission
    fn has_permission(&self, permission: fn(&Self::Permissions) -> bool) -> bool;

    /// Get metadata associated with this user
    fn metadata(&self) -> &std::collections::HashMap<String, String>;
}

/// Trait for creating users
pub trait UserBuilder: Send + Sync {
    /// The type of user to create
    type User: User;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new user
    fn new_user(
        &self,
        name: String,
        public_key: Vec<u8>,
        role: <Self::User as User>::Role,
        permissions: <Self::User as User>::Permissions,
    ) -> Result<Self::User, Self::Error>;

    /// Create the genesis user
    ///
    /// **Security Note**: This takes ONLY the public key. The private key
    /// MUST be kept externally and is NEVER accessible through this system.
    fn new_genesis_user(
        &self,
        name: String,
        public_key: Vec<u8>,
    ) -> Result<Self::User, Self::Error>;
}

// ============================================================================
// BLOCK IDENTIFICATION
// ============================================================================

/// Trait for a block identifier
///
/// The identifier is typically the hash of the block's content,
/// making it content-addressable like in Git.
pub trait BlockId:
    Clone + Eq + std::hash::Hash + AsRef<[u8]> + Debug + Serialize + DeserializeOwned + Send + Sync
{
    /// Create a new block ID from raw bytes
    fn from_bytes(bytes: Vec<u8>) -> Self;

    /// Create a block ID from content (typically a hash of the content)
    fn from_content(content: &[u8]) -> Self;

    /// Convert to raw bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Return the size in bytes
    fn size(&self) -> usize;
}

// ============================================================================
// BLOCK
// ============================================================================

/// Trait for a block of data
///
/// Blocks are the fundamental units of storage, identified by their content hash.
/// Each block has an owner (a user).
pub trait Block: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// The type of identifier for this block
    type Id: BlockId;
    /// The type of user identifier for the owner
    type UserId: UserId;

    /// Return the unique identifier of this block
    fn id(&self) -> &Self::Id;

    /// Return the data stored in this block
    fn data(&self) -> &[u8];

    /// Return the identifier of the owner of this block
    fn owner_id(&self) -> &Self::UserId;

    /// Return the timestamp when this block was created (Unix timestamp)
    fn created_at(&self) -> i64;

    /// Return the type of this block (e.g., "data", "system", "executable")
    fn block_type(&self) -> &str;

    /// Check if this is a system block
    fn is_system_block(&self) -> bool;

    /// Get metadata associated with this block
    fn metadata(&self) -> &std::collections::HashMap<String, String>;

    /// Return the size of the data in bytes
    fn data_size(&self) -> usize;
}

/// Trait for building blocks
pub trait BlockBuilder: Send + Sync {
    /// The type of block to create
    type Block: Block;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new block with the given data and owner
    fn new_block(
        &self,
        data: Vec<u8>,
        owner_id: <Self::Block as Block>::UserId,
        block_type: String,
    ) -> Result<Self::Block, Self::Error>;
}

// ============================================================================
// EXECUTABLE BLOCK
// ============================================================================

/// Trait for an executable block
///
/// Executable blocks contain code that can be executed by the OS.
/// They are owned by users, typically the genesis user for system code.
pub trait ExecutableBlock: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// The type of underlying block
    type Block: Block;
    /// The type of block identifier
    type BlockId: BlockId;
    /// The type of user identifier
    type UserId: UserId;

    /// Return the underlying block
    fn block(&self) -> &Self::Block;

    /// Return the block identifier
    fn id(&self) -> &Self::BlockId;

    /// Return the code stored in this executable block
    fn code(&self) -> &[u8];

    /// Return the identifier of the owner of this executable block
    fn owner_id(&self) -> &Self::UserId;

    /// Return the entry point for execution
    fn entry_point(&self) -> &str;

    /// Return the type of executable (e.g., "kernel", "bootstrap", "driver", "service")
    fn executable_type(&self) -> &str;
}

/// Trait for building executable blocks
pub trait ExecutableBlockBuilder: Send + Sync {
    /// The type of executable block to create
    type ExecutableBlock: ExecutableBlock;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new executable block
    fn new_executable(
        &self,
        code: Vec<u8>,
        owner_id: <Self::ExecutableBlock as ExecutableBlock>::UserId,
        entry_point: String,
        executable_type: String,
    ) -> Result<Self::ExecutableBlock, Self::Error>;
}

// ============================================================================
// USER SYSTEM
// ============================================================================

/// Error type for user system operations
#[derive(Debug, thiserror::Error)]
pub enum UserSystemError {
    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("System not initialized")]
    SystemNotInitialized,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Block not found")]
    BlockNotFound,

    #[error("Block already exists")]
    BlockAlreadyExists,
}

impl From<String> for UserSystemError {
    fn from(error: String) -> Self {
        UserSystemError::InternalError(error)
    }
}

impl From<&str> for UserSystemError {
    fn from(error: &str) -> Self {
        UserSystemError::InternalError(error.to_string())
    }
}

/// Result type for user system operations
pub type UserSystemResult<T> = Result<T, UserSystemError>;

/// Trait for a user system
///
/// Manages users, their blocks, and system executable blocks.
/// The genesis user "futuros" owns all system executable blocks.
pub trait UserSystem: Send + Sync {
    /// The type of user
    type User: User;
    /// The type of user identifier
    type UserId: UserId;
    /// The type of block
    type Block: Block<UserId = Self::UserId>;
    /// The type of block identifier
    type BlockId: BlockId;
    /// The type of executable block
    type ExecutableBlock: ExecutableBlock<
        Block = Self::Block,
        BlockId = Self::BlockId,
        UserId = Self::UserId,
    >;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Initialize the user system with the genesis user
    ///
    /// **Security Note**: This takes ONLY the public key. The private key
    /// MUST be kept externally and is NEVER accessible through this system.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the genesis user (default: "futuros")
    /// * `public_key` - The PUBLIC KEY only of the genesis user
    fn initialize(&mut self, name: String, public_key: Vec<u8>) -> Result<(), Self::Error>;

    /// Initialize with the default "futuros" user
    fn initialize_with_futuros(&mut self, public_key: Vec<u8>) -> Result<(), Self::Error>;

    /// Check if the system is initialized
    fn is_initialized(&self) -> bool;

    /// Get the genesis user
    fn get_genesis_user(&self) -> Option<Self::User>;

    /// Get a user by ID
    fn get_user(&self, user_id: &Self::UserId) -> Option<Self::User>;

    /// Get a user by public key
    fn get_user_by_public_key(&self, public_key: &[u8]) -> Option<Self::User>;

    /// Check if a user is the genesis user
    fn is_genesis_user(&self, user_id: &Self::UserId) -> bool;

    /// Create a new block owned by a user
    fn create_block(
        &mut self,
        data: Vec<u8>,
        owner_id: Self::UserId,
        block_type: String,
    ) -> Result<Self::BlockId, Self::Error>;

    /// Register a system executable owned by the genesis user
    ///
    /// This is the primary method for storing OS executable code.
    /// All system executables are owned by the genesis user "futuros".
    fn register_system_executable(
        &mut self,
        name: String,
        code: Vec<u8>,
        entry_point: String,
        executable_type: String,
    ) -> Result<Self::BlockId, Self::Error>;

    /// Get a block by ID
    fn get_block(&self, block_id: &Self::BlockId) -> Option<Self::Block>;

    /// Get the owner of a block
    fn get_block_owner(&self, block_id: &Self::BlockId) -> Option<Self::UserId>;

    /// Get all blocks owned by a user
    fn get_user_blocks(&self, user_id: &Self::UserId) -> Vec<Self::BlockId>;

    /// Get all system blocks (owned by genesis user)
    fn get_system_blocks(&self) -> Vec<Self::BlockId>;

    /// Get a system executable by block ID
    fn get_system_executable(&self, block_id: &Self::BlockId) -> Option<Self::ExecutableBlock>;

    /// List all system executables
    fn list_system_executables(&self) -> Vec<Self::ExecutableBlock>;

    /// Verify that a block is a system block (owned by genesis user)
    fn is_system_block(&self, block_id: &Self::BlockId) -> bool;
}

/// Trait for a user system factory
pub trait UserSystemFactory: Send + Sync {
    /// The type of user system to create
    type UserSystem: UserSystem;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new user system
    fn create(&self) -> Self::UserSystem;

    /// Create and initialize a new user system with the genesis user
    fn create_initialized(
        &self,
        name: String,
        public_key: Vec<u8>,
    ) -> Result<Self::UserSystem, Self::Error>;

    /// Create a user system with the default genesis user "futuros"
    fn create_with_futuros(&self, public_key: Vec<u8>) -> Result<Self::UserSystem, Self::Error>;
}

// ============================================================================
// AUTHENTICATION
// ============================================================================

/// Authentication token
#[derive(Debug, Clone)]
pub struct AuthToken {
    pub token: Vec<u8>,
    pub user_id: Vec<u8>,
    pub expires_at: i64,
    pub issued_at: i64,
}

impl AuthToken {
    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        now > self.expires_at
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub token: AuthToken,
    pub user_id: Vec<u8>,
}

/// Trait for user authentication
pub trait UserAuthenticator: Send + Sync {
    /// The type of user system
    type UserSystem: UserSystem;
    /// The type of user
    type User: User;
    /// The type of user identifier
    type UserId: UserId;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new authenticator for the user system
    fn new(user_system: Self::UserSystem) -> Self;

    /// Authenticate a user by verifying a signature
    ///
    /// In a real system, this would verify a cryptographic signature
    /// made with the user's private key. For now, this is a stub.
    ///
    /// **Security Note**: The actual signature verification must be done
    /// externally using the user's private key, which is NEVER accessible
    /// through this system.
    fn authenticate(
        &self,
        user_id: &Self::UserId,
        challenge: &[u8],
        signature: &[u8], // Signature is verified externally
    ) -> Result<AuthResult, Self::Error>;

    /// Generate a random challenge for authentication
    fn generate_challenge(&self) -> Vec<u8>;

    /// Validate an authentication token
    fn validate_token(&self, token: &AuthToken) -> Result<Self::User, Self::Error>;

    /// Get the user system
    fn user_system(&self) -> &Self::UserSystem;

    /// Get the genesis user
    fn get_genesis_user(&self) -> Result<Self::User, Self::Error>;
}

/// Trait for authentication provider
///
/// Provides external authentication services (e.g., PQC signature verification)
pub trait AuthenticationProvider: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Verify a signature using a public key
    fn verify_signature(
        &self,
        data: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, Self::Error>;

    /// Sign data with a private key (for system operations)
    ///
    /// **Security Note**: This is only used for system-level operations
    /// where the private key is provided externally. User private keys
    /// are NEVER stored in or accessible through this system.
    fn sign(
        &self,
        data: &[u8],
        private_key: &[u8],
        public_key: &[u8],
    ) -> Result<Vec<u8>, Self::Error>;

    /// Generate a new key pair (for demo/testing only)
    ///
    /// **Warning**: The private key returned should be handled with care.
    /// In production, private keys should be generated and stored in secure
    /// external systems.
    fn generate_key_pair(&self) -> Result<(Vec<u8>, Vec<u8>), Self::Error>;
}

// ============================================================================
// BLOCKCHAIN INTEGRATION
// ============================================================================

/// Trait for user system blockchain integration
///
/// Allows the user system to integrate with a blockchain backend
/// for storing user data, blocks, and transactions.
pub trait UserSystemBlockchain: Send + Sync {
    /// The type of user system
    type UserSystem: UserSystem;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Store user data on the blockchain
    fn store_user_on_blockchain(
        &self,
        user_system: &Self::UserSystem,
        user_id: &<Self::UserSystem as UserSystem>::UserId,
    ) -> Result<(), Self::Error>;

    /// Store a block on the blockchain
    fn store_block_on_blockchain(
        &self,
        user_system: &Self::UserSystem,
        block_id: &<Self::UserSystem as UserSystem>::BlockId,
    ) -> Result<(), Self::Error>;

    /// Verify user data from the blockchain
    fn verify_user_from_blockchain(
        &self,
        user_id: &[u8],
    ) -> Result<<Self::UserSystem as UserSystem>::User, Self::Error>;

    /// Synchronize user system with blockchain state
    fn sync_with_blockchain(&self, user_system: &mut Self::UserSystem) -> Result<(), Self::Error>;
}

// ============================================================================
// USER STORAGE BACKEND
// ============================================================================

/// Trait for user storage backend
///
/// Provides persistent storage for user data, allowing different
/// implementations (in-memory, database, blockchain, distributed, etc.)
pub trait UserStorageBackend: Send + Sync {
    /// The type of user
    type User: User;
    /// The type of user identifier
    type UserId: UserId;
    /// The type of block
    type Block: Block<UserId = Self::UserId>;
    /// The type of block identifier
    type BlockId: BlockId;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Store a user
    fn store_user(&mut self, user: Self::User) -> Result<(), Self::Error>;

    /// Retrieve a user by ID
    fn retrieve_user(&self, user_id: &Self::UserId) -> Result<Self::User, Self::Error>;

    /// Retrieve a user by public key
    fn retrieve_user_by_public_key(&self, public_key: &[u8]) -> Result<Self::User, Self::Error>;

    /// Store a block
    fn store_block(&mut self, block: Self::Block) -> Result<(), Self::Error>;

    /// Retrieve a block by ID
    fn retrieve_block(&self, block_id: &Self::BlockId) -> Result<Self::Block, Self::Error>;

    /// List all users
    fn list_users(&self) -> Result<Vec<Self::User>, Self::Error>;

    /// List all blocks owned by a user
    fn list_user_blocks(&self, user_id: &Self::UserId) -> Result<Vec<Self::BlockId>, Self::Error>;

    /// Flush all changes to persistent storage
    fn flush(&mut self) -> Result<(), Self::Error>;

    /// Clear all data
    fn clear(&mut self) -> Result<(), Self::Error>;
}

// ============================================================================
// USER INDEX
// ============================================================================

/// Trait for user index
///
/// Provides indexing capabilities for fast user and block lookups.
pub trait UserIndex: Send + Sync {
    /// The type of user identifier
    type UserId: UserId;
    /// The type of block identifier
    type BlockId: BlockId;

    /// Index a user by ID
    fn index_user(&mut self, user_id: Self::UserId);

    /// Index a user by public key
    fn index_user_by_public_key(&mut self, public_key: Vec<u8>, user_id: Self::UserId);

    /// Index a block by ID and owner
    fn index_block(&mut self, block_id: Self::BlockId, owner_id: Self::UserId);

    /// Lookup user ID by public key
    fn lookup_user_by_public_key(&self, public_key: &[u8]) -> Option<Self::UserId>;

    /// Lookup blocks by owner
    fn lookup_blocks_by_owner(&self, owner_id: &Self::UserId) -> Vec<Self::BlockId>;

    /// Check if a user exists
    fn contains_user(&self, user_id: &Self::UserId) -> bool;

    /// Check if a block exists
    fn contains_block(&self, block_id: &Self::BlockId) -> bool;

    /// Clear all indexes
    fn clear(&mut self);
}
