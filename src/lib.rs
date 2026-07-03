//! Distributed Post-Quantum Secure Operating System
//!
//! A novel OS architecture treating all storage as content-addressed,
//! encrypted memory blocks with immutable history on a distributed blockchain.

pub mod block;
pub mod blockchain;
pub mod crypto;
pub mod error;
pub mod integration;
pub mod memory;
pub mod network;
pub mod storage;
pub mod users;

pub use error::{Error, Result};
pub use integration::create_system_integration_with_demo_keys;
pub use storage::local::create_pqdos_system_storage;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Test that the library can be imported
        assert_eq!(2 + 2, 4);
    }
}
