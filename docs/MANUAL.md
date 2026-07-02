# pqdos Shell CLI - User Manual

## Table of Contents

1. [Overview](#overview)
2. [Installation](#installation)
3. [Getting Started](#getting-started)
4. [Command Reference](#command-reference)
5. [Authentication](#authentication)
6. [File System Operations](#file-system-operations)
7. [Memory Block Management](#memory-block-management)
8. [System Administration](#system-administration)
9. [Examples](#examples)
10. [Security Notes](#security-notes)
11. [Troubleshooting](#troubleshooting)

---

## Overview

The **pqdos Shell** is the command-line interface for the Post-Quantum Distributed Operating System. It provides a unified environment where:

- **Files are treated as content-addressed encrypted memory blocks** (similar to Git)
- **All operations are cryptographically verified** using post-quantum algorithms
- **User authentication is mandatory** for accessing personal memory blocks
- **The genesis user "pqdos"** owns all system executable blocks
- **Blockchain-backed history** ensures immutable version tracking

---

## Installation

### Prerequisites

- Rust 1.70+
- Standard development tools (gcc, make, etc.)
- Optional: liboqs-rs for post-quantum cryptography

### Building

```bash
# Clone the repository
git clone https://github.com/your-repo/pqos.git
cd pqos

# Build in release mode
cargo build --release

# The binary will be available at:
# target/release/pqos
```

### Running

```bash
# Run the shell
./target/release/pqos

# Or with cargo
cargo run --release
```

---

## Getting Started

### First Launch

When you first launch pqdos Shell, it will:

1. **Verify self-integrity** - Check that the current binary matches the latest version registered in the blockchain by user "pqdos"
2. **Prompt for authentication** - You must login or create an account to proceed
3. **Display the shell prompt** - Once authenticated, you'll see the interactive prompt

```
╔══════════════════════════════════════════════════════════════════════╗
║  Post-Quantum Distributed Operating System (pqdos) Shell           ║
║                        vX.X.X - CLI Interface                         ║
╚══════════════════════════════════════════════════════════════════════╝

🔍 Verifying self-integrity...
✅ Self-integrity verified - Binary matches system blockchain

🚀 pqdos Shell ready. Type 'help' for available commands.

pqos:$ 
```

### Shell Prompt

The shell prompt displays:
- **Username** (if authenticated)
- **Current working directory**

Format: `pqos@{user}:{current_directory} $`

---

## Command Reference

### General Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `help` | `?` | Display this help message |
| `version` | `-v`, `--version` | Display pqdos version |
| `clear` | `cls` | Clear the screen |
| `exit` | `quit`, `q` | Exit the shell |

### Authentication Commands

| Command | Description |
|---------|-------------|
| `login` | `auth` | Login to the system |
| `logout` | Logout from the system |

### File System Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `pwd` | | Show current directory |
| `cd <path>` | | Change directory (supports virtual paths) |
| `ls [path]` | `dir` | List files in directory |
| `ll [path]` | `ls -l`, `ls -la` | List files with details |
| `cp <src> <dest>` | `copy` | Copy file (to virtual dir converts to block) |
| `rm <file\|block>` | `remove`, `del` | Remove file or block |

### Memory Block Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `blocks` | `myblocks` | List all memory blocks owned by current user |
| `blockinfo <id>` | `binfo` | Show detailed information about a block |

---

## Authentication

### Logging In

To access memory block features, you must authenticate:

```bash
# Start authentication
login

# You will be prompted for:
# Username: [your_username]

# For demo purposes, any username (except "futuros") will work
# In production, you would provide a cryptographic signature
```

### Creating an Account

```bash
# Start authentication menu
login

# Select option 2 to create account
2

# Enter your desired username
Username: myuser

# Account created!
✅ Account created for 'myuser'
```

### Genesis User "futuros"

The **futuros** user is a special system user:

- **Owns all system executable blocks** (kernel, shell, bootstrap)
- **Cannot be logged into normally** - Only accessible with the private key (which is NEVER stored in the system)
- **Used for system integrity verification** - The shell checks that the running binary matches a block owned by pqdos

⚠️ **SECURITY NOTE**: The private key for "pqdos" is **NEVER stored or accessible** through pqdos. It must be kept in a secure external location (Hardware Security Module, secure enclave, etc.).

### Authentication Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Authentication Process                        │
├─────────────────────────────────────────────────────────────┤
│  1. User requests login                                         │
│  2. System generates random challenge                           │
│  3. User signs challenge with private key (EXTERNAL PROCESS)    │
│  4. System verifies signature with stored public key           │
│  5. If valid: authentication token issued                       │
│  6. Token expires after 1 hour (configurable)                   │
└─────────────────────────────────────────────────────────────┘
```

### Logout

```bash
# Logout from current session
logout

# You will return to unauthenticated state
# Memory block commands will be unavailable until login
```

---

## File System Operations

### Navigation

```bash
# Show current directory
pwd

# Change to home directory
cd

# Change to specific directory
cd /path/to/directory

# Change to parent directory
cd ..

# Change to virtual blocks directory (authenticated users only)
cd /users/myuser/blocks
```

### Listing Files

```bash
# List files in current directory
ls

# List files in specific directory
ls /path/to/directory

# List with details (size, date, permissions)
ll
ll /path/to/directory
```

### Copying Files

**Important**: When copying a file to your virtual blocks directory, it is **automatically converted to a signed memory block**.

```bash
# Copy a local file to your blocks directory
cp /path/to/local/file.txt /users/myuser/blocks/

# The system will:
# 1. Read the file content
# 2. Compute its content hash (SHA256)
# 3. Create a memory block with your ownership
# 4. Store it in your virtual directory
# 5. Return the block ID

✅ File converted to memory block
   Block ID: a1b2c3d4e5f6...
   Owner: myuser
   Original Name: file.txt
   The file is now stored as a content-addressed block
```

### Removing Files and Blocks

```bash
# Remove a local file
rm local_file.txt

# Remove a memory block (by block ID)
rm a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890

# Note: Block IDs are 64-character hex strings
```

---

## Memory Block Management

### Understanding Memory Blocks

In pqdos, **everything is a memory block**:

- **Content-Addressed**: Each block is identified by a cryptographic hash of its content (like Git)
- **Encrypted**: All blocks are encrypted at rest
- **Signed**: Blocks can be cryptographically signed
- **Owned**: Each block has an owner (a user)
- **Typed**: Blocks have types (file, system, executable, etc.)
- **Metadata**: Blocks can have arbitrary metadata attached

### Block Types

| Type | Description |
|------|-------------|
| `file` | Regular file data |
| `system` | System configuration or data |
| `executable` | Executable code (owned by futuros for system code) |
| `kernel` | OS kernel executable |
| `shell` | Shell executable |
| `bootstrap` | System bootstrap code |
| `driver` | Hardware driver |
| `service` | System service |

### Listing Your Blocks

```bash
# List all blocks you own
blocks

# Output:
Memory Blocks owned by myuser:

#  Block ID                                 Type          Size      Created              Name
----------------------------------------------------------------------------------------------------
1  a1b2c3d4e5f6...7890               file          1.5 KB   2024-07-01 14:30:22   file.txt
2  b2c3d4e5f6a7...8901               file          2.3 KB   2024-07-01 14:31:45   document.pdf

Total: 2 block(s)
```

### Block Information

```bash
# Get detailed information about a block
blockinfo a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890

# Output:
Block Information:
  ID: a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890
  Type: file
  Owner: myuser
  Original Name: file.txt
  Size: 1536 bytes (1 KB)
  MIME Type: text/plain
  Original Path: /home/user/file.txt
  Created: 2024-07-01 14:30:22
  Signature: a1b2c3d4...
```

### Virtual Directory Structure

Each authenticated user has a virtual directory at:
```
/users/<username>/blocks/
```

This directory contains **symlinks or references** to all blocks owned by that user.

```bash
# Navigate to your virtual directory
cd /users/myuser/blocks

# List your blocks
ls

# Output:
file.txt
document.pdf
image.png

# These are references to memory blocks, not actual files
```

---

## System Administration

### Self-Integrity Verification

On startup, pqdos Shell automatically verifies that:

1. The current binary's hash matches a block in the system blockchain
2. That block is owned by the genesis user "pqdos"
3. That block is marked as a system executable

This ensures that **only authorized system code can run**.

```
🔍 Verifying self-integrity...
✅ Self-integrity verified - Binary matches system blockchain
```

If verification fails:
```
⚠️  Self-integrity: Binary not found in system blocks
   This is expected during development
   In production, this would prevent execution if verification fails
```

### System Blocks

System executable blocks are owned by **futuros** and include:

- **kernel** - The OS kernel code
- **shell** - The CLI shell executable
- **bootstrap** - System initialization code
- **drivers** - Hardware drivers
- **services** - System services

These blocks **cannot be modified** without the pqdos private key.

### Version Information

```bash
# Check pqdos version
version

# Output:
pqdos Shell v1.0.0
Post-Quantum Distributed Operating System
```

---

## Examples

### Example 1: Basic File Operations

```bash
# Start the shell
./pqos

# Login (or create account)
login
1
Username: myuser

# Navigate to home directory
cd

# List files
ls

# Copy a file to memory blocks
cp ~/important.doc /users/myuser/blocks/

# List your blocks
blocks

# View block info
blockinfo [block_id]

# Logout
logout

# Exit
exit
```

### Example 2: Working with Memory Blocks

```bash
# Login
login
1
Username: myuser

# Navigate to virtual directory
cd /users/myuser/blocks

# List blocks (appears as files)
ls

# View detailed block information
blockinfo a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890

# Copy another file to blocks
cp ~/data.csv /users/myuser/blocks/

# List blocks again
blocks

# Remove a block
rm a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890
```

### Example 3: Multiple Sessions

```bash
# Session 1: Create account and add files
./pqos
login
2
Username: alice

# Add some files
cp file1.txt /users/alice/blocks/
cp file2.txt /users/alice/blocks/

exit

# Session 2: Login as alice and access files
./pqos
login
1
Username: alice

# View blocks from previous session
blocks
cd /users/alice/blocks
ls
```

---

## Security Notes

### ⚠️ CRITICAL SECURITY PRINCIPLES

1. **Private Keys Are NEVER Stored**
   - pqdos **never** stores private keys
   - Only **public keys** are accessible through the system
   - Private keys must be kept in **external secure storage** (HSM, secure enclave, offline storage)

2. **Genesis User "pqdos"**
   - Owns all system executable blocks
   - Private key is **completely inaccessible** through pqdos
   - Used only for **system integrity verification**

3. **Authentication**
   - In **demo mode**: Any username works (for development/testing)
   - In **production mode**: Requires cryptographic signature verification
   - Signatures are verified using stored public keys
   - Actual signing must be done **externally** with private keys

4. **Block Ownership**
   - Each block is owned by a specific user
   - Users can only access their own blocks (unless granted permissions)
   - System blocks are owned by "futuros" and cannot be modified by regular users

5. **Content Addressing**
   - Blocks are identified by **content hash**, not by name
   - Identical content will have the **same block ID** (deduplication)
   - Changing a single byte changes the entire block ID

### Encryption

All data in pqdos is:

- **Encrypted at rest** using symmetric encryption (AES-256-GCM by default)
- **Content-addressed** using cryptographic hashing (SHA-256 by default)
- **Signed** for integrity verification (when applicable)

### Blockchain Security

- **Immutable history**: Once a block is added to the blockchain, it cannot be modified
- **Cryptographic linking**: Each block contains the hash of the previous block
- **Consensus verification**: All nodes must agree on the state (when distributed)

---

## Troubleshooting

### Common Issues

#### "Self-integrity: Binary not found in system blocks"

**Cause**: This is normal during development. The running binary doesn't match any block in the system blockchain.

**Solution**: 
- In development: This is expected and safe
- In production: Register your binary as a system block owned by "futuros"

#### "Please login first" or "Permission denied"

**Cause**: You're trying to access memory block features without authentication.

**Solution**:
```bash
login
# Enter your username
```

#### "cd: permission denied: not your blocks directory"

**Cause**: You're trying to access another user's blocks directory.

**Solution**: Only access your own blocks directory:
```bash
cd /users/your_username/blocks
```

#### "Block not found"

**Cause**: The block ID doesn't exist or you don't have permission to access it.

**Solution**:
- Verify the block ID is correct (64 hex characters)
- Check you're logged in as the correct user
- Use `blocks` to see your available blocks

#### "Destination must be in your blocks directory"

**Cause**: When copying to virtual directory, destination must be in `/users/<yourname>/blocks/`

**Solution**:
```bash
# Correct:
cp file.txt /users/myuser/blocks/

# Incorrect:
cp file.txt /users/otheruser/blocks/
```

### Error Messages

| Message | Meaning | Action |
|---------|---------|--------|
| `❌ Username cannot be empty` | Empty username provided | Provide a valid username |
| `❌ Username 'futuros' is reserved` | Attempting to create futuros account | Use a different username |
| `❌ Permission denied: please login first` | Not authenticated | Run `login` first |
| `❌ Destination must be in your blocks directory` | Invalid copy destination | Use correct virtual path |
| `❌ Source file not found` | File doesn't exist | Verify file path |
| `❌ Block not found` | Block ID doesn't exist | Verify block ID |

---

## Command Summary Cheat Sheet

```
╔══════════════════════════════════════════════════════════════════════════╗
║                         pqdos SHELL COMMANDS                              ║
╠══════════════════════════════════════════════════════════════════════════╣
║                                                                       ║
║  📋 SYSTEM                                                           ║
║     help, ?           Show help                                      ║
║     version, -v       Show version                                   ║
║     clear, cls        Clear screen                                    ║
║     exit, quit, q     Exit shell                                      ║
║                                                                       ║
║  🔐 AUTHENTICATION                                                   ║
║     login, auth      Login to system                                  ║
║     logout           Logout from system                               ║
║                                                                       ║
║  📁 FILE SYSTEM                                                      ║
║     pwd             Show current directory                           ║
║     cd <path>       Change directory                                   ║
║     ls [path]       List files                                        ║
║     ll [path]       List files with details                            ║
║     cp <src> <dest> Copy file (to blocks/ converts to block)             ║
║     rm <file>       Remove file                                       ║
║     rm <block_id>   Remove memory block                                ║
║                                                                       ║
║  🧱 MEMORY BLOCKS (Requires Authentication)                           ║
║     blocks, myblocks    List your memory blocks                        ║
║     blockinfo <id>      Show block information                         ║
║     cd /users/<you>/blocks/   Navigate to your blocks directory        ║
║     cp <file> /users/<you>/blocks/    Convert file to memory block    ║
║                                                                       ║
╚══════════════════════════════════════════════════════════════════════════╝
```

---

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────────────────┐
│  pqdos SHELL - QUICK REFERENCE                                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  NAVIGATION:                                                           │
│    pwd                    Show current directory                      │
│    cd <path>             Change directory                              │
│    cd                    Go to home directory                         │
│    cd /users/me/blocks  Go to my memory blocks                        │
│                                                                         │
│  FILE OPERATIONS:                                                      │
│    ls                    List files                                   │
│    ll                    List with details                            │
│    cp src dest          Copy file (to blocks/ = convert to block)     │
│    rm file              Remove file                                  │
│    rm <64-hex>          Remove memory block                           │
│                                                                         │
│  BLOCK OPERATIONS:                                                     │
│    blocks               List my memory blocks                         │
│    blockinfo <id>       Show block details                            │
│    cp file /users/me/blocks/   Add file as memory block               │
│                                                                         │
│  SYSTEM:                                                               │
│    help                 This help                                    │
│    version              Show version                                 │
│    clear                Clear screen                                 │
│    exit                 Exit shell                                   │
│    login                Authenticate                                 │
│    logout               End session                                  │
│                                                                         │
│  TIP: Use TAB for auto-completion (if supported by your terminal)      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` | Previous command |
| `↓` | Next command |
| `Tab` | Auto-complete (if supported) |
| `Ctrl+C` | Cancel current input |
| `Ctrl+D` | Exit shell (same as `exit`) |
| `Ctrl+L` | Clear screen (same as `clear`) |

---

## Support

For issues, questions, or contributions:

- Check the [Architecture Documentation](./ARCHITECTURE.md) for technical details
- Review this manual for usage questions
- For bugs, please report with:
  - pqdos version (`version` command)
  - Steps to reproduce
  - Expected vs actual behavior

---

*Last updated: 2024 | pqdos v1.0.0 | Post-Quantum Distributed OS*
