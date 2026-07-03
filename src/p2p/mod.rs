//! Module P2P pour PQDOS.
//!
//! Ce module fournit une implémentation d'un réseau P2P pour accéder aux blocs mémoire
//! de manière distribuée, avec un cache local pour améliorer les performances.

pub mod traits;
pub mod peer;
pub mod network;
pub mod cache;
pub mod discovery;
pub mod block_fetcher;

// Ré-exports.
pub use traits::{
    P2PNetwork, Peer, NetworkMessage, BlockFetcher, CacheManager,
};
pub use peer::{P2PPeer, PeerAddress};
pub use network::P2PNetworkImpl;
pub use cache::lru::LRUCache;
pub use cache::storage::CacheStorage;
pub use block_fetcher::P2PBlockFetcher;
