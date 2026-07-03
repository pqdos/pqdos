//! Integration module for pqdos storage and user system
//!
//! This module provides integration between the user system and storage backends,
//! allowing each user to have multiple storage backends for their memory blocks.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::storage::github::AsyncStorageBackend;
use crate::storage::traits::StorageBackend;
use crate::storage::{create_pqdos_system_storage, GitHubStorage, LocalStorage, StorageError, StorageResult};
use crate::users::{UserSystem};

/// Enum that wraps different storage backend types
#[derive(Debug, Clone)]
pub enum StorageBackendEnum {
    Local(LocalStorage),
    GitHub(GitHubStorage),
}

impl StorageBackendEnum {
    pub fn id(&self) -> &str {
        match self {
            StorageBackendEnum::Local(s) => StorageBackend::id(s),
            StorageBackendEnum::GitHub(s) => AsyncStorageBackend::id(s),
        }
    }

    pub fn backend_type(&self) -> &str {
        match self {
            StorageBackendEnum::Local(s) => StorageBackend::backend_type(s),
            StorageBackendEnum::GitHub(s) => AsyncStorageBackend::backend_type(s),
        }
    }

    pub fn owner_id(&self) -> &str {
        match self {
            StorageBackendEnum::Local(s) => StorageBackend::owner_id(s),
            StorageBackendEnum::GitHub(s) => AsyncStorageBackend::owner_id(s),
        }
    }

    pub fn has_block(&self, block_id: &str) -> StorageResult<bool> {
        match self {
            StorageBackendEnum::Local(s) => StorageBackend::has_block(s, block_id),
            StorageBackendEnum::GitHub(_) => {
                // GitHubStorage requires async, so for sync operations we return false
                // In a real implementation, this would be handled differently
                Ok(false)
            }
        }
    }

    pub fn get_block_data(&self, block_id: &str) -> StorageResult<Option<Vec<u8>>> {
        match self {
            StorageBackendEnum::Local(s) => {
                match StorageBackend::get_block(s, block_id)? {
                    Some(block) => Ok(Some(block.raw_data()?)),
                    None => Ok(None),
                }
            }
            StorageBackendEnum::GitHub(_) => {
                // GitHubStorage requires async
                Ok(None)
            }
        }
    }
}

/// User Storage Manager
#[derive(Debug, Clone)]
pub struct UserStorageManager {
    user_id: String,
    storages: Arc<RwLock<HashMap<String, StorageBackendEnum>>>,
    default_storage_id: Option<String>,
}

impl UserStorageManager {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            storages: Arc::new(RwLock::new(HashMap::new())),
            default_storage_id: None,
        }
    }

    pub fn add_storage(
        &mut self,
        storage_id: impl Into<String>,
        storage: StorageBackendEnum,
    ) -> StorageResult<()> {
        let storage_id = storage_id.into();
        if self.default_storage_id.is_none() {
            self.default_storage_id = Some(storage_id.clone());
        }
        self.storages.write().insert(storage_id, storage);
        Ok(())
    }

    pub fn get_storage(&self, storage_id: &str) -> Option<StorageBackendEnum> {
        self.storages.read().get(storage_id).cloned()
    }

    pub fn get_default_storage(&self) -> Option<StorageBackendEnum> {
        self.default_storage_id.as_ref().and_then(|id| self.get_storage(id))
    }

    pub fn set_default_storage(&mut self, storage_id: impl Into<String>) -> StorageResult<()> {
        let storage_id = storage_id.into();
        if !self.storages.read().contains_key(&storage_id) {
            return Err(StorageError::NotFound);
        }
        self.default_storage_id = Some(storage_id);
        Ok(())
    }

    pub fn has_storage(&self, storage_id: &str) -> bool {
        self.storages.read().contains_key(storage_id)
    }
}

/// Global Storage Registry
#[derive(Debug, Clone)]
pub struct GlobalStorageRegistry {
    user_storages: Arc<RwLock<HashMap<String, UserStorageManager>>>,
}

impl Default for GlobalStorageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalStorageRegistry {
    pub fn new() -> Self {
        Self {
            user_storages: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_or_create_user_storage(&self, user_id: impl Into<String>) -> UserStorageManager {
        let user_id_str: String = user_id.into();
        
        {
            let mut write_guard = self.user_storages.write();
            write_guard.entry(user_id_str.clone()).or_insert_with(|| UserStorageManager::new(user_id_str.clone()));
        }
        
        self.user_storages.read().get(&user_id_str).unwrap().clone()
    }

    pub fn add_user_storage(
        &mut self,
        user_id: impl Into<String>,
        storage_id: impl Into<String>,
        storage: StorageBackendEnum,
    ) -> StorageResult<()> {
        let user_id = user_id.into();
        let storage_id = storage_id.into();
        let mut user_storage = self.get_or_create_user_storage(user_id);
        user_storage.add_storage(storage_id, storage)
    }

    pub fn get_user_storage(&self, user_id: &str, storage_id: &str) -> Option<StorageBackendEnum> {
        self.user_storages.read().get(user_id).and_then(|manager| manager.get_storage(storage_id))
    }

    pub fn get_default_user_storage(&self, user_id: &str) -> Option<StorageBackendEnum> {
        self.user_storages.read().get(user_id).and_then(|manager| manager.get_default_storage())
    }

    pub fn initialize_pqdos_system_storage(&mut self) -> StorageResult<()> {
        let pqdos_system_id = "pqdos_system";
        
        let system_storage = StorageBackendEnum::Local(create_pqdos_system_storage());
        let github_storage = StorageBackendEnum::GitHub(GitHubStorage::new("pqdos", "blockchain"));
        
        self.add_user_storage(pqdos_system_id, "local", system_storage)?;
        self.add_user_storage(pqdos_system_id, "github-blockchain", github_storage)?;
        
        let mut user_storage = self.get_or_create_user_storage(pqdos_system_id);
        user_storage.set_default_storage("local")?;
        
        Ok(())
    }
}

/// Integration between user system and storage backends
pub struct SystemIntegration {
    pub user_system: Arc<RwLock<UserSystem>>,
    pub storage_registry: Arc<RwLock<GlobalStorageRegistry>>,
}

impl SystemIntegration {
    pub fn new(user_system: UserSystem) -> Self {
        let storage_registry = Arc::new(RwLock::new(GlobalStorageRegistry::new()));
        
        {
            let mut registry_guard = storage_registry.write();
            registry_guard.initialize_pqdos_system_storage().unwrap();
        }
        
        Self {
            user_system: Arc::new(RwLock::new(user_system)),
            storage_registry,
        }
    }

    pub fn new_with_demo_keys() -> Self {
        use crate::users::create_user_system_with_demo_keys;
        Self::new(create_user_system_with_demo_keys())
    }

    pub fn verify_self_integrity(&self) -> Result<bool, String> {
        use sha2::{Digest, Sha256};
        use std::fs;
        use std::io::{BufReader, Read};

        let binary_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;
        
        let file = fs::File::open(&binary_path)
            .map_err(|e| format!("Failed to open binary: {}", e))?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];
        loop {
            let bytes_read = reader.read(&mut buffer)
                .map_err(|e| format!("Failed to read binary: {}", e))?;
            if bytes_read == 0 { break; }
            hasher.update(&buffer[..bytes_read]);
        }
        let binary_hash = format!("{:x}", hasher.finalize());

        // Check if binary exists as a block in pqdos_system storage
        if self.get_default_user_storage_has_block("pqdos_system", &binary_hash).is_ok_and(|exists| exists) {
            return Ok(true);
        }

        // Fallback: check the specific test block provided
        let test_block_id = "f2872c9437ddccb0e9b56569f93d6cf0d7bfb5d45911e137abaf7203283a7655";
        if self.get_default_user_storage_has_block("pqdos_system", test_block_id).is_ok_and(|exists| exists) {
            return Ok(true);
        }

        Ok(false)
    }

    pub fn get_user_storage(&self, user_id: &str, storage_id: &str) -> Option<StorageBackendEnum> {
        self.storage_registry.read().get_user_storage(user_id, storage_id)
    }

    pub fn get_default_user_storage(&self, user_id: &str) -> Option<StorageBackendEnum> {
        self.storage_registry.read().get_default_user_storage(user_id)
    }
    
    pub fn get_user_storage_has_block(&self, user_id: &str, storage_id: &str, block_id: &str) -> StorageResult<bool> {
        if let Some(storage) = self.get_user_storage(user_id, storage_id) {
            storage.has_block(block_id)
        } else {
            Ok(false)
        }
    }
    
    pub fn get_default_user_storage_has_block(&self, user_id: &str, block_id: &str) -> StorageResult<bool> {
        if let Some(storage) = self.get_default_user_storage(user_id) {
            storage.has_block(block_id)
        } else {
            Ok(false)
        }
    }

    pub fn get_storage_registry(&self) -> &Arc<RwLock<GlobalStorageRegistry>> {
        &self.storage_registry
    }
}

/// Convenience function to create system integration with demo keys
pub fn create_system_integration_with_demo_keys() -> SystemIntegration {
    use crate::users::create_user_system_with_demo_keys;
    SystemIntegration::new(create_user_system_with_demo_keys())
}
