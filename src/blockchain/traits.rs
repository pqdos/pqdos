//! Abstract blockchain traits for Post-Quantum Secure OS
//!
//! These traits define interfaces for blockchain functionality,
//! including transactions, consensus algorithms, and distributed ledger management.

use crate::block::traits::{Block, BlockStorage};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;

// Type aliases for complex callback signatures to satisfy clippy
pub type NewBlockCallback<B> = Arc<dyn Fn(&B) + Send + Sync>;
pub type NewTransactionCallback<T> = Arc<dyn Fn(&T) + Send + Sync>;
pub type ConsensusCallback = Arc<dyn Fn(ConsensusState) + Send + Sync>;
pub type SyncCallback = Arc<dyn Fn(&[u8], u64) + Send + Sync>;

/// Trait for a transaction identifier
pub trait TransactionId:
    Clone + Eq + std::hash::Hash + AsRef<[u8]> + Debug + Serialize + DeserializeOwned + Send + Sync
{
    fn from_bytes(bytes: Vec<u8>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}

/// Trait for a transaction in the blockchain
///
/// Transactions represent state changes that are recorded in the blockchain.
pub trait Transaction:
    Clone + Debug + Serialize + DeserializeOwned + Eq + std::hash::Hash + Send + Sync
{
    /// The type of identifier for this transaction
    type Id: TransactionId;

    /// Return the unique identifier of this transaction
    fn id(&self) -> &Self::Id;

    /// Return the sender's identifier (e.g., public key hash)
    fn sender(&self) -> &[u8];

    /// Return the recipient's identifier (optional, for transfers)
    fn recipient(&self) -> Option<&[u8]>;

    /// Return the transaction data/payload
    fn payload(&self) -> &[u8];

    /// Return the timestamp when this transaction was created
    fn timestamp(&self) -> i64;

    /// Return the signature of this transaction
    fn signature(&self) -> &[u8];

    /// Return the public key that signed this transaction
    fn signer(&self) -> &[u8];

    /// Return the transaction fee
    fn fee(&self) -> u64;

    /// Return the nonce (to prevent replay attacks)
    fn nonce(&self) -> u64;

    /// Serialize the transaction to bytes (excluding signature)
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>;

    /// Deserialize a transaction from bytes
    fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>>
    where
        Self: Sized;

    /// Validate the transaction (signature, format, etc.)
    fn validate(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    /// Return the transaction type
    fn transaction_type(&self) -> TransactionType;
}

/// Types of transactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Data storage transaction
    DataStorage,
    /// Key management transaction
    KeyManagement,
    /// Access control transaction
    AccessControl,
    /// System configuration transaction
    SystemConfig,
    /// Custom transaction type
    Custom(u8),
}

/// Trait for a transaction builder
pub trait TransactionBuilder: Send + Sync {
    /// The type of transaction this builder creates
    type Transaction: Transaction;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Create a new unsigned transaction
    fn new_transaction(&mut self, payload: Vec<u8>) -> Result<Self::Transaction, Self::Error>;

    /// Set the sender for the transaction
    fn with_sender(&mut self, sender: Vec<u8>) -> &mut Self;

    /// Set the recipient for the transaction
    fn with_recipient(&mut self, recipient: Vec<u8>) -> &mut Self;

    /// Set the fee for the transaction
    fn with_fee(&mut self, fee: u64) -> &mut Self;

    /// Set the nonce for the transaction
    fn with_nonce(&mut self, nonce: u64) -> &mut Self;

    /// Set the timestamp for the transaction
    fn with_timestamp(&mut self, timestamp: i64) -> &mut Self;

    /// Set the transaction type
    fn with_type(&mut self, transaction_type: TransactionType) -> &mut Self;
}

/// Trait for a transaction signer
pub trait TransactionSigner: Send + Sync {
    /// The type of transaction to sign
    type Transaction: Transaction;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Sign a transaction with the given private key
    fn sign(
        &self,
        transaction: &mut Self::Transaction,
        private_key: &[u8],
        public_key: &[u8],
    ) -> Result<(), Self::Error>;

    /// Verify the signature on a transaction
    fn verify_signature(&self, transaction: &Self::Transaction) -> Result<bool, Self::Error>;
}

/// Trait for a transaction pool
///
/// Manages pending transactions that have not yet been included in a block.
pub trait TransactionPool: Send + Sync {
    /// The type of transaction stored in the pool
    type Transaction: Transaction;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Add a transaction to the pool
    fn add_transaction(&mut self, transaction: Self::Transaction) -> Result<(), Self::Error>;

    /// Get a transaction by its identifier
    fn get_transaction(
        &self,
        id: &<Self::Transaction as Transaction>::Id,
    ) -> Result<Self::Transaction, Self::Error>;

    /// Remove a transaction from the pool
    fn remove_transaction(
        &mut self,
        id: &<Self::Transaction as Transaction>::Id,
    ) -> Result<(), Self::Error>;

    /// Get all transactions in the pool
    fn get_all(&self) -> Result<Vec<Self::Transaction>, Self::Error>;

    /// Get transactions ready to be included in a block
    fn get_ready(&self) -> Result<Vec<Self::Transaction>, Self::Error>;

    /// Get the number of pending transactions
    fn pending_count(&self) -> usize;

    /// Check if a transaction exists in the pool
    fn contains(&self, id: &<Self::Transaction as Transaction>::Id) -> bool;

    /// Clear all transactions from the pool
    fn clear(&mut self) -> Result<(), Self::Error>;
}

/// Peer identifier type
pub type PeerId = Vec<u8>;

/// Trait for peer information
pub trait PeerInfo: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// Return the peer's unique identifier
    fn id(&self) -> &[u8];

    /// Return the peer's network address
    fn address(&self) -> SocketAddr;

    /// Return the peer's public key
    fn public_key(&self) -> &[u8];

    /// Return the peer's current blockchain height
    fn height(&self) -> u64;

    /// Return the last time this peer was seen (Unix timestamp)
    fn last_seen(&self) -> i64;

    /// Check if the peer is currently connected
    fn is_connected(&self) -> bool;
}

/// Consensus algorithm state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusState {
    /// No consensus in progress
    Idle,
    /// Proposing a new block
    Proposing,
    /// Voting on proposals
    Voting,
    /// Committed to a block
    Committed,
    /// Consensus failed
    Failed,
}

/// Trait for a consensus algorithm
///
/// Defines the interface for distributed consensus protocols
/// (Raft, BFT, Paxos, etc.)
pub trait ConsensusAlgorithm: Send + Sync {
    /// The type of block used in this blockchain
    type Block: Block;
    /// The type of transaction used in this blockchain
    type Transaction: Transaction;
    /// Peer identifier type
    type PeerId: Clone + Eq + std::hash::Hash + Debug + Serialize + DeserializeOwned + Send + Sync;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Propose a new transaction to the consensus
    fn propose_transaction(&mut self, transaction: Self::Transaction) -> Result<(), Self::Error>;

    /// Vote on a proposed block
    fn vote(
        &mut self,
        peer: Self::PeerId,
        block_id: &<Self::Block as Block>::Id,
        approve: bool,
    ) -> Result<(), Self::Error>;

    /// Finalize consensus and produce a new block
    fn finalize(&mut self) -> Result<Self::Block, Self::Error>;

    /// Receive a consensus message from a peer
    fn receive_message(&mut self, peer: Self::PeerId, message: &[u8]) -> Result<(), Self::Error>;

    /// Get the current consensus state
    fn state(&self) -> ConsensusState;

    /// Get the current leader/proposer (if applicable)
    fn leader(&self) -> Option<Self::PeerId>;

    /// Get the current set of voters
    fn voters(&self) -> Vec<Self::PeerId>;

    /// Get the current block being voted on
    fn current_block(&self) -> Option<&Self::Block>;

    /// Get the current height
    fn height(&self) -> u64;
}

/// Trait for a blockchain
///
/// The main interface for the distributed ledger.
pub trait Blockchain: Send + Sync {
    /// The type of block used in this blockchain
    type Block: Block;
    /// The type of transaction used in this blockchain
    type Transaction: Transaction;
    /// The consensus algorithm type
    type Consensus: ConsensusAlgorithm<Block = Self::Block, Transaction = Self::Transaction>;
    /// The block storage backend type
    type Storage: BlockStorage<Block = Self::Block>;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Add a transaction to the transaction pool
    fn add_transaction(
        &mut self,
        transaction: Self::Transaction,
    ) -> Result<<Self::Transaction as Transaction>::Id, Self::Error>;

    /// Create a new block with pending transactions
    fn create_block(&mut self) -> Result<<Self::Block as Block>::Id, Self::Error>;

    /// Add a block to the blockchain
    fn add_block(&mut self, block: Self::Block) -> Result<(), Self::Error>;

    /// Get a block by its identifier
    fn get_block(&self, id: &<Self::Block as Block>::Id) -> Result<Self::Block, Self::Error>;

    /// Get the genesis block
    fn get_genesis(&self) -> Result<Self::Block, Self::Error>;

    /// Get the latest block
    fn get_latest(&self) -> Result<Self::Block, Self::Error>;

    /// Get a block by its height in the chain
    fn get_block_at_height(&self, height: u64) -> Result<Self::Block, Self::Error>;

    /// Get all blocks in the chain from a given block
    fn get_chain_from(
        &self,
        from: &<Self::Block as Block>::Id,
    ) -> Result<Vec<Self::Block>, Self::Error>;

    /// Verify the integrity of the entire blockchain
    fn verify_chain(&self) -> Result<bool, Self::Error>;

    /// Get the current height of the blockchain
    fn height(&self) -> Result<u64, Self::Error>;

    /// Get a transaction by its identifier
    fn get_transaction(
        &self,
        id: &<Self::Transaction as Transaction>::Id,
    ) -> Result<Self::Transaction, Self::Error>;

    /// Get all transactions in a block
    fn get_transactions_in_block(
        &self,
        block_id: &<Self::Block as Block>::Id,
    ) -> Result<Vec<Self::Transaction>, Self::Error>;

    /// Synchronize with a peer
    fn sync_with_peer(&mut self, peer_id: &[u8]) -> Result<(), Self::Error>;

    /// Get the blockchain's identifier
    fn id(&self) -> &[u8];

    /// Get the genesis block's identifier
    fn genesis_id(&self) -> &<Self::Block as Block>::Id;

    /// Get the storage backend
    fn storage(&self) -> &Self::Storage;

    /// Get the consensus algorithm
    fn consensus(&self) -> &Self::Consensus;
}

/// Trait for a blockchain node
///
/// Represents a participant in the distributed blockchain network.
pub trait BlockchainNode: Send + Sync {
    /// The type of blockchain this node participates in
    type Blockchain: Blockchain;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Get the blockchain instance
    fn blockchain(&self) -> &Self::Blockchain;

    /// Get a mutable reference to the blockchain instance
    fn blockchain_mut(&mut self) -> &mut Self::Blockchain;

    /// Start the node
    fn start(&mut self) -> Result<(), Self::Error>;

    /// Stop the node
    fn stop(&mut self) -> Result<(), Self::Error>;

    /// Check if the node is running
    fn is_running(&self) -> bool;

    /// Get the node's unique identifier
    fn id(&self) -> &[u8];

    /// Get the node's network address
    fn address(&self) -> SocketAddr;

    /// Get the node's public key
    fn public_key(&self) -> &[u8];

    /// Connect to another node
    fn connect(&mut self, peer_address: SocketAddr) -> Result<(), Self::Error>;

    /// Disconnect from a node
    fn disconnect(&mut self, peer_id: &[u8]) -> Result<(), Self::Error>;

    /// Get the list of connected peers
    fn peers(&self) -> Vec<PeerId>;

    /// Broadcast a transaction to the network
    fn broadcast_transaction(
        &self,
        transaction: <Self::Blockchain as Blockchain>::Transaction,
    ) -> Result<(), Self::Error>;

    /// Broadcast a block to the network
    fn broadcast_block(
        &self,
        block: <Self::Blockchain as Blockchain>::Block,
    ) -> Result<(), Self::Error>;
}

/// Trait for blockchain configuration
pub trait BlockchainConfig: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// Return the blockchain identifier
    fn id(&self) -> &[u8];

    /// Return the genesis block data
    fn genesis_data(&self) -> &[u8];

    /// Return the block time (seconds between blocks)
    fn block_time(&self) -> u64;

    /// Return the maximum block size in bytes
    fn max_block_size(&self) -> u64;

    /// Return the maximum transaction size in bytes
    fn max_transaction_size(&self) -> u64;

    /// Return the consensus algorithm name
    fn consensus_algorithm(&self) -> &str;

    /// Return the hash function name
    fn hash_function(&self) -> &str;

    /// Return the signature scheme name
    fn signature_scheme(&self) -> &str;

    /// Return the encryption algorithm name (for encrypted blocks)
    fn encryption_algorithm(&self) -> Option<&str>;

    /// Return the difficulty target (for PoW-based consensus)
    fn difficulty_target(&self) -> Option<u64>;
}

/// Trait for blockchain events
///
/// Allows subscribing to blockchain events.
pub trait BlockchainEvents: Send + Sync {
    /// The type of blockchain
    type Blockchain: Blockchain;

    /// Subscribe to new block events
    #[allow(clippy::type_complexity)]
    fn on_new_block(
        &mut self,
        callback: Arc<dyn Fn(&<Self::Blockchain as Blockchain>::Block) + Send + Sync>,
    );

    /// Subscribe to new transaction events
    #[allow(clippy::type_complexity)]
    fn on_new_transaction(
        &mut self,
        callback: Arc<dyn Fn(&<Self::Blockchain as Blockchain>::Transaction) + Send + Sync>,
    );

    /// Subscribe to consensus state change events
    fn on_consensus_state_change(&mut self, callback: ConsensusCallback);

    /// Subscribe to sync events
    fn on_sync(&mut self, callback: SyncCallback); // peer_id, height
}

/// Trait for blockchain synchronization
pub trait BlockchainSync: Send + Sync {
    /// The type of blockchain
    type Blockchain: Blockchain;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Synchronize with a specific peer
    fn sync_with_peer(&mut self, peer_id: &[u8]) -> Result<SyncResult, Self::Error>;

    /// Synchronize with all connected peers
    fn sync_all(&mut self) -> Result<Vec<SyncResult>, Self::Error>;

    /// Get the synchronization status
    fn sync_status(&self) -> SyncStatus;

    /// Get the list of blocks that need to be synchronized
    #[allow(clippy::type_complexity)]
    fn pending_blocks(
        &self,
    ) -> Result<Vec<<<Self::Blockchain as Blockchain>::Block as Block>::Id>, Self::Error>;
}

/// Synchronization result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub peer_id: Vec<u8>,
    pub blocks_received: u64,
    pub blocks_sent: u64,
    pub height_before: u64,
    pub height_after: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Synchronization status
#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub current_peer: Option<Vec<u8>>,
    pub blocks_remaining: u64,
    pub last_sync_time: Option<i64>,
}

/// Trait for a blockchain factory
///
/// Creates blockchain instances with specific configurations.
pub trait BlockchainFactory: Send + Sync {
    /// The type of blockchain to create
    type Blockchain: Blockchain;
    /// Configuration type
    type Config: BlockchainConfig;
    /// Node type
    type Node: BlockchainNode<Blockchain = Self::Blockchain>;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create a new blockchain with the given configuration
    fn create(&self, config: Self::Config) -> Result<Self::Blockchain, Self::Error>;

    /// Create a new blockchain with default configuration
    fn create_default(&self) -> Result<Self::Blockchain, Self::Error>;

    /// Create a blockchain node
    fn create_node(
        &self,
        config: Self::Config,
        address: SocketAddr,
    ) -> Result<Self::Node, Self::Error>;
}

/// Trait for blockchain validation
pub trait BlockchainValidator: Send + Sync {
    /// The type of blockchain to validate
    type Blockchain: Blockchain;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Validate the entire blockchain
    fn validate(&self, blockchain: &Self::Blockchain) -> Result<bool, Self::Error>;

    /// Validate a specific block
    fn validate_block(
        &self,
        blockchain: &Self::Blockchain,
        block: &<Self::Blockchain as Blockchain>::Block,
    ) -> Result<bool, Self::Error>;

    /// Validate a transaction
    fn validate_transaction(
        &self,
        transaction: &<Self::Blockchain as Blockchain>::Transaction,
    ) -> Result<bool, Self::Error>;

    /// Check if the blockchain is in a valid state
    fn is_valid(&self, blockchain: &Self::Blockchain) -> Result<bool, Self::Error>;
}
