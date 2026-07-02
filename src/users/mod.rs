//! User management module for Post-Quantum Distributed OS
//!
//! This module provides a comprehensive user system with:
//! - Abstract traits for extensibility (see `traits` module)
//! - Genesis user "futuros" who owns all system executable blocks
//! - Authentication system
//! - Block ownership tracking
//! - Simple in-memory implementation (see `simple` module)
//!
//! **Security Note**: The genesis user's private key is NEVER stored or accessible
//! through this system. Only the public key is stored. The private key must be
//! kept in a secure external location.

mod simple;
pub mod traits;

// ============================================================================
// RE-EXPORTS FROM TRAITS
// ============================================================================

pub use traits::{
    AuthResult,
    // Types
    AuthToken,
    AuthenticationProvider,
    Block as BlockTrait,
    BlockBuilder as BlockBuilderTrait,
    BlockId as BlockIdTrait,
    ExecutableBlock as ExecutableBlockTrait,
    ExecutableBlockBuilder as ExecutableBlockBuilderTrait,
    User as UserTrait,
    UserAuthenticator as UserAuthenticatorTrait,
    UserBuilder as UserBuilderTrait,
    // Traits
    UserId as UserIdTrait,
    UserIndex,
    UserPermissions as UserPermissionsTrait,
    UserRole as UserRoleTrait,
    UserStorageBackend,
    UserSystem as UserSystemTrait,
    UserSystemError,
    UserSystemFactory as UserSystemFactoryTrait,
    UserSystemResult,

    // Constants
    GENESIS_USER_NAME,
};

// ============================================================================
// RE-EXPORTS FROM SIMPLE IMPLEMENTATION
// ============================================================================

pub use simple::{
    create_user_system, create_user_system_with_demo_keys, Block, BlockId, ExecutableBlock,
    SimpleBlockBuilder, SimpleExecutableBlockBuilder, SimpleUserBuilder, SimpleUserSystemFactory,
    User, UserId, UserPermissions, UserRole, UserSystem,
};

// ============================================================================
// CONVENIENCE RE-EXPORTS
// ============================================================================

/// The name of the genesis user - owner of all system executable blocks
pub const GENESIS_USER: &str = GENESIS_USER_NAME;

// ============================================================================
// AUTHENTICATION SYSTEM
// ============================================================================

/// Error type for user system operations (original implementation)
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("System not initialized")]
    SystemNotInitialized,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<UserSystemError> for UserError {
    fn from(error: UserSystemError) -> Self {
        match error {
            | UserSystemError::UserNotFound => UserError::UserNotFound,
            | UserSystemError::UserAlreadyExists => UserError::UserAlreadyExists,
            | UserSystemError::SystemNotInitialized => UserError::SystemNotInitialized,
            | UserSystemError::PermissionDenied => UserError::PermissionDenied,
            | UserSystemError::InvalidOperation(msg) => UserError::InvalidOperation(msg),
            | UserSystemError::InternalError(msg) => UserError::InternalError(msg),
            | UserSystemError::BlockNotFound => {
                UserError::InvalidOperation("Block not found".to_string())
            },
            | UserSystemError::BlockAlreadyExists => {
                UserError::InvalidOperation("Block already exists".to_string())
            },
        }
    }
}

/// Authentication token
#[derive(Debug, Clone)]
pub struct AuthTokenCompat {
    pub token: Vec<u8>,
    pub user_id: UserId,
    pub expires_at: i64,
    pub issued_at: i64,
}

impl AuthTokenCompat {
    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        now > self.expires_at
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResultCompat {
    pub token: AuthTokenCompat,
    pub user: User,
}

/// User authentication system
pub struct UserAuthenticatorCompat {
    user_system: UserSystem,
}

impl UserAuthenticatorCompat {
    /// Create a new authenticator for the user system
    pub fn new(user_system: UserSystem) -> Self {
        Self { user_system }
    }

    /// Authenticate a user by verifying a signature
    pub fn authenticate(
        &self,
        user_id: &UserId,
        challenge: &[u8],
        _signature: &[u8],
    ) -> Result<AuthResultCompat, UserError> {
        let user = self.user_system.get_user(user_id).ok_or(UserError::UserNotFound)?;

        use sha2::{Digest, Sha256};
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

        let mut hasher = Sha256::new();
        hasher.update(user_id.as_bytes());
        hasher.update(challenge);
        hasher.update(now.to_be_bytes());
        let token_bytes = hasher.finalize().to_vec();

        let token = AuthTokenCompat {
            token: token_bytes,
            user_id: user_id.clone(),
            expires_at: now + 3600,
            issued_at: now,
        };

        Ok(AuthResultCompat { token, user })
    }

    /// Generate a random challenge for authentication
    pub fn generate_challenge(&self) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();

        let mut hasher = Sha256::new();
        hasher.update(now.to_be_bytes());
        hasher.finalize().to_vec()
    }

    /// Validate an authentication token
    pub fn validate_token(&self, token: &AuthTokenCompat) -> Result<User, UserError> {
        if token.is_expired() {
            return Err(UserError::InvalidOperation("Token expired".to_string()));
        }

        let user = self.user_system.get_user(&token.user_id).ok_or(UserError::UserNotFound)?;

        Ok(user)
    }

    /// Get the user system
    pub fn user_system(&self) -> &UserSystem {
        &self.user_system
    }

    /// Get the genesis user
    pub fn get_genesis_user(&self) -> Result<User, UserError> {
        self.user_system.get_genesis_user().ok_or(UserError::UserNotFound)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_system_initialization() {
        let mut system = UserSystem::new();
        let public_key = vec![1u8; 64];

        system.initialize_with_futuros(public_key.clone()).unwrap();

        assert!(system.is_initialized());

        let genesis_user = system.get_genesis_user().unwrap();
        assert_eq!(genesis_user.name, GENESIS_USER);
        assert_eq!(genesis_user.public_key, public_key);
        assert!(genesis_user.is_genesis());
    }

    #[test]
    fn test_system_block_ownership() {
        let mut system = UserSystem::new();
        let public_key = vec![2u8; 64];

        system.initialize_with_futuros(public_key).unwrap();

        let kernel_code = vec![0u8, 1, 2, 3, 4];
        let kernel_block_id = system
            .register_system_executable(
                "kernel".to_string(),
                kernel_code.clone(),
                "main".to_string(),
                "kernel".to_string(),
            )
            .unwrap();

        let owner_id = system.get_block_owner(&kernel_block_id).unwrap();
        let genesis_user = system.get_genesis_user().unwrap();

        assert_eq!(owner_id, genesis_user.id);
        assert!(system.is_system_block(&kernel_block_id));

        let executable = system.get_system_executable(&kernel_block_id).unwrap();
        assert_eq!(executable.code(), &kernel_code);
    }

    #[test]
    fn test_authentication() {
        let mut system = UserSystem::new();
        let public_key = vec![3u8; 64];

        system.initialize_with_futuros(public_key).unwrap();

        let authenticator = UserAuthenticatorCompat::new(system);
        let genesis_user = authenticator.get_genesis_user().unwrap();

        let challenge = authenticator.generate_challenge();
        let dummy_signature = vec![0u8; 64];

        let auth_result =
            authenticator.authenticate(&genesis_user.id, &challenge, &dummy_signature).unwrap();

        let validated_user = authenticator.validate_token(&auth_result.token).unwrap();
        assert_eq!(validated_user.id, genesis_user.id);
    }

    #[test]
    fn test_private_key_inaccessible() {
        let mut system = UserSystem::new();
        let public_key = vec![4u8; 64];

        system.initialize_with_futuros(public_key.clone()).unwrap();

        let genesis_user = system.get_genesis_user().unwrap();

        assert_eq!(genesis_user.public_key, public_key);
    }
}
