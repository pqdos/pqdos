//! Traits abstraits pour le réseau P2P de PQDOS.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::block::simple::SimpleBlock;
use crate::error::Error;

/// Énumération pour les messages réseau.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    BlockRequest { block_id: Vec<u8>, requester_id: String },
    BlockResponse { 
        block_id: Vec<u8>, 
        encrypted_data: Vec<u8>,  // Données chiffrées du bloc.
        owner_id: Vec<u8>,       // ID du propriétaire du bloc.
        nonce: Vec<u8>,          // Nonce utilisé pour le chiffrement.
    },
    BlockAvailability { block_id: Vec<u8>, peer_id: String },
    Ping { peer_id: String },
    Pong { peer_id: String },
}

impl NetworkMessage {
    /// Type du message (ex: "BLOCK_REQUEST", "PING").
    pub fn message_type(&self) -> &str {
        match self {
            NetworkMessage::BlockRequest { .. } => "BLOCK_REQUEST",
            NetworkMessage::BlockResponse { .. } => "BLOCK_RESPONSE",
            NetworkMessage::BlockAvailability { .. } => "BLOCK_AVAILABILITY",
            NetworkMessage::Ping { .. } => "PING",
            NetworkMessage::Pong { .. } => "PONG",
        }
    }

    /// ID du bloc associé au message (si applicable).
    pub fn block_id(&self) -> Option<&[u8]> {
        match self {
            NetworkMessage::BlockRequest { block_id, .. } => Some(block_id),
            NetworkMessage::BlockResponse { block_id, .. } => Some(block_id),
            NetworkMessage::BlockAvailability { block_id, .. } => Some(block_id),
            NetworkMessage::Ping { .. } => None,
            NetworkMessage::Pong { .. } => None,
        }
    }
}

/// Trait pour un peer dans le réseau P2P.
#[async_trait]
pub trait Peer: Send + Sync {
    /// Adresse du peer.
    type Address: Send + Sync + std::fmt::Display;

    /// Retourne l'ID unique du peer.
    fn id(&self) -> &str;

    /// Retourne l'adresse du peer.
    fn address(&self) -> &Self::Address;

    /// Envoie un message au peer.
    async fn send_message(&self, message: NetworkMessage) -> Result<(), Error>;

    /// Vérifie si le peer est connecté (via un ping).
    async fn ping(&self) -> Result<bool, Error>;
}

/// Trait pour le réseau P2P.
#[async_trait]
pub trait P2PNetwork: Send + Sync {
    /// Type de peer utilisé par le réseau.
    type Peer: Peer + Clone;

    /// Ajoute un peer connu au réseau.
    async fn add_peer(&mut self, peer: Self::Peer) -> Result<(), Error>;

    /// Découvre de nouveaux peers.
    async fn discover_peers(&self) -> Result<Vec<Self::Peer>, Error>;

    /// Envoie un message à tous les peers connectés.
    async fn broadcast(&self, message: NetworkMessage) -> Result<(), Error>;

    /// Envoie un message à un peer spécifique.
    async fn send_to_peer(&self, peer_id: &str, message: NetworkMessage) -> Result<(), Error>;

    /// Écoute les messages entrants.
    async fn listen(&mut self) -> Result<(), Error>;
}

/// Trait pour la récupération de blocs depuis le réseau.
#[async_trait]
pub trait BlockFetcher: Send + Sync {
    /// Récupère un bloc depuis le réseau ou le cache.
    /// Retourne `None` si le bloc n'est pas accessible (pas de permissions).
    async fn fetch_block(
        &self, 
        block_id: &[u8], 
        user_id: &[u8],
        user_private_key: Option<&[u8]>, // Clé privée de l'utilisateur (pour déchiffrer ses propres blocs).
        shared_keys: &std::collections::HashMap<Vec<u8>, Vec<u8>>, // Clés partagées (block_id -> clé de déchiffrement).
    ) -> Result<Option<SimpleBlock>, Error>;

    /// Vérifie si un bloc est disponible localement ou sur le réseau.
    async fn has_block(&self, block_id: &[u8]) -> Result<bool, Error>;
}

/// Trait pour la gestion du cache local.
#[async_trait]
pub trait CacheManager: Send + Sync {
    /// Ajoute un bloc au cache (chiffré).
    async fn cache_block(
        &mut self, 
        block: SimpleBlock, 
        owner_id: &[u8],
    ) -> Result<(), Error>;

    /// Récupère un bloc depuis le cache (déchiffré si l'utilisateur a les permissions).
    async fn get_cached_block(
        &self, 
        block_id: &[u8], 
        user_id: &[u8],
        user_private_key: Option<&[u8]>, // Clé privée de l'utilisateur.
        shared_keys: &std::collections::HashMap<Vec<u8>, Vec<u8>>, // Clés partagées.
    ) -> Result<Option<SimpleBlock>, Error>;

    /// Vérifie si un bloc est en cache.
    async fn contains(&self, block_id: &[u8]) -> Result<bool, Error>;

    /// Supprime un bloc du cache.
    async fn remove_block(&mut self, block_id: &[u8]) -> Result<(), Error>;

    /// Nettoie le cache (supprime les entrées LRU).
    async fn cleanup(&mut self) -> Result<(), Error>;

    /// Taille actuelle du cache (en octets).
    async fn size(&self) -> Result<u64, Error>;
}
