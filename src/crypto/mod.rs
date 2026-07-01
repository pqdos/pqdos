//! Cryptographic module for Post-Quantum Secure OS
//!
//! This module provides abstract cryptographic traits that allow for
//! pluggable cryptography implementations (PQC, classical, etc.).

pub mod traits;

pub use traits::{
    CryptoAlgorithms,
    CryptoProvider,
    CryptoResult,
    HashFunction,
    Kdf,
    Kem,
    KeyManager,
    SecureRng,
    SignatureScheme,
    SymmetricEncryption,
};
