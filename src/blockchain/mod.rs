//! Blockchain module for Post-Quantum Secure OS
//!
//! This module provides abstract blockchain traits for
//! distributed ledger management with consensus algorithms.

pub mod traits;

pub use traits::{
    Blockchain,
    BlockchainConfig,
    BlockchainFactory,
    BlockchainNode,
    BlockchainSync,
    BlockchainValidator,
    ConsensusAlgorithm,
    ConsensusState,
    // NetworkMessage, // Not used in blockchain module
    // PeerId, // Defined in network module
    PeerInfo,
    SyncResult,
    SyncStatus,
    Transaction,
    TransactionBuilder,
    TransactionId,
    TransactionPool,
    TransactionSigner,
    TransactionType,
    BlockchainEvents,
};
