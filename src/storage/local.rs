//! Local Storage Backend for pqdos
//!
//! This module provides an in-memory and filesystem-based storage backend
//! for development and testing purposes.

use base64::{engine::general_purpose, Engine as _};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::traits::{
    StorageBackend, StorageConfig, StorageError, StorageResult, StoredBlock, StoredBlockchain,
};

/// Local Storage Block - in-memory representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalBlock {
    pub id: String,
    pub previous_id: Option<String>,
    pub data: String,
    pub owner_id: String,
    pub block_type: String,
    pub timestamp: i64,
    pub signature: Option<String>,
    pub signer: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl LocalBlock {
    pub fn new(
        id: impl Into<String>,
        data: impl AsRef<[u8]>,
        owner_id: impl Into<String>,
        block_type: impl Into<String>,
    ) -> Self {
        use base64::{engine::general_purpose, Engine as _};
        use chrono;

        Self {
            id: id.into(),
            previous_id: None,
            data: general_purpose::STANDARD.encode(data.as_ref()),
            owner_id: owner_id.into(),
            block_type: block_type.into(),
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
            signer: None,
            metadata: HashMap::new(),
        }
    }

    pub fn new_with_previous(
        id: impl Into<String>,
        data: impl AsRef<[u8]>,
        owner_id: impl Into<String>,
        block_type: impl Into<String>,
        previous_id: impl Into<String>,
    ) -> Self {
        use base64::{engine::general_purpose, Engine as _};
        use chrono;

        Self {
            id: id.into(),
            previous_id: Some(previous_id.into()),
            data: general_purpose::STANDARD.encode(data.as_ref()),
            owner_id: owner_id.into(),
            block_type: block_type.into(),
            timestamp: chrono::Utc::now().timestamp(),
            signature: None,
            signer: None,
            metadata: HashMap::new(),
        }
    }

    pub fn compute_id(data: &[u8]) -> String {
        format!("{:x}", Sha256::digest(data))
    }

    pub fn raw_data(&self) -> StorageResult<Vec<u8>> {
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::STANDARD
            .decode(&self.data)
            .map_err(|e| StorageError::DeserializationError(format!("Base64 decode error: {}", e)))
    }
}

impl StoredBlock for LocalBlock {
    fn id(&self) -> &str {
        &self.id
    }
    fn previous_id(&self) -> Option<&str> {
        self.previous_id.as_deref()
    }
    fn data(&self) -> &str {
        &self.data
    }
    fn owner_id(&self) -> &str {
        &self.owner_id
    }
    fn block_type(&self) -> &str {
        &self.block_type
    }
    fn timestamp(&self) -> i64 {
        self.timestamp
    }
    fn signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }
    fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

/// Local Storage Blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalBlockchain {
    pub name: String,
    pub genesis_block: String,
    pub head_block: String,
    pub blocks: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub description: Option<String>,
}

impl LocalBlockchain {
    pub fn new(name: impl Into<String>, genesis: impl Into<String>) -> Self {
        use chrono;
        let now = chrono::Utc::now().timestamp();
        let genesis = genesis.into();
        Self {
            name: name.into(),
            genesis_block: genesis.clone(),
            head_block: genesis.clone(),
            blocks: vec![genesis],
            created_at: now,
            updated_at: now,
            description: None,
        }
    }

    pub fn with_block(mut self, block: impl Into<String>) -> Self {
        use chrono;
        self.blocks.push(block.into());
        self.head_block = self.blocks.last().cloned().unwrap_or_default();
        self.updated_at = chrono::Utc::now().timestamp();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl StoredBlockchain for LocalBlockchain {
    fn name(&self) -> &str {
        &self.name
    }
    fn genesis_block(&self) -> &str {
        &self.genesis_block
    }
    fn head_block(&self) -> &str {
        &self.head_block
    }
    fn blocks(&self) -> &[String] {
        &self.blocks
    }
    fn created_at(&self) -> i64 {
        self.created_at
    }
    fn updated_at(&self) -> i64 {
        self.updated_at
    }
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

/// Local Storage Backend
///
/// In-memory storage backend for development and testing.
/// Can optionally persist to filesystem.
#[derive(Debug, Clone)]
pub struct LocalStorage {
    pub storage_id: String,
    pub owner_id: String,
    blocks: Arc<RwLock<HashMap<String, LocalBlock>>>,
    blockchains: Arc<RwLock<HashMap<String, LocalBlockchain>>>,
    base_path: Option<PathBuf>,
}

impl LocalStorage {
    /// Create a new in-memory local storage
    pub fn new(owner_id: impl Into<String>) -> Self {
        let owner_id_str: String = owner_id.into();
        let storage_id = format!("local-{}", owner_id_str);
        Self {
            storage_id,
            owner_id: owner_id_str,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            blockchains: Arc::new(RwLock::new(HashMap::new())),
            base_path: None,
        }
    }

    /// Create a new local storage with a filesystem base path
    pub fn with_path(owner_id: impl Into<String>, base_path: impl AsRef<Path>) -> Self {
        let owner_id_str: String = owner_id.into();
        let storage_id = format!("local-{}", owner_id_str);
        let base_path = base_path.as_ref().to_path_buf();
        Self {
            storage_id,
            owner_id: owner_id_str,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            blockchains: Arc::new(RwLock::new(HashMap::new())),
            base_path: Some(base_path),
        }
    }

    /// Create from a StorageConfig
    pub fn from_config(config: &StorageConfig) -> Self {
        let owner_id = config.owner_id.clone();
        let storage_id = config.id.clone();

        let base_path = config.parameters.get("path").map(|p| PathBuf::from(p.as_str()));

        Self {
            storage_id,
            owner_id,
            blocks: Arc::new(RwLock::new(HashMap::new())),
            blockchains: Arc::new(RwLock::new(HashMap::new())),
            base_path,
        }
    }

    /// Load blocks from filesystem (if base_path is set)
    pub fn load_from_filesystem(&mut self) -> StorageResult<()> {
        if let Some(base_path) = &self.base_path {
            // Try to load blocks directory
            let blocks_dir = base_path.join("blocks");
            if blocks_dir.exists() && blocks_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&blocks_dir) {
                    for entry in entries.flatten() {
                        if entry.path().extension().is_some_and(|ext| ext == "json") {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                if let Ok(block) = serde_json::from_str::<LocalBlock>(&content) {
                                    self.blocks.write().insert(block.id.clone(), block);
                                }
                            }
                        }
                    }
                }
            }

            // Try to load chains directory
            let chains_dir = base_path.join("chains");
            if chains_dir.exists() && chains_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&chains_dir) {
                    for entry in entries.flatten() {
                        if entry.path().extension().is_some_and(|ext| ext == "json") {
                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                if let Ok(chain) = serde_json::from_str::<LocalBlockchain>(&content)
                                {
                                    self.blockchains.write().insert(chain.name.clone(), chain);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Save blocks to filesystem (if base_path is set)
    pub fn save_to_filesystem(&self) -> StorageResult<()> {
        if let Some(base_path) = &self.base_path {
            // Save blocks
            let blocks_dir = base_path.join("blocks");
            std::fs::create_dir_all(&blocks_dir)?;
            for block in self.blocks.read().values() {
                let path = blocks_dir.join(format!("{}.json", block.id));
                let content = serde_json::to_string(block)?;
                std::fs::write(path, content)?;
            }

            // Save blockchains
            let chains_dir = base_path.join("chains");
            std::fs::create_dir_all(&chains_dir)?;
            for chain in self.blockchains.read().values() {
                let path = chains_dir.join(format!("{}.json", chain.name));
                let content = serde_json::to_string(chain)?;
                std::fs::write(path, content)?;
            }
        }
        Ok(())
    }
}

impl StorageBackend for LocalStorage {
    type Block = LocalBlock;
    type Blockchain = LocalBlockchain;
    type Error = StorageError;

    fn id(&self) -> &str {
        &self.storage_id
    }
    fn backend_type(&self) -> &str {
        "local"
    }
    fn owner_id(&self) -> &str {
        &self.owner_id
    }
    fn config(&self) -> String {
        match &self.base_path {
            | Some(path) => format!("local:path={}", path.display()),
            | None => "local:in-memory".to_string(),
        }
    }

    fn initialize(&mut self) -> StorageResult<()> {
        self.load_from_filesystem()
    }

    fn is_accessible(&self) -> StorageResult<bool> {
        Ok(true)
    }

    fn store_block(&self, block: &Self::Block) -> StorageResult<()> {
        self.blocks.write().insert(block.id.clone(), block.clone());
        self.save_to_filesystem()
    }

    fn get_block(&self, block_id: &str) -> StorageResult<Option<Self::Block>> {
        Ok(self.blocks.read().get(block_id).cloned())
    }

    fn has_block(&self, block_id: &str) -> StorageResult<bool> {
        Ok(self.blocks.read().contains_key(block_id))
    }

    fn delete_block(&self, block_id: &str) -> StorageResult<()> {
        self.blocks.write().remove(block_id);
        self.save_to_filesystem()
    }

    fn list_block_ids(&self) -> StorageResult<Vec<String>> {
        Ok(self.blocks.read().keys().cloned().collect())
    }

    fn list_blocks_by_owner(&self, owner_id: &str) -> StorageResult<Vec<Self::Block>> {
        Ok(self.blocks.read().values().filter(|b| b.owner_id == owner_id).cloned().collect())
    }

    fn store_blockchain(&self, chain: &Self::Blockchain) -> StorageResult<()> {
        self.blockchains.write().insert(chain.name.clone(), chain.clone());
        self.save_to_filesystem()
    }

    fn get_blockchain(&self, name: &str) -> StorageResult<Option<Self::Blockchain>> {
        Ok(self.blockchains.read().get(name).cloned())
    }

    fn list_blockchain_names(&self) -> StorageResult<Vec<String>> {
        Ok(self.blockchains.read().keys().cloned().collect())
    }

    fn create_block(
        &self,
        data: &[u8],
        owner_id: &str,
        block_type: &str,
        previous_id: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> StorageResult<Self::Block> {
        let id = LocalBlock::compute_id(data);
        let mut block = LocalBlock::new(id.clone(), data, owner_id, block_type);
        if let Some(prev) = previous_id {
            block.previous_id = Some(prev.to_string());
        }
        block.metadata = metadata;
        Ok(block)
    }
}

/// Default LocalStorage factory
pub struct LocalStorageFactory;

impl Default for LocalStorageFactory {
    fn default() -> Self {
        Self
    }
}

impl LocalStorageFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn create(&self, owner_id: impl Into<String>) -> LocalStorage {
        LocalStorage::new(owner_id)
    }

    pub fn create_with_path(
        &self,
        owner_id: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> LocalStorage {
        LocalStorage::with_path(owner_id, path)
    }
}

/// System LocalStorage with pre-loaded pqdos_system data
/// This is used for the pqdos_system user to have access to the blockchain repo
pub fn create_pqdos_system_storage() -> LocalStorage {
    let storage = LocalStorage::new("pqdos_system");

    // Add the test block provided by the user
    let mut test_block_metadata = HashMap::new();
    test_block_metadata.insert("name".to_string(), "first_block".to_string());
    test_block_metadata.insert("description".to_string(), "First test block in pqdos".to_string());
    test_block_metadata.insert("mime_type".to_string(), "text/plain".to_string());

    let _test_data = general_purpose::STANDARD.decode("SGVsbG8sIHBrZG9zIQ==").unwrap_or_default();

    // Manually create the block with all fields to match the provided JSON
    let block_with_metadata = LocalBlock {
        id: "f2872c9437ddccb0e9b56569f93d6cf0d7bfb5d45911e137abaf7203283a7655".to_string(),
        previous_id: Some(
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        ),
        data: "SGVsbG8sIHBrZG9zIQ==".to_string(),
        owner_id: "pqdos_system".to_string(),
        block_type: "data".to_string(),
        timestamp: 1717000060,
        signature: Some(
            "304502201a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7"
                .to_string(),
        ),
        signer: None,
        metadata: test_block_metadata,
    };

    storage.blocks.write().insert(
        "f2872c9437ddccb0e9b56569f93d6cf0d7bfb5d45911e137abaf7203283a7655".to_string(),
        block_with_metadata,
    );

    storage
}
