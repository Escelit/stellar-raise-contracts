/**
 * @title WASM Build Pipeline Caching
 * @notice Utility for managing WASM build artifact caching in the frontend UI
 * @dev Provides type-safe, secure caching for WASM build outputs with
 *      validation, cache invalidation, and developer-friendly error handling.
 * @author Stellar Raise Contracts Team
 *
 * @notice SECURITY: All cache keys and values are validated before storage.
 *         Dangerous patterns are rejected to prevent cache poisoning attacks.
 */

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/**
 * @notice Prefix applied to every cache entry to namespace it in storage
 */
export const CACHE_KEY_PREFIX = 'wasm_build_cache_';

/**
 * @notice Default TTL for cached WASM artifacts (24 hours in milliseconds)
 */
export const DEFAULT_CACHE_TTL_MS = 24 * 60 * 60 * 1000;

/**
 * @notice Maximum allowed size for a single cached value (5 MB)
 */
export const MAX_CACHE_VALUE_BYTES = 5 * 1024 * 1024;

/**
 * @notice Regex that valid cache keys must satisfy
 * @dev Allows alphanumeric characters, hyphens, underscores, and dots only
 */
const VALID_KEY_REGEX = /^[a-zA-Z0-9_\-\.]+$/;

/**
 * @notice Patterns that must never appear in cached values
 * @dev Guards against script injection via the cache layer
 */
const DANGEROUS_VALUE_PATTERNS = [
  /<script[\s>]/i,
  /javascript:/i,
  /data:text\/html/i,
  /on\w+\s*=/i, // inline event handlers
];

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/**
 * @notice Metadata stored alongside every cached WASM artifact
 */
export interface WasmCacheEntry {
  /** Opaque build hash (e.g. SHA-256 of the WASM binary) */
  hash: string;
  /** Unix timestamp (ms) when this entry was written */
  createdAt: number;
  /** Unix timestamp (ms) after which this entry is considered stale */
  expiresAt: number;
  /** Serialised WASM artifact or build output */
  value: string;
  /** Optional human-readable label for debugging */
  label?: string;
}

/**
 * @notice Options accepted by `WasmBuildCache.set`
 */
export interface WasmCacheSetOptions {
  /** Custom TTL in milliseconds; falls back to DEFAULT_CACHE_TTL_MS */
  ttlMs?: number;
  /** Human-readable label stored with the entry */
  label?: string;
}

/**
 * @notice Result returned by `WasmBuildCache.getStats`
 */
export interface WasmCacheStats {
  totalEntries: number;
  expiredEntries: number;
  validEntries: number;
  keys: string[];
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/**
 * @title WasmCacheError
 * @notice Custom error class for WASM cache operations
 */
export class WasmCacheError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'WasmCacheError';
  }
}

// ---------------------------------------------------------------------------
// Validator
// ---------------------------------------------------------------------------

/**
 * @title WasmCacheValidator
 * @notice Static helpers for validating cache keys and values
 */
export class WasmCacheValidator {
  /**
   * @notice Validates a cache key
   * @param key The raw key string
   * @throws WasmCacheError if the key is empty or contains invalid characters
   */
  static validateKey(key: string): void {
    if (!key || key.trim().length === 0) {
      throw new WasmCacheError('Cache key must not be empty.');
    }
    if (!VALID_KEY_REGEX.test(key)) {
      throw new WasmCacheError(
        `Invalid cache key "${key}". Only alphanumeric characters, hyphens, underscores, and dots are allowed.`
      );
    }
  }

  /**
   * @notice Validates a cache value for dangerous patterns and size limits
   * @param value The serialised value to store
   * @throws WasmCacheError if the value is unsafe or exceeds the size limit
   */
  static validateValue(value: string): void {
    if (Buffer.byteLength(value, 'utf8') > MAX_CACHE_VALUE_BYTES) {
    if (new TextEncoder().encode(value).length > MAX_CACHE_VALUE_BYTES) {
      throw new WasmCacheError(
        `Cache value exceeds the maximum allowed size of ${MAX_CACHE_VALUE_BYTES} bytes.`
      );
    }
    for (const pattern of DANGEROUS_VALUE_PATTERNS) {
      if (pattern.test(value)) {
        throw new WasmCacheError(
          'Cache value contains a potentially dangerous pattern and was rejected.'
        );
      }
    }
  }

  /**
   * @notice Validates a build hash string
   * @param hash The hash to validate (must be a non-empty hex string)
   * @throws WasmCacheError if the hash is invalid
   */
  static validateHash(hash: string): void {
    if (!hash || !/^[a-fA-F0-9]+$/.test(hash)) {
      throw new WasmCacheError(
        `Invalid build hash "${hash}". Hash must be a non-empty hexadecimal string.`
      );
    }
  }
}

// ---------------------------------------------------------------------------
// Cache implementation
// ---------------------------------------------------------------------------

/**
 * @title WasmBuildCache
 * @notice Manages caching of WASM build artifacts for the frontend UI.
 *
 * @dev Uses an in-memory Map as the backing store so it works in both browser
 *      and Node/test environments without requiring Web Storage APIs.
 *      Swap `_store` for a `localStorage`/`sessionStorage` adapter when
 *      persistence across page loads is required.
 */
export class WasmBuildCache {
  private _store: Map<string, WasmCacheEntry>;

  constructor() {
    this._store = new Map();
  }

  // -------------------------------------------------------------------------
  // Private helpers
  // -------------------------------------------------------------------------

  /**
   * @notice Builds the namespaced storage key
   */
  private _buildKey(key: string): string {
    return `${CACHE_KEY_PREFIX}${key}`;
  }

  // -------------------------------------------------------------------------
  // Public API
  // -------------------------------------------------------------------------

  /**
   * @notice Stores a WASM build artifact in the cache
   * @param key   Unique identifier for this build artifact
   * @param value Serialised artifact content
   * @param hash  Build hash used for cache-busting
   * @param opts  Optional TTL and label overrides
   * @throws WasmCacheError on validation failure
   */
  set(key: string, value: string, hash: string, opts: WasmCacheSetOptions = {}): void {
    WasmCacheValidator.validateKey(key);
    WasmCacheValidator.validateValue(value);
    WasmCacheValidator.validateHash(hash);

    const now = Date.now();
    const ttl = opts.ttlMs ?? DEFAULT_CACHE_TTL_MS;

    const entry: WasmCacheEntry = {
      hash,
      value,
      createdAt: now,
      expiresAt: now + ttl,
      label: opts.label,
    };

    this._store.set(this._buildKey(key), entry);
  }

  /**
   * @notice Retrieves a cached artifact if it exists and has not expired
   * @param key The cache key
   * @returns The cache entry, or `null` if missing / expired
   * @throws WasmCacheError if the key is invalid
   */
  get(key: string): WasmCacheEntry | null {
    WasmCacheValidator.validateKey(key);

    const entry = this._store.get(this._buildKey(key));
    if (!entry) return null;

    if (Date.now() > entry.expiresAt) {
      // Evict stale entry eagerly
      this._store.delete(this._buildKey(key));
      return null;
    }

    return entry;
  }

  /**
   * @notice Checks whether a valid (non-expired) entry exists for the given key
   * @param key The cache key
   * @throws WasmCacheError if the key is invalid
   */
  has(key: string): boolean {
    return this.get(key) !== null;
  }

  /**
   * @notice Removes a single entry from the cache
   * @param key The cache key
   * @throws WasmCacheError if the key is invalid
   */
  delete(key: string): void {
    WasmCacheValidator.validateKey(key);
    this._store.delete(this._buildKey(key));
  }

  /**
   * @notice Removes all entries from the cache
   */
  clear(): void {
    this._store.clear();
  }

  /**
   * @notice Evicts all expired entries from the cache
   * @returns Number of entries removed
   */
  evictExpired(): number {
    const now = Date.now();
    let removed = 0;
    for (const [k, entry] of this._store.entries()) {
      if (now > entry.expiresAt) {
        this._store.delete(k);
        removed++;
      }
    }
    return removed;
  }

  /**
   * @notice Returns cache statistics for monitoring / debugging
   */
  getStats(): WasmCacheStats {
    const now = Date.now();
    let expired = 0;
    const keys: string[] = [];

    for (const [k, entry] of this._store.entries()) {
      const rawKey = k.replace(CACHE_KEY_PREFIX, '');
      keys.push(rawKey);
      if (now > entry.expiresAt) expired++;
    }

    return {
      totalEntries: this._store.size,
      expiredEntries: expired,
      validEntries: this._store.size - expired,
      keys,
    };
  }

  /**
   * @notice Checks whether a cached entry is still valid for the given hash.
   *         Returns false if the entry is missing, expired, or the hash differs.
   * @param key  The cache key
   * @param hash The expected build hash
   */
  isValid(key: string, hash: string): boolean {
    WasmCacheValidator.validateKey(key);
    WasmCacheValidator.validateHash(hash);

    const entry = this.get(key);
    return entry !== null && entry.hash === hash;
  }
}

// ---------------------------------------------------------------------------
// Singleton helper
// ---------------------------------------------------------------------------

/**
 * @notice Module-level singleton cache instance
 * @dev Import and use this directly for the common case.
 */
export const wasmBuildCache = new WasmBuildCache();

export default WasmBuildCache;
