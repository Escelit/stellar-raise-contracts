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

- [Rust](https://rustup.rs/) (stable)
- The `wasm32-unknown-unknown` target:

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- [Stellar CLI](https://soroban.stellar.org/docs/getting-started/setup) (optional, for deployment)

## Getting Started

```bash
# Clone the repo
git clone https://github.com/<your-org>/stellar-raise-contracts.git
cd stellar-raise-contracts

# Build the contract
cargo build --release --target wasm32-unknown-unknown

# Run tests
cargo test --workspace
```

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

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a full history of notable changes.

## Security

Please review our [Security Policy](SECURITY.md) for responsible disclosure guidelines.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
