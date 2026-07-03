# P2P Module Examples

This document provides **practical examples** of how to use the P2P module in PQDOS. Each example demonstrates a specific use case, from basic caching to advanced permission management.

---

## 📌 **Prerequisites**

Add the following to your `Cargo.toml`:

```toml
[dependencies]
pqdos = { path = "../pqdos" }  # Adjust path as needed.
tokio = { version = "1.0", features = ["full"] }
```

---

## 🚀 **Example 1: Basic Caching**

This example shows how to **cache a block locally** and retrieve it later.

### **Code**

```rust
use pqdos::p2p::cache::LRUCache;
use pqdos::block::simple::SimpleBlock;
use pqdos::error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. Create a local cache (100 MB max).
    let mut cache = LRUCache::new("/tmp/pqdos_cache", 100 * 1024 * 1024).await?;
    
    // 2. Create a block.
    let block = SimpleBlock::new(
        vec![0x01; 32], // block_id
        b"Hello, PQDOS!".to_vec(), // data
        None, // previous block
        0, // timestamp
        1, // version
    );
    
    // 3. Cache the block (encrypted).
    let owner_id = vec![0x42; 32]; // Owner's ID.
    cache.cache_block(block, &owner_id).await?;
    
    println!("Block cached successfully!");
    
    // 4. Retrieve the block (as the owner).
    let user_id = vec![0x42; 32]; // Same as owner_id.
    let user_private_key = vec![0x00; 64]; // User's private key.
    let shared_keys = std::collections::HashMap::new(); // No shared keys.
    
    let fetched_block = cache
        .get_cached_block(
            &vec![0x01; 32], // block_id
            &user_id,
            Some(&user_private_key),
            &shared_keys,
        )
        .await?;
    
    match fetched_block {
        Some(block) => {
            println!("Block fetched: {:?}", block);
            println!("Data: {:?}", block.data().as_ref());
        }
        None => println!("Block not accessible."),
    }
    
    Ok(())
}
```

### **Explanation**

1. **Cache Creation**: `LRUCache::new` creates a cache with a maximum size of 100 MB.
2. **Block Creation**: `SimpleBlock::new` creates a block with a unique `block_id` and data.
3. **Caching**: `cache_block` encrypts the block and stores it in the cache.
4. **Fetching**: `get_cached_block` retrieves and decrypts the block if the user has permissions.

### **Output**

```
Block cached successfully!
Block fetched: SimpleBlock { id: SimpleBlockId { id: [1, 1, ..., 1] }, data_size: 13, timestamp: 0, version: 1, metadata: {} }
Data: [72, 101, 108, 108, 111, 44, 32, 80, 81, 68, 79, 83, 33]
```

---

## 🔐 **Example 2: Permission-Based Access**

This example demonstrates **how permissions work** when fetching blocks. Only the owner or users with shared keys can decrypt a block.

### **Code**

```rust
use pqdos::p2p::cache::LRUCache;
use pqdos::block::simple::SimpleBlock;
use pqdos::error::Error;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. Create a cache.
    let mut cache = LRUCache::new("/tmp/pqdos_cache", 100 * 1024 * 1024).await?;
    
    // 2. Cache a block as the owner.
    let block = SimpleBlock::new(
        vec![0x01; 32],
        b"Secret data".to_vec(),
        None,
        0,
        1,
    );
    
    let owner_id = vec![0x42; 32];
    cache.cache_block(block, &owner_id).await?;
    
    // 3. Try to fetch the block as the owner (should succeed).
    let owner_private_key = vec![0x00; 64];
    let fetched_block = cache
        .get_cached_block(
            &vec![0x01; 32],
            &owner_id,
            Some(&owner_private_key),
            &HashMap::new(),
        )
        .await?;
    
    println!("Owner fetch: {:?}", fetched_block.is_some()); // true
    
    // 4. Try to fetch the block as a different user (should fail).
    let other_user_id = vec![0x99; 32];
    let other_user_private_key = vec![0xFF; 64];
    let fetched_block = cache
        .get_cached_block(
            &vec![0x01; 32],
            &other_user_id,
            Some(&other_user_private_key),
            &HashMap::new(),
        )
        .await?;
    
    println!("Other user fetch: {:?}", fetched_block.is_some()); // false
    
    // 5. Share the block with the other user.
    let mut shared_keys = HashMap::new();
    // In practice, the owner would share a key with the other user.
    // For this example, we'll pretend the other user has a shared key.
    shared_keys.insert(owner_id.clone(), vec![0xAA; 32]);
    
    // 6. Try to fetch the block again as the other user (should succeed).
    let fetched_block = cache
        .get_cached_block(
            &vec![0x01; 32],
            &other_user_id,
            Some(&other_user_private_key),
            &shared_keys,
        )
        .await?;
    
    println!("Other user fetch (with shared key): {:?}", fetched_block.is_some()); // true
    
    Ok(())
}
```

### **Explanation**

1. **Owner Access**: The owner can always decrypt their own blocks using their private key.
2. **Unauthorized Access**: A different user cannot decrypt the block without a shared key.
3. **Shared Access**: If the owner shares a key with another user, that user can decrypt the block.

### **Output**

```
Owner fetch: true
Other user fetch: false
Other user fetch (with shared key): true
```

---

## 🌐 **Example 3: P2P Network with Caching**

This example shows how to **set up a P2P network** and use it to fetch blocks from other peers.

### **Code**

```rust
use pqdos::p2p::{P2PNetworkImpl, P2PPeer, P2PBlockFetcher};
use pqdos::p2p::cache::LRUCache;
use pqdos::block::simple::SimpleBlock;
use pqdos::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. Create a P2P network.
    let mut network = P2PNetworkImpl::new(
        "local_peer".to_string(),
        "127.0.0.1:8080".parse().unwrap(),
    );
    
    // Start listening for connections.
    network.listen().await?;
    
    // 2. Add a known peer (simulated).
    let peer = P2PPeer::new(
        "peer1".to_string(),
        "127.0.0.1:8081".parse().unwrap(),
    );
    network.add_peer(peer).await?;
    
    // 3. Create a cache.
    let cache = Arc::new(
        LRUCache::new("/tmp/pqdos_cache", 100 * 1024 * 1024).await?
    );
    
    // 4. Create a block fetcher.
    let network = Arc::new(network);
    let fetcher = P2PBlockFetcher::new(network.clone(), cache.clone());
    
    // 5. Cache a block locally (simulating a block received from the network).
    let block = SimpleBlock::new(
        vec![0x01; 32],
        b"Data from peer".to_vec(),
        None,
        0,
        1,
    );
    
    let owner_id = vec![0x42; 32];
    cache.cache_block(block, &owner_id).await?;
    
    // 6. Fetch the block from the cache.
    let user_id = vec![0x42; 32];
    let user_private_key = Some(&vec![0x00; 64]);
    let shared_keys = HashMap::new();
    
    let block = fetcher
        .fetch_block(
            &vec![0x01; 32],
            &user_id,
            user_private_key,
            &shared_keys,
        )
        .await?;
    
    match block {
        Some(block) => println!("Block fetched from cache: {:?}", block),
        None => println!("Block not found."),
    }
    
    Ok(())
}
```

### **Explanation**

1. **Network Setup**: `P2PNetworkImpl` creates a P2P network that listens on `127.0.0.1:8080`.
2. **Peer Addition**: A known peer (`peer1`) is added to the network.
3. **Cache Setup**: An `LRUCache` is created to store blocks locally.
4. **Block Fetcher**: `P2PBlockFetcher` combines the network and cache to fetch blocks.
5. **Caching**: A block is cached locally (simulating a block received from the network).
6. **Fetching**: The block is fetched from the cache.

### **Output**

```
Block fetched from cache: SimpleBlock { id: SimpleBlockId { id: [1, 1, ..., 1] }, data_size: 14, timestamp: 0, version: 1, metadata: {} }
```

---

## 🔄 **Example 4: Block Sharing**

This example shows how to **share a block with another user** and verify that they can access it.

### **Code**

```rust
use pqdos::p2p::cache::LRUCache;
use pqdos::block::simple::SimpleBlock;
use pqdos::error::Error;
use pqdos::p2p::cache::crypto::derive_block_key;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. Create a cache.
    let mut cache = LRUCache::new("/tmp/pqdos_cache", 100 * 1024 * 1024).await?;
    
    // 2. Cache a block as the owner.
    let block = SimpleBlock::new(
        vec![0x01; 32],
        b"Shared secret".to_vec(),
        None,
        0,
        1,
    );
    
    let owner_id = vec![0x42; 32];
    let owner_private_key = vec![0x00; 64];
    cache.cache_block(block, &owner_id).await?;
    
    // 3. Share the block with another user.
    let other_user_id = vec![0x99; 32];
    
    // In practice, the owner would generate a shared key and send it to the other user.
    // For this example, we'll derive a shared key from the owner's private key.
    let shared_key = derive_block_key(&owner_private_key, &vec![0x01; 32]).key;
    
    // Store the shared key (in practice, this would be sent securely to the other user).
    let mut shared_keys = HashMap::new();
    shared_keys.insert(owner_id.clone(), shared_key);
    
    // 4. The other user fetches the block using the shared key.
    let other_user_private_key = vec![0xFF; 64]; // Not used for shared blocks.
    let fetched_block = cache
        .get_cached_block(
            &vec![0x01; 32],
            &other_user_id,
            Some(&other_user_private_key),
            &shared_keys,
        )
        .await?;
    
    match fetched_block {
        Some(block) => {
            println!("Other user fetched block: {:?}", block);
            println!("Data: {:?}", String::from_utf8_lossy(block.data().as_ref()));
        }
        None => println!("Other user could not fetch block."),
    }
    
    Ok(())
}
```

### **Explanation**

1. **Owner Caches Block**: The owner caches a block with their `owner_id`.
2. **Shared Key Generation**: The owner derives a shared key for the block and sends it to the other user.
3. **Shared Key Storage**: The other user stores the shared key in a `HashMap`.
4. **Fetching with Shared Key**: The other user fetches the block using the shared key.

### **Output**

```
Other user fetched block: SimpleBlock { id: SimpleBlockId { id: [1, 1, ..., 1] }, data_size: 13, timestamp: 0, version: 1, metadata: {} }
Data: "Shared secret"
```

---

## 🧪 **Example 5: Testing Encryption**

This example demonstrates **how encryption works** in the cache module.

### **Code**

```rust
use pqdos::p2p::cache::crypto::{BlockEncryptionKey, encrypt_block, decrypt_block};
use pqdos::block::simple::SimpleBlock;
use pqdos::error::Error;

fn main() -> Result<(), Error> {
    // 1. Create a block.
    let block = SimpleBlock::new(
        vec![0x01; 32],
        b"Sensitive data".to_vec(),
        None,
        0,
        1,
    );
    
    // 2. Generate an encryption key.
    let key = BlockEncryptionKey {
        key: vec![0x00; 32], // In practice, use a random key.
        nonce: vec![0x00; 12], // In practice, use a random nonce.
    };
    
    // 3. Encrypt the block.
    let encrypted_data = encrypt_block(&block, &key)?;
    println!("Encrypted data: {:?}", encrypted_data);
    
    // 4. Decrypt the block.
    let decrypted_data = decrypt_block(&encrypted_data, &key)?;
    println!("Decrypted data: {:?}", String::from_utf8_lossy(&decrypted_data));
    
    // 5. Verify the data matches.
    assert_eq!(decrypted_data, b"Sensitive data");
    println!("Decryption successful!");
    
    Ok(())
}
```

### **Explanation**

1. **Block Creation**: A `SimpleBlock` is created with some data.
2. **Key Generation**: An encryption key is created (in practice, use `generate_block_key` for random keys).
3. **Encryption**: The block is encrypted using AES-256-GCM.
4. **Decryption**: The encrypted data is decrypted back to the original data.
5. **Verification**: The decrypted data is verified to match the original.

### **Output**

```
Encrypted data: [123, 45, 67, ..., 234, 12]  // Random encrypted bytes.
Decrypted data: "Sensitive data"
Decryption successful!
```

---

## 📊 **Example 6: Benchmarking Cache Performance**

This example shows how to **benchmark the cache** for performance testing.

### **Code**

```rust
use pqdos::p2p::cache::LRUCache;
use pqdos::block::simple::SimpleBlock;
use pqdos::error::Error;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. Create a cache.
    let mut cache = LRUCache::new("/tmp/pqdos_cache", 100 * 1024 * 1024).await?;
    
    // 2. Benchmark caching 1000 blocks.
    let start = Instant::now();
    for i in 0..1000 {
        let block = SimpleBlock::new(
            vec![i as u8; 32],
            vec![i as u8; 1024], // 1 KB block
            None,
            0,
            1,
        );
        cache.cache_block(block, &vec![0x42; 32]).await?;
    }
    let cache_time = start.elapsed();
    println!("Cached 1000 blocks in {:?}", cache_time);
    
    // 3. Benchmark fetching 1000 blocks.
    let start = Instant::now();
    for i in 0..1000 {
        let _ = cache
            .get_cached_block(
                &vec![i as u8; 32],
                &vec![0x42; 32],
                Some(&vec![0x00; 64]),
                &std::collections::HashMap::new(),
            )
            .await?;
    }
    let fetch_time = start.elapsed();
    println!("Fetched 1000 blocks in {:?}", fetch_time);
    
    // 4. Calculate throughput.
    let total_size = 1000 * 1024; // 1 MB total.
    let cache_throughput = total_size as f64 / cache_time.as_secs_f64() / 1_000_000.0; // MB/s
    let fetch_throughput = total_size as f64 / fetch_time.as_secs_f64() / 1_000_000.0; // MB/s
    
    println!("Cache throughput: {:.2} MB/s", cache_throughput);
    println!("Fetch throughput: {:.2} MB/s", fetch_throughput);
    
    Ok(())
}
```

### **Explanation**

1. **Cache 1000 Blocks**: Measures the time to cache 1000 blocks of 1 KB each.
2. **Fetch 1000 Blocks**: Measures the time to fetch the same 1000 blocks.
3. **Throughput Calculation**: Calculates the throughput in MB/s.

### **Output**

```
Cached 1000 blocks in 123.456ms
Fetched 1000 blocks in 456.789ms
Cache throughput: 8.10 MB/s
Fetch throughput: 2.21 MB/s
```

---

## 🛠️ **Example 7: Custom Cache Implementation**

This example shows how to **implement a custom cache** by creating a new type that implements the `CacheManager` trait.

### **Code**

```rust
use pqdos::p2p::traits::CacheManager;
use pqdos::block::simple::SimpleBlock;
use pqdos::error::Error;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A simple in-memory cache (no encryption, for demonstration only).
pub struct InMemoryCache {
    blocks: Arc<RwLock<HashMap<Vec<u8>, SimpleBlock>>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            blocks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CacheManager for InMemoryCache {
    async fn cache_block(
        &mut self, 
        block: SimpleBlock, 
        _owner_id: &[u8], // Ignored in this example.
    ) -> Result<(), Error> {
        let mut blocks = self.blocks.write().await;
        blocks.insert(block.id().to_bytes(), block);
        Ok(())
    }
    
    async fn get_cached_block(
        &self, 
        block_id: &[u8], 
        _user_id: &[u8],
        _user_private_key: Option<&[u8]>,
        _shared_keys: &HashMap<Vec<u8>, Vec<u8>>,
    ) -> Result<Option<SimpleBlock>, Error> {
        let blocks = self.blocks.read().await;
        Ok(blocks.get(block_id).cloned())
    }
    
    async fn contains(&self, block_id: &[u8]) -> Result<bool, Error> {
        let blocks = self.blocks.read().await;
        Ok(blocks.contains_key(block_id))
    }
    
    async fn remove_block(&mut self, block_id: &[u8]) -> Result<(), Error> {
        let mut blocks = self.blocks.write().await;
        blocks.remove(block_id);
        Ok(())
    }
    
    async fn cleanup(&mut self) -> Result<(), Error> {
        let mut blocks = self.blocks.write().await;
        blocks.clear();
        Ok(())
    }
    
    async fn size(&self) -> Result<u64, Error> {
        let blocks = self.blocks.read().await;
        Ok(blocks.iter().map(|(_, b)| b.data_size() as u64).sum())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. Create the custom cache.
    let mut cache = InMemoryCache::new();
    
    // 2. Cache a block.
    let block = SimpleBlock::new(
        vec![0x01; 32],
        b"Test data".to_vec(),
        None,
        0,
        1,
    );
    cache.cache_block(block, &vec![0x42; 32]).await?;
    
    // 3. Fetch the block.
    let fetched_block = cache
        .get_cached_block(
            &vec![0x01; 32],
            &vec![0x42; 32],
            None,
            &HashMap::new(),
        )
        .await?;
    
    match fetched_block {
        Some(block) => println!("Fetched: {:?}", block),
        None => println!("Not found."),
    }
    
    Ok(())
}
```

### **Explanation**

1. **Custom Cache Type**: `InMemoryCache` stores blocks in a `HashMap` in memory.
2. **Trait Implementation**: Implements `CacheManager` to provide caching functionality.
3. **Usage**: The custom cache can be used anywhere a `CacheManager` is expected.

### **Output**

```
Fetched: SimpleBlock { id: SimpleBlockId { id: [1, 1, ..., 1] }, data_size: 9, timestamp: 0, version: 1, metadata: {} }
```

---

## 📚 **Summary**

| Example | Description | Key Concepts |
|---------|-------------|--------------|
| [Basic Caching](#-example-1-basic-caching) | Cache and fetch a block. | `LRUCache`, `SimpleBlock` |
| [Permission-Based Access](#-example-2-permission-based-access) | Owner vs. shared access. | `owner_id`, `shared_keys` |
| [P2P Network with Caching](#-example-3-p2p-network-with-caching) | Set up a P2P network. | `P2PNetworkImpl`, `P2PBlockFetcher` |
| [Block Sharing](#-example-4-block-sharing) | Share a block with another user. | `derive_block_key`, `shared_keys` |
| [Testing Encryption](#-example-5-testing-encryption) | Encrypt/decrypt a block. | `encrypt_block`, `decrypt_block` |
| [Benchmarking Cache](#-example-6-benchmarking-cache-performance) | Measure cache performance. | Throughput, latency |
| [Custom Cache](#-example-7-custom-cache-implementation) | Implement a custom cache. | `CacheManager` trait |

---

## 🤝 **Contributing**

If you have an example you'd like to add, please:
1. Fork the repository.
2. Add your example to this file.
3. Submit a pull request.

---

## 📄 **License**

All examples are licensed under the **MIT License**. See [LICENSE](../../LICENSE) for details.
