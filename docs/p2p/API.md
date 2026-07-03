# P2P Module API Reference

This document describes the **public API** of the P2P module for PQDOS. All types and functions are exposed via the `pqdos::p2p` module.

---

## 📦 **Module Structure**

```
pub mod p2p {
    pub mod traits;          // Abstract traits (P2PNetwork, Peer, BlockFetcher, CacheManager)
    pub mod peer;            // Peer implementation (P2PPeer)
    pub mod network;         // Network messages and P2PNetworkImpl
    pub mod cache;           // Cache module (LRUCache, CacheStorage, crypto)
    pub mod discovery;       // Peer discovery (TODO)
    pub mod block_fetcher;   // Block fetching (P2PBlockFetcher)
}
```

---

## 🏗️ **Traits**

### **1. `Peer` Trait**

Represents a peer in the P2P network.

```rust
#[async_trait]
pub trait Peer: Send + Sync {
    type Address: Send + Sync + Display;
    
    /// Returns the unique ID of the peer.
    fn id(&self) -> &str;
    
    /// Returns the address of the peer.
    fn address(&self) -> &Self::Address;
    
    /// Sends a message to the peer.
    async fn send_message(&self, message: NetworkMessage) -> Result<(), Error>;
    
    /// Checks if the peer is connected (via ping).
    async fn ping(&self) -> Result<bool, Error>;
}
```

---

### **2. `P2PNetwork` Trait**

Represents a P2P network.

```rust
#[async_trait]
pub trait P2PNetwork: Send + Sync {
    type Peer: Peer + Clone;
    
    /// Adds a known peer to the network.
    async fn add_peer(&mut self, peer: Self::Peer) -> Result<(), Error>;
    
    /// Discovers new peers in the network.
    async fn discover_peers(&self) -> Result<Vec<Self::Peer>, Error>;
    
    /// Broadcasts a message to all connected peers.
    async fn broadcast(&self, message: NetworkMessage) -> Result<(), Error>;
    
    /// Sends a message to a specific peer.
    async fn send_to_peer(&self, peer_id: &str, message: NetworkMessage) -> Result<(), Error>;
    
    /// Listens for incoming messages.
    async fn listen(&mut self) -> Result<(), Error>;
}
```

---

### **3. `BlockFetcher` Trait**

Fetches blocks from the network or local cache.

```rust
#[async_trait]
pub trait BlockFetcher: Send + Sync {
    /// Fetches a block from the network or cache.
    /// Returns `None` if the block is not accessible (no permissions).
    async fn fetch_block(
        &self, 
        block_id: &[u8], 
        user_id: &[u8],
        user_private_key: Option<&[u8]>, // For decrypting owned blocks.
        shared_keys: &HashMap<Vec<u8>, Vec<u8>>, // For shared blocks.
    ) -> Result<Option<SimpleBlock>, Error>;
    
    /// Checks if a block is available locally or on the network.
    async fn has_block(&self, block_id: &[u8]) -> Result<bool, Error>;
}
```

---

### **4. `CacheManager` Trait**

Manages the local cache for blocks.

```rust
#[async_trait]
pub trait CacheManager: Send + Sync {
    /// Adds a block to the cache (encrypted).
    async fn cache_block(
        &mut self, 
        block: SimpleBlock, 
        owner_id: &[u8],
    ) -> Result<(), Error>;
    
    /// Retrieves a block from the cache (decrypted if user has permissions).
    async fn get_cached_block(
        &self, 
        block_id: &[u8], 
        user_id: &[u8],
        user_private_key: Option<&[u8]>, // For decrypting owned blocks.
        shared_keys: &HashMap<Vec<u8>, Vec<u8>>, // For shared blocks.
    ) -> Result<Option<SimpleBlock>, Error>;
    
    /// Checks if a block is in the cache.
    async fn contains(&self, block_id: &[u8]) -> Result<bool, Error>;
    
    /// Removes a block from the cache.
    async fn remove_block(&mut self, block_id: &[u8]) -> Result<(), Error>;
    
    /// Cleans up the cache (removes LRU entries).
    async fn cleanup(&mut self) -> Result<(), Error>;
    
    /// Returns the current size of the cache (in bytes).
    async fn size(&self) -> Result<u64, Error>;
}
```

---

## 📡 **Network Messages**

### **`NetworkMessage` Enum**

Enum for all P2P network messages.

```rust
pub enum NetworkMessage {
    /// Request a block from the network.
    BlockRequest {
        block_id: Vec<u8>,      // ID of the requested block.
        requester_id: String,  // ID of the peer requesting the block.
    },
    
    /// Response with an encrypted block.
    BlockResponse {
        block_id: Vec<u8>,         // ID of the block.
        encrypted_data: Vec<u8>,   // Encrypted block data.
        owner_id: Vec<u8>,         // ID of the block owner.
        nonce: Vec<u8>,            // Nonce used for encryption.
    },
    
    /// Announce block availability.
    BlockAvailability {
        block_id: Vec<u8>,      // ID of the available block.
        peer_id: String,        // ID of the peer that has the block.
    },
    
    /// Ping message (check connectivity).
    Ping {
        peer_id: String,        // ID of the peer sending the ping.
    },
    
    /// Pong message (response to ping).
    Pong {
        peer_id: String,        // ID of the peer responding to the ping.
    },
}
```

### **Methods**

```rust
impl NetworkMessage {
    /// Returns the type of the message (e.g., "BLOCK_REQUEST").
    pub fn message_type(&self) -> &str;
    
    /// Returns the block ID associated with the message (if any).
    pub fn block_id(&self) -> Option<&[u8]>;
}
```

---

## 🗃️ **Cache Types**

### **1. `LRUCache`**

LRU (Least Recently Used) cache for blocks.

#### **Fields**

```rust
pub struct LRUCache {
    cache_dir: PathBuf,                       // Directory for cache storage.
    entries: Arc<RwLock<HashMap<Vec<u8>, CacheEntry>>>, // block_id -> CacheEntry.
    max_size: u64,                            // Maximum cache size (bytes).
    current_size: Arc<RwLock<u64>>,           // Current cache size.
}
```

#### **Methods**

```rust
impl LRUCache {
    /// Creates a new LRU cache.
    pub async fn new(cache_dir: impl AsRef<Path>, max_size: u64) -> Result<Self, Error>;
}
```

#### **Trait Implementations**

Implements [`CacheManager`](#4-cachemanager-trait).

---

### **2. `CacheEntry`**

Metadata for a cached block.

```rust
pub struct CacheEntry {
    pub block_id: Vec<u8>,      // ID of the block.
    pub path: PathBuf,          // Path to the encrypted block file.
    pub size: u64,              // Size of the encrypted block (bytes).
    pub last_accessed: i64,     // Timestamp of last access.
    pub ttl: Option<i64>,       // Time-to-live (None = no expiry).
    pub owner_id: Vec<u8>,      // ID of the block owner.
    pub nonce: Vec<u8>,         // Nonce used for encryption.
}
```

---

### **3. `CacheStorage`**

Physical storage of blocks in the cache.

#### **Fields**

```rust
pub struct CacheStorage {
    cache_dir: PathBuf,       // Directory for cache storage.
}
```

#### **Methods**

```rust
impl CacheStorage {
    pub fn new(cache_dir: PathBuf) -> Self;
    
    /// Returns the path to a block in the cache.
    pub fn block_path(&self, block_id: &[u8]) -> PathBuf;
    
    /// Writes a block to the cache.
    pub async fn write_block(&self, block_id: &[u8], data: &[u8]) -> Result<(), Error>;
    
    /// Reads a block from the cache.
    pub async fn read_block(&self, block_id: &[u8]) -> Result<Option<Vec<u8>>, Error>;
    
    /// Deletes a block from the cache.
    pub async fn delete_block(&self, block_id: &[u8]) -> Result<(), Error>;
}
```

---

## 🔐 **Encryption Types**

### **1. `BlockEncryptionKey`**

Encryption key for a block.

```rust
pub struct BlockEncryptionKey {
    pub key: Vec<u8>,      // AES-256 key (32 bytes).
    pub nonce: Vec<u8>,    // Nonce (12 bytes).
}
```

#### **Methods**

```rust
impl BlockEncryptionKey {
    /// Creates a new encryption key (for testing; use `generate_block_key` in production).
    pub fn new() -> Self;
    
    /// Creates an encryption key from a user key (for testing).
    pub fn from_user_key(user_key: &[u8]) -> Self;
}
```

---

### **2. Encryption Functions**

```rust
/// Encrypts a block with a given key.
pub fn encrypt_block(block: &SimpleBlock, key: &BlockEncryptionKey) -> Result<Vec<u8>, Error>;

/// Decrypts a block with a given key.
pub fn decrypt_block(encrypted_data: &[u8], key: &BlockEncryptionKey) -> Result<Vec<u8>, Error>;

/// Generates a random encryption key for a block.
pub fn generate_block_key() -> BlockEncryptionKey;

/// Derives an encryption key from a user's private key and block ID.
pub fn derive_block_key(user_private_key: &[u8], block_id: &[u8]) -> BlockEncryptionKey;

/// Checks if a user can decrypt a block.
pub fn can_decrypt_block(
    user_id: &[u8],
    block_owner_id: &[u8],
    shared_keys: &HashMap<Vec<u8>, Vec<u8>>,
) -> bool;
```

---

## 👥 **Peer Types**

### **1. `P2PPeer`**

Represents a peer in the P2P network.

#### **Fields**

```rust
pub struct P2PPeer {
    id: String,               // Unique ID of the peer.
    address: PeerAddress,    // Network address of the peer.
    stream: Option<TcpStream>, // TCP connection (if connected).
}
```

#### **Methods**

```rust
impl P2PPeer {
    /// Creates a new peer.
    pub fn new(id: String, address: SocketAddr) -> Self;
    
    /// Connects to the peer.
    pub async fn connect(&mut self) -> Result<(), Error>;
    
    /// Disconnects from the peer.
    pub fn disconnect(&mut self);
    
    /// Sends raw bytes to the peer.
    pub async fn send_raw(&mut self, message: &[u8]) -> Result<(), Error>;
    
    /// Receives raw bytes from the peer.
    pub async fn recv_raw(&mut self) -> Result<Vec<u8>, Error>;
}
```

#### **Trait Implementations**

Implements [`Peer`](#1-peer-trait).

---

### **2. `PeerAddress`**

Wrapper around `SocketAddr` for peer addresses.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAddress(pub SocketAddr);

impl Display for PeerAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}
```

---

## 🌐 **Network Types**

### **1. `P2PNetworkImpl`**

Implementation of the P2P network.

#### **Fields**

```rust
pub struct P2PNetworkImpl {
    peers: Arc<RwLock<HashMap<String, P2PPeer>>>, // peer_id -> Peer
    local_peer_id: String,                        // ID of this node.
    listen_addr: SocketAddr,                     // Address to listen on.
}
```

#### **Methods**

```rust
impl P2PNetworkImpl {
    /// Creates a new P2P network.
    pub fn new(local_peer_id: String, listen_addr: SocketAddr) -> Self;
    
    /// Starts the server to listen for incoming connections.
    pub async fn start_server(&mut self) -> Result<(), Error>;
}
```

#### **Trait Implementations**

Implements [`P2PNetwork`](#2-p2pnetwork-trait).

---

## 📥 **Block Fetcher Types**

### **1. `P2PBlockFetcher`**

Fetches blocks from the P2P network or local cache.

#### **Fields**

```rust
pub struct P2PBlockFetcher {
    network: Arc<P2PNetworkImpl>,   // P2P network.
    cache: Arc<dyn CacheManager>,   // Local cache.
}
```

#### **Methods**

```rust
impl P2PBlockFetcher {
    /// Creates a new block fetcher.
    pub fn new(network: Arc<P2PNetworkImpl>, cache: Arc<dyn CacheManager>) -> Self;
}
```

#### **Trait Implementations**

Implements [`BlockFetcher`](#3-blockfetcher-trait).

---

## 📌 **Usage Examples**

### **1. Creating a P2P Network**

```rust
use pqdos::p2p::{P2PNetworkImpl, PeerAddress};
use std::net::SocketAddr;

let mut network = P2PNetworkImpl::new(
    "local_peer".to_string(),
    "127.0.0.1:8080".parse().unwrap(),
);

// Start listening for connections.
network.listen().await?;

// Add a known peer.
let peer = pqdos::p2p::P2PPeer::new(
    "peer1".to_string(),
    "127.0.0.1:8081".parse().unwrap(),
);
network.add_peer(peer).await?;
```

---

### **2. Creating a Cache**

```rust
use pqdos::p2p::cache::LRUCache;

let cache = LRUCache::new("/tmp/pqdos_cache", 100 * 1024 * 1024).await?;
```

---

### **3. Caching a Block**

```rust
use pqdos::block::simple::SimpleBlock;

let block = SimpleBlock::new(
    vec![0x01; 32], // block_id
    b"Hello, world!".to_vec(), // data
    None, // previous
    0, // timestamp
    1, // version
);

let owner_id = vec![0x42; 32]; // Owner's ID.
cache.cache_block(block, &owner_id).await?;
```

---

### **4. Fetching a Block**

```rust
use std::collections::HashMap;

let block_id = vec![0x01; 32];
let user_id = vec![0x42; 32]; // Same as owner_id.
let user_private_key = vec![0x00; 64]; // User's private key.
let shared_keys = HashMap::new(); // No shared keys.

let fetched_block = cache.get_cached_block(
    &block_id,
    &user_id,
    Some(&user_private_key),
    &shared_keys,
).await?;

match fetched_block {
    Some(block) => println!("Block fetched: {:?}", block),
    None => println!("Block not accessible."),
}
```

---

### **5. Fetching a Block from the Network**

```rust
use pqdos::p2p::{P2PBlockFetcher, P2PNetworkImpl};
use std::sync::Arc;

let network = Arc::new(P2PNetworkImpl::new(
    "local_peer".to_string(),
    "127.0.0.1:8080".parse().unwrap(),
));

let cache = Arc::new(LruCache::new("/tmp/pqdos_cache", 100 * 1024 * 1024).await?);

let fetcher = P2PBlockFetcher::new(network, cache);

let block_id = vec![0x01; 32];
let user_id = vec![0x42; 32];
let user_private_key = Some(&vec![0x00; 64]);
let shared_keys = HashMap::new();

let block = fetcher.fetch_block(
    &block_id,
    &user_id,
    user_private_key,
    &shared_keys,
).await?;

match block {
    Some(block) => println!("Block fetched: {:?}", block),
    None => println!("Block not found or not accessible."),
}
```

---

## 🔧 **Error Handling**

The P2P module uses the [`Error`](crate::error::Error) type from the main PQDOS crate. Common errors include:

| Error | Description |
|-------|-------------|
| `Error::CryptoError` | Encryption/decryption failed. |
| `Error::BlockNotFound` | Block not found in cache or network. |
| `Error::PeerNotFound` | Peer not found in the network. |
| `Error::PeerNotConnected` | Peer is not connected. |
| `Error::IoError` | Filesystem I/O error. |
| `Error::SerializationError` | Failed to serialize/deserialize a message. |

---

## 📚 **See Also**

- [P2P Design](./DESIGN.md): Detailed architecture and design decisions.
- [Encryption](./ENCRYPTION.md): How blocks are encrypted and decrypted.
- [Examples](./EXAMPLES.md): More usage examples.
- [PQDOS Architecture](../../ARCHITECTURE.md): Overall PQDOS architecture.
