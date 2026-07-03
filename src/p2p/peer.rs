//! Implémentation d'un peer dans le réseau P2P.

use std::net::SocketAddr;
use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use crate::p2p::traits::{Peer, NetworkMessage};
use crate::error::Error;

/// Adresse d'un peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAddress(pub SocketAddr);

impl std::fmt::Display for PeerAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Représente un peer dans le réseau P2P.
pub struct P2PPeer {
    id: String,               // ID unique du peer.
    address: PeerAddress,    // Adresse réseau.
    stream: Option<TcpStream>, // Connexion TCP (si connecté).
}

impl Clone for P2PPeer {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            address: self.address.clone(),
            stream: None, // On ne clone pas la connexion TCP.
        }
    }
}

impl P2PPeer {
    pub fn new(id: String, address: SocketAddr) -> Self {
        Self {
            id,
            address: PeerAddress(address),
            stream: None,
        }
    }

    /// Connecte le peer.
    pub async fn connect(&mut self) -> Result<(), Error> {
        let stream = TcpStream::connect(self.address.0).await?;
        self.stream = Some(stream);
        Ok(())
    }

    /// Déconnecte le peer.
    pub fn disconnect(&mut self) {
        self.stream = None;
    }

    /// Envoie un message brut.
    pub async fn send_raw(&mut self, message: &[u8]) -> Result<(), Error> {
        if let Some(stream) = &mut self.stream {
            stream.write_all(message).await?;
            Ok(())
        } else {
            Err(Error::PeerNotConnected(self.id.clone()))
        }
    }

    /// Reçoit un message brut.
    pub async fn recv_raw(&mut self) -> Result<Vec<u8>, Error> {
        if let Some(stream) = &mut self.stream {
            let mut buf = vec![0u8; 1024];
            let n = stream.read(&mut buf).await?;
            buf.truncate(n);
            Ok(buf)
        } else {
            Err(Error::PeerNotConnected(self.id.clone()))
        }
    }
}

#[async_trait]
impl Peer for P2PPeer {
    type Address = PeerAddress;

    fn id(&self) -> &str {
        &self.id
    }

    fn address(&self) -> &Self::Address {
        &self.address
    }

    async fn send_message(&self, message: NetworkMessage) -> Result<(), Error> {
        // Sérialiser le message.
        let serialized = bincode::serialize(&message)?;
        // En pratique, il faudrait une connexion mutable.
        // Ici, on simule l'envoi.
        Ok(())
    }

    async fn ping(&self) -> Result<bool, Error> {
        // En pratique, il faudrait envoyer un message Ping et attendre une réponse.
        Ok(true)
    }
}
