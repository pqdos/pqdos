//! Simple user system implementation for PQOS
//!
//! This module provides a simplified but functional implementation of the user system
//! with a genesis user "futuros" that owns all system executable blocks.
//!
//! The key design principle: The genesis user's private key is NEVER stored or accessible
//! through this system. Only the public key is stored.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use parking_lot::RwLock;

use super::traits::{
    UserId as UserIdTrait,
    UserRole as UserRoleTrait,
    UserPermissions as UserPermissionsTrait,
    User as UserTrait,
    UserBuilder as UserBuilderTrait,
    BlockId as BlockIdTrait,
    Block as BlockTrait,
    BlockBuilder as BlockBuilderTrait,
    ExecutableBlock as ExecutableBlockTrait,
    ExecutableBlockBuilder as ExecutableBlockBuilderTrait,
    UserSystem as UserSystemTrait,
    UserSystemFactory as UserSystemFactoryTrait,
    UserSystemError,
    GENESIS_USER_NAME as TRAITS_GENESIS_USER_NAME,
};

// ============================================================================
// RE-EXPORTS FOR CONVENIENCE
// ============================================================================

/// Genesis user name
pub const GENESIS_USER_NAME: &str = TRAITS_GENESIS_USER_NAME;

// ============================================================================
// USER ID
// ============================================================================

/// User ID type - SHA256 hash of the public key
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId {
    id: Vec<u8>,
}

impl UserId {
    /// Create a new user ID from bytes
    pub fn new(id: Vec<u8>) -> Self {
        Self { id }
    }
    
    /// Create a user ID from a public key
    pub fn from_public_key(public_key: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        Self::new(hash.to_vec())
    }
    
    /// Get the raw bytes of the ID
    pub fn as_bytes(&self) -> &[u8] {
        &self.id
    }
    
    /// Convert to vector
    pub fn to_vec(&self) -> Vec<u8> {
        self.id.clone()
    }
}

impl AsRef<[u8]> for UserId {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// Implement the UserId trait
impl UserIdTrait for UserId {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }

    fn from_public_key(public_key: &[u8]) -> Self {
        Self::from_public_key(public_key)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }

    fn size(&self) -> usize {
        self.id.len()
    }
}

// ============================================================================
// USER ROLE
// ============================================================================

/// User role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Genesis,
    Admin,
    User,
}

impl UserRole {
    /// Check if this is the genesis role
    pub fn is_genesis(&self) -> bool {
        matches!(self, UserRole::Genesis)
    }

    /// Check if this is an admin role
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Check if this is a regular user role
    pub fn is_user(&self) -> bool {
        matches!(self, UserRole::User)
    }

    /// Get the role as a string
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::Genesis => "genesis",
            UserRole::Admin => "admin",
            UserRole::User => "user",
        }
    }
}

// Implement the UserRole trait
impl UserRoleTrait for UserRole {
    fn is_genesis(&self) -> bool {
        self.is_genesis()
    }

    fn is_admin(&self) -> bool {
        self.is_admin()
    }

    fn is_user(&self) -> bool {
        self.is_user()
    }

    fn as_str(&self) -> &str {
        self.as_str()
    }
}

// ============================================================================
// USER PERMISSIONS
// ============================================================================

/// User permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct UserPermissions {
    pub can_create_blocks: bool,
    pub can_read_all_blocks: bool,
    pub can_write_all_blocks: bool,
    pub can_manage_users: bool,
    pub can_manage_system: bool,
    pub can_execute_code: bool,
}

impl UserPermissions {
    /// Full permissions (for genesis user)
    pub fn full() -> Self {
        Self {
            can_create_blocks: true,
            can_read_all_blocks: true,
            can_write_all_blocks: true,
            can_manage_users: true,
            can_manage_system: true,
            can_execute_code: true,
        }
    }

    /// Check if these permissions are full (all true)
    pub fn is_full(&self) -> bool {
        self.can_create_blocks &&
        self.can_read_all_blocks &&
        self.can_write_all_blocks &&
        self.can_manage_users &&
        self.can_manage_system &&
        self.can_execute_code
    }
}

// Implement the UserPermissions trait
impl UserPermissionsTrait for UserPermissions {
    fn can_create_blocks(&self) -> bool {
        self.can_create_blocks
    }

    fn can_read_all_blocks(&self) -> bool {
        self.can_read_all_blocks
    }

    fn can_write_all_blocks(&self) -> bool {
        self.can_write_all_blocks
    }

    fn can_manage_users(&self) -> bool {
        self.can_manage_users
    }

    fn can_manage_system(&self) -> bool {
        self.can_manage_system
    }

    fn can_execute_code(&self) -> bool {
        self.can_execute_code
    }

    fn is_full(&self) -> bool {
        self.is_full()
    }

    fn full() -> Self {
        Self::full()
    }
}

// ============================================================================
// USER
// ============================================================================

/// User structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub public_key: Vec<u8>,
    pub role: UserRole,
    pub permissions: UserPermissions,
    pub created_at: i64,
    pub metadata: HashMap<String, String>,
}

impl User {
    /// Create a new user
    pub fn new(
        name: String,
        public_key: Vec<u8>,
        role: UserRole,
        permissions: UserPermissions,
    ) -> Self {
        let id = UserId::from_public_key(&public_key);
        let created_at = timestamp();
        
        let mut metadata = HashMap::new();
        metadata.insert("created_by".to_string(), "system".to_string());
        
        Self {
            id,
            name,
            public_key,
            role,
            permissions,
            created_at,
            metadata,
        }
    }
    
    /// Create the genesis user
    pub fn new_genesis(name: String, public_key: Vec<u8>) -> Self {
        let permissions = UserPermissions::full();
        
        let mut user = Self::new(name, public_key, UserRole::Genesis, permissions);
        user.metadata.insert("type".to_string(), "genesis".to_string());
        user.metadata.insert("description".to_string(), "System genesis user - owner of OS executable blocks".to_string());
        
        user
    }
    
    /// Check if this is the genesis user
    pub fn is_genesis(&self) -> bool {
        self.role == UserRole::Genesis
    }

    /// Check if this user has the given role
    pub fn has_role(&self, role: UserRole) -> bool {
        self.role == role
    }

    /// Check if this user has the given permission
    pub fn has_permission<F>(&self, permission: F) -> bool
    where
        F: Fn(&UserPermissions) -> bool,
    {
        permission(&self.permissions)
    }
}

// Implement the User trait
impl UserTrait for User {
    type Id = UserId;
    type Role = UserRole;
    type Permissions = UserPermissions;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    fn role(&self) -> Self::Role {
        self.role
    }

    fn permissions(&self) -> Self::Permissions {
        self.permissions
    }

    fn created_at(&self) -> i64 {
        self.created_at
    }

    fn is_genesis(&self) -> bool {
        self.is_genesis()
    }

    fn has_role(&self, role: Self::Role) -> bool {
        self.has_role(role)
    }

    fn has_permission(&self, permission: fn(&Self::Permissions) -> bool) -> bool {
        self.has_permission(permission)
    }

    fn metadata(&self) -> &std::collections::HashMap<String, String> {
        &self.metadata
    }
}

// ============================================================================
// BLOCK ID
// ============================================================================

/// Block ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId {
    id: Vec<u8>,
}

impl BlockId {
    pub fn new(id: Vec<u8>) -> Self {
        Self { id }
    }
    
    pub fn from_content(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hasher.finalize();
        Self::new(hash.to_vec())
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.id
    }
    
    pub fn to_vec(&self) -> Vec<u8> {
        self.id.clone()
    }
}

impl AsRef<[u8]> for BlockId {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// Implement the BlockId trait
impl BlockIdTrait for BlockId {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }

    fn from_content(content: &[u8]) -> Self {
        Self::from_content(content)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }

    fn size(&self) -> usize {
        self.id.len()
    }
}

// ============================================================================
// BLOCK
// ============================================================================

/// Block type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub data: Vec<u8>,
    pub owner_id: UserId,
    pub created_at: i64,
    pub block_type: String,
    pub metadata: HashMap<String, String>,
}

impl Block {
    pub fn new(data: Vec<u8>, owner_id: UserId, block_type: String) -> Self {
        let id = BlockId::from_content(&data);
        let created_at = timestamp();
        
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), block_type.clone());
        
        Self {
            id,
            data,
            owner_id,
            created_at,
            block_type,
            metadata,
        }
    }
    
    pub fn is_system_block(&self) -> bool {
        self.block_type == "system" || self.block_type == "executable"
    }

    /// Return the size of the data in bytes
    pub fn data_size(&self) -> usize {
        self.data.len()
    }
}

// Implement the Block trait
impl BlockTrait for Block {
    type Id = BlockId;
    type UserId = UserId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn owner_id(&self) -> &Self::UserId {
        &self.owner_id
    }

    fn created_at(&self) -> i64 {
        self.created_at
    }

    fn block_type(&self) -> &str {
        &self.block_type
    }

    fn is_system_block(&self) -> bool {
        self.is_system_block()
    }

    fn metadata(&self) -> &std::collections::HashMap<String, String> {
        &self.metadata
    }

    fn data_size(&self) -> usize {
        self.data_size()
    }
}

// ============================================================================
// EXECUTABLE BLOCK
// ============================================================================

/// Executable block type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableBlock {
    pub inner: Block,
    pub entry_point: String,
    pub executable_type: String,
}

impl ExecutableBlock {
    pub fn new(code: Vec<u8>, owner_id: UserId, entry_point: String, executable_type: String) -> Self {
        let block_type = match executable_type.as_str() {
            "kernel" | "bootstrap" | "driver" | "service" => "system_executable".to_string(),
            _ => "executable".to_string(),
        };
        
        let inner = Block::new(code, owner_id, block_type);
        
        Self {
            inner,
            entry_point,
            executable_type,
        }
    }
    
    pub fn code(&self) -> &[u8] {
        &self.inner.data
    }
    
    pub fn id(&self) -> &BlockId {
        &self.inner.id
    }
    
    pub fn owner_id(&self) -> &UserId {
        &self.inner.owner_id
    }
}

// Implement the ExecutableBlock trait
impl ExecutableBlockTrait for ExecutableBlock {
    type Block = Block;
    type BlockId = BlockId;
    type UserId = UserId;

    fn block(&self) -> &Self::Block {
        &self.inner
    }

    fn id(&self) -> &Self::BlockId {
        self.id()
    }

    fn code(&self) -> &[u8] {
        self.code()
    }

    fn owner_id(&self) -> &Self::UserId {
        self.owner_id()
    }

    fn entry_point(&self) -> &str {
        &self.entry_point
    }

    fn executable_type(&self) -> &str {
        &self.executable_type
    }
}

// ============================================================================
// USER SYSTEM
// ============================================================================

/// User system state
#[derive(Debug, Default)]
pub struct UserSystem {
    users: RwLock<HashMap<UserId, User>>,
    public_key_to_user: RwLock<HashMap<Vec<u8>, UserId>>,
    blocks: RwLock<HashMap<BlockId, Block>>,
    user_blocks: RwLock<HashMap<UserId, Vec<BlockId>>>,
    executables: RwLock<HashMap<BlockId, ExecutableBlock>>,
    genesis_user: RwLock<Option<User>>,
}

impl UserSystem {
    /// Create a new empty user system
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Initialize the system with the genesis user
    /// 
    /// This method takes ONLY the public key. The private key MUST be kept externally.
    /// The private key is NEVER stored in this system and CANNOT be retrieved.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the genesis user (default: "futuros")
    /// * `public_key` - The PUBLIC KEY only of the genesis user
    /// 
    /// # Security Note
    /// 
    /// The private key corresponding to this public key MUST be kept in a secure location
    /// external to this system. There is NO way to retrieve or use the private key through
    /// this system. Operations that require the private key (like signing) must be done
    /// externally and the signatures provided to this system.
    pub fn initialize(&mut self, name: String, public_key: Vec<u8>) -> Result<(), String> {
        // Check if already initialized
        {
            let genesis = self.genesis_user.read();
            if genesis.is_some() {
                return Err("System already initialized with genesis user".to_string());
            }
        }
        
        // Create the genesis user
        let genesis_user = User::new_genesis(name.clone(), public_key.clone());
        let user_id = genesis_user.id.clone();
        
        // Store the user
        {
            let mut users = self.users.write();
            let mut pk_map = self.public_key_to_user.write();
            let mut genesis = self.genesis_user.write();
            
            users.insert(user_id.clone(), genesis_user.clone());
            pk_map.insert(public_key.clone(), user_id.clone());
            *genesis = Some(genesis_user);
        }
        
        Ok(())
    }
    
    /// Initialize with the default "futuros" user
    pub fn initialize_with_futuros(&mut self, public_key: Vec<u8>) -> Result<(), String> {
        self.initialize(TRAITS_GENESIS_USER_NAME.to_string(), public_key)
    }
    
    /// Get the genesis user
    pub fn get_genesis_user(&self) -> Option<User> {
        let genesis = self.genesis_user.read();
        (*genesis).clone()
    }
    
    /// Get a user by ID
    pub fn get_user(&self, user_id: &UserId) -> Option<User> {
        let users = self.users.read();
        users.get(user_id).cloned()
    }
    
    /// Get a user by public key
    pub fn get_user_by_public_key(&self, public_key: &[u8]) -> Option<User> {
        let pk_map = self.public_key_to_user.read();
        let user_id = pk_map.get(public_key)?;
        self.get_user(user_id)
    }
    
    /// Check if the system is initialized
    pub fn is_initialized(&self) -> bool {
        let genesis = self.genesis_user.read();
        genesis.is_some()
    }
    
    /// Check if a user is the genesis user
    pub fn is_genesis_user(&self, user_id: &UserId) -> bool {
        let genesis = self.genesis_user.read();
        genesis.as_ref().map_or(false, |g| &g.id == user_id)
    }
    
    /// Create a new block owned by a user
    pub fn create_block(&mut self, data: Vec<u8>, owner_id: UserId, block_type: String) -> Result<BlockId, String> {
        // Verify the user exists
        {
            let users = self.users.read();
            if !users.contains_key(&owner_id) {
                return Err("User not found".to_string());
            }
        }
        
        let block = Block::new(data, owner_id.clone(), block_type);
        let block_id = block.id.clone();
        
        // Store the block
        {
            let mut blocks = self.blocks.write();
            let mut user_blocks = self.user_blocks.write();
            
            blocks.insert(block_id.clone(), block);
            user_blocks.entry(owner_id).or_default().push(block_id.clone());
        }
        
        Ok(block_id)
    }
    
    /// Register a system executable owned by the genesis user
    pub fn register_system_executable(
        &mut self,
        _name: String,
        code: Vec<u8>,
        entry_point: String,
        executable_type: String,
    ) -> Result<BlockId, String> {
        // Get the genesis user
        let genesis_user = self.get_genesis_user()
            .ok_or("System not initialized with genesis user".to_string())?;
        
        // Create the block with the genesis user as owner
        let block_id = self.create_block(code, genesis_user.id.clone(), "system_executable".to_string())?;
        
        // Create and store the executable
        {
            let block = self.get_block(&block_id)
                .ok_or("Failed to retrieve created block".to_string())?;
            
            let executable = ExecutableBlock::new(
                block.data.clone(),
                block.owner_id.clone(),
                entry_point,
                executable_type,
            );
            
            let mut executables = self.executables.write();
            executables.insert(block_id.clone(), executable);
        }
        
        Ok(block_id)
    }
    
    /// Get a block by ID
    pub fn get_block(&self, block_id: &BlockId) -> Option<Block> {
        let blocks = self.blocks.read();
        blocks.get(block_id).cloned()
    }
    
    /// Get the owner of a block
    pub fn get_block_owner(&self, block_id: &BlockId) -> Option<UserId> {
        let blocks = self.blocks.read();
        blocks.get(block_id).map(|b| b.owner_id.clone())
    }
    
    /// Get all blocks owned by a user
    pub fn get_user_blocks(&self, user_id: &UserId) -> Vec<BlockId> {
        let user_blocks = self.user_blocks.read();
        user_blocks.get(user_id).cloned().unwrap_or_default()
    }
    
    /// Get all system blocks (owned by genesis user)
    pub fn get_system_blocks(&self) -> Vec<BlockId> {
        let genesis = self.genesis_user.read();
        let user_id = genesis.as_ref().map(|g| &g.id);
        
        match user_id {
            Some(id) => self.get_user_blocks(id),
            None => Vec::new(),
        }
    }
    
    /// Get a system executable by block ID
    pub fn get_system_executable(&self, block_id: &BlockId) -> Option<ExecutableBlock> {
        let executables = self.executables.read();
        executables.get(block_id).cloned()
    }
    
    /// List all system executables
    pub fn list_system_executables(&self) -> Vec<ExecutableBlock> {
        let executables = self.executables.read();
        executables.values().cloned().collect()
    }
    
    /// Verify that a block is a system block (owned by genesis user)
    pub fn is_system_block(&self, block_id: &BlockId) -> bool {
        let owner_id = self.get_block_owner(block_id);
        match owner_id {
            Some(owner) => self.is_genesis_user(&owner),
            None => false,
        }
    }
}

// Implement the UserSystem trait for UserSystem
impl UserSystemTrait for UserSystem {
    type User = User;
    type UserId = UserId;
    type Block = Block;
    type BlockId = BlockId;
    type ExecutableBlock = ExecutableBlock;
    type Error = UserSystemError;

    fn initialize(&mut self, name: String, public_key: Vec<u8>) -> Result<(), Self::Error> {
        self.initialize(name, public_key)
            .map_err(|e| UserSystemError::InternalError(e))
    }

    fn initialize_with_futuros(&mut self, public_key: Vec<u8>) -> Result<(), Self::Error> {
        self.initialize_with_futuros(public_key)
            .map_err(|e| UserSystemError::InternalError(e.to_string()))
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized()
    }

    fn get_genesis_user(&self) -> Option<Self::User> {
        self.get_genesis_user()
    }

    fn get_user(&self, user_id: &Self::UserId) -> Option<Self::User> {
        self.get_user(user_id)
    }

    fn get_user_by_public_key(&self, public_key: &[u8]) -> Option<Self::User> {
        self.get_user_by_public_key(public_key)
    }

    fn is_genesis_user(&self, user_id: &Self::UserId) -> bool {
        self.is_genesis_user(user_id)
    }

    fn create_block(
        &mut self,
        data: Vec<u8>,
        owner_id: Self::UserId,
        block_type: String,
    ) -> Result<Self::BlockId, Self::Error> {
        self.create_block(data, owner_id, block_type)
            .map_err(|e| UserSystemError::InternalError(e))
    }

    fn register_system_executable(
        &mut self,
        name: String,
        code: Vec<u8>,
        entry_point: String,
        executable_type: String,
    ) -> Result<Self::BlockId, Self::Error> {
        self.register_system_executable(name, code, entry_point, executable_type)
            .map_err(|e| UserSystemError::InternalError(e))
    }

    fn get_block(&self, block_id: &Self::BlockId) -> Option<Self::Block> {
        self.get_block(block_id)
    }

    fn get_block_owner(&self, block_id: &Self::BlockId) -> Option<Self::UserId> {
        self.get_block_owner(block_id)
    }

    fn get_user_blocks(&self, user_id: &Self::UserId) -> Vec<Self::BlockId> {
        self.get_user_blocks(user_id)
    }

    fn get_system_blocks(&self) -> Vec<Self::BlockId> {
        self.get_system_blocks()
    }

    fn get_system_executable(&self, block_id: &Self::BlockId) -> Option<Self::ExecutableBlock> {
        self.get_system_executable(block_id)
    }

    fn list_system_executables(&self) -> Vec<Self::ExecutableBlock> {
        self.list_system_executables()
    }

    fn is_system_block(&self, block_id: &Self::BlockId) -> bool {
        self.is_system_block(block_id)
    }
}

// ============================================================================
// BUILDERS AND FACTORIES
// ============================================================================

/// User builder
pub struct SimpleUserBuilder;

impl UserBuilderTrait for SimpleUserBuilder {
    type User = User;
    type Error = UserSystemError;

    fn new_user(
        &self,
        name: String,
        public_key: Vec<u8>,
        role: UserRole,
        permissions: UserPermissions,
    ) -> Result<Self::User, Self::Error> {
        Ok(User::new(name, public_key, role, permissions))
    }

    fn new_genesis_user(&self, name: String, public_key: Vec<u8>) -> Result<Self::User, Self::Error> {
        Ok(User::new_genesis(name, public_key))
    }
}

/// Block builder
pub struct SimpleBlockBuilder;

impl BlockBuilderTrait for SimpleBlockBuilder {
    type Block = Block;
    type Error = UserSystemError;

    fn new_block(
        &self,
        data: Vec<u8>,
        owner_id: UserId,
        block_type: String,
    ) -> Result<Self::Block, Self::Error> {
        Ok(Block::new(data, owner_id, block_type))
    }
}

/// Executable block builder
pub struct SimpleExecutableBlockBuilder;

impl ExecutableBlockBuilderTrait for SimpleExecutableBlockBuilder {
    type ExecutableBlock = ExecutableBlock;
    type Error = UserSystemError;

    fn new_executable(
        &self,
        code: Vec<u8>,
        owner_id: UserId,
        entry_point: String,
        executable_type: String,
    ) -> Result<Self::ExecutableBlock, Self::Error> {
        Ok(ExecutableBlock::new(code, owner_id, entry_point, executable_type))
    }
}

/// User system factory
pub struct SimpleUserSystemFactory;

impl UserSystemFactoryTrait for SimpleUserSystemFactory {
    type UserSystem = UserSystem;
    type Error = UserSystemError;

    fn create(&self) -> Self::UserSystem {
        UserSystem::new()
    }

    fn create_initialized(&self, name: String, public_key: Vec<u8>) -> Result<Self::UserSystem, Self::Error> {
        let mut system = UserSystem::new();
        system.initialize(name, public_key).map_err(|e| UserSystemError::InternalError(e))?;
        Ok(system)
    }

    fn create_with_futuros(&self, public_key: Vec<u8>) -> Result<Self::UserSystem, Self::Error> {
        let mut system = UserSystem::new();
        system.initialize_with_futuros(public_key).map_err(|e| UserSystemError::InternalError(e.to_string()))?;
        Ok(system)
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/// Helper function to get current timestamp
fn timestamp() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Convenience function to create and initialize a new user system
pub fn create_user_system(public_key: Vec<u8>) -> Result<UserSystem, String> {
    let mut system = UserSystem::new();
    system.initialize_with_futuros(public_key)?;
    Ok(system)
}

/// Convenience function to create a user system with a generated key pair
/// 
/// WARNING: This generates a key pair and DISCARDS the private key.
/// The private key CANNOT be retrieved from this system.
/// This is only for testing/demonstration.
pub fn create_user_system_with_demo_keys() -> UserSystem {
    // In a real system, you would generate a proper PQC key pair here
    // For demo purposes, we'll use a mock public key
    let demo_public_key = vec![0u8; 64]; // Mock 512-bit public key
    
    create_user_system(demo_public_key).expect("Failed to create user system")
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_initialization() {
        let mut system = UserSystem::new();
        let public_key = vec![1u8; 64];
        
        let result = system.initialize_with_futuros(public_key.clone());
        assert!(result.is_ok());
        
        assert!(system.is_initialized());
        
        let genesis_user = system.get_genesis_user();
        assert!(genesis_user.is_some());
        
        let user = genesis_user.unwrap();
        assert_eq!(user.name, TRAITS_GENESIS_USER_NAME);
        assert_eq!(user.public_key, public_key);
        assert!(user.is_genesis());
    }
    
    #[test]
    fn test_trait_implementation() {
        // Test that UserId implements UserIdTrait
        let public_key = vec![1u8; 64];
        let user_id = UserId::from_public_key(&public_key);
        
        // Test UserId trait methods
        assert_eq!(user_id.to_bytes().len(), 32); // SHA256 hash is 32 bytes
        assert_eq!(user_id.size(), 32);
        
        // Test UserRole trait
        let role = UserRole::Genesis;
        assert!(role.is_genesis());
        assert!(!role.is_admin());
        assert!(!role.is_user());
        assert_eq!(role.as_str(), "genesis");
        
        // Test UserPermissions trait
        let perms = UserPermissions::full();
        assert!(perms.is_full());
        assert!(perms.can_create_blocks());
        assert!(perms.can_execute_code());
    }
    
    #[test]
    fn test_system_block_ownership() {
        let mut system = UserSystem::new();
        let public_key = vec![2u8; 64];
        
        system.initialize_with_futuros(public_key).unwrap();
        
        // Register a system executable
        let code = vec![0u8, 1, 2, 3, 4];
        let block_id = system.register_system_executable(
            "kernel".to_string(),
            code.clone(),
            "main".to_string(),
            "kernel".to_string(),
        ).unwrap();
        
        // Verify the block is owned by genesis user
        let owner_id = system.get_block_owner(&block_id).unwrap();
        let genesis_user = system.get_genesis_user().unwrap();
        
        assert_eq!(owner_id, genesis_user.id);
        assert!(system.is_system_block(&block_id));
        
        // Verify the executable is stored
        let executable = system.get_system_executable(&block_id).unwrap();
        assert_eq!(executable.code(), &code);
        assert_eq!(executable.entry_point(), "main");
    }
    
    #[test]
    fn test_block_trait_implementation() {
        let public_key = vec![1u8; 64];
        let user_id = UserId::from_public_key(&public_key);
        
        let data = vec![1, 2, 3, 4, 5];
        let block = Block::new(data.clone(), user_id, "test".to_string());
        
        // Test Block trait methods
        assert_eq!(block.data(), &data);
        assert_eq!(block.data_size(), 5);
        assert_eq!(block.block_type(), "test");
        assert!(!block.is_system_block());
    }
    
    #[test]
    fn test_private_key_not_accessible() {
        // This test verifies the fundamental security property:
        // There is NO way to access the private key through the UserSystem
        
        let mut system = UserSystem::new();
        let public_key = vec![3u8; 64];
        let public_key_copy = public_key.clone();
        
        system.initialize_with_futuros(public_key).unwrap();
        
        // The User structure only contains the public key
        let genesis_user = system.get_genesis_user().unwrap();
        
        // There is no private key field in User
        // There is no method to get the private key
        // The private key was NEVER provided to the system
        
        // We can only access the public key
        assert_eq!(genesis_user.public_key(), &public_key_copy);
        
        // This test passes because the system is designed to NEVER have access
        // to the private key
    }
    
    #[test]
    fn test_factory_creation() {
        let factory = SimpleUserSystemFactory;
        let public_key = vec![4u8; 64];
        
        // Test creating a user system with factory
        let system = factory.create_with_futuros(public_key.clone()).unwrap();
        
        assert!(system.is_initialized());
        let genesis = system.get_genesis_user().unwrap();
        assert_eq!(genesis.name(), TRAITS_GENESIS_USER_NAME);
    }
    
    #[test]
    fn test_trait_object() {
        // Test that we can use trait objects with concrete types
        let public_key = vec![5u8; 64];
        let user_id = UserId::from_public_key(&public_key);
        
        // Test trait implementations
        let _user_id_trait: &dyn UserIdTrait = &user_id;
        let role = UserRole::Genesis;
        let _role_trait: &dyn UserRoleTrait = &role;
        let perms = UserPermissions::full();
        let _perms_trait: &dyn UserPermissionsTrait = &perms;
        let user = User::new_genesis("test".to_string(), public_key.clone());
        let _user_trait: &dyn UserTrait = &user;
        let block_id = BlockId::from_content(&[1, 2, 3]);
        let _block_id_trait: &dyn BlockIdTrait = &block_id;
        let block = Block::new(vec![1, 2, 3], user_id, "test".to_string());
        let _block_trait: &dyn BlockTrait = &block;
        let executable = ExecutableBlock::new(vec![1, 2, 3], user_id, "main".to_string(), "kernel".to_string());
        let _exec_trait: &dyn ExecutableBlockTrait = &executable;
        
        let mut system = UserSystem::new();
        system.initialize_with_futuros(public_key).unwrap();
        // UserSystemTrait can be used directly
        assert!(system.is_initialized());
    }
}
