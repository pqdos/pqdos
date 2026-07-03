# P2P Network Module

The **P2P Network Module** for PQDOS enables distributed access to memory blocks across a peer-to-peer network. This module provides:

- **Decentralized Block Access**: Fetch any memory block from the P2P network.
- **Local Cache**: Store frequently accessed blocks in a local cache (`/tmp/pqdos_cache/`).
- **End-to-End Encryption**: All blocks are **encrypted at rest** in the cache.
- **Permission-Based Access**: Users can only decrypt blocks they own or have been explicitly shared with.

---

## 🎯 **Purpose**

PQDOS treats all storage as a **unified memory hierarchy**, where files, RAM, and network storage are content-addressed, encrypted blocks. The P2P module extends this architecture to a **distributed environment**, allowing:

1. **Decentralized Access**: No single point of failure. Blocks can be fetched from any peer in the network.
2. **Offline Operation**: Frequently accessed blocks are cached locally, enabling offline access.
3. **Security**: All blocks are encrypted, and access is restricted to authorized users.
4. **Performance**: Local caching reduces latency and network load.

---

## 🏗️ **Architecture Overview**

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                            PQDOS Node (Local)                                    │
├───────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────────┐    ┌─────────────┐    ┌───────────────────────────────────┐  │
│  │  P2P Layer  │    │  Cache      │    │  Block Manager (Local Storage)    │  │
│  │             │    │  Manager    │    │                                   │  │
│  │ - Peer      │    │             │    │ - Stores blocks locally          │  │
│  │   Discovery │◄───►│ - LRU Cache │    │   (encrypted files)               │  │
│  │ - Message   │    │ - TTL       │    │ - Manages block access           │  │
│  │   Routing   │    │ - Size Limit│    │   (via BlockTrait)                │  │
│  │ - Connection│    │             │    │                                   │  │
│  └─────────────┘    └─────────────┘    └───────────────────────────────────┘  │
│           ▲                                                                     │
│           │                                                                     │
│  ┌─────────────────────────────────────────────────────────────────────────┐  │
│  │                        Blockchain Layer (Optional)                       │  │
│  │  - Verifies block integrity (via PQDOS blockchain)                     │  │
│  │  - Stores metadata (owner, permissions, etc.)                            │  │
│  └─────────────────────────────────────────────────────────────────────────┘  │
│                                                                               │
└───────────────────────────────────────────────────────────────────────────────┘
                              ▲
                              │
                              ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│                            P2P Network (Other Peers)                            │
├───────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                      │
│  │   Peer A    │    │   Peer B    │    │   Peer C    │                      │
│  │             │    │             │    │             │                      │
│  │ - Cache     │    │ - Cache     │    │ - Cache     │                      │
│  │ - Blocks    │    │ - Blocks    │    │ - Blocks    │                      │
│  └─────────────┘    └─────────────┘    └─────────────┘                      │
└───────────────────────────────────────────────────────────────────────────────┘
```

---

## 📦 **Modules**

| Module | Description | Status |
|--------|-------------|--------|
| [`traits`](./traits.md) | Abstract traits for P2P networking (`Peer`, `P2PNetwork`, `BlockFetcher`, `CacheManager`). | ✅ Implemented |
| [`peer`](./peer.md) | Peer implementation (`P2PPeer`). | ✅ Implemented |
| [`network`](./network.md) | Network messages and `P2PNetworkImpl`. | ✅ Implemented |
| [`cache`](./cache/) | Local cache with encryption (`LRUCache`, `CacheStorage`). | ✅ Implemented |
| [`discovery`](./discovery.md) | Peer discovery (mDNS, DHT). | ⏳ TODO |
| [`block_fetcher`](./block_fetcher.md) | Block fetching from network/cache. | ✅ Implemented |

---

## 🔐 **Security Model**

### **Encryption**
- **All blocks are encrypted** before being stored in the cache.
- **AES-256-GCM** is used for encryption (post-quantum alternatives like Kyber can be integrated later).
- Each block is encrypted with a **unique key** derived from:
  - The **owner's private key** (or a shared key).
  - The **`block_id`** (to ensure uniqueness per block).

### **Access Control**
- A user can only decrypt a block if:
  1. They are the **owner** of the block (verified via `owner_id`).
  2. They have been **explicitly shared** the block by the owner (via `shared_keys`).
- **No plaintext storage**: Blocks are **always encrypted** in the cache.

---

## 🚀 **Getting Started**

### **Basic Usage**

```rust
use pqdos::p2p::cache::{LRUCache, CacheManager};
use pqdos::block::simple::SimpleBlock;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a local cache.
    let mut cache = LRUCache::new("/tmp/pqdos_cache", 100 * 1024 * 1024).await?;

    // 2. Create a block.
    let block = SimpleBlock::new(
        vec![0x01; 32], // block_id
        b"Hello, world!".to_vec(), // data
        None, // previous
        0, // timestamp
        1, // version
    );

    // 3. Cache the block (encrypted).
    let owner_id = vec![0x42; 32]; // Owner's ID.
    cache.cache_block(block, &owner_id).await?;

    // 4. Fetch the block (decrypted if user has permissions).
    let user_id = vec![0x42; 32]; // Same as owner.
    let user_private_key = vec![0x00; 64]; // User's private key.
    let shared_keys = HashMap::new(); // No shared keys.

    let fetched_block = cache.get_cached_block(
        &vec![0x01; 32], // block_id
        &user_id,
        Some(&user_private_key),
        &shared_keys,
    ).await?;

    match fetched_block {
        Some(block) => println!("Block fetched: {:?}", block),
        None => println!("Block not accessible (no permissions)."),
    }

    Ok(())
}
```

---

## 📚 **Documentation**

- [Design](./DESIGN.md): Detailed architecture and design decisions.
- [Encryption](./ENCRYPTION.md): How blocks are encrypted and decrypted.
- [API](./API.md): Public API reference.
- [Examples](./EXAMPLES.md): Usage examples.

---

## 🔧 **Configuration**

### **Cache Settings**
| Parameter | Description | Default |
|-----------|-------------|---------|
| `cache_dir` | Directory for cache storage. | `/tmp/pqdos_cache` |
| `max_size` | Maximum cache size in bytes. | `100 * 1024 * 1024` (100 MB) |

### **Network Settings**
| Parameter | Description | Default |
|-----------|-------------|---------|
| `listen_addr` | Address to listen for P2P connections. | `127.0.0.1:8080` |
| `local_peer_id` | Unique ID for this peer. | Randomly generated |

---

## 🛠️ **Roadmap**

- [x] Basic P2P networking.
- [x] Local cache with LRU eviction.
- [x] End-to-end encryption for cached blocks.
- [x] Permission-based access control.
- [ ] Peer discovery (mDNS, DHT).
- [ ] Blockchain integration for integrity verification.
- [ ] Post-quantum cryptography (Kyber, Dilithium).
- [ ] Parallel block fetching.
- [ ] Cache compression (zstd).

---

## 🤝 **Contributing**

Contributions are welcome! See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

## 📄 **License**

This module is part of PQDOS and is licensed under the **MIT License**. See [LICENSE](../../LICENSE) for details.
