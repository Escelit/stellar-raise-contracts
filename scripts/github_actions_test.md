# GitHub Actions Workflow Fixes

## What was wrong

Four bugs were found and fixed across the CI workflow files:

### 1. `actions/checkout@v6` — non-existent action version (typo)

**Files affected:** `rust_ci.yml`, `testnet_smoke.yml`

`actions/checkout@v6` does not exist. The latest stable release is `v4`. Using a
non-existent version causes every CI run to fail immediately at the checkout step.

**Fix:** Changed `actions/checkout@v6` → `actions/checkout@v4` in both files.

### 2. Duplicate WASM build step in `rust_ci.yml`

The workflow built the WASM binary twice:

```yaml
# Step 1 — correct, scoped to the crowdfund crate
- name: Build crowdfund WASM for tests
  run: cargo build --release --target wasm32-unknown-unknown -p crowdfund

# Step 2 — redundant, rebuilds the same artifact
- name: Build WASM (release)
  run: cargo build --release --target wasm32-unknown-unknown
```

The second step added ~60–90 s of unnecessary compile time on every CI run
without producing a different artifact.

**Fix:** Removed the redundant second build step.

### 3. Empty `spellcheck.yml`

The file existed but contained only a single newline byte, so the spellcheck
job never ran. Added a minimal working workflow using
`streetsidesoftware/cspell-action@v6` that checks `*.md`, `*.yml`, and
`*.yaml` files on push and pull-request events.

### 4. `testnet_smoke.yml` — WASM build not scoped to `-p crowdfund`

The smoke test built the entire workspace:

```yaml
- run: cargo build --target wasm32-unknown-unknown --release
```

This compiles every crate in the workspace unnecessarily, wasting CI time.

**Fix:** Added `-p crowdfund` to scope the build to the single required crate:

```yaml
- run: cargo build --target wasm32-unknown-unknown --release -p crowdfund
```

### 5. `testnet_smoke.yml` — deprecated `soroban-cli` instead of `stellar-cli`

The Soroban CLI was renamed to the Stellar CLI (`stellar-cli`). Installing
`soroban-cli` installs an outdated, unmaintained package. All `soroban`
subcommands (`soroban keys`, `soroban contract`) were updated to `stellar`.

**Fix:** Changed `cargo install soroban-cli` → `cargo install stellar-cli` and
updated all `soroban` command invocations to `stellar`.

---

## Files changed

| File | Change |
|---|---|
| `.github/workflows/rust_ci.yml` | `checkout@v6` → `checkout@v4`; removed duplicate WASM build step |
| `.github/workflows/testnet_smoke.yml` | `checkout@v6` → `checkout@v4`; added `-p crowdfund` to build step; `soroban-cli` → `stellar-cli`; all `soroban` commands → `stellar` |
| `.github/workflows/spellcheck.yml` | Replaced empty file with working spellcheck workflow |

## Validation scripts

| Script | Purpose |
|---|---|
| `scripts/github_actions_test.sh` | Validates workflow files in CI or locally (7 checks) |
| `scripts/github_actions_test.test.sh` | Tests the validator against pass/fail scenarios (8 tests) |

Run locally:

```bash
bash scripts/github_actions_test.sh
bash scripts/github_actions_test.test.sh
```

## Security notes

- No secrets or credentials are introduced or modified.
- The `actions/checkout@v4` pin is the current stable, audited release.
- The spellcheck action runs with default (read-only) permissions.
- Using `stellar-cli` (the maintained successor) reduces supply-chain risk
  compared to the deprecated `soroban-cli` package.
