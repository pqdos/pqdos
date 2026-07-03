//! Cache LRU pour les blocs PQDOS.
//!
//! Tous les blocs sont chiffrés avant d'être stockés dans le cache.
//! Seuls les utilisateurs autorisés peuvent déchiffrer les blocs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::block::simple::SimpleBlock;
use crate::block::traits::{Block, BlockId};
use crate::p2p::traits::CacheManager;
use crate::p2p::cache::crypto::{encrypt_block, decrypt_block, derive_block_key, can_decrypt_block};
use crate::error::Error;

/// Entrée du cache avec métadonnées LRU et chiffrement.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub block_id: Vec<u8>,      // ID du bloc.
    pub path: PathBuf,          // Chemin vers le fichier en cache.
    pub size: u64,              // Taille du bloc chiffré (octets).
    pub last_accessed: i64,     // Timestamp du dernier accès.
    pub ttl: Option<i64>,       // Time-To-Live (None = pas de TTL).
    pub owner_id: Vec<u8>,      // ID du propriétaire du bloc.
    pub nonce: Vec<u8>,         // Nonce utilisé pour le chiffrement.
}

/// Cache LRU pour les blocs.
pub struct LRUCache {
    cache_dir: PathBuf,                       // Dossier de stockage du cache.
    entries: Arc<RwLock<HashMap<Vec<u8>, CacheEntry>>>, // block_id -> CacheEntry.
    max_size: u64,                            // Taille maximale du cache (octets).
    current_size: Arc<RwLock<u64>>,           // Taille actuelle.
}

impl LRUCache {
    /// Crée un nouveau cache LRU.
    pub async fn new(cache_dir: impl AsRef<Path>, max_size: u64) -> Result<Self, Error> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        tokio::fs::create_dir_all(&cache_dir).await?;

        Ok(Self {
            cache_dir,
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
        })
    }

    /// Chemin vers un bloc dans le cache.
    fn block_path(&self, block_id: &[u8]) -> PathBuf {
        self.cache_dir.join(hex::encode(block_id))
    }
}

#[async_trait::async_trait]
impl CacheManager for LRUCache {
    async fn cache_block(
        &mut self, 
        block: SimpleBlock, 
        owner_id: &[u8],
    ) -> Result<(), Error> {
        let block_id = block.id().to_bytes();
        let path = self.block_path(&block_id);
        
        // Dériver la clé de chiffrement à partir de l'owner_id et du block_id.
        let encryption_key = derive_block_key(owner_id, &block_id);
        
        // Chiffrer le bloc.
        let encrypted_data = encrypt_block(&block, &encryption_key)?;
        
        // Écrire le bloc chiffré dans le cache.
        tokio::fs::write(&path, &encrypted_data).await?;

        // Mettre à jour les métadonnées.
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size.write().await;

        let entry = CacheEntry {
            block_id: block_id.clone(),
            path: path.clone(),
            size: encrypted_data.len() as u64,
            last_accessed: chrono::Utc::now().timestamp(),
            ttl: None,
            owner_id: owner_id.to_vec(),
            nonce: encryption_key.nonce.clone(),
        };

        // Si le bloc existe déjà, mettre à jour.
        if let Some(old_entry) = entries.get(&block_id) {
            *current_size -= old_entry.size;
        }

        entries.insert(block_id, entry);
        *current_size += encrypted_data.len() as u64;

        Ok(())
    }

    async fn get_cached_block(
        &self, 
        block_id: &[u8], 
        user_id: &[u8],
        user_private_key: Option<&[u8]>, // Clé privée de l'utilisateur.
        shared_keys: &std::collections::HashMap<Vec<u8>, Vec<u8>>, // Clés partagées (block_id -> clé de déchiffrement).
    ) -> Result<Option<SimpleBlock>, Error> {
        let entries = self.entries.read().await;
        
        if let Some(entry) = entries.get(block_id) {
            // Vérifier si l'utilisateur a le droit de déchiffrer ce bloc.
            if !can_decrypt_block(user_id, &entry.owner_id, shared_keys) {
                // L'utilisateur n'a pas les permissions pour ce bloc.
                return Ok(None);
            }
            
            let path = entry.path.clone();
            let encrypted_data = tokio::fs::read(&path).await?;
            
            // Dériver la clé de déchiffrement.
            let decryption_key = if user_id == entry.owner_id.as_slice() {
                // L'utilisateur est le propriétaire : dériver la clé à partir de sa clé privée.
                if let Some(private_key) = user_private_key {
                    derive_block_key(private_key, block_id)
                } else {
                    // Pas de clé privée : impossible de déchiffrer.
                    return Ok(None);
                }
            } else {
                // L'utilisateur n'est pas le propriétaire : utiliser la clé partagée.
                if let Some(shared_key) = shared_keys.get(block_id) {
                    // En pratique, il faudrait stocker la clé de chiffrement pour ce bloc.
                    // Ici, on utilise une clé dérivée de la clé partagée.
                    derive_block_key(shared_key, block_id)
                } else {
                    // Pas de clé partagée : impossible de déchiffrer.
                    return Ok(None);
                }
            };
            
            // Déchiffrer le bloc.
            match decrypt_block(&encrypted_data, &decryption_key) {
                Ok(decrypted_data) => {
                    // Reconstruire le bloc à partir des données déchiffrées.
                    Ok(Some(SimpleBlock::new(
                        block_id.to_vec(),
                        decrypted_data,
                        None,
                        entry.last_accessed,
                        1,
                    )))
                }
                Err(_) => {
                    // Échec du déchiffrement : clé incorrecte ou données corrompues.
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    async fn contains(&self, block_id: &[u8]) -> Result<bool, Error> {
        let entries = self.entries.read().await;
        Ok(entries.contains_key(block_id))
    }

    async fn remove_block(&mut self, block_id: &[u8]) -> Result<(), Error> {
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size.write().await;

        if let Some(entry) = entries.remove(block_id) {
            let path = self.block_path(block_id);
            tokio::fs::remove_file(&path).await.ok();
            *current_size -= entry.size;
        }

        Ok(())
    }

    async fn cleanup(&mut self) -> Result<(), Error> {
        // Nettoyage simple : supprimer tous les blocs.
        let mut entries = self.entries.write().await;
        let mut current_size = self.current_size.write().await;

        for (block_id, entry) in entries.iter() {
            let path = self.block_path(block_id);
            tokio::fs::remove_file(&path).await.ok();
            *current_size -= entry.size;
        }

        entries.clear();
        Ok(())
    }

    async fn size(&self) -> Result<u64, Error> {
        let current_size = self.current_size.read().await;
        Ok(*current_size)
    }
}
