# WASM Build Pipeline Caching

Utility for managing WASM build artifact caching in the Stellar Raise frontend UI.

## Overview

`wasm_build_pipeline.tsx` provides a type-safe, secure in-memory cache for WASM build outputs. It improves developer experience by avoiding redundant rebuilds during development and gives the frontend a reliable way to check whether a cached artifact is still valid before re-fetching or re-compiling.

## Features

- Key and value validation with allowlist-based security
- Configurable TTL per entry (default: 24 hours)
- Hash-based cache invalidation (`isValid`)
- Automatic eviction of stale entries on read
- Manual bulk eviction via `evictExpired()`
- Cache statistics for monitoring and debugging
- Zero external dependencies — works in browser and Node/test environments

## Security

All inputs are validated before storage:

| Threat | Mitigation |
|---|---|
| Key injection | Keys must match `/^[a-zA-Z0-9_\-\.]+$/` |
| Script injection in values | `<script>`, `javascript:`, `data:text/html`, and inline event handlers are rejected |
| Oversized payloads | Values are capped at 5 MB (`MAX_CACHE_VALUE_BYTES`) |
| Hash tampering | Hashes must be non-empty hexadecimal strings |

## Usage

### Basic set / get

```ts
import { wasmBuildCache } from './wasm_build_pipeline';

// Store a build artifact
wasmBuildCache.set('crowdfund-v1', wasmBase64, buildHash);

// Retrieve it
const entry = wasmBuildCache.get('crowdfund-v1');
if (entry) {
  console.log(entry.value, entry.hash);
}
```

### Custom TTL and label

```ts
wasmBuildCache.set('crowdfund-v1', wasmBase64, buildHash, {
  ttlMs: 60 * 60 * 1000, // 1 hour
  label: 'crowdfund contract release build',
});
```

### Hash-based invalidation

```ts
const isStillValid = wasmBuildCache.isValid('crowdfund-v1', latestBuildHash);
if (!isStillValid) {
  // Re-fetch or re-compile
}
```

### Evict stale entries

```ts
const removed = wasmBuildCache.evictExpired();
console.log(`Evicted ${removed} stale entries`);
```

### Cache statistics

```ts
const stats = wasmBuildCache.getStats();
console.log(stats.totalEntries, stats.expiredEntries, stats.validEntries);
```

## API Reference

### `WasmBuildCache`

| Method | Description |
|---|---|
| `set(key, value, hash, opts?)` | Store an artifact |
| `get(key)` | Retrieve an artifact (returns `null` if missing or expired) |
| `has(key)` | Check existence without returning the value |
| `delete(key)` | Remove a single entry |
| `clear()` | Remove all entries |
| `evictExpired()` | Remove all expired entries; returns count removed |
| `getStats()` | Return `WasmCacheStats` object |
| `isValid(key, hash)` | Returns `true` if entry exists, is not expired, and hash matches |

### `WasmCacheValidator` (static)

| Method | Description |
|---|---|
| `validateKey(key)` | Throws `WasmCacheError` if key is invalid |
| `validateValue(value)` | Throws `WasmCacheError` if value is unsafe or too large |
| `validateHash(hash)` | Throws `WasmCacheError` if hash is not a hex string |

### Exported constants

| Constant | Value | Description |
|---|---|---|
| `CACHE_KEY_PREFIX` | `'wasm_build_cache_'` | Namespace prefix for all keys |
| `DEFAULT_CACHE_TTL_MS` | `86400000` (24 h) | Default entry lifetime |
| `MAX_CACHE_VALUE_BYTES` | `5242880` (5 MB) | Maximum value size |

## Running Tests

```bash
npx jest src/components/wasm_build_pipeline.test.tsx --coverage
```

Expected coverage: ≥ 95 % statements, branches, functions, and lines.

## Notes

- The backing store is an in-memory `Map`. Entries do not persist across page reloads. Swap `_store` for a `localStorage` adapter if persistence is needed.
- `WasmCacheError` is thrown for all validation failures — wrap calls in `try/catch` in production code.
