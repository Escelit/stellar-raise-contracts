# Installation Edge Cases — Stellar Raise Contracts

> Companion to the main [README.md](../README.md).  
> Run `./scripts/verify_env.sh` to check your environment automatically.

---

## Minimum Requirements

| Requirement | Minimum | Notes |
|---|---|---|
| OS | Linux (x86-64) or macOS 12+ | WSL2 on Windows |
| RAM | 4 GB | 8 GB recommended for `--release` builds |
| Disk | 2 GB free | Rust toolchain + WASM target + node_modules |
| Rust | stable (≥ 1.74) | `rustup update stable` |
| Stellar CLI | ≥ 20.0.0 | Renamed from `soroban` in v20 |
| Node.js | ≥ 18 | Required for frontend UI and JS tests |
| npm | ≥ 9 | Bundled with Node 18+ |
| Disk | 2 GB free | Rust toolchain + WASM target |
| Rust | stable (≥ 1.74) | `rustup update stable` |
| Stellar CLI | ≥ 20.0.0 | Renamed from `soroban` in v20 |
| Node.js | ≥ 18 | For frontend and JS tests |

---

## Edge Case 1 — WASM Target Not Installed

**Symptom**

```
error[E0463]: can't find crate for `std`
  = note: the `wasm32-unknown-unknown` target may not be installed
```

**Fix**

```bash
rustup target add wasm32-unknown-unknown
rustup target list --installed | grep wasm32
```

**Verify**

```bash
cargo build --release --target wasm32-unknown-unknown -p crowdfund
```

---

## Edge Case 2 — Stellar CLI Version / Rename

**Symptom**

```
soroban: command not found
# or
error: unexpected argument '--source-account' found
```

**Background**  
The CLI was renamed from `soroban` to `stellar` in v20. Scripts using `soroban`
will fail on newer installs.

**Fix**

```bash
curl -Ls https://soroban.stellar.org/install-soroban.sh | sh
source ~/.bashrc    # or ~/.zshrc / ~/.profile
stellar --version   # should print stellar-cli x.y.z
```

**Verify**

```bash
stellar contract --help
```

---

## Edge Case 3 — Testnet vs. Futurenet Identity Setup

### Testnet (recommended for development)

Friendbot automatically funds new testnet identities.

```bash
stellar keys generate --global alice --network testnet
stellar keys address alice
# Fund via friendbot (automatic on first use with --network testnet)
```

### Futurenet (pre-release features)

Futurenet requires manual funding and a separate network config.

```bash
stellar network add futurenet \
  --rpc-url https://rpc-futurenet.stellar.org:443 \
  --network-passphrase "Test SDF Future Network ; October 2022"

stellar keys generate --global alice-futurenet --network futurenet
# Fund manually via https://friendbot-futurenet.stellar.org
```

> **Security assumption**: The `.soroban/` directory and `~/.config/stellar/`
> contain **plaintext secret keys**. Never commit them to version control.
> `.soroban/` is already in `.gitignore` — verify with `git check-ignore -v .soroban`.

---

## Edge Case 4 — Toolchain Drift After `rustup update`

After updating Rust, the WASM target may need to be re-added.

```bash
rustup update stable
rustup target add wasm32-unknown-unknown
cargo clean && cargo build --release --target wasm32-unknown-unknown
```

---

## Edge Case 5 — `cargo test` Hangs or Times Out

Soroban tests spin up an in-process ledger. Running many tests in parallel can
exhaust memory on low-RAM machines.

```bash
# Limit parallelism
cargo test --workspace -- --test-threads=2
```

---

## Edge Case 6 — Node.js Version Mismatch (Frontend UI)

**Symptom**

```
SyntaxError: Unexpected token '?'
# or npm WARN EBADENGINE Unsupported engine { required: { node: '>=18' } }
```

**Fix**

```bash
# Install nvm if not present
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
source ~/.bashrc

nvm install 18
nvm use 18
node --version   # v18.x.x
```

**Verify**

```bash
cd frontend && npm install && npm test -- --run
```

---

## Edge Case 7 — npm Peer Dependency Conflicts

**Symptom**

```
npm ERR! ERESOLVE unable to resolve dependency tree
```

**Fix**

```bash
npm install --legacy-peer-deps
```

> Only use `--legacy-peer-deps` if you understand the dependency tree. Prefer
> resolving the conflict explicitly in `package.json` for production builds.

---

## Edge Case 8 — Frontend Dev Server Port Conflict

**Symptom**

```
Error: EADDRINUSE: address already in use :::3000
```

**Fix**

```bash
# Find and kill the process occupying port 3000
lsof -ti:3000 | xargs kill -9
npm run dev
```

Alternatively, configure a different port in `vite.config.ts` (or `next.config.js`):

```ts
// vite.config.ts
export default { server: { port: 3001 } };
```

---

## Edge Case 9 — CSS Variables Not Resolving in Frontend

**Symptom**  
Design tokens (`--color-docs-bg`, `--color-docs-accent`, etc.) render as empty
strings or fall back to defaults unexpectedly.

**Cause**  
The `useDocsCssVariable` hook reads from `getComputedStyle` at runtime. If the
stylesheet containing the variable declarations hasn't loaded yet (e.g., SSR or
lazy CSS), the hook returns the fallback value.

**Fix**

1. Ensure the global CSS file that declares the variables is imported in your
   app entry point (e.g., `_app.tsx` or `main.tsx`):

   ```ts
   import '../styles/globals.css';
   ```

2. Provide meaningful fallback values in every `useDocsCssVariable` call:

   ```ts
   const bg = useDocsCssVariable('--color-docs-bg', '#FAFAFA');
   ```

3. For SSR environments, guard the hook with a `typeof window !== 'undefined'`
   check or use a `useEffect` to defer resolution to the client.

---

## Automated Verification

Run the environment check script before opening a PR or filing a bug report:

```bash
./scripts/verify_env.sh
```

The script checks:
- `rustc`, `cargo`, `stellar` are on `$PATH`
- `wasm32-unknown-unknown` target is installed
- `node` and `npm` meet minimum version requirements
- A dry-run build of the crowdfund contract succeeds

Exit codes: `0` = all checks passed, `1` = one or more checks failed.

---

## Security Notes

- Never commit `.soroban/` or `~/.config/stellar/` — they contain secret keys.
- Use a multisig or governance contract as the `admin` for mainnet deployments,
  not a plain keypair (see `docs/admin_upgrade_mechanism.md`).
- Rotate keys immediately if a secret is accidentally pushed to a public repo.
  Use `stellar keys remove <name>` and generate a new identity.
- CSS variable values are validated against an allowlist by `CssVariableValidator`
  to prevent CSS injection (see `docs/css_variables_usage.md`).
- Never embed secret keys, XDR payloads, or account mnemonics in frontend error
  messages — the `FrontendGlobalErrorBoundary` strips stack traces in production,
  but the message string itself is still surfaced to users.
