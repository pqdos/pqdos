# Post-Quantum Distributed OS (PQOS)

A novel operating system architecture that unifies memory hierarchy through content-addressed, encrypted blocks with immutable history tracked on a distributed blockchain.

## Quick Overview

PQOS treats all storage as a **unified memory layer** where:
- **Files = Encrypted Memory Blocks** identified by cryptographic signatures (similar to Git)
- **Content Addressing** ensures deduplication and integrity
- **Blockchain Ledger** maintains immutable version history and access control
- **Post-Quantum Cryptography** protects against quantum adversaries
- **Distributed Consensus** enables decentralized operation

## Documentation

📖 **Full documentation is available in [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)**

This includes:
- Complete architecture overview
- Detailed module documentation (Crypto, Block, Blockchain, Network, Memory, **Users**)
- Abstract traits system explanation
- Implementation patterns and examples
- Security architecture and considerations
- Development guidelines

## Module Structure

```
pqos/
├── Cargo.toml                 # Project configuration
├── README.md                  # This file
├── src/
│   ├── lib.rs                 # Library entry point
│   ├── error.rs               # Global error types
│   ├── crypto/                # Cryptographic traits
│   │   └── traits.rs
│   ├── block/                 # Content-addressed blocks
│   │   └── traits.rs
│   ├── blockchain/            # Distributed ledger
│   │   └── traits.rs
│   ├── network/               # P2P communication
│   │   └── traits.rs
│   ├── memory/                # Unified memory abstraction
│   │   └── traits.rs
│   └── users/                 # User management system
│       ├── mod.rs
│       ├── traits.rs          # Abstract user traits
│       └── simple.rs          # Reference implementation
└── docs/
    └── ARCHITECTURE.md        # Complete documentation
```

## Key Features

- ✅ **Post-Quantum Secure**: NIST-approved PQC algorithms (ML-KEM/Kyber, ML-DSA/Dilithium)
- ✅ **Content-Addressed**: All data identified by cryptographic hash
- ✅ **Blockchain-Backed**: Immutable history with cryptographic integrity
- ✅ **Distributed**: P2P network with pluggable consensus
- ✅ **Unified Memory**: RAM, files, network storage use same addressing
- ✅ **Evolvable Architecture**: Trait-based design allows pluggable implementations

## Getting Started

### Prerequisites

- Rust 1.70+
- liboqs-rs (optional, for post-quantum crypto)
- Standard development tools

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

## Development

The project uses a **trait-based architecture** for maximum evolvability. All core functionality is defined through abstract traits that can be implemented with various technologies.

See [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) for complete development guidelines.

## Contributing

We welcome contributions! The trait-based architecture makes it easy to add:

- New cryptographic implementations (PQC algorithms)
- New storage backends (databases, distributed systems)
- New network protocols (P2P, QUIC, etc.)
- New consensus algorithms (Raft, BFT, etc.)

Please see [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) for implementation patterns and guidelines.

## License

This project is licensed under the **MIT License** - see LICENSE file for details.

---

## Quick Links

- 📖 [Full Documentation](./docs/ARCHITECTURE.md)
- 🔐 [Crypto Module](./src/crypto/traits.rs)
- 🧱 [Block Module](./src/block/traits.rs)
- ⛓️ [Blockchain Module](./src/blockchain/traits.rs)
- 🌐 [Network Module](./src/network/traits.rs)
- 💾 [Memory Module](./src/memory/traits.rs)
- 👥 [Users Module](./src/users/traits.rs) - *Newly documented!*
