//! Implémentation simple d'un bloc PQDOS.

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use serde::{Serialize, Deserialize};
use crate::block::traits::{Block, BlockId};

/// ID de bloc simple (wrapper autour de `Vec<u8>`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimpleBlockId {
    id: Vec<u8>,
}

impl SimpleBlockId {
    pub fn new(id: Vec<u8>) -> Self {
        Self { id }
    }
}

impl AsRef<[u8]> for SimpleBlockId {
    fn as_ref(&self) -> &[u8] {
        &self.id
    }
}

impl BlockId for SimpleBlockId {
    fn from_bytes(bytes: Vec<u8>) -> Self {
        Self::new(bytes)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.id.clone()
    }

    fn size(&self) -> usize {
        self.id.len()
    }
}

/// Données d'un bloc simple.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimpleBlockData {
    data: Vec<u8>,
}

impl AsRef<[u8]> for SimpleBlockData {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

/// Bloc simple pour PQDOS.
#[derive(Clone, Serialize, Deserialize)]
pub struct SimpleBlock {
    id: SimpleBlockId,
    data: SimpleBlockData,
    previous: Option<SimpleBlockId>,
    timestamp: i64,
    signature: Option<Vec<u8>>,
    signer: Option<Vec<u8>>,
    version: u8,
    metadata: HashMap<String, String>,
}

impl Debug for SimpleBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleBlock")
            .field("id", &self.id)
            .field("data_size", &self.data.data.len())
            .field("timestamp", &self.timestamp)
            .field("version", &self.version)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl SimpleBlock {
    pub fn new(
        id: Vec<u8>,
        data: Vec<u8>,
        previous: Option<Vec<u8>>,
        timestamp: i64,
        version: u8,
    ) -> Self {
        Self {
            id: SimpleBlockId::new(id),
            data: SimpleBlockData { data },
            previous: previous.map(SimpleBlockId::new),
            timestamp,
            signature: None,
            signer: None,
            version,
            metadata: HashMap::new(),
        }
    }
}

impl Block for SimpleBlock {
    type Id = SimpleBlockId;
    type Data = SimpleBlockData;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }

    fn previous(&self) -> Option<&Self::Id> {
        self.previous.as_ref()
    }

    fn timestamp(&self) -> i64 {
        self.timestamp
    }

    fn signature(&self) -> Option<&[u8]> {
        self.signature.as_deref()
    }

    fn signer(&self) -> Option<&[u8]> {
        self.signer.as_deref()
    }

    fn data_size(&self) -> usize {
        self.data.data.len()
    }

    fn is_valid(&self) -> bool {
        // Vérification basique : l'ID correspond au hash des données.
        // En pratique, il faudrait calculer le hash et comparer.
        true
    }

    fn version(&self) -> u8 {
        self.version
    }

    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}
