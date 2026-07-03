//! Distributed Post-Quantum Secure Operating System
//!
//! A novel OS architecture treating all storage as content-addressed,
//! encrypted memory blocks with immutable history on a distributed blockchain.

pub mod block;
pub mod blockchain;
pub mod crypto;
pub mod error;
pub mod memory;
pub mod network;
pub mod users;
pub mod p2p;

pub use error::{Error, Result};

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
