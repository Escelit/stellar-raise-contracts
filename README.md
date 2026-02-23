# 🚀 Stellar Raise Contracts

![codecov](https://codecov.io/gh/Mac-5/stellar-raise-contracts/branch/develop/graph/badge.svg)

A **crowdfunding smart contract** built on the [Stellar](https://stellar.org/) network using [Soroban](https://soroban.stellar.org/).

## Overview

Stellar Raise lets anyone create a crowdfunding campaign on-chain. Contributors pledge tokens toward a goal before a deadline. If the goal is met, the creator withdraws the funds. If not, contributors are refunded automatically.

### Key Features

| Feature        | Description                                        |
| :------------- | :------------------------------------------------- |
| **Initialize** | Create a campaign with a goal, deadline, and token |
| **Contribute** | Pledge tokens before the deadline |
| **Withdraw** | Creator claims funds after a successful campaign |
| **Refund** | Contributors individually reclaim tokens if the goal is missed (pull-based) |
| **Contribute** | Pledge tokens before the deadline                  |
| **Withdraw**   | Creator claims funds after a successful campaign   |
| **Refund**     | Contributors reclaim tokens if the goal is missed  |

## Architecture

The following diagram shows the high-level relationship between the frontend, Soroban smart contracts, and the Stellar network:

```mermaid
graph TD
    A[User Browser] -->|HTTP / HTTPS| B[Frontend<br/>Next.js / React]

    B -->|Soroban SDK calls| C[Soroban RPC Node]
    B -->|Horizon API calls| D[Horizon API Server]

    C -->|Deploy / Invoke| E[Soroban Smart Contracts<br/>Crowdfund Contract<br/>Factory Contract]
    D -->|Account / Transaction data| B

    E -->|Read / Write state| F[Stellar Ledger]
    C -->|Submit transactions| F

    F -->|Ledger events| G[Off-chain Indexer<br/>Optional]
    G -->|Indexed campaign data| B

    subgraph network["Stellar Network"]
        C
        D
        F
    end

    subgraph contracts["Smart Contracts Layer"]
        E
    end

    subgraph services["Off-chain Services"]
        G
    end
```

### Layer Summary

| Layer                 | Technology            | Responsibility                           |
| :-------------------- | :-------------------- | :--------------------------------------- |
| **Frontend**          | Next.js / React       | User interface and wallet interaction    |
| **Smart Contracts**   | Soroban / Rust        | Campaign logic, fund management          |
| **Stellar Network**   | Soroban RPC + Horizon | Transaction processing and ledger state  |
| **Off-chain Indexer** | Optional              | Event indexing for faster data retrieval |

## Project Structure

```text
stellar-raise-contracts/
├── .github/workflows/rust_ci.yml   # CI pipeline
├── contracts/crowdfund/
│   ├── src/
│   │   ├── lib.rs                  # Smart contract logic
│   │   └── test.rs                 # Unit tests
│   └── Cargo.toml                  # Contract dependencies
├── Cargo.toml                      # Workspace config
├── CONTRIBUTING.md
├── README.md
└── LICENSE
```

## Prerequisites

| Requirement | Minimum | Notes |
| :--- | :--- | :--- |
| OS | Linux x86-64 or macOS 12+ | WSL2 on Windows |
| RAM | 4 GB | 8 GB recommended for `--release` builds |
| Rust | stable ≥ 1.74 | `rustup update stable` |
| Stellar CLI | ≥ 20.0.0 | Renamed from `soroban` in v20 |
| Node.js | ≥ 18 | Required for frontend UI and JS tests |

### Install Rust and WASM target

```bash
# Install Rust via rustup (https://rustup.rs)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Add the WASM compilation target
rustup target add wasm32-unknown-unknown

# Verify
rustc --version
rustup target list --installed | grep wasm32
```

### Install Stellar CLI

```bash
curl -Ls https://soroban.stellar.org/install-soroban.sh | sh
source ~/.bashrc   # or ~/.zshrc
stellar --version  # should print stellar-cli x.y.z
```

### Install Node.js (frontend UI)

Node.js ≥ 18 is required to run the frontend and its test suite.

```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
source ~/.bashrc
nvm install 18
nvm use 18
node --version   # v18.x.x
npm --version
```

> Alternatively, download directly from [nodejs.org](https://nodejs.org/).

## Getting Started

```bash
# Clone the repo
git clone https://github.com/<your-org>/stellar-raise-contracts.git
cd stellar-raise-contracts

# Build the contract
cargo build --release --target wasm32-unknown-unknown

# Run contract tests
cargo test --workspace

# Install frontend dependencies
cd frontend
npm install

# Run frontend tests (single pass, no watch)
npm test -- --run

# Start the frontend dev server (run manually in your terminal)
# npm run dev
```

> See [`docs/readme_md_installation.md`](docs/readme_md_installation.md) for edge cases, automated environment verification, and frontend-specific troubleshooting.

## Contract Interface

```rust
// Create a new campaign
fn initialize(env, creator, token, goal, deadline, min_contribution);

// Pledge tokens to the campaign
fn contribute(env, contributor, amount);

// Creator withdraws after successful campaign
fn withdraw(env);

// Individual contributor claims refund if goal not met (pull-based)
fn refund_single(env, contributor);

// View functions
fn total_raised(env) -> i128;
fn goal(env) -> i128;
fn deadline(env) -> u64;
fn contribution(env, contributor) -> i128;
fn min_contribution(env) -> i128;
```

## Pull-based Refund Model

This contract uses a **pull-based refund** pattern for scalability and gas efficiency.

### Why Pull-based?

A traditional "push" refund (where one transaction refunds all contributors) would:
- Fail with thousands of contributors due to resource limits
- Be expensive and unpredictable in cost
- Create a single point of failure

### How it Works

If the campaign goal is **not met** by the deadline:
1. Each contributor must claim their own refund by calling `refund_single`
2. Contributors can claim at any time after the deadline
3. The refund is processed immediately and securely

### Example: Claiming Your Refund

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <YOUR_SECRET_KEY> \
  -- refund_single \
  --contributor <YOUR_ADDRESS>
```

## Upgrading the Contract

Once deployed, the contract can be upgraded to a new WASM implementation without changing its address or losing stored data. This allows the project to ship fixes and improvements without redeploying.

### Upgrade Procedure

1. **Build the new WASM binary:**

   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```

2. **Upload the new WASM to the network:**

   ```bash
   stellar contract install \
     --wasm target/wasm32-unknown-unknown/release/crowdfund.wasm \
     --network testnet \
     --source <YOUR_SECRET_KEY>
   ```

   This returns the WASM hash (SHA-256).

3. **Invoke the upgrade function:**
   ```bash
   stellar contract invoke \
     --id <CONTRACT_ADDRESS> \
     --fn upgrade \
     --arg <WASM_HASH> \
     --network testnet \
     --source <YOUR_SECRET_KEY>
   ```

### Important Notes

- Only the **admin** (set to the campaign creator at initialization) can call the upgrade function.
- The upgrade is **irreversible** — ensure the new WASM is thoroughly tested before upgrading.
- All contract storage and state persist across upgrades.
- The contract address remains the same after an upgrade.
- **Recommendation:** Have at least two reviewers approve upgrade PRs before merging to production.

## Deployment

### Using the Deployment Script

We provide automated scripts to simplify deploying and interacting with the crowdfund contract on testnet.

#### Prerequisites

1. **Install Stellar CLI (v20+):**

   ```bash
   curl -Ls https://soroban.stellar.org/install-soroban.sh | sh
   source ~/.bashrc   # or ~/.zshrc
   stellar --version  # should print stellar-cli x.y.z
   ```

2. **Configure your Stellar identity:**

   ```bash
   stellar keys generate --global alice
1. **Install Soroban CLI:**

   ```bash
   curl -Ls https://soroban.stellar.org/install-soroban.sh | sh
   ```

2. **Configure your Soroban identity:**

   ```bash
   soroban keys generate --global <alice>
   ```

3. **Add the testnet network:**
   ```bash
   stellar network add testnet \
     --rpc-url https://soroban-testnet.stellar.org:443 \
     --network-passphrase "Test SDF Network ; September 2015"
   soroban network add testnet --rpc-url https://soroban-testnet.stellar.org:443 --network-passphrase "Test SDF Network ; September 2015"
   ```

#### Deploy Script

The deploy script builds the WASM, deploys to testnet, and initializes a campaign.

```bash
./scripts/deploy.sh <creator> <token> <goal> <deadline> <min_contribution>
```

**Parameters:**
| Parameter | Description |
| :--- | :--- |
| `creator` | Stellar address of the campaign creator |
| `token` | Stellar address of the token contract |
| `goal` | Funding goal (in stroops/lumens) |
| `deadline` | Unix timestamp for campaign end |
| `min_contribution` | Minimum contribution amount (default: 1) |

**Example:**

```bash
# Example: Deploy a campaign with 1000 XLM goal, 30-day deadline
DEADLINE=$(date -d "+30 days" +%s)
./scripts/deploy.sh GAAAAH4D... GAAAAH4D... 1000 $DEADLINE 10
```

**Output:**

```
Building WASM...
Deploying contract to testnet...
Contract deployed: C...
Campaign initialized successfully.
Contract ID: C...
Save this Contract ID for interacting with the campaign.
```

#### Interact Script

After deployment, use the interact script for common actions:

```bash
./scripts/interact.sh <contract_id> <action> [args...]
```

**Actions:**

| Action       | Description                                   | Arguments                         |
| :----------- | :-------------------------------------------- | :-------------------------------- |
| `contribute` | Contribute tokens to campaign                 | `contributor` (address), `amount` |
| `withdraw`   | Creator withdraws funds (after success)       | `creator` (address)               |
| `refund`     | Contributor requests refund (if goal not met) | `caller` (address)                |

**Examples:**

```bash
# Contribute 100 tokens to the campaign
./scripts/interact.sh C... contribute GCCCC... 100

# Creator withdraws funds after successful campaign
./scripts/interact.sh C... withdraw GAAAAH4D...

# Contributor requests refund if goal not met
./scripts/interact.sh C... refund GCCCC...
```

#### Manual Deployment

If you prefer manual deployment:

```bash
# Build the optimized WASM
cargo build --release --target wasm32-unknown-unknown

# Deploy using Soroban CLI
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/crowdfund.wasm \
  --network testnet \
  --source <YOUR_SECRET_KEY>

# Initialize the campaign
stellar contract invoke \
soroban contract invoke \
  --id <CONTRACT_ADDRESS> \
  --network testnet \
  --source <YOUR_SECRET_KEY> \
  -- initialize \
  --creator <CREATOR> \
  --token <TOKEN> \
  --goal <GOAL> \
  --deadline <DEADLINE> \
  --min_contribution <MIN>
```


## Code of Conduct

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

## Troubleshooting

### WASM target missing

```bash
# Symptom: error[E0463]: can't find crate for `std`
rustup target add wasm32-unknown-unknown
rustup target list --installed | grep wasm32
```

### Stellar CLI not found or wrong version

```bash
# Symptom: stellar: command not found  OR  unexpected argument '--source-account'
# The CLI was renamed from `soroban` to `stellar` in v20. Install the latest:
curl -Ls https://soroban.stellar.org/install-soroban.sh | sh
source ~/.bashrc   # or ~/.zshrc
stellar --version  # should print stellar-cli x.y.z
```

### Testnet vs. Futurenet identity setup

```bash
# Generate a funded testnet identity (friendbot auto-funds on testnet)
stellar keys generate --global alice --network testnet
stellar keys address alice

# For Futurenet (manual funding required):
stellar network add futurenet \
  --rpc-url https://rpc-futurenet.stellar.org:443 \
  --network-passphrase "Test SDF Future Network ; October 2022"
stellar keys generate --global alice-futurenet --network futurenet
```

> **Security**: Never commit `.soroban/` or `~/.config/stellar/` directories.
> They contain plaintext secret keys. Add `.soroban/` to `.gitignore`.

### cargo build fails after `rustup update`

```bash
rustup update stable
rustup target add wasm32-unknown-unknown   # re-add after toolchain update
cargo clean && cargo build --release --target wasm32-unknown-unknown
```

### cargo test hangs or times out

Soroban tests spin up an in-process ledger. On machines with limited RAM,
running all tests in parallel can exhaust memory.

```bash
# Limit test thread parallelism
cargo test --workspace -- --test-threads=2
```

### Node.js version mismatch (frontend)

```bash
# Symptom: SyntaxError or "engine" warning during npm install
node --version   # must be >= 18
nvm install 18 && nvm use 18
npm install
```

### npm install fails with peer dependency errors

```bash
# Use legacy peer deps flag if needed (Node 18+)
npm install --legacy-peer-deps
```

### Frontend dev server port conflict

```bash
# Symptom: EADDRINUSE: address already in use :::3000
# Kill the process on port 3000 and restart
lsof -ti:3000 | xargs kill -9
npm run dev
```

For a full edge-case checklist and automated environment verification, see
[`docs/readme_md_installation.md`](docs/readme_md_installation.md) and run:

```bash
./scripts/verify_env.sh
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a full history of notable changes.

## Security

Please review our [Security Policy](SECURITY.md) for responsible disclosure guidelines.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
