# PQOS Documentation

Welcome to the documentation for the **Post-Quantum Distributed Operating System (PQOS)**.

## Documentation Structure

```
docs/
├── README.md              # This file - Documentation overview
└── ARCHITECTURE.md        # Complete architecture documentation
    ├── Overview
    ├── Core Design Principles
    ├── Module Architecture
    ├── Users Module - Deep Dive
    ├── Abstract Traits System
    ├── Implementation Patterns
    ├── Security Architecture
    └── Development Guidelines
```

## Quick Start

### Documentation Map

| Document | Purpose | Audience |
|----------|---------|----------|
| `ARCHITECTURE.md` | Complete technical architecture | Developers, Architects |
| This file | Documentation overview | All users |

### Reading Order

1. **First time?** Start with `ARCHITECTURE.md` - it contains everything you need
2. **Looking for something specific?** Use the table of contents in `ARCHITECTURE.md`
3. **Need to implement something?** See the Development Guidelines section

---

## Project Overview

**PQOS (Post-Quantum Distributed Operating System)** is a novel OS architecture that treats all storage as a **unified memory layer** of content-addressed, encrypted blocks with immutable history tracked on a distributed blockchain.

### Key Features

- ✅ **Post-Quantum Secure**: Uses NIST-approved PQC algorithms (ML-KEM/Kyber, ML-DSA/Dilithium)
- ✅ **Content-Addressed**: All data identified by cryptographic hash (like Git)
- ✅ **Blockchain-Backed**: Immutable ledger for version history and access control
- ✅ **Distributed**: P2P network with consensus algorithms
- ✅ **Unified Memory**: RAM, files, network storage all use same addressing scheme
- ✅ **Evolvable Architecture**: Trait-based design allows pluggable implementations

### What Makes PQOS Unique?

1. **Everything is a Block**: Files, memory, configuration - all are content-addressed encrypted blocks
2. **No Technology Lock-in**: Abstract traits allow any cryptographic library, network stack, or storage backend
3. **Security by Design**: Private keys NEVER stored in system, encryption at rest, immutable history
4. **Genesis User Model**: Special "futuros" user owns all system executable code

---

## Module Documentation

### 🔐 Crypto Module

**Purpose**: Abstract cryptographic primitives  
**Location**: `src/crypto/traits.rs`  
**Status**: Traits defined  

Provides technology-agnostic interfaces for:
- Hash functions (SHA3, BLAKE3, etc.)
- Symmetric encryption (AES-GCM, ChaCha20-Poly1305)
- Key Encapsulation Mechanisms (ML-KEM/Kyber, BIKE, NTRU)
- Digital signatures (ML-DSA/Dilithium, ECDSA, Ed25519)
- Key derivation (HKDF, PBKDF2)
- Secure random generation
- Key lifecycle management

**Example Implementations**:
- `liboqs-rs` (NIST PQC)
- `OpenSSL` (classical crypto)
- Custom implementations

### 🧱 Block Module

**Purpose**: Content-addressed block storage  
**Location**: `src/block/traits.rs`  
**Status**: Traits defined  

Defines the fundamental storage unit with:
- Content-based addressing (hash of content = address)
- Immutable block structure
- Support for encrypted and signed blocks
- Flexible storage backends

**Key Traits**:
- `BlockId` - Content-based identifier
- `Block` - Immutable data block
- `BlockStorage` - Persistent storage
- `BlockVerifier` - Integrity validation
- `ContentAddressedStorage` - High-level interface

### ⛓️ Blockchain Module

**Purpose**: Distributed ledger functionality  
**Location**: `src/blockchain/traits.rs`  
**Status**: Traits defined  

Provides blockchain functionality with:
- Transaction processing
- Consensus algorithms (Raft, BFT, etc.)
- P2P synchronization
- Immutable history

**Key Traits**:
- `Transaction` - State change
- `ConsensusAlgorithm` - Consensus protocol
- `Blockchain` - Main ledger interface
- `BlockchainNode` - Network participant
- `BlockchainSync` - Synchronization

### 🌐 Network Module

**Purpose**: P2P communication  
**Location**: `src/network/traits.rs`  
**Status**: Traits defined  

Provides network communication with:
- Peer discovery
- Message routing
- Connection management
- Network encryption

**Key Traits**:
- `Peer` - Network node
- `NetworkMessage` - Message structure
- `P2PNetwork` - Network interface
- `NetworkProtocol` - Low-level protocol

### 💾 Memory Module

**Purpose**: Unified memory abstraction  
**Location**: `src/memory/traits.rs`  
**Status**: Traits defined  

**The Core Innovation**: Treats all storage as content-addressed encrypted blocks

**Key Traits**:
- `MemoryManager` - Memory management
- `AddressSpace` - Virtual addressing
- `ContentAddressedFileSystem` - Files as blocks
- `MemoryAllocator` - Allocation

### 👥 Users Module ⭐ (Newly Documented)

**Purpose**: User management and authentication  
**Location**: `src/users/`  
**Status**: ✅ **Fully implemented with traits and reference implementation**  

#### Structure

```
src/users/
├── mod.rs              # Module exports, compatibility layer
├── traits.rs           # Abstract traits (715 lines)
└── simple.rs           # Reference implementation (1132 lines)
```

#### What's Included

**Abstract Traits (traits.rs)**:
- `UserIdTrait` - User identifier
- `UserRoleTrait` - User roles (Genesis, Admin, User)
- `UserPermissionsTrait` - Permission system
- `UserTrait` - User structure
- `BlockIdTrait` - Block identifier
- `BlockTrait` - Block structure
- `ExecutableBlockTrait` - Executable code blocks
- `UserSystemTrait` - Main user system interface
- `UserBuilderTrait` - User construction
- `BlockBuilderTrait` - Block construction
- `ExecutableBlockBuilderTrait` - Executable construction
- `UserSystemFactoryTrait` - System factory
- `UserAuthenticatorTrait` - Authentication
- `AuthenticationProvider` - External auth services
- `UserSystemBlockchain` - Blockchain integration
- `UserStorageBackend` - Storage backend
- `UserIndex` - Indexing

**Reference Implementation (simple.rs)**:
- Complete in-memory implementation
- SHA256 for content addressing
- Thread-safe with `RwLock`
- Genesis user "futuros" support
- System executable registration
- Full trait implementation
- Comprehensive tests

#### Key Features

1. **Genesis User "futuros"**
   - Owner of all system executable blocks
   - Full permissions
   - Private key NEVER accessible through system

2. **Security Model**
   - Only public keys stored
   - Private keys kept externally
   - External signature verification
   - Time-limited authentication tokens

3. **Block Ownership**
   - All blocks have an owner
   - System executables owned by genesis user
   - Regular users own their data blocks

4. **Permission System**
   - Genesis: All permissions
   - Admin: Most permissions
   - User: Limited permissions

#### Usage Example

```rust
use pqos::users::{UserSystem, create_user_system, GENESIS_USER};

// Create user system with genesis user
let mut system = create_user_system(vec![0x01; 64]).unwrap();

// Register kernel executable (owned by futuros)
let kernel_id = system.register_system_executable(
    "kernel".to_string(),
    vec![0x00, 0x01, 0x02],
    "main".to_string(),
    "kernel".to_string(),
).unwrap();

// Verify it's a system block
assert!(system.is_system_block(&kernel_id));

// Get genesis user
let genesis = system.get_genesis_user().unwrap();
assert_eq!(genesis.name(), GENESIS_USER);
```

#### Extending the Users Module

To add a new backend (e.g., blockchain-based):

```rust
// 1. Create new module
pub mod blockchain_users {
    use super::traits::*;
    
    // 2. Define types
    pub struct BlockchainUserId(Vec<u8>);
    pub struct BlockchainUser { /* ... */ }
    
    // 3. Implement traits
    impl UserIdTrait for BlockchainUserId { /* ... */ }
    impl UserTrait for BlockchainUser { /* ... */ }
    
    // 4. Implement UserSystem
    pub struct BlockchainUserSystem { /* ... */ }
    impl UserSystemTrait for BlockchainUserSystem { /* ... */ }
}

// 5. Export in mod.rs
pub use blockchain_users::*;
```

**See `docs/ARCHITECTURE.md` for complete details on the Users module architecture.**

---

## Architecture Highlights

### Design Principles

1. **Technology Agnosticism**: No concrete implementations in core
2. **Pluggable Architecture**: Any backend can be substituted
3. **Content Addressing**: All data identified by cryptographic hash
4. **Security by Design**: Private keys NEVER stored, encryption at rest
5. **Evolutionary**: New implementations can be added without breaking changes

### Trait System

All functionality is defined through **Rust traits**:

```rust
// Example: User trait
pub trait User: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    type Id: UserId;
    type Role: UserRole;
    type Permissions: UserPermissions;
    
    fn id(&self) -> &Self::Id;
    fn name(&self) -> &str;
    fn public_key(&self) -> &[u8];  // ONLY public key!
    // ... other methods
}
```

Benefits:
- ✅ Zero-cost abstractions (Rust monomorphization)
- ✅ Compile-time type safety
- ✅ Flexible composition
- ✅ Clear separation of concerns

### Error Handling

Consistent error handling with `thiserror`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum UserSystemError {
    #[error("User not found")]
    UserNotFound,
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Internal error: {0}")]
    InternalError(String),
}
```

---

## Development

### Prerequisites

- Rust 1.70+
- liboqs-rs (optional, for PQC)
- Standard development tools

### Building

```bash
# Build the library
cargo build

# Build with optimizations
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test --lib users

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin
```

### Project Structure

```
pqos/
├── Cargo.toml                 # Project configuration
├── README.md                  # Project overview
├── src/                       # Source code
│   ├── lib.rs                 # Library entry point
│   ├── error.rs               # Global error types
│   ├── crypto/                # Cryptographic traits
│   │   └── traits.rs
│   ├── block/                 # Block storage traits
│   │   └── traits.rs
│   ├── blockchain/            # Blockchain traits
│   │   └── traits.rs
│   ├── network/               # Network traits
│   │   └── traits.rs
│   ├── memory/                # Memory abstraction traits
│   │   └── traits.rs
│   └── users/                 # User management (NEW!)
│       ├── mod.rs
│       ├── traits.rs          # User traits
│       └── simple.rs          # Reference implementation
└── docs/                      # Documentation
    ├── README.md              # This file
    └── ARCHITECTURE.md        # Complete architecture
```

---

## Contributing

We welcome contributions! The trait-based architecture makes it easy to add:

- **New cryptographic implementations** (PQC algorithms)
- **New storage backends** (databases, distributed systems)
- **New network protocols** (P2P, QUIC, etc.)
- **New consensus algorithms** (Raft, BFT, etc.)

### Getting Started

1. Fork the repository
2. Read `docs/ARCHITECTURE.md`
3. Pick a trait to implement
4. Create a new module with your implementation
5. Add tests
6. Submit a pull request

### Implementation Checklist

- [ ] Trait correctly implemented
- [ ] All methods have documentation
- [ ] All associated types defined
- [ ] Error handling implemented
- [ ] Thread safety (Send + Sync)
- [ ] Serialization support (Serialize + DeserializeOwned)
- [ ] Unit tests added
- [ ] Integration tests considered
- [ ] Security invariants verified

---

## Security

### Core Security Principles

1. **Private Key Protection**: Private keys are NEVER stored or accessible through the system
2. **Encryption at Rest**: All blocks are encrypted with symmetric encryption
3. **Immutable History**: Blockchain ledger prevents tampering with past data
4. **Post-Quantum Ready**: All cryptographic traits support PQC algorithms
5. **Least Privilege**: Users have only necessary permissions

### Security Testing

All implementations MUST verify:
- Private keys cannot be accessed
- Signatures are properly verified
- Permissions are enforced
- Data integrity is maintained
- Content addressing works correctly

### Reporting Security Issues

If you find a security vulnerability:
1. **DO NOT** open a public issue
2. Email security@pqos.dev (or appropriate contact)
3. Include steps to reproduce
4. Allow reasonable time for fix before disclosure

---

## Resources

### Post-Quantum Cryptography

- [NIST PQC Project](https://csrc.nist.gov/projects/post-quantum-cryptography/)
- [CRYSTALS-Kyber (ML-KEM)](https://pq-crystals.org/kyber/)
- [CRYSTALS-Dilithium (ML-DSA)](https://pq-crystals.org/dilithium/)
- [liboqs-rs](https://github.com/open-quantum-safe/liboqs-rs)

### Distributed Systems

- [Raft Consensus](https://raft.github.io/)
- [Byzantine Fault Tolerance](https://en.wikipedia.org/wiki/Byzantine_fault_tolerance)
- [IPFS - Content Addressed Storage](https://ipfs.tech/)

### Related Concepts

- [Git Internals - Content Addressing](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)
- [Designing Data-Intensive Applications (Book)](https://www.oreilly.com/library/view/designing-data-intensive-applications/9781491903063/)

---

## License

This project is licensed under the **MIT License**. See LICENSE file for details.

---

## Next Steps

Ready to dive deeper?

📖 **Read the complete architecture**: See `docs/ARCHITECTURE.md`

💻 **Explore the code**: Check out `src/users/` for the Users module implementation

🔧 **Start developing**: Pick a trait and implement it!

---

*Documentation last updated: July 1, 2026*
