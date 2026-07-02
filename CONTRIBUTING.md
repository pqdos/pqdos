# Contributing to pqdos

Thank you for your interest in contributing to **pqdos** - the Post-Quantum Distributed Operating System!

This document provides guidelines for setting up your development environment and contributing code to the project.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Environment Setup](#development-environment-setup)
3. [Project Structure](#project-structure)
4. [Coding Guidelines](#coding-guidelines)
5. [Formatting and Linting](#formatting-and-linting)
6. [Testing](#testing)
7. [Commit Guidelines](#commit-guidelines)
8. [Pull Request Process](#pull-request-process)
9. [Code Review](#code-review)

---

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust 1.70+** (recommended: latest stable)
- **Cargo** (comes with Rust)
- **Git**
- **CMake 3.15+**
- **Ninja** (optional, but recommended for faster builds)
- **Clang/LLVM** (for build dependencies)
- **pkg-config**

### Quick Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add required components
rustup component add rustfmt clippy rust-src

# Clone the repository
git clone https://github.com/pqdos/pqdos.git
cd pqdos

# Build the project
cargo build --release

# Run tests
cargo test --all-features
```

---

## Development Environment Setup

### Required System Dependencies

#### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    cmake \
    ninja-build \
    libclang-dev \
    git
```

#### macOS (using Homebrew)

```bash
brew install \
    cmake \
    ninja \
    llvm \
    pkg-config
```

#### Windows (using Chocolatey)

```powershell
choco install \
    cmake \
    ninja \
    llvm \
    git
```

### Rust Toolchain Configuration

The project uses a standard Rust toolchain with the following components:

- **rustc**: Latest stable
- **cargo**: Latest stable
- **rustfmt**: Code formatting
- **clippy**: Linting

To ensure you have the correct components:

```bash
rustup component add rustfmt clippy
```

### Verify Your Setup

```bash
# Check Rust version
rustc --version

# Check components
rustup component list

# Run a test build
cargo check --all-features
```

---

## Project Structure

```
pqdos/
├── Cargo.toml                 # Project configuration
├── Cargo.lock                 # Dependency lock file
├── .rustfmt.toml              # Rustfmt configuration
├── README.md                  # Project overview
├── CONTRIBUTING.md            # This file
├── src/
│   ├── lib.rs                 # Library entry point
│   ├── main.rs                # CLI entry point
│   ├── error.rs               # Global error types
│   ├── crypto/
│   │   └── traits.rs          # Cryptographic traits
│   ├── block/
│   │   └── traits.rs          # Content-addressed block traits
│   ├── blockchain/
│   │   └── traits.rs          # Distributed ledger traits
│   ├── network/
│   │   └── traits.rs          # P2P communication traits
│   ├── memory/
│   │   └── traits.rs          # Unified memory abstraction
│   └── users/
│       ├── mod.rs             # User module
│       ├── traits.rs          # Abstract user traits
│       └── simple.rs          # Reference implementation
└── docs/
    └── ARCHITECTURE.md        # Complete documentation
```

---

## Coding Guidelines

### General Principles

1. **Trait-Based Architecture**: All core functionality should be defined through abstract traits. This ensures the system remains evolvable and technology-agnostic.

2. **Error Handling**: Use `anyhow` for application-level errors and custom error types for library errors.

3. **Async-First**: Prefer async/await for I/O operations.

4. **Type Safety**: Leverage Rust's type system to prevent errors at compile time.

5. **Zero-Cost Abstractions**: Use Rust's monomorphization to avoid runtime overhead.

---

## Formatting and Linting

### Automatic Formatting with rustfmt

The project uses **rustfmt** for consistent code formatting. A configuration file `.rustfmt.toml` is provided at the repository root.

#### Check Formatting

```bash
cargo fmt --all -- --check
```

#### Apply Formatting

```bash
cargo fmt --all
```

#### Configuration

The `.rustfmt.toml` file contains the following settings:

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Max"
match_arm_leading_pipes = "Always"
match_block_trailing_comma = true
```

**Important:** All code must pass `cargo fmt --all -- --check` before being committed.

### Linting with Clippy

Clippy is used to catch common mistakes and improve code quality.

#### Run Clippy

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

#### Common Clippy Warnings

- **type_complexity**: Simplify complex type signatures with type aliases
- **unused_imports**: Remove unused imports
- **dead_code**: Remove or annotate unused code with `#[allow(dead_code)]`
- **ptr_arg**: Prefer `&Path` over `&PathBuf`
- **map_clone**: Use `Option::cloned()` or remove unnecessary clones

**Note:** Some deprecated functions (e.g., `chrono::NaiveDateTime::from_timestamp_opt`) are explicitly allowed with `#[allow(deprecated)]` where necessary for compatibility.

---

## Testing

### Run All Tests

```bash
cargo test --all-features --all-targets
```

### Run Tests for a Specific Module

```bash
cargo test --package pqdos --lib users
```

### Run Tests with Coverage (requires tarpaulin)

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --all-features
```

### Test Workflow

1. Always run tests before committing
2. Ensure all tests pass in CI
3. Add new tests for new functionality
4. Update existing tests when behavior changes

---

## Commit Guidelines

### Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/) style:

```
<type>(<scope>): <description>

[body]

[footer]
```

#### Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (formatting)
- `refactor`: Code changes that neither fix a bug nor add a feature
- `perf`: Code changes that improve performance
- `test`: Adding or fixing tests
- `chore`: Changes to the build process or auxiliary tools
- `config`: Configuration changes

#### Examples

```bash
# Good commit messages
git commit -m "feat(users): add user authentication"
git commit -m "fix(crypto): correct signature verification"
git commit -m "style: format code with rustfmt"
git commit -m "docs: update architecture documentation"
git commit -m "chore: update dependencies"

# Bad commit messages (avoid)
git commit -m "fixed bug"
git commit -m "changes"
git commit -m "WIP"
```

### Atomic Commits

- Each commit should represent a single logical change
- Keep commits small and focused
- Avoid mixing unrelated changes in a single commit

---

## Pull Request Process

### Before Submitting

1. **Rebase your branch** on the latest `develop` branch:
   ```bash
   git fetch origin
   git rebase origin/develop
   ```

2. **Run all checks locally:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features --all-targets
   ```

3. **Squash related commits** into logical units

### Pull Request Template

```markdown
## Description

[Brief description of the changes]

## Related Issue

[Link to relevant issue, if any]

## Changes Made

- [ ] Bug fix
- [ ] New feature
- [ ] Documentation
- [ ] Tests
- [ ] Refactoring
- [ ] Other

## Testing

- [ ] All tests pass
- [ ] Formatting is correct
- [ ] Clippy warnings resolved
- [ ] Manual testing performed

## Checklist

- [ ] Code follows the project's coding guidelines
- [ ] All tests pass
- [ ] Documentation updated (if applicable)
- [ ] Commit messages are descriptive
```

### PR Submission

1. Push your branch to GitHub:
   ```bash
   git push origin feature/your-feature
   ```

2. Open a Pull Request to the `develop` branch

3. Fill out the PR template

4. Request review from at least one maintainer

---

## Code Review

### Review Process

1. **Automated Checks**: All PRs must pass CI (formatting, clippy, tests)
2. **Human Review**: At least one maintainer must approve the PR
3. **Address Feedback**: Respond to review comments and update the PR accordingly

### Review Guidelines for Contributors

- Be responsive to review comments
- Update the PR to address feedback
- Add clarification if you disagree with a suggestion
- Keep discussions constructive and respectful

### Review Guidelines for Maintainers

- Provide clear, actionable feedback
- Explain the rationale behind suggestions
- Be timely with reviews
- Acknowledge good work
- Use approval emoji (👍, ❤️) for minor changes

---

## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Clippy Documentation](https://rust-lang.github.io/rust-clippy/)
- [rustfmt Documentation](https://rust-lang.github.io/rustfmt/)

---

## Getting Help

If you have questions or need assistance:

1. **Check existing issues**: Your question may have already been answered
2. **Open a new issue**: For bug reports or feature requests
3. **Join the discussion**: [Link to community forum/discord if available]

---

## License

By contributing to pqdos, you agree that your contributions will be licensed under the **MIT License**. See [LICENSE](LICENSE) for details.

---

**Thank you for contributing to pqdos!** Your contributions help make this project better for everyone.
