//! Module de chiffrement pour le cache P2P.
//!
//! Ce module gère le chiffrement et déchiffrement des blocs stockés dans le cache local.
//! Tous les blocs sont chiffrés avant d'être stockés, et seuls les utilisateurs autorisés
//! peuvent les déchiffrer.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use crate::block::simple::SimpleBlock;
use crate::block::traits::Block;
use crate::error::Error;

/// Taille du nonce pour AES-256-GCM (96 bits = 12 octets).
const NONCE_SIZE: usize = 12;

/// Clé de chiffrement pour un bloc.
/// Contient la clé AES-256 et le nonce utilisé pour le chiffrement.
#[derive(Debug, Clone)]
pub struct BlockEncryptionKey {
    pub key: Vec<u8>,      // Clé AES-256 (32 octets).
    pub nonce: Vec<u8>,    // Nonce (12 octets).
}

impl BlockEncryptionKey {
    pub fn new() -> Self {
        let key = vec![0u8; 32]; // À générer aléatoirement en pratique.
        let nonce = vec![0u8; NONCE_SIZE];
        Self { key, nonce }
    }

    pub fn from_user_key(user_key: &[u8]) -> Self {
        // En pratique, il faudrait dériver la clé de manière sécurisée (ex: HKDF).
        // Ici, on utilise une clé fixe pour l'exemple.
        let mut key = vec![0u8; 32];
        key.copy_from_slice(&user_key[..32.min(user_key.len())]);
        let nonce = vec![0u8; NONCE_SIZE];
        Self { key, nonce }
    }
}

/// Chiffre un bloc avec une clé donnée.
pub fn encrypt_block(block: &SimpleBlock, key: &BlockEncryptionKey) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new_from_slice(&key.key)
        .map_err(|e| Error::CryptoError(format!("Failed to create cipher: {}", e)))?;
    
    let nonce = Nonce::from_slice(&key.nonce);
    let data = block.data().as_ref();
    
    cipher
        .encrypt(nonce, data)
        .map_err(|e| Error::CryptoError(format!("Failed to encrypt block: {}", e)))
}

/// Déchiffre un bloc avec une clé donnée.
pub fn decrypt_block(encrypted_data: &[u8], key: &BlockEncryptionKey) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new_from_slice(&key.key)
        .map_err(|e| Error::CryptoError(format!("Failed to create cipher: {}", e)))?;
    
    let nonce = Nonce::from_slice(&key.nonce);
    
    cipher
        .decrypt(nonce, encrypted_data)
        .map_err(|e| Error::CryptoError(format!("Failed to decrypt block: {}", e)))
}

/// Génère une clé aléatoire pour un bloc.
pub fn generate_block_key() -> BlockEncryptionKey {
    use rand::RngCore;
    let mut key = vec![0u8; 32];
    OsRng.fill_bytes(&mut key);
    
    let mut nonce = vec![0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce);
    
    BlockEncryptionKey { key, nonce }
}

/// Dérive une clé de chiffrement à partir de la clé privée de l'utilisateur et du block_id.
/// Cela permet de s'assurer que chaque bloc a une clé unique, même pour le même utilisateur.
pub fn derive_block_key(user_private_key: &[u8], block_id: &[u8]) -> BlockEncryptionKey {
    // En pratique, il faudrait utiliser une fonction de dérivation de clé (ex: HKDF).
    // Ici, on utilise une approche simplifiée pour l'exemple.
    use sha3::{Sha3_256, Digest};
    
    let mut hasher = Sha3_256::new();
    hasher.update(user_private_key);
    hasher.update(block_id);
    let hash = hasher.finalize();
    
    let mut key = vec![0u8; 32];
    key.copy_from_slice(&hash[..32]);
    
    let mut nonce = vec![0u8; NONCE_SIZE];
    nonce.copy_from_slice(&hash[32..32 + NONCE_SIZE]);
    
    BlockEncryptionKey { key, nonce }
}

/// Vérifie si l'utilisateur a le droit de déchiffrer un bloc.
/// Un utilisateur peut déchiffrer un bloc si :
/// - Il est le propriétaire du bloc.
/// - Il a reçu la clé de déchiffrement du propriétaire.
pub fn can_decrypt_block(
    user_id: &[u8],
    block_owner_id: &[u8],
    shared_keys: &std::collections::HashMap<Vec<u8>, Vec<u8>>,
) -> bool {
    // Si l'utilisateur est le propriétaire, il peut déchiffrer.
    if user_id == block_owner_id {
        return true;
    }
    
    // Sinon, vérifier si l'utilisateur a une clé partagée pour ce bloc.
    // En pratique, il faudrait stocker les permissions par bloc.
    // Ici, on vérifie simplement si l'utilisateur a une clé pour le propriétaire.
    shared_keys.contains_key(block_owner_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt() {
        let key = BlockEncryptionKey::new();
        let block = SimpleBlock::new(
            vec![0x01; 32],
            b"Hello, world!".to_vec(),
            None,
            0,
            1,
        );
        
        let encrypted = encrypt_block(&block, &key).unwrap();
        let decrypted = decrypt_block(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, b"Hello, world!");
    }
    
    #[test]
    fn test_derive_block_key() {
        let user_key = vec![0x42; 64];
        let block_id = vec![0x01; 32];
        
        let key1 = derive_block_key(&user_key, &block_id);
        let key2 = derive_block_key(&user_key, &block_id);
        
        // La même clé doit être dérivée pour le même utilisateur et block_id.
        assert_eq!(key1.key, key2.key);
        assert_eq!(key1.nonce, key2.nonce);
    }
}
