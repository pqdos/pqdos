//! Abstract network traits for Post-Quantum Secure OS
//!
//! These traits define interfaces for P2P network communication,
//! allowing for pluggable network implementations (Tokio, async-std, etc.).

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;
use std::time::SystemTime;

/// Trait for a peer identifier
pub trait PeerId:
    Clone + Eq + std::hash::Hash + AsRef<[u8]> + Debug + Serialize + DeserializeOwned + Send + Sync
{
    /// Create a new peer ID from raw bytes
    fn from_bytes(bytes: Vec<u8>) -> Self;

    /// Convert to raw bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Return the size in bytes
    fn size(&self) -> usize;
}

/// Trait for peer information
///
/// Represents a node in the P2P network.
pub trait Peer: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// The type of peer identifier
    type Id: PeerId;

    /// Return the peer's unique identifier
    fn id(&self) -> &Self::Id;

    /// Return the peer's network address
    fn address(&self) -> SocketAddr;

    /// Return the peer's public key (for authentication)
    fn public_key(&self) -> &[u8];

    /// Return the peer's current blockchain height (if applicable)
    fn height(&self) -> u64;

    /// Return the last time this peer was seen
    fn last_seen(&self) -> SystemTime;

    /// Check if the peer is currently connected
    fn is_connected(&self) -> bool;

    /// Return the peer's capabilities (supported protocols, etc.)
    fn capabilities(&self) -> &[u8];

    /// Return the peer's user agent/version
    fn user_agent(&self) -> &str;
}

/// Message type for network communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Initial handshake message
    Handshake,
    /// Ping message (keep-alive)
    Ping,
    /// Pong message (response to ping)
    Pong,
    /// Block data message
    Block,
    /// Transaction message
    Transaction,
    /// Consensus vote message
    ConsensusVote,
    /// Blockchain synchronization request
    SyncRequest,
    /// Blockchain synchronization response
    SyncResponse,
    /// Peer discovery request
    PeerDiscovery,
    /// Peer discovery response
    PeerList,
    /// Error message
    Error,
    /// Custom message type
    Custom(u8),
}

/// Trait for a network message
///
/// Represents a message that can be sent over the network.
pub trait NetworkMessage: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// The type of message payload
    type Payload: AsRef<[u8]> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync;

    /// Return the message type
    fn message_type(&self) -> MessageType;

    /// Return the message payload
    fn payload(&self) -> &Self::Payload;

    /// Return the sender's peer ID
    fn sender(&self) -> &[u8];

    /// Return the message timestamp (Unix timestamp)
    fn timestamp(&self) -> i64;

    /// Return the message ID (for request-response matching)
    fn message_id(&self) -> u64;

    /// Return the TTL (time to live) in seconds
    fn ttl(&self) -> u8;

    /// Check if the message has expired
    fn is_expired(&self) -> bool;

    /// Return the hash of the message content
    fn hash(&self) -> &[u8];
}

/// Trait for a message serializer
///
/// Handles serialization and deserialization of network messages.
pub trait MessageSerializer: Send + Sync {
    /// The type of message to serialize
    type Message: NetworkMessage;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Serialize a message to bytes
    fn serialize(&self, message: &Self::Message) -> Result<Vec<u8>, Self::Error>;

    /// Deserialize bytes to a message
    fn deserialize(&self, data: &[u8]) -> Result<Self::Message, Self::Error>;

    /// Get the maximum message size
    fn max_message_size(&self) -> usize;

    /// Get the protocol version
    fn protocol_version(&self) -> u8;
}

/// Trait for a network protocol
///
/// Defines the low-level protocol for network communication.
pub trait NetworkProtocol: Send + Sync {
    /// The type of peer
    type Peer: Peer;
    /// The type of message
    type Message: NetworkMessage;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Connect to a peer at the given address
    fn connect(
        &self,
        address: SocketAddr,
    ) -> impl std::future::Future<Output = Result<Self::Peer, Self::Error>> + Send;

    /// Disconnect from a peer
    fn disconnect(
        &self,
        peer_id: &<Self::Peer as Peer>::Id,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Send a message to a specific peer
    fn send(
        &self,
        peer_id: &<Self::Peer as Peer>::Id,
        message: Self::Message,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Broadcast a message to all connected peers
    fn broadcast(
        &self,
        message: Self::Message,
    ) -> impl std::future::Future<Output = Result<Vec<<Self::Peer as Peer>::Id>, Self::Error>> + Send;

    /// Receive a message (non-blocking)
    fn receive(
        &self,
    ) -> impl std::future::Future<Output = Option<(Self::Peer, Self::Message)>> + Send;

    /// Get all connected peers
    fn connected_peers(&self) -> Vec<Self::Peer>;

    /// Get the local peer information
    fn local_peer(&self) -> &Self::Peer;

    /// Get the message serializer
    fn serializer(
        &self,
    ) -> &dyn MessageSerializer<
        Message = Self::Message,
        Error = Box<dyn std::error::Error + Send + Sync + 'static>,
    >;

    /// Set a message handler for incoming messages
    fn set_message_handler(
        &mut self,
        handler: Arc<dyn Fn(Self::Peer, Self::Message) + Send + Sync>,
    );

    /// Set a connection handler for new connections
    fn set_connection_handler(&mut self, handler: Arc<dyn Fn(Self::Peer) + Send + Sync>);

    /// Set a disconnection handler
    #[allow(clippy::type_complexity)]
    fn set_disconnection_handler(
        &mut self,
        handler: Arc<dyn Fn(&<Self::Peer as Peer>::Id) + Send + Sync>,
    );
}

/// Trait for a P2P network
///
/// High-level interface for peer-to-peer network communication.
pub trait P2PNetwork: Send + Sync {
    /// The type of protocol used
    type Protocol: NetworkProtocol;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get the underlying protocol
    fn protocol(&self) -> &Self::Protocol;

    /// Start the network, listening on the given address
    fn start(
        &mut self,
        listen_addr: SocketAddr,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Stop the network
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Check if the network is running
    fn is_running(&self) -> bool;

    /// Connect to a specific peer
    fn connect(
        &mut self,
        address: SocketAddr,
    ) -> impl std::future::Future<
        Output = Result<<Self::Protocol as NetworkProtocol>::Peer, Self::Error>,
    > + Send;

    /// Disconnect from a specific peer
    fn disconnect(
        &mut self,
        peer_id: &[u8],
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Get all connected peers
    fn peers(&self) -> Vec<<Self::Protocol as NetworkProtocol>::Peer>;

    /// Get the local peer ID
    fn local_peer_id(&self) -> &[u8];

    /// Discover peers on the network
    fn discover_peers(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<SocketAddr>, Self::Error>> + Send;

    /// Join the network using bootstrap nodes
    fn join_network(
        &mut self,
        bootstrap_nodes: Vec<SocketAddr>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Leave the network
    fn leave_network(
        &mut self,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Get network statistics
    fn stats(&self) -> NetworkStats;
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub peer_count: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub connections_opened: u64,
    pub connections_closed: u64,
    pub uptime_seconds: u64,
}

/// Trait for peer discovery
///
/// Handles discovery of other peers in the network.
pub trait PeerDiscovery: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;

    /// Discover peers using a bootstrap node
    fn discover(
        &self,
        bootstrap_addr: SocketAddr,
    ) -> impl std::future::Future<Output = Result<Vec<SocketAddr>, Self::Error>> + Send;

    /// Get a list of known peers
    fn known_peers(&self) -> Vec<SocketAddr>;

    /// Add a known peer to the list
    fn add_known_peer(&mut self, address: SocketAddr);

    /// Remove a peer from the known list
    fn remove_known_peer(&mut self, address: &SocketAddr);

    /// Get the local peer's address
    fn local_address(&self) -> SocketAddr;
}

/// Trait for a network transport
///
/// Low-level transport layer for network communication.
pub trait NetworkTransport: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;
    /// Connection type
    type Connection: Connection;

    /// Listen for incoming connections on the given address
    fn listen(
        &mut self,
        addr: SocketAddr,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Connect to a remote address
    fn connect(
        &mut self,
        addr: SocketAddr,
    ) -> impl std::future::Future<Output = Result<Self::Connection, Self::Error>> + Send;

    /// Stop listening
    fn stop(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Get the next incoming connection
    fn accept(
        &mut self,
    ) -> impl std::future::Future<Output = Option<Result<Self::Connection, Self::Error>>> + Send;
}

/// Trait for a network connection
///
/// Represents an active connection to a peer.
pub trait Connection: Send + Sync {
    /// Error type for connection operations
    type Error: std::error::Error + Send + Sync + 'static;

    /// Send data over the connection
    fn send(
        &mut self,
        data: &[u8],
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Receive data from the connection
    fn receive(&mut self)
        -> impl std::future::Future<Output = Result<Vec<u8>, Self::Error>> + Send;

    /// Close the connection
    fn close(&mut self) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Get the remote address
    fn remote_address(&self) -> SocketAddr;

    /// Get the local address
    fn local_address(&self) -> SocketAddr;

    /// Check if the connection is open
    fn is_open(&self) -> bool;

    /// Set the read timeout
    fn set_read_timeout(&mut self, timeout: std::time::Duration) -> Result<(), Self::Error>;

    /// Set the write timeout
    fn set_write_timeout(&mut self, timeout: std::time::Duration) -> Result<(), Self::Error>;
}

/// Trait for a network address book
///
/// Manages known peers and their information.
pub trait AddressBook: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;
    /// Peer type
    type Peer: Peer;

    /// Add a peer to the address book
    fn add_peer(&mut self, peer: Self::Peer) -> Result<(), Self::Error>;

    /// Remove a peer from the address book
    fn remove_peer(&mut self, peer_id: &[u8]) -> Result<(), Self::Error>;

    /// Get a peer by ID
    fn get_peer(&self, peer_id: &[u8]) -> Option<Self::Peer>;

    /// Get a peer by address
    fn get_peer_by_address(&self, address: &SocketAddr) -> Option<Self::Peer>;

    /// Get all peers in the address book
    fn get_all_peers(&self) -> Vec<Self::Peer>;

    /// Get peers that are currently connected
    fn get_connected_peers(&self) -> Vec<Self::Peer>;

    /// Mark a peer as connected
    fn mark_connected(&mut self, peer_id: &[u8], address: SocketAddr) -> Result<(), Self::Error>;

    /// Mark a peer as disconnected
    fn mark_disconnected(&mut self, peer_id: &[u8]) -> Result<(), Self::Error>;

    /// Update a peer's information
    fn update_peer(&mut self, peer: Self::Peer) -> Result<(), Self::Error>;

    /// Save the address book to persistent storage
    fn save(&self) -> Result<(), Self::Error>;

    /// Load the address book from persistent storage
    fn load(&mut self) -> Result<(), Self::Error>;
}

/// Trait for a network factory
///
/// Creates network instances with specific configurations.
pub trait NetworkFactory: Send + Sync {
    /// The type of P2P network to create
    type Network: P2PNetwork;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;
    /// Configuration type
    type Config: NetworkConfig;

    /// Create a new P2P network with the given configuration
    fn create(
        &self,
        config: Self::Config,
    ) -> impl std::future::Future<Output = Result<Self::Network, Self::Error>> + Send;

    /// Create a new P2P network with default configuration
    fn create_default(
        &self,
    ) -> impl std::future::Future<Output = Result<Self::Network, Self::Error>> + Send;
}

/// Trait for network configuration
pub trait NetworkConfig: Clone + Debug + Serialize + DeserializeOwned + Send + Sync {
    /// Return the listen address
    fn listen_address(&self) -> SocketAddr;

    /// Return the list of bootstrap nodes
    fn bootstrap_nodes(&self) -> Vec<SocketAddr>;

    /// Return the maximum number of connections
    fn max_connections(&self) -> usize;

    /// Return the connection timeout
    fn connection_timeout(&self) -> std::time::Duration;

    /// Return the read timeout
    fn read_timeout(&self) -> std::time::Duration;

    /// Return the write timeout
    fn write_timeout(&self) -> std::time::Duration;

    /// Return the ping interval
    fn ping_interval(&self) -> std::time::Duration;

    /// Return whether to enable NAT traversal
    fn enable_nat_traversal(&self) -> bool;

    /// Return whether to enable encryption
    fn enable_encryption(&self) -> bool;

    /// Return the encryption algorithm (if enabled)
    fn encryption_algorithm(&self) -> Option<&str>;

    /// Return the protocol version
    fn protocol_version(&self) -> u8;

    /// Return the user agent string
    fn user_agent(&self) -> &str;
}

/// Trait for network encryption
///
/// Handles encryption and decryption of network traffic.
pub trait NetworkEncryption: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Encrypt data for a specific peer
    fn encrypt(&self, data: &[u8], peer_public_key: &[u8]) -> Result<Vec<u8>, Self::Error>;

    /// Decrypt data from a specific peer
    fn decrypt(&self, data: &[u8], peer_id: &[u8]) -> Result<Vec<u8>, Self::Error>;

    /// Generate a new key pair for this node
    fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>), Self::Error>; // (public, private)

    /// Get the local node's public key
    fn public_key(&self) -> &[u8];

    /// Get the local node's private key
    fn private_key(&self) -> &[u8];

    /// Add a peer's public key
    fn add_peer_key(&mut self, peer_id: Vec<u8>, public_key: Vec<u8>);

    /// Remove a peer's public key
    fn remove_peer_key(&mut self, peer_id: &[u8]);

    /// Get a peer's public key
    fn get_peer_key(&self, peer_id: &[u8]) -> Option<&[u8]>;
}

/// Trait for message authentication
///
/// Handles authentication of network messages.
pub trait MessageAuthenticator: Send + Sync {
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Sign a message
    fn sign(&self, message: &[u8], private_key: &[u8]) -> Result<Vec<u8>, Self::Error>;

    /// Verify a message signature
    fn verify(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, Self::Error>;

    /// Generate a message authentication code (MAC)
    fn generate_mac(&self, message: &[u8], key: &[u8]) -> Result<Vec<u8>, Self::Error>;

    /// Verify a message authentication code
    fn verify_mac(&self, message: &[u8], mac: &[u8], key: &[u8]) -> Result<bool, Self::Error>;
}

/// Trait for a network middleware
///
/// Allows for intercepting and processing network messages.
pub trait NetworkMiddleware: Send + Sync {
    /// The type of message
    type Message: NetworkMessage;
    /// Error type
    type Error: std::error::Error + Send + Sync + 'static;
    /// Peer type
    type Peer: Peer;

    /// Process an outgoing message before sending
    fn on_send(&mut self, message: Self::Message) -> Result<Option<Self::Message>, Self::Error>;

    /// Process an incoming message after receiving
    fn on_receive(&mut self, message: Self::Message) -> Result<Option<Self::Message>, Self::Error>;

    /// Process a connection event
    fn on_connect(&mut self, peer: &Self::Peer) -> Result<(), Self::Error>;

    /// Process a disconnection event
    fn on_disconnect(&mut self, peer_id: &[u8]) -> Result<(), Self::Error>;
}
