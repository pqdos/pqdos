//! Messages réseau et implémentation de base pour le réseau P2P.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use crate::p2p::traits::{P2PNetwork, Peer, NetworkMessage};
use crate::p2p::peer::P2PPeer;
use crate::error::Error;

pub struct P2PNetworkImpl {
    peers: Arc<RwLock<HashMap<String, P2PPeer>>>, // peer_id -> Peer
    local_peer_id: String,                        // ID de ce noeud.
    listen_addr: SocketAddr,                     // Adresse d'écoute.
}

impl P2PNetworkImpl {
    pub fn new(local_peer_id: String, listen_addr: SocketAddr) -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            local_peer_id,
            listen_addr,
        }
    }

    /// Démarre le serveur pour écouter les connexions entrantes.
    pub async fn start_server(&mut self) -> Result<(), Error> {
        let listener = tokio::net::TcpListener::bind(self.listen_addr).await?;
        let _peers = self.peers.clone();
        let _local_peer_id = self.local_peer_id.clone();

        tokio::spawn(async move {
            loop {
                let (_stream, addr) = listener.accept().await.unwrap();
                println!("[P2P] Nouvelle connexion depuis {}", addr);
                // En pratique, il faudrait gérer la connexion ici.
            }
        });

        Ok(())
    }
}

#[async_trait]
impl P2PNetwork for P2PNetworkImpl {
    type Peer = P2PPeer;

    async fn add_peer(&mut self, peer: Self::Peer) -> Result<(), Error> {
        let mut peers = self.peers.write().await;
        peers.insert(peer.id().to_string(), peer);
        Ok(())
    }

    async fn discover_peers(&self) -> Result<Vec<Self::Peer>, Error> {
        let peers = self.peers.read().await;
        Ok(peers.values().cloned().collect())
    }

    async fn broadcast(&self, message: NetworkMessage) -> Result<(), Error> {
        let peers = self.peers.read().await;
        for peer in peers.values() {
            peer.send_message(message.clone()).await?;
        }
        Ok(())
    }

    async fn send_to_peer(&self, peer_id: &str, message: NetworkMessage) -> Result<(), Error> {
        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(peer_id) {
            peer.send_message(message).await
        } else {
            Err(Error::PeerNotFound(peer_id.to_string()))
        }
    }

    async fn listen(&mut self) -> Result<(), Error> {
        self.start_server().await
    }
}
