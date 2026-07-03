//! Stockage physique des blocs dans le cache.

use std::path::PathBuf;
use crate::error::Error;

/// Gère le stockage physique des blocs dans le cache.
pub struct CacheStorage {
    cache_dir: PathBuf,
}

impl CacheStorage {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Chemin vers un bloc dans le cache.
    pub fn block_path(&self, block_id: &[u8]) -> PathBuf {
        self.cache_dir.join(hex::encode(block_id))
    }

    /// Écrit un bloc dans le cache.
    pub async fn write_block(&self, block_id: &[u8], data: &[u8]) -> Result<(), Error> {
        let path = self.block_path(block_id);
        tokio::fs::write(&path, data).await?;
        Ok(())
    }

    /// Lit un bloc depuis le cache.
    pub async fn read_block(&self, block_id: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let path = self.block_path(block_id);
        if path.exists() {
            let data = tokio::fs::read(&path).await?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    /// Supprime un bloc du cache.
    pub async fn delete_block(&self, block_id: &[u8]) -> Result<(), Error> {
        let path = self.block_path(block_id);
        tokio::fs::remove_file(&path).await?;
        Ok(())
    }
}
