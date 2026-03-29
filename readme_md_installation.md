# Comprehensive Installation Guide for Stellar Raise Contracts

## Table of Contents
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Detailed Setup](#detailed-setup)
- [Verification](#verification)
- [Deployment](#deployment)
- [Logging Bounds](#logging-bounds)
- [Troubleshooting](#troubleshooting)
- [Security Assumptions](#security-assumptions)
- [Testing](#testing)
- [Development](#development)

## Prerequisites
| Tool | Version | Install Command |
|------|---------|-----------------|
| Rust | stable | [rustup.rs](https://rustup.rs) |
| wasm32 target | - | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | latest | `curl -Ls https://soroban.stellar.org/install-soroban.sh \| sh` |
| Node.js | 18+ | [nodejs.org](https://nodejs.org) |
| Git | 2.0+ | OS package manager |

**Windows Users**: Use WSL2 for best compatibility.

## Quick Start
```bash
git clone https://github.com/Mac-5/stellar-raise-contracts.git
cd stellar-raise-contracts
rustup target add wasm32-unknown-unknown
curl -Ls https://soroban.stellar.org/install-soroban.sh | sh
npm ci
npm ci  # Frontend deps
cargo build --release --target wasm32-unknown-unknown
cargo test
npm test
```

## Detailed Setup

### 1. Clone Repository
```bash
git clone https://github.com/Mac-5/stellar-raise-contracts.git
cd stellar-raise-contracts
git checkout develop
```

### 2. Install Rust & WASM Target
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup target add wasm32-unknown-unknown
```

### 3. Install Stellar CLI
```bash
curl -Ls https://soroban.stellar.org/install-soroban.sh | sh
# Verify
stellar --version
```

**Note**: `soroban` commands are now `stellar` (updated CLI).

### 4. Frontend Dependencies
```bash
npm ci
```

### 5. Build Contracts
```bash
cargo build --release --target wasm32-unknown-unknown -p crowdfund
# Output: target/wasm32-unknown-unknown/release/crowdfund.wasm
# Output: contracts/crowdfund/target/wasm32-unknown-unknown/release/crowdfund.wasm
```

## Verification
Run `readme_md_installation.test.js`:
```bash
npm test readme_md_installation.test.js
```

Expected: All checks pass (Rust, wasm target, Stellar CLI, cargo build).

## Deployment

### Automated Script
```bash
DEADLINE=$(date -d '+30 days' +%s)
./scripts/deploy.sh \
### Automated Script
```bash
DEADLINE=$(date -d '+30 days' +%s)
./scripts/deploy.sh \
  'GYOUR_CREATOR_ADDRESS' 'GTOKEN_ADDRESS' 1000000000 $DEADLINE 10000000
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Missing tool or argument |
| 2 | Build failure |
| 3 | Deploy failure |
| 4 | Initialize failure |
| 1 | Missing tool |
| 2 | Bad args |
| 3 | Build fail |
| 4 | Deploy fail |
| 5 | Init fail |

### Manual
```bash
# Build
cargo build --release --target wasm32-unknown-unknown -p crowdfund

# Install WASM
stellar contract install \
  --wasm target/wasm32-unknown-unknown/release/crowdfund.wasm \
  --source YOUR_SECRET \
  --network testnet

# Initialize
stellar contract invoke ... -- initialize \
  --admin ADMIN --creator CREATOR --token TOKEN \
  --goal GOAL --deadline DEADLINE --min_contribution MIN
```

## Logging Bounds

All scripts emit structured `[LOG]` lines to stdout. Each line has the form:

```
[LOG] key=value key=value ...
```

### `deploy.sh` — maximum 7 log lines per run

| Line | When emitted | Fields |
|------|-------------|--------|
| 1 | Build starts | `step=build status=start` |
| 2 | Build succeeds | `step=build status=ok` |
| 3 | Deploy starts | `step=deploy status=start network=<net>` |
| 4 | Deploy succeeds | `step=deploy status=ok contract_id=<id>` |
| 5 | Initialize starts | `step=initialize status=start` |
| 6 | Initialize succeeds | `step=initialize status=ok` |
| 7 | Script complete | `step=done contract_id=<id>` |

### `interact.sh` — exactly 2 log lines per action

| Line | When emitted | Fields |
|------|-------------|--------|
| 1 | Action starts | `action=<action> status=start <args>` |
| 2 | Action succeeds | `action=<action> status=ok <args>` |

On unknown action: 1 error line then `exit 1`.

### Why bounded logging?

- **Predictable output** — CI pipelines and monitoring tools can assert on
  exact log counts without parsing free-form text.
- **No unbounded loops** — scripts never emit a log line per contributor or
  per ledger entry; output size is O(1) regardless of campaign size.
- **Grep-friendly** — `grep '\[LOG\]'` extracts all structured output;
  `grep 'status=error'` surfaces failures instantly.

### Parsing log output

```bash
# Extract contract ID after deploy
CONTRACT_ID=$(./scripts/deploy.sh ... | grep 'step=done' | grep -oP 'contract_id=\K\S+')

# Check for any error
./scripts/interact.sh ... | grep -q 'status=error' && echo "FAILED"
stellar contract install \\
  --wasm contracts/crowdfund/target/wasm32-unknown-unknown/release/crowdfund.wasm \\
  --source YOUR_SECRET \\
  --network testnet

# Initialize
stellar contract invoke ... -- initialize \
  --admin ADMIN --creator CREATOR --token TOKEN \
  --goal GOAL --deadline DEADLINE --min_contribution MIN
```

## Logging Bounds

All scripts emit structured `[LOG]` lines to stdout. Each line has the form:

```
[LOG] key=value key=value ...
```

### `deploy.sh` — maximum 7 log lines per run

| Line | When emitted | Fields |
|------|-------------|--------|
| 1 | Build starts | `step=build status=start` |
| 2 | Build succeeds | `step=build status=ok` |
| 3 | Deploy starts | `step=deploy status=start network=<net>` |
| 4 | Deploy succeeds | `step=deploy status=ok contract_id=<id>` |
| 5 | Initialize starts | `step=initialize status=start` |
| 6 | Initialize succeeds | `step=initialize status=ok` |
| 7 | Script complete | `step=done contract_id=<id>` |

### `interact.sh` — exactly 2 log lines per action

| Line | When emitted | Fields |
|------|-------------|--------|
| 1 | Action starts | `action=<action> status=start <args>` |
| 2 | Action succeeds | `action=<action> status=ok <args>` |

On unknown action: 1 error line then `exit 1`.

### Why bounded logging?

- **Predictable output** — CI pipelines and monitoring tools can assert on
  exact log counts without parsing free-form text.
- **No unbounded loops** — scripts never emit a log line per contributor or
  per ledger entry; output size is O(1) regardless of campaign size.
- **Grep-friendly** — `grep '\[LOG\]'` extracts all structured output;
  `grep 'status=error'` surfaces failures instantly.

### Parsing log output

```bash
# Extract contract ID after deploy
CONTRACT_ID=$(./scripts/deploy.sh ... | grep 'step=done' | grep -oP 'contract_id=\K\S+')

# Check for any error
./scripts/interact.sh ... | grep -q 'status=error' && echo "FAILED"
```

## Troubleshooting
| Issue | Solution |
|-------|----------|
| `wasm32-unknown-unknown` not found | `rustup target add wasm32-unknown-unknown` |
| `stellar: command not found` | Re-run Stellar CLI install script |
| `cargo build` fails | `rustup update stable` |
| Windows path issues | Use WSL2 |
| Tests timeout | `cargo test -- --test-threads=1` |
| No `[LOG]` output | Ensure script is executable: `chmod +x scripts/*.sh` |

## Security Assumptions
- **Admin Auth**: Only creator/admin can `initialize`, `withdraw`, `upgrade`.
- **Contributor Auth**: `contribute`/`refund_single` requires caller auth.
- **Pull Refunds**: Individual claims prevent gas DoS.
- **Upgrade Safety**: WASM hash validated; storage preserved.
- **Bounds**: Goal/deadline/min_contribution validated.
- **Platform Fee**: Capped at 100%.
- **Log Injection**: `[LOG]` lines contain only alphanumeric values and `=`;
  user-supplied addresses are not interpolated into log field values beyond
  what the shell already escapes.

## Testing
```bash
cargo test --workspace   # Contracts
npm test                 # Frontend + installation tests
```

## Development
| Tests timeout | Increase `cargo test -- --test-threads=1` |

## Security Assumptions
- **Admin Auth**: Only creator/admin can `initialize`, `withdraw`, `upgrade`.
- **Contributor Auth**: `contribute`/`refund_single` requires caller auth.
- **Pull Refunds**: Individual claims prevent gas DoS.
- **Upgrade Safety**: WASM hash validated; storage preserved.
- **Bounds**: Goal/deadline/min_contribution validated.
- **Platform Fee**: Capped at 100%.
- **Log Injection**: `[LOG]` lines contain only alphanumeric values and `=`;
  user-supplied addresses are not interpolated into log field values beyond
  what the shell already escapes.

## Testing
```bash
cargo test --workspace   # Contracts
npm test                 # Frontend + installation tests
```

## Development
- Branch: `git checkout -b feat/your-feature develop`
- Format: `cargo fmt --all`
- Lint: `cargo clippy --all-targets`
- PR to `develop`
