# WASM Build Pipeline Caching – Scripts

Utility for managing WASM build artifact caching for the Stellar Raise deployment and interaction scripts.

## Overview

`wasm_build_pipeline.tsx` provides two complementary caches:

- `WasmBuildCache` — general-purpose cache for WASM binary artifacts (frontend UI and CI).
- `ScriptBuildCache` — specialised cache that models the `deploy.sh → interact.sh` workflow, storing contract IDs, deploy parameters, and WASM hashes so scripts can skip redundant builds and re-deployments.

## Features

- Key/value/hash validation with strict allowlists
- Stellar contract ID and address format validation
- WASM path and network validation
- Configurable TTL per entry (general: 24 h, script: 1 h)
- Hash-based cache invalidation (`isValid`, `isWasmHashValid`)
- Automatic eviction of stale entries on read
- Manual bulk eviction via `evictExpired()`
- Cross-network isolation (keys scoped by `<network>.<contractId>`)
- Zero external dependencies
 - Bounded memory: configurable max entries with oldest-entry eviction to
   avoid unbounded memory growth in long-running processes (CI, dev servers)

## Security

| Threat | Mitigation |
|---|---|
| Key injection | Keys must match `/^[a-zA-Z0-9_\-\.]+$/` |
| Script injection in values | `<script>`, `javascript:`, `data:text/html`, inline event handlers rejected |
| Oversized payloads | Values capped at 5 MB |
| Hash tampering | Hashes must be non-empty hex strings |
| Invalid contract IDs | Must match `C[A-Z2-7]{55}` |
| Invalid Stellar addresses | Must match `G[A-Z2-7]{55}` |
| Cross-network collisions | Keys namespaced by network |

## Usage

### General WASM artifact cache

```ts
import { wasmBuildCache } from './wasm_build_pipeline';

wasmBuildCache.set('crowdfund-v1', wasmBase64, buildHash);
const entry = wasmBuildCache.get('crowdfund-v1');
```

### Script deploy cache (deploy.sh integration)

After a successful `deploy.sh` run, store the result:

```ts
import { scriptBuildCache } from './wasm_build_pipeline';

scriptBuildCache.setDeployEntry({
  contractId:      'CAAA...', // 56-char contract address
  creator:         'GAAA...', // 56-char Stellar address
  token:           'GBBB...', // 56-char token address
  goal:            1000,
  deadline:        1735689600,
  minContribution: 10,
  network:         'testnet',
  wasmPath:        'target/wasm32-unknown-unknown/release/crowdfund.wasm',
  wasmHash:        'deadbeef...',
});
```

### interact.sh integration

Before invoking on-chain actions, verify the cached WASM is still current:

```ts
const isStillValid = scriptBuildCache.isWasmHashValid(
  'testnet', contractId, latestWasmHash
);
if (!isStillValid) {
  // Re-build and re-deploy
}

const entry = scriptBuildCache.getDeployEntry('testnet', contractId);
if (entry) {
  console.log('Using cached contract:', entry.contractId);
}
```

### Evict stale entries

```ts
const removed = scriptBuildCache.evictExpired();
console.log(`Evicted ${removed} stale deploy entries`);
```

## API Reference

### `WasmBuildCache`

| Method | Description |
|---|---|
| `set(key, value, hash, opts?)` | Store an artifact |
| `get(key)` | Retrieve (null if missing/expired) |
| `has(key)` | Check existence |
| `delete(key)` | Remove single entry |
| `clear()` | Remove all entries |
| `evictExpired()` | Remove expired entries; returns count |
| `getStats()` | Return `WasmCacheStats` |
| `isValid(key, hash)` | True if entry exists, not expired, hash matches |

### `ScriptBuildCache`

| Method | Description |
|---|---|
| `setDeployEntry(input)` | Store a deploy result |
| `getDeployEntry(network, contractId)` | Retrieve (null if missing/expired) |
| `hasDeployEntry(network, contractId)` | Check existence |
| `isWasmHashValid(network, contractId, hash)` | True if WASM hash matches cached value |
| `deleteDeployEntry(network, contractId)` | Remove single entry |
| `clear()` | Remove all entries |
| `evictExpired()` | Remove expired entries; returns count |
| `getStats()` | Return `WasmCacheStats` |

### `WasmCacheValidator` (static)

| Method | Description |
|---|---|
| `validateKey(key)` | Throws if key is invalid |
| `validateValue(value)` | Throws if value is unsafe or too large |
| `validateHash(hash)` | Throws if hash is not hex |
| `validateContractId(id)` | Throws if not a valid Stellar contract ID |
| `validateStellarAddress(addr, field?)` | Throws if not a valid G-address |
| `validateWasmPath(path)` | Throws if path doesn't end in `.wasm` |
| `validateNetwork(network)` | Throws if network is not in `SUPPORTED_NETWORKS` |
| `validateDeployInput(input)` | Validates all fields of a `ScriptDeployInput` |

### Exported constants

| Constant | Value | Description |
|---|---|---|
| `CACHE_KEY_PREFIX` | `'wasm_build_cache_'` | Namespace for general cache keys |
| `SCRIPT_CACHE_KEY_PREFIX` | `'wasm_script_cache_'` | Namespace for script cache keys |
| `DEFAULT_CACHE_TTL_MS` | `86400000` (24 h) | Default TTL for artifact cache |
| `SCRIPT_CACHE_TTL_MS` | `3600000` (1 h) | Default TTL for deploy cache |
| `MAX_CACHE_VALUE_BYTES` | `5242880` (5 MB) | Maximum value size |
| `MAX_CACHE_ENTRIES` | `100` | Maximum entries in `WasmBuildCache` before LRU eviction |
| `SUPPORTED_NETWORKS` | `testnet`, `mainnet`, `futurenet`, `localnet` | Valid network names |

## Running Tests

```bash
npx jest scripts/wasm_build_pipeline.test.tsx --coverage
```

Expected coverage: 100% statements, branches, functions, and lines (132 tests).
Expected coverage: ≥ 95% statements, branches, functions, and lines.

## Notes

- The backing store is an in-memory `Map`. Entries do not persist across process restarts.
- `WasmCacheError` is thrown for all validation failures.
- Cache keys for `ScriptBuildCache` are scoped as `<network>.<contractId>` to prevent cross-network collisions.

# WASM Build Pipeline Constants (`wasm_build_pipeline.tsx`)

## Overview
Extracts hardcoded WASM build constants from:
- `Cargo.toml` `[profile.release]`
- `.cargo/config.toml` `[target.wasm32-unknown-unknown]`
- Build scripts (`deploy.sh`, CI workflows)

Improves:
- **Testing reproducibility**: Consistent flags across runs.
- **Caching efficiency**: Identical flags enable cargo/sccache hits.
- **Readability**: Centralized config vs scattered strings.

## Usage
```ts
import { WASM_BUILD_CONSTANTS } from './wasm_build_pipeline';

const cmd = ['cargo', 'build', ...WASM_BUILD_CONSTANTS.cargo_flags].join(' ');
```

In Node/CI:
```bash
node -e "
const { WASM_BUILD_CONSTANTS } = require('./scripts/wasm_build_pipeline');
console.log(['cargo', 'build', ...WASM_BUILD_CONSTANTS.cargo_flags].join(' '));
" | bash
```

## Constants Reference (NatSpec)

### @notice WASM Target
- `target: 'wasm32-unknown-unknown'` - Soroban contract target.

### @notice Release Profile (Optimizations)
| Key | Value | Purpose |
|-----|-------|---------|
| `opt_level` | `'z'` | Size-optimized |
| `overflow_checks` | `true` | Security |
| `debug` | `0` | No debug info |
| `strip` | `'symbols'` | Strip symbols |
| `debug_assertions` | `false` | Release |
| `panic` | `'abort'` | WASM compatible |
| `codegen_units` | `1` | Max opt |
| `lto` | `true` | Link-time opt |

### @notice Rustflags
- `stack-size: 65536` - Prevent stack overflow.
- `target-feature=-reference-types,-multivalue` - Soroban compatibility.

## Security Assumptions
- Constants are compile-time; no runtime impact.
- Validated against source configs.
- Tests ensure immutability/type-safety.

## Testing
- `npm test scripts/wasm_build_pipeline.test.tsx`
- 100% coverage.
- Edge: Type checks, presence validation.

## Integration
Update `deploy.sh`, `rust_ci.yml` to use generated flags from TS (future).

Timeframe met: Efficient extraction.

