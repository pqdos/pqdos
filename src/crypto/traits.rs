//! Abstract cryptographic traits for Post-Quantum Secure OS
//!
//! These traits define interfaces for cryptographic operations without
//! committing to specific implementations or libraries.
//! They allow for pluggable cryptography (PQC, classical, etc.).

use std::sync::Arc;

/// Error type for cryptographic operations
pub type CryptoResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Trait for hash functions (SHA3, BLAKE3, etc.)
pub trait HashFunction: Send + Sync {
    /// Output type of the hash function
    type Output: AsRef<[u8]> + Clone + Eq + std::hash::Hash + Send + Sync;

    /// Hash input data
    fn hash(&self, input: &[u8]) -> Self::Output;

    /// Hash data from a stream/iterator
    fn hash_stream<'a>(&self, chunks: impl Iterator<Item = &'a [u8]>) -> Self::Output;

    /// Verify that input hashes to expected value
    fn verify(&self, input: &[u8], expected: &[u8]) -> bool {
        self.hash(input).as_ref() == expected
    }

    /// Return the hash function name
    fn name(&self) -> &str;

    /// Return the output size in bytes
    fn output_size(&self) -> usize;
}

/// Trait for symmetric encryption schemes (AES-GCM, ChaCha20-Poly1305, etc.)
pub trait SymmetricEncryption: Send + Sync {
    /// Key type for encryption
    type Key: Clone + AsRef<[u8]> + Send + Sync;
    /// Nonce/IV type for encryption
    type Nonce: Clone + AsRef<[u8]> + Send + Sync;
    /// Error type for encryption operations
    type Error: std::error::Error + Send + Sync;

    /// Encrypt plaintext with given key and nonce
    fn encrypt(
        &self,
        key: &Self::Key,
        nonce: &Self::Nonce,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, Self::Error>;

    /// Decrypt ciphertext with given key and nonce
    fn decrypt(
        &self,
        key: &Self::Key,
        nonce: &Self::Nonce,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, Self::Error>;

    /// Generate a new random key
    fn generate_key(&self) -> Self::Key;

    /// Generate a new random nonce
    fn generate_nonce(&self) -> Self::Nonce;

    /// Return the encryption scheme name
    fn name(&self) -> &str;

    /// Return the key size in bytes
    fn key_size(&self) -> usize;

    /// Return the nonce size in bytes
    fn nonce_size(&self) -> usize;

    /// Return the authentication tag size in bytes (for AEAD schemes)
    fn tag_size(&self) -> usize;
}

/// Trait for Key Encapsulation Mechanisms (KEM) - Post-Quantum key exchange
///
/// Used for establishing shared secrets (e.g., ML-KEM/Kyber, BIKE, etc.)
pub trait Kem: Send + Sync {
    /// Public key type
    type PublicKey: Clone + AsRef<[u8]> + Send + Sync;
    /// Private key type
    type PrivateKey: Clone + AsRef<[u8]> + Send + Sync;
    /// Shared secret type
    type SharedSecret: Clone + AsRef<[u8]> + Send + Sync;
    /// Ciphertext type for encapsulated key
    type Ciphertext: Clone + AsRef<[u8]> + Send + Sync;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Generate a new key pair
    fn generate_keypair(&self) -> (Self::PublicKey, Self::PrivateKey);

    /// Encapsulate: generate shared secret and encapsulate it with recipient's public key
    fn encapsulate(
        &self,
        recipient_public_key: &Self::PublicKey,
    ) -> Result<(Self::SharedSecret, Self::Ciphertext), Self::Error>;

    /// Decapsulate: recover shared secret from ciphertext using private key
    fn decapsulate(
        &self,
        private_key: &Self::PrivateKey,
        ciphertext: &[u8],
    ) -> Result<Self::SharedSecret, Self::Error>;

    /// Return the KEM scheme name
    fn name(&self) -> &str;

    /// Return the public key size in bytes
    fn public_key_size(&self) -> usize;

    /// Return the private key size in bytes
    fn private_key_size(&self) -> usize;

    /// Return the shared secret size in bytes
    fn shared_secret_size(&self) -> usize;

    /// Return the ciphertext size in bytes
    fn ciphertext_size(&self) -> usize;
}

/// Trait for digital signature schemes (ML-DSA/Dilithium, etc.)
pub trait SignatureScheme: Send + Sync {
    /// Public key type
    type PublicKey: Clone + AsRef<[u8]> + Send + Sync;
    /// Private/Signing key type
    type PrivateKey: Clone + AsRef<[u8]> + Send + Sync;
    /// Signature type
    type Signature: Clone + AsRef<[u8]> + Send + Sync;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Generate a new key pair for signing
    fn generate_keypair(&self) -> (Self::PublicKey, Self::PrivateKey);

    /// Sign a message with the private key
    fn sign(
        &self,
        private_key: &Self::PrivateKey,
        message: &[u8],
    ) -> Result<Self::Signature, Self::Error>;

    /// Verify a signature with the public key
    fn verify(
        &self,
        public_key: &Self::PublicKey,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, Self::Error>;

    /// Return the signature scheme name
    fn name(&self) -> &str;

    /// Return the public key size in bytes
    fn public_key_size(&self) -> usize;

    /// Return the private key size in bytes
    fn private_key_size(&self) -> usize;

    /// Return the signature size in bytes
    fn signature_size(&self) -> usize;
}

/// Trait for key derivation functions (HKDF, etc.)
pub trait Kdf: Send + Sync {
    type Output: AsRef<[u8]> + Clone + Send + Sync;
    type Error: std::error::Error + Send + Sync;

    /// Derive a key from input key material and context
    fn derive(
        &self,
        ikm: &[u8],
        salt: Option<&[u8]>,
        info: &[u8],
        output_size: usize,
    ) -> Result<Self::Output, Self::Error>;

    /// Return the KDF name
    fn name(&self) -> &str;
}

/// Trait for cryptographic random number generation
pub trait SecureRng: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    /// Fill a buffer with random bytes
    fn fill_bytes(&self, dest: &mut [u8]) -> Result<(), Self::Error>;

    /// Generate random bytes
    fn generate_bytes(&self, size: usize) -> Result<Vec<u8>, Self::Error>;

    /// Return the RNG name
    fn name(&self) -> &str;
}

/// Trait for key management (storage, rotation, derivation)
pub trait KeyManager: Send + Sync {
    /// Unique identifier for keys
    type KeyId: Clone + Eq + std::hash::Hash + AsRef<[u8]> + Send + Sync;
    /// Key material type
    type Key: Clone + AsRef<[u8]> + Send + Sync;
    /// Error type
    type Error: std::error::Error + Send + Sync;

    /// Generate a new cryptographic key
    fn generate_key(&mut self, algorithm: &str) -> Result<Self::KeyId, Self::Error>;

    /// Retrieve a key by its identifier
    fn get_key(&self, key_id: &Self::KeyId) -> Result<Self::Key, Self::Error>;

    /// Store a key with an identifier
    fn store_key(&mut self, key_id: Self::KeyId, key: Self::Key) -> Result<(), Self::Error>;

    /// Remove a key
    fn remove_key(&mut self, key_id: &Self::KeyId) -> Result<(), Self::Error>;

    /// Rotate a key: generate new key, re-encrypt data, remove old key
    fn rotate_key(&mut self, old_key_id: &Self::KeyId) -> Result<Self::KeyId, Self::Error>;

    /// Derive a new key from an existing key with context
    fn derive_key(
        &self,
        base_key_id: &Self::KeyId,
        context: &[u8],
    ) -> Result<Self::KeyId, Self::Error>;

    /// List all available key identifiers
    fn list_keys(&self) -> Result<Vec<Self::KeyId>, Self::Error>;

    /// Check if a key exists
    fn has_key(&self, key_id: &Self::KeyId) -> Result<bool, Self::Error>;
}

/// Trait for a cryptographic provider factory
///
/// Allows creating cryptographic primitives with consistent configuration
pub trait CryptoProvider: Send + Sync {
    /// Hash function type
    type Hash: HashFunction;
    /// Symmetric encryption type
    type SymmetricEncryption: SymmetricEncryption;
    /// KEM type
    type Kem: Kem;
    /// Signature scheme type
    type SignatureScheme: SignatureScheme;
    /// KDF type
    type Kdf: Kdf;
    /// Secure RNG type
    type SecureRng: SecureRng;
    /// Key manager type
    type KeyManager: KeyManager;

    /// Create a hash function instance
    fn hash_function(&self, algorithm: &str) -> Self::Hash;

    /// Create a symmetric encryption instance
    fn symmetric_encryption(&self, algorithm: &str) -> Self::SymmetricEncryption;

    /// Create a KEM instance
    fn kem(&self, algorithm: &str) -> Self::Kem;

    /// Create a signature scheme instance
    fn signature_scheme(&self, algorithm: &str) -> Self::SignatureScheme;

    /// Create a KDF instance
    fn kdf(&self, algorithm: &str) -> Self::Kdf;

    /// Create a secure RNG instance
    fn secure_rng(&self) -> Self::SecureRng;

    /// Create a key manager instance
    fn key_manager(&self) -> Self::KeyManager;

    /// List available algorithms for each category
    fn available_algorithms(&self) -> CryptoAlgorithms;
}

/// Available cryptographic algorithms
#[derive(Debug, Clone)]
pub struct CryptoAlgorithms {
    pub hash: Vec<String>,
    pub symmetric_encryption: Vec<String>,
    pub kem: Vec<String>,
    pub signature: Vec<String>,
    pub kdf: Vec<String>,
}
