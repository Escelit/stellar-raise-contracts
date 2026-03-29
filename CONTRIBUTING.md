# Contributing Guide

Thank you for your interest in contributing to this project!
Please read this guide carefully before opening a Pull Request.

## Prerequisites

- Rust (stable): https://rustup.rs
- Soroban CLI: `cargo install soroban-cli`
- wasm32 target: `rustup target add wasm32-unknown-unknown`

## Setting Up Locally

1. Fork and clone the repository:

```bash
git clone https://github.com/<your-username>/stellar-raise-contracts.git
cd stellar-raise-contracts
```

2. Build the contract:

```bash
cargo build --target wasm32-unknown-unknown --release
```

3. Run the test suite:

```bash
cargo test
```

## Branching Convention

Always branch off `develop`, never `main`:

```bash
git checkout develop
git pull origin develop
git checkout -b feature/your-feature-name
```

## Submitting a Pull Request

1. Ensure all tests pass:

```bash
cargo test
```

2. Ensure code is formatted:

```bash
cargo fmt --all
```

3. Ensure no Clippy warnings:

```bash
cargo clippy --all-targets -- -D warnings
```

4. Push your branch and open a PR targeting `develop`.
5. Reference the issue number with `Closes #<issue-number>` in your PR description.

## Code Style

- Follow standard Rust formatting enforced by `cargo fmt`.
- All public functions must have `///` doc comments.
- All new features must include tests.
- Commit messages must follow conventional commits format:
  - `feat:`
  - `fix:`
  - `docs:`
  - `test:`
  - `ci:`
  - `chore:`

## Need Help?

Open a Discussion or comment on the relevant issue. We are happy to help.
