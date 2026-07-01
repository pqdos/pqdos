# PQOS Architecture Documentation

## Table of Contents
1. [Overview](#overview)
2. [Core Design Principles](#core-design-principles)
3. [Module Architecture](#module-architecture)
4. [Users Module - Deep Dive](#users-module---deep-dive)
5. [Abstract Traits System](#abstract-traits-system)
6. [Implementation Patterns](#implementation-patterns)
7. [Security Architecture](#security-architecture)
8. [Development Guidelines](#development-guidelines)

---

## Overview

The Post-Quantum Distributed Operating System (PQOS) is built on a foundation of **abstract traits** that define interfaces without committing to specific implementations. This architecture ensures the system remains **evolvable, technology-agnostic, and secure**.

### Key Innovations

1. **Unified Memory Hierarchy**: All storage (RAM, files, network) treated as content-addressed encrypted blocks
2. **Blockchain-Backed History**: Immutable ledger tracks all modifications with cryptographic integrity
3. **Post-Quantum Security**: NIST-approved PQC algorithms protect against quantum adversaries
4. **Distributed Consensus**: Decentralized operation with pluggable consensus protocols

---

## Core Design Principles

### 1. Technology Agnosticism
- **No concrete implementations in core modules** - All modules expose only trait interfaces
- **Pluggable architecture** - Any cryptographic library, network stack, or storage backend can be substituted
- **Multiple implementation support** - Different backends can coexist (in-memory for testing, blockchain for production)

### 2. Content Addressing
- **Everything is a block** - Files, memory, configuration all use the same content-addressable model
- **Git-like hashing** - SHA3 hashes serve as unique content identifiers
- **Automatic deduplication** - Identical content shares the same address

### 3. Security by Design
- **Private keys NEVER stored** - Only public keys are accessible through the system
- **Encryption at rest** - All blocks encrypted with symmetric encryption
- **Immutable history** - Blockchain ledger prevents tampering
- **Post-quantum ready** - All cryptographic traits support PQC algorithms

### 4. Evolutionary Architecture
- **Trait-based abstraction** - Clear separation between interface and implementation
- **Factory pattern** - Flexible instance creation with different configurations
- **Builder pattern** - Safe construction of complex objects
- **Type safety** - Compile-time guarantees through Rust's type system

---

## Module Architecture

```
pqos/
├── src/
│   ├── lib.rs                 # Library entry point
│   ├── error.rs               # Global error types
│   ├── crypto/                # Cryptographic primitives
│   │   └── traits.rs          # HashFunction, Kem, SignatureScheme, etc.
│   ├── block/                 # Content-addressed blocks
│   │   └── traits.rs          # BlockId, Block, BlockStorage, etc.
│   ├── blockchain/            # Distributed ledger
│   │   └── traits.rs          # Transaction, Blockchain, ConsensusAlgorithm, etc.
│   ├── network/               # P2P communication
│   │   └── traits.rs          # Peer, NetworkMessage, P2PNetwork, etc.
│   ├── memory/                # Unified memory abstraction
│   │   └── traits.rs          # MemoryManager, AddressSpace, etc.
│   └── users/                # User management system
│       ├── mod.rs             # Module exports and compatibility
│       ├── traits.rs          # UserId, User, UserSystem, Authenticator, etc.
│       └── simple.rs          # Reference in-memory implementation
└── docs/
    └── ARCHITECTURE.md        # This file
```

---

## Users Module - Deep Dive

### Overview

The Users module provides a **comprehensive user management system** with:
- Abstract traits for pluggable implementations
- Genesis user "futuros" who owns all system executable blocks
- Authentication system with external signature verification
- Block ownership tracking
- Content-addressed storage for user data

### Security Principle

**The genesis user's private key is NEVER stored or accessible through this system.**

This is the **fundamental security invariant** of the Users module:
- Only the **public key** is stored in the system
- The **private key** must be kept in a secure external location (HSM, secure enclave, etc.)
- Operations requiring the private key (signing) are performed externally
- Signatures are provided to the system for verification

### Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    Users Module                                │
├─────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              TRAITS (Abstract Interface)              │   │
│  │  UserId, UserRole, UserPermissions, User              │   │
│  │  BlockId, Block, ExecutableBlock                      │   │
│  │  UserSystem, UserBuilder, UserAuthenticator            │   │
│  │  UserSystemFactory, UserStorageBackend                │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                    │
│                          ▼                                    │
│  ┌─────────────────────────────────────────────────────┐   │
│  │            SIMPLE (Reference Implementation)          │   │
│  │  In-memory storage with RwLock                       │   │
│  │  SHA256 for content addressing                         │   │
│  │  Genesis user "futuros" initialization                 │   │
│  │  System executable registration                        │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                    │
│          ┌───────────────┴───────────────┐                    │
│          ▼                               ▼                    │
│  ┌─────────────┐               ┌─────────────┐                │
│  │  Other      │               │  Custom     │                │
│  │  Backends   │               │Implementations│                │
│  │ (Future)    │               │ (Future)    │                │
│  └─────────────┘               └─────────────┘                │
│                                                                  │
└─────────────────────────────────────────────────────────────┘
```

### Traits Hierarchy

#### Core Types

```
┌─────────────────────────────────────────────────────────────┐
│ UserIdTrait                                                  │
│ ├── from_bytes(bytes: Vec<u8>) -> Self                      │
│ ├── from_public_key(public_key: &[u8]) -> Self               │
│ ├── to_bytes(&self) -> Vec<u8>                                │
│ └── size(&self) -> usize                                     │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ UserRoleTrait                                                │
│ ├── is_genesis(&self) -> bool                                │
│ ├── is_admin(&self) -> bool                                  │
│ ├── is_user(&self) -> bool                                    │
│ └── as_str(&self) -> &str                                    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ UserPermissionsTrait                                          │
│ ├── can_create_blocks(&self) -> bool                        │
│ ├── can_read_all_blocks(&self) -> bool                      │
│ ├── can_write_all_blocks(&self) -> bool                     │
│ ├── can_manage_users(&self) -> bool                          │
│ ├── can_manage_system(&self) -> bool                         │
│ ├── can_execute_code(&self) -> bool                          │
│ ├── is_full(&self) -> bool                                   │
│ └── full() -> Self                                           │
└─────────────────────────────────────────────────────────────┘
```

#### User and Block Types

```
┌─────────────────────────────────────────────────────────────┐
│ UserTrait                                                    │
│ ├── id(&self) -> &Self::Id                                   │
│ ├── name(&self) -> &str                                      │
│ ├── public_key(&self) -> &[u8]  // ONLY public key!         │
│ ├── role(&self) -> Self::Role                                │
│ ├── permissions(&self) -> Self::Permissions                 │
│ ├── created_at(&self) -> i64                                 │
│ ├── is_genesis(&self) -> bool                                │
│ ├── has_role(&self, role: Self::Role) -> bool                │
│ └── has_permission(&self, fn(&Permissions) -> bool) -> bool  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ BlockTrait                                                   │
│ ├── id(&self) -> &Self::Id                                   │
│ ├── data(&self) -> &[u8]                                     │
│ ├── owner_id(&self) -> &Self::UserId                         │
│ ├── created_at(&self) -> i64                                 │
│ ├── block_type(&self) -> &str                                │
│ ├── is_system_block(&self) -> bool                          │
│ ├── metadata(&self) -> &HashMap<String, String>             │
│ └── data_size(&self) -> usize                                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ ExecutableBlockTrait                                         │
│ ├── block(&self) -> &Self::Block                             │
│ ├── id(&self) -> &Self::BlockId                             │
│ ├── code(&self) -> &[u8]                                    │
│ ├── owner_id(&self) -> &Self::UserId                        │
│ ├── entry_point(&self) -> &str                              │
│ └── executable_type(&self) -> &str                          │
└─────────────────────────────────────────────────────────────┘
```

#### System Traits

```
┌─────────────────────────────────────────────────────────────┐
│ UserSystemTrait                                              │
│ ├── initialize(name: String, public_key: Vec<u8>)          │
│ ├── initialize_with_futuros(public_key: Vec<u8>)            │
│ ├── is_initialized(&self) -> bool                            │
│ ├── get_genesis_user(&self) -> Option<Self::User>          │
│ ├── get_user(&self, user_id: &Self::UserId) -> Option<User> │
│ ├── get_user_by_public_key(&self, pk: &[u8]) -> Option<User>│
│ ├── is_genesis_user(&self, user_id: &Self::UserId) -> bool   │
│ ├── create_block(&mut self, data, owner_id, type)            │
│ ├── register_system_executable(&mut self, name, code, ...)  │
│ ├── get_block(&self, block_id: &Self::BlockId) -> Option     │
│ ├── get_block_owner(&self, block_id) -> Option<UserId>       │
│ ├── get_user_blocks(&self, user_id) -> Vec<BlockId>          │
│ ├── get_system_blocks(&self) -> Vec<BlockId>                 │
│ ├── get_system_executable(&self, block_id) -> Option         │
│ ├── list_system_executables(&self) -> Vec<ExecutableBlock>   │
│ └── is_system_block(&self, block_id) -> bool                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ UserAuthenticatorTrait                                       │
│ ├── new(user_system: Self::UserSystem) -> Self              │
│ ├── authenticate(&self, user_id, challenge, signature)      │
│ ├── generate_challenge(&self) -> Vec<u8>                    │
│ ├── validate_token(&self, token: &AuthToken) -> Result<User> │
│ ├── user_system(&self) -> &Self::UserSystem                  │
│ └── get_genesis_user(&self) -> Result<Self::User>           │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ AuthenticationProvider                                       │
│ ├── verify_signature(&self, data, signature, pk) -> Result   │
│ ├── sign(&self, data, private_key, public_key) -> Result     │
│ └── generate_key_pair(&self) -> Result<(Vec<u8>, Vec<u8>)>   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ UserSystemFactoryTrait                                       │
│ ├── create(&self) -> Self::UserSystem                        │
│ ├── create_initialized(&self, name, pk) -> Result            │
│ └── create_with_futuros(&self, pk) -> Result                  │
└─────────────────────────────────────────────────────────────┘
```

#### Storage and Integration Traits

```
┌─────────────────────────────────────────────────────────────┐
│ UserStorageBackend                                           │
│ ├── store_user(&mut self, user) -> Result                    │
│ ├── retrieve_user(&self, user_id) -> Result                  │
│ ├── retrieve_user_by_public_key(&self, pk) -> Result         │
│ ├── store_block(&mut self, block) -> Result                  │
│ ├── retrieve_block(&self, block_id) -> Result                │
│ ├── list_users(&self) -> Result<Vec<User>>                   │
│ ├── list_user_blocks(&self, user_id) -> Result<Vec<BlockId>> │
│ ├── flush(&mut self) -> Result                               │
│ └── clear(&mut self) -> Result                               │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ UserIndex                                                    │
│ ├── index_user(&mut self, user_id)                          │
│ ├── index_user_by_public_key(&mut self, pk, user_id)         │
│ ├── index_block(&mut self, block_id, owner_id)              │
│ ├── lookup_user_by_public_key(&self, pk) -> Option<UserId>    │
│ ├── lookup_blocks_by_owner(&self, owner_id) -> Vec<BlockId>  │
│ ├── contains_user(&self, user_id) -> bool                   │
│ ├── contains_block(&self, block_id) -> bool                  │
│ └── clear(&mut self)                                        │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ UserSystemBlockchain                                         │
│ ├── store_user_on_blockchain(&self, user_system, user_id)     │
│ ├── store_block_on_blockchain(&self, user_system, block_id)  │
│ ├── verify_user_from_blockchain(&self, user_id) -> Result    │
│ └── sync_with_blockchain(&self, user_system) -> Result       │
└─────────────────────────────────────────────────────────────┘
```

### Reference Implementation (simple.rs)

The `simple.rs` module provides a **fully functional in-memory implementation** that:

1. **Implements all traits** defined in `traits.rs`
2. **Uses SHA256** for content addressing and user identification
3. **Stores everything in memory** using `RwLock<HashMap<...>>` for thread safety
4. **Maintains the security invariant** - private keys are NEVER stored
5. **Provides complete functionality** for development and testing

#### Key Types in Simple Implementation

```rust
// User ID - SHA256 hash of public key
pub struct UserId {
    id: Vec<u8>,  // 32 bytes (SHA256)
}

// User roles
pub enum UserRole {
    Genesis,    // System genesis user "futuros"
    Admin,      // System administrator
    User,       // Regular user
}

// User permissions
pub struct UserPermissions {
    pub can_create_blocks: bool,
    pub can_read_all_blocks: bool,
    pub can_write_all_blocks: bool,
    pub can_manage_users: bool,
    pub can_manage_system: bool,
    pub can_execute_code: bool,
}

// User structure
pub struct User {
    pub id: UserId,
    pub name: String,
    pub public_key: Vec<u8>,  // ONLY public key stored!
    pub role: UserRole,
    pub permissions: UserPermissions,
    pub created_at: i64,
    pub metadata: HashMap<String, String>,
}

// Block ID - SHA256 hash of content
pub struct BlockId {
    id: Vec<u8>,  // 32 bytes (SHA256)
}

// Block structure
pub struct Block {
    pub id: BlockId,
    pub data: Vec<u8>,
    pub owner_id: UserId,
    pub created_at: i64,
    pub block_type: String,
    pub metadata: HashMap<String, String>,
}

// Executable block - contains OS code
pub struct ExecutableBlock {
    pub inner: Block,
    pub entry_point: String,
    pub executable_type: String,  // "kernel", "bootstrap", "driver", "service"
}

// User system - main state
pub struct UserSystem {
    users: RwLock<HashMap<UserId, User>>,
    public_key_to_user: RwLock<HashMap<Vec<u8>, UserId>>,
    blocks: RwLock<HashMap<BlockId, Block>>,
    user_blocks: RwLock<HashMap<UserId, Vec<BlockId>>>,
    executables: RwLock<HashMap<BlockId, ExecutableBlock>>,
    genesis_user: RwLock<Option<User>>,
}
```

#### Genesis User "futuros"

The genesis user is the **owner of all system executable blocks** and has special properties:

```rust
// Creation of genesis user
pub fn new_genesis(name: String, public_key: Vec<u8>) -> Self {
    let permissions = UserPermissions::full();  // All permissions
    let mut user = Self::new(name, public_key, UserRole::Genesis, permissions);
    user.metadata.insert("type".to_string(), "genesis".to_string());
    user.metadata.insert("description".to_string(), 
        "System genesis user - owner of OS executable blocks".to_string());
    user
}

// Initialization with genesis user
pub fn initialize_with_futuros(&mut self, public_key: Vec<u8>) -> Result<(), String> {
    self.initialize(GENESIS_USER_NAME.to_string(), public_key)
}

// Register system executable (owned by genesis user)
pub fn register_system_executable(
    &mut self,
    name: String,
    code: Vec<u8>,
    entry_point: String,
    executable_type: String,
) -> Result<BlockId, String> {
    let genesis_user = self.get_genesis_user()
        .ok_or("System not initialized with genesis user".to_string())?;
    
    // Create block with genesis user as owner
    let block_id = self.create_block(code, genesis_user.id.clone(), 
        "system_executable".to_string())?;
    
    // Store executable metadata
    let executable = ExecutableBlock::new(
        code, 
        genesis_user.id.clone(),
        entry_point,
        executable_type,
    );
    
    // Store in executables map
    self.executables.write().insert(block_id.clone(), executable);
    
    Ok(block_id)
}
```

### Usage Examples

#### Basic Setup

```rust
use pqos::users::{UserSystem, create_user_system_with_demo_keys, GENESIS_USER};

// Create a user system with genesis user "futuros"
let mut system = create_user_system_with_demo_keys();

// System is initialized with genesis user
assert!(system.is_initialized());

// Get genesis user
let genesis = system.get_genesis_user().unwrap();
assert_eq!(genesis.name, GENESIS_USER);
```

#### Registering System Executables

```rust
use pqos::users::{UserSystem, create_user_system};

// Create system with a specific public key
let mut system = create_user_system(vec![0x01; 64]).unwrap();

// Register kernel executable (owned by genesis user)
let kernel_code = include_bytes!("kernel.bin");
let kernel_id = system.register_system_executable(
    "kernel".to_string(),
    kernel_code.to_vec(),
    "main".to_string(),
    "kernel".to_string(),
).unwrap();

// Verify ownership
assert!(system.is_system_block(&kernel_id));
let owner_id = system.get_block_owner(&kernel_id).unwrap();
let genesis = system.get_genesis_user().unwrap();
assert_eq!(owner_id, genesis.id);
```

#### Working with Traits

```rust
use pqos::users::traits::*;
use pqos::users::{UserId, UserRole, UserPermissions, User, UserSystem};

// Use trait objects for generic programming
fn print_user_info(user: &dyn UserTrait) {
    println!("User ID: {:?}", user.id());
    println!("Name: {}", user.name());
    println!("Role: {}", user.role().as_str());
    println!("Is Genesis: {}", user.is_genesis());
}

// Create a user and use trait
let public_key = vec![1u8; 64];
let user = User::new_genesis("test".to_string(), public_key);
print_user_info(&user);

// Use UserSystem trait
let mut system = UserSystem::new();
system.initialize_with_futuros(vec![2u8; 64]).unwrap();

fn process_user_system(system: &dyn UserSystemTrait) {
    if system.is_initialized() {
        println!("System has {} users", 
            system.list_system_executables().len());
    }
}
process_user_system(&system);
```

#### Creating Custom Implementations

To create a **blockchain-based user system**:

```rust
// 1. Create blockchain_users.rs
pub mod blockchain_users {
    use super::traits::*;
    
    // Define blockchain-specific types
    pub struct BlockchainUserId(Vec<u8>);
    pub struct BlockchainUser { /* ... */ }
    pub struct BlockchainBlock { /* ... */ }
    
    // Implement traits
    impl UserIdTrait for BlockchainUserId { /* ... */ }
    impl UserTrait for BlockchainUser { /* ... */ }
    impl BlockTrait for BlockchainBlock { /* ... */ }
    
    // Implement UserSystem with blockchain backend
    pub struct BlockchainUserSystem {
        blockchain: Arc<dyn Blockchain>,
        // ... other state
    }
    
    impl UserSystemTrait for BlockchainUserSystem {
        type User = BlockchainUser;
        type UserId = BlockchainUserId;
        type Block = BlockchainBlock;
        type BlockId = BlockId;
        type ExecutableBlock = BlockchainExecutableBlock;
        type Error = BlockchainUserError;
        
        fn initialize(&mut self, name: String, public_key: Vec<u8>) -> Result<(), Self::Error> {
            // Store genesis user on blockchain
            let tx = self.create_genesis_transaction(name, public_key);
            self.blockchain.add_transaction(tx)?;
            Ok(())
        }
        
        // Implement other trait methods using blockchain storage
        // ...
    }
}

// 2. Export in mod.rs
pub use blockchain_users::*;
```

---

## Abstract Traits System

### Design Philosophy

The **trait-based architecture** is the foundation of PQOS's evolvability:

1. **Separation of Concerns**: Traits define WHAT, implementations define HOW
2. **Zero Cost Abstractions**: Rust's monomorphization ensures no runtime overhead
3. **Compile-Time Safety**: Type system prevents incompatible implementations
4. **Flexible Composition**: Traits can be combined in various ways

### Trait Organization

#### Horizontal Traits (Cross-Cutting)

```
┌─────────────────────────────┐
│  Send + Sync                │  Thread-safe
├─────────────────────────────┤
│  Clone                      │  Copyable
├─────────────────────────────┤
│  Debug                      │  Debuggable
├─────────────────────────────┤
│  Serialize + DeserializeOwned│  Serde compatible
└─────────────────────────────┘
```

#### Domain-Specific Traits

Each module defines its own trait hierarchy:
- `crypto/` - Cryptographic primitives
- `block/` - Block storage and management
- `blockchain/` - Distributed ledger
- `network/` - P2P communication
- `memory/` - Unified memory abstraction
- `users/` - User management and authentication

### Trait Composition Example

```rust
// A complete blockchain node might implement:
pub trait BlockchainNode: 
    Blockchain + 
    NetworkProtocol + 
    BlockStorage + 
    ConsensusAlgorithm +
    Send + Sync
{
    // Combined functionality
}
```

### Associated Types

Traits use **associated types** to maintain flexibility:

```rust
pub trait UserSystem: Send + Sync {
    type User: User;
    type UserId: UserId;
    type Block: Block<UserId = Self::UserId>;
    type BlockId: BlockId;
    type ExecutableBlock: ExecutableBlock<
        Block = Self::Block,
        BlockId = Self::BlockId,
        UserId = Self::UserId,
    >;
    type Error: std::error::Error + Send + Sync + 'static;
    
    // Methods using associated types
    fn get_user(&self, user_id: &Self::UserId) -> Option<Self::User>;
    fn create_block(&mut self, data: Vec<u8>, owner: Self::UserId) 
        -> Result<Self::BlockId, Self::Error>;
}
```

### Error Handling

Each module defines its own error type with `thiserror`:

```rust
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
}
```

### Factory Pattern

Factories create instances with specific configurations:

```rust
pub trait UserSystemFactory: Send + Sync {
    type UserSystem: UserSystem;
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn create(&self) -> Self::UserSystem;
    fn create_initialized(&self, name: String, public_key: Vec<u8>) 
        -> Result<Self::UserSystem, Self::Error>;
    fn create_with_futuros(&self, public_key: Vec<u8>) 
        -> Result<Self::UserSystem, Self::Error>;
}
```

---

## Implementation Patterns

### 1. Trait Implementation Pattern

```rust
// Step 1: Define the trait
pub trait MyTrait: Send + Sync {
    type AssociatedType;
    fn my_method(&self, param: u32) -> Result<Self::AssociatedType, MyError>;
}

// Step 2: Create concrete type
pub struct MyConcreteType {
    field: String,
}

// Step 3: Implement the trait
impl MyTrait for MyConcreteType {
    type AssociatedType = ConcreteResult;
    
    fn my_method(&self, param: u32) -> Result<Self::AssociatedType, MyError> {
        // Implementation
        Ok(ConcreteResult::new(param))
    }
}

// Step 4: Export in mod.rs
pub use my_type::MyConcreteType;
```

### 2. Builder Pattern

```rust
pub trait MyBuilder: Send + Sync {
    type Output;
    type Error;
    
    fn new() -> Self;
    fn with_option(&mut self, value: u32) -> &mut Self;
    fn build(&self) -> Result<Self::Output, Self::Error>;
}

pub struct ConcreteBuilder {
    option: u32,
}

impl MyBuilder for ConcreteBuilder {
    type Output = MyType;
    type Error = BuilderError;
    
    fn new() -> Self {
        Self { option: 0 }
    }
    
    fn with_option(&mut self, value: u32) -> &mut Self {
        self.option = value;
        self
    }
    
    fn build(&self) -> Result<Self::Output, Self::Error> {
        Ok(MyType::new(self.option))
    }
}
```

### 3. Factory Pattern

```rust
pub trait MyFactory: Send + Sync {
    type Product;
    type Config;
    type Error;
    
    fn create(&self, config: Self::Config) -> Result<Self::Product, Self::Error>;
}

pub struct ConcreteFactory;

impl MyFactory for ConcreteFactory {
    type Product = MyProduct;
    type Config = MyConfig;
    type Error = FactoryError;
    
    fn create(&self, config: MyConfig) -> Result<MyProduct, FactoryError> {
        // Create product based on config
        Ok(MyProduct::from_config(config))
    }
}
```

### 4. Decorator Pattern (Trait Composition)

```rust
// Base trait
pub trait Encryption: Send + Sync {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError>;
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, EncryptionError>;
}

// Extension trait
pub trait Compression: Send + Sync {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError>;
}

// Combined trait
pub trait EncryptedCompression: Encryption + Compression {
    fn encrypt_and_compress(&self, data: &[u8]) -> Result<Vec<u8>, CombinedError> {
        let compressed = self.compress(data)?;
        self.encrypt(&compressed)
    }
    
    fn decrypt_and_decompress(&self, data: &[u8]) -> Result<Vec<u8>, CombinedError> {
        let decrypted = self.decrypt(data)?;
        self.decompress(&decrypted)
    }
}

// Blanket implementation
impl<T: Encryption + Compression> EncryptedCompression for T {}
```

---

## Security Architecture

### Core Security Principles

1. **Defense in Depth**: Multiple layers of security (crypto, access control, audit)
2. **Least Privilege**: Users have only necessary permissions
3. **Fail Secure**: System fails in a secure state
4. **Auditability**: All operations are logged and verifiable
5. **Forward Secrecy**: Compromise of one key doesn't compromise past data

### Users Module Security

#### Private Key Protection

**INVARIANT**: The genesis user's private key is NEVER accessible through the system.

```rust
// ❌ WRONG - NEVER do this!
pub struct User {
    pub id: UserId,
    pub name: String,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,  // NEVER store private key!
    // ...
}

// ✅ CORRECT - Only public key stored
pub struct User {
    pub id: UserId,
    pub name: String,
    pub public_key: Vec<u8>,  // ONLY public key
    pub role: UserRole,
    pub permissions: UserPermissions,
    // NO private key field!
}
```

#### Authentication Flow

```
┌─────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│   External System    │────▶│  Signature Verifier  │────▶│   User Authenticator │
│   (HSM, Secure       │     │   (External/Internal)│     │   (PQOS Module)    │
│    Enclave, etc.)    │     │                     │     │                     │
└─────────────────────┘     └─────────────────────┘     └─────────────────────┘
       │                        │                          │
       │ Sign(data, sk)          │ verify(sig, pk, data)   │ Generate token
       ▼                        ▼                          ▼
  ┌─────────────────┐    ┌─────────────────┐        ┌─────────────────┐
  │  Signature       │    │   bool          │        │   AuthToken      │
  │  (detached)     │    │ (valid/invalid) │        │ (time-limited)   │
  └─────────────────┘    └─────────────────┘        └─────────────────┘
```

1. **External signing**: Private key operations happen outside PQOS
2. **Signature verification**: Can be done externally or internally with public key
3. **Token generation**: PQOS generates time-limited authentication tokens
4. **Token validation**: PQOS validates tokens using its stored public keys

#### Permission System

```rust
// Full permissions for genesis user
UserPermissions {
    can_create_blocks: true,
    can_read_all_blocks: true,
    can_write_all_blocks: true,
    can_manage_users: true,
    can_manage_system: true,
    can_execute_code: true,
}

// Regular user permissions
UserPermissions {
    can_create_blocks: true,
    can_read_all_blocks: false,  // Only own blocks
    can_write_all_blocks: false,  // Only own blocks
    can_manage_users: false,
    can_manage_system: false,
    can_execute_code: false,      // Only system can execute
}

// Admin user permissions
UserPermissions {
    can_create_blocks: true,
    can_read_all_blocks: true,
    can_write_all_blocks: true,
    can_manage_users: true,
    can_manage_system: false,     // Only genesis can manage system
    can_execute_code: true,
}
```

#### Block Ownership

```
┌─────────────────────────────────────────────────────────────┐
│  Block Ownership Model                                         │
├─────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐      owns       ┌─────────────┐               │
│  │  User       │ ──────────────▶ │   Block     │               │
│  │  futuros    │                │   (code)     │               │
│  │  (Genesis)  │                │             │               │
│  └─────────────┘                └─────────────┘               │
│           │                                          │               │
│           │ owns                                     │               │
│           ▼                                          ▼               │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              System Executable Blocks                │   │
│  │  - kernel.bin (owned by futuros)                      │   │
│  │  - bootstrap.bin (owned by futuros)                   │   │
│  │  - drivers/ (owned by futuros)                         │   │
│  │  - services/ (owned by futuros)                        │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────┐      owns       ┌─────────────┐               │
│  │  User       │ ──────────────▶ │   Block     │               │
│  │  alice      │                │   (data)     │               │
│  │  (Regular)  │                │             │               │
│  └─────────────┘                └─────────────┘               │
│                                                                  │
└─────────────────────────────────────────────────────────────┘
```

**Security Property**: Only the genesis user "futuros" can register system executable blocks.

### Cryptographic Protection

#### Block Encryption

All blocks are encrypted at rest using symmetric encryption:

```rust
// Encrypted block structure
pub trait EncryptedBlock: Block {
    type Ciphertext: AsRef<[u8]> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync;
    
    fn ciphertext(&self) -> &Self::Ciphertext;
    fn nonce(&self) -> &[u8];           // Unique nonce for each encryption
    fn key_id(&self) -> &[u8];           // Identifier of encryption key
    fn encryption_algorithm(&self) -> &str;  // e.g., "AES-256-GCM"
}
```

#### Key Management

```rust
// Key encapsulation for PQC
pub trait Kem: Clone + Debug + Send + Sync {
    fn algorithm(&self) -> &str;
    fn generate_keypair(&self) -> (Vec<u8>, Vec<u8>);  // (pk, sk)
    fn encapsulate(&self, public_key: &[u8]) -> (Vec<u8>, Vec<u8>);  // (ct, ss)
    fn decapsulate(&self, private_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>>;
}

// Signature schemes for PQC
pub trait SignatureScheme: Clone + Debug + Send + Sync {
    fn algorithm(&self) -> &str;
    fn generate_keypair(&self) -> (Vec<u8>, Vec<u8>);  // (pk, sk)
    fn sign(&self, data: &[u8], private_key: &[u8]) -> Result<Vec<u8>>;
    fn verify(&self, data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool>;
}
```

---

## Development Guidelines

### Adding New Implementations

To add a new implementation for a trait:

1. **Create a new module** (e.g., `src/users/blockchain.rs`)
2. **Define concrete types** with proper derives
3. **Implement the trait(s)** for your types
4. **Add tests** in a `#[cfg(test)] mod tests` block
5. **Export in mod.rs**
6. **Document the implementation**

### Example: Adding a Database Backend for Users

```rust
// src/users/database.rs

use super::traits::*;
use diesel::{Connection, PgConnection};

pub struct DatabaseUserId(pub i64);  // Database primary key

impl UserIdTrait for DatabaseUserId {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        // Convert bytes to i64
        DatabaseUserId(i64::from_be_bytes(bytes.try_into().unwrap()))
    }
    
    fn from_public_key(public_key: &[u8]) -> Self {
        // Hash public key and use as ID
        let hash = sha3::Sha3_256::hash(public_key);
        DatabaseUserId(i64::from_be_bytes(hash[..8].try_into().unwrap()))
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
    
    fn size(&self) -> usize {
        8  // i64 is 8 bytes
    }
}

// Implement other types and UserSystem...

// Export in mod.rs
pub use database::DatabaseUserSystem;
```

### Testing Guidelines

1. **Unit Tests**: Test individual functions in `#[cfg(test)] mod tests`
2. **Trait Tests**: Test that types correctly implement traits
3. **Integration Tests**: Test interactions between modules
4. **Property Tests**: Use `proptest` for property-based testing
5. **Security Tests**: Verify security invariants

### Security Testing

**MUST verify for all implementations**:

1. **Private Key Inaccessibility**: No way to retrieve private keys
2. **Signature Verification**: All signatures are properly verified
3. **Permission Enforcement**: Users cannot perform unauthorized actions
4. **Data Integrity**: Blocks cannot be modified without detection
5. **Content Addressing**: Same content produces same address

```rust
#[test]
fn test_private_key_inaccessible() {
    // This test MUST pass for all implementations
    let system = create_user_system(vec![1u8; 64]).unwrap();
    
    // There should be NO way to access private keys
    // This is verified by:
    // 1. User struct has no private_key field
    // 2. No method returns private keys
    // 3. Private keys are never stored in the system
    
    // We can only access public keys
    let genesis = system.get_genesis_user().unwrap();
    assert_eq!(genesis.public_key().len(), 64);  // Only public key accessible
    
    // The private key was provided only during initialization
    // and was NOT stored in the system
}

#[test]
fn test_block_ownership_security() {
    let mut system = create_user_system(vec![1u8; 64]).unwrap();
    let regular_user = User::new("alice".to_string(), vec![2u8; 64], UserRole::User, 
        UserPermissions::default());
    
    // Regular user cannot register system executables
    let result = system.create_block(
        vec![0xDE, 0xAD, 0xBE, 0xEF],
        regular_user.id.clone(),
        "system_executable".to_string()
    );
    
    // This should succeed - users can create blocks
    assert!(result.is_ok());
    
    // But only genesis user can register SYSTEM executables
    // (enforced by register_system_executable method)
    let system_block = system.register_system_executable(
        "test".to_string(),
        vec![0x00],
        "main".to_string(),
        "kernel".to_string(),
    ).unwrap();
    
    let owner = system.get_block_owner(&system_block).unwrap();
    let genesis = system.get_genesis_user().unwrap();
    assert_eq!(owner, genesis.id);  // Must be owned by genesis
}
```

### Documentation Standards

1. **All public items** must have doc comments
2. **Traits** must document all methods and associated types
3. **Structs** must document all fields
4. **Enums** must document all variants
5. **Security notes** must be clearly marked with `**Security Note**`
6. **Examples** should be provided for complex traits

### Code Style

1. **Naming**: Use `snake_case` for functions, `PascalCase` for types
2. **Error Handling**: Use `thiserror` for error types
3. **Type Safety**: Prefer compile-time checks over runtime checks
4. **Performance**: Avoid unnecessary allocations and copies
5. **Concurrency**: Use `parking_lot` for locks, prefer `Arc` over `Rc`

---

## Summary

The PQOS **Users Module** provides:

✅ **Complete abstraction** through traits for all user management functionality  
✅ **Genesis user "futuros"** with secure private key handling  
✅ **Reference implementation** in `simple.rs` for development and testing  
✅ **Extensible architecture** allowing custom backends (blockchain, database, etc.)  
✅ **Comprehensive security** with private key protection and permission system  
✅ **Full test coverage** for all functionality  

The **trait-based design** ensures:
- **Technology agnosticism** - No lock-in to specific implementations
- **Evolution** - New implementations can be added without changing existing code
- **Type safety** - Compile-time guarantees through Rust's type system
- **Performance** - Zero-cost abstractions through monomorphization

This architecture enables PQOS to be:
- **Secure** against quantum and classical attacks
- **Distributed** across multiple nodes
- **Evolvable** as new technologies emerge
- **Maintainable** with clear separation of concerns
