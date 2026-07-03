//! Module de cache pour le réseau P2P.
//!
//! Fournit des implémentations de cache local pour les blocs mémoire.
//! Tous les blocs sont chiffrés avant d'être stockés.

pub mod lru;
pub mod storage;
pub mod crypto;

pub use lru::LRUCache;
pub use storage::CacheStorage;
pub use crypto::{BlockEncryptionKey, encrypt_block, decrypt_block, derive_block_key, can_decrypt_block};
