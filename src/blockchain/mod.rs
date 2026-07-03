//! Blockchain module for Post-Quantum Secure OS
//!
//! This module provides abstract blockchain traits for
//! distributed ledger management with consensus algorithms.

pub mod traits;

pub use traits::{
    Blockchain,
    BlockchainConfig,
    BlockchainEvents,
    BlockchainFactory,
    BlockchainNode,
    BlockchainSync,
    BlockchainValidator,
    ConsensusAlgorithm,
    ConsensusState,
    PeerInfo,
    SyncResult,
    SyncStatus,
    Transaction,
    TransactionBuilder,
    TransactionId,
    TransactionPool,
    TransactionSigner,
    TransactionType,
};
