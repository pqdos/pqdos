//! Network module for Post-Quantum Secure OS
//!
//! This module provides abstract network traits for
//! P2P communication with pluggable implementations.

pub mod traits;

pub use traits::{
    AddressBook,
    Connection,
    MessageAuthenticator,
    MessageSerializer,
    MessageType,
    NetworkConfig,
    NetworkEncryption,
    NetworkFactory,
    // NetworkMessage, // Causes unresolved import
    NetworkMiddleware,
    NetworkProtocol,
    NetworkStats,
    NetworkTransport,
    P2PNetwork,
    Peer,
    PeerDiscovery,
    PeerId,
};
