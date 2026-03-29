/**
 * @title WASM Build Pipeline Caching
 * @notice Utility for managing WASM build artifact caching for both the
 *         frontend UI and deployment/interaction scripts.
 * @dev Provides type-safe, secure caching for WASM build outputs with
 *      validation, cache invalidation, and developer-friendly error handling.
 *      Includes a specialised ScriptBuildCache layer that models the
 *      deploy.sh / interact.sh workflow (WASM path, contract ID, deploy
 *      metadata) so scripts can skip redundant builds and re-deployments.
 * @author Stellar Raise Contracts Team
 *
 * @notice SECURITY: All cache keys and values are validated before storage.
 *         Dangerous patterns are rejected to prevent cache poisoning attacks.
 */

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/**
 * @notice Prefix applied to every general cache entry
 */
export const CACHE_KEY_PREFIX = 'wasm_build_cache_';

/**
 * @notice Prefix applied to every script-layer cache entry
 */
export const SCRIPT_CACHE_KEY_PREFIX = 'wasm_script_cache_';

/**
 * @notice Default TTL for cached WASM artifacts (24 hours in milliseconds)
 */
export const DEFAULT_CACHE_TTL_MS = 24 * 60 * 60 * 1000;

/**
 * @notice Shorter default TTL for script deploy metadata (1 hour)
 * @dev Contract IDs are network-specific and should be re-validated more often
 */
export const SCRIPT_CACHE_TTL_MS = 60 * 60 * 1000;

/**
 * @notice Maximum allowed size for a single cached value (5 MB)
 */
export const MAX_CACHE_VALUE_BYTES = 5 * 1024 * 1024;

/**
 * @notice Maximum number of entries kept in the general WasmBuildCache.
 * @dev When exceeded, the cache evicts the oldest entries (simple LRU by
 *      insertion time) to avoid unbounded memory growth in long-running
 *      processes (CI runners, dev servers). Pick a conservative default
 *      that keeps tests fast but prevents denial-of-service via many keys.
 */
export const MAX_CACHE_ENTRIES = 100;

/**
 * @notice Supported Stellar networks
 */
export const SUPPORTED_NETWORKS = ['testnet', 'mainnet', 'futurenet', 'localnet'] as const;

/**
 * @notice Regex that valid cache keys must satisfy
 */
const VALID_KEY_REGEX = /^[a-zA-Z0-9_\-\.]+$/;

/**
 * @notice Patterns that must never appear in cached values
 */
const DANGEROUS_VALUE_PATTERNS = [
  /<script[\s>]/i,
  /javascript:/i,
  /data:text\/html/i,
  /on\w+\s*=/i,
];

/**
 * @notice Regex for a valid Stellar contract ID (C + 55 base32 chars)
 */
const CONTRACT_ID_REGEX = /^C[A-Z2-7]{55}$/;

/**
 * @notice Regex for a valid Stellar account address (G + 55 base32 chars)
 */
const STELLAR_ADDRESS_REGEX = /^G[A-Z2-7]{55}$/;

/**
 * @notice Regex for a valid WASM file path
 */
const WASM_PATH_REGEX = /^[a-zA-Z0-9_\-\.\/]+\.wasm$/;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type SupportedNetwork = typeof SUPPORTED_NETWORKS[number];

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
 * @notice Result returned by `WasmBuildCache.getStats` and `ScriptBuildCache.getStats`
 */
export interface WasmCacheStats {
  totalEntries: number;
  expiredEntries: number;
  validEntries: number;
  keys: string[];
}

/**
 * @notice Cached result of a successful `deploy.sh` run
 * @dev Stores everything a subsequent `interact.sh` call needs
 */
export interface ScriptDeployEntry {
  /** Deployed contract ID (C + 55 chars) */
  contractId: string;
  /** Stellar address of the campaign creator */
  creator: string;
  /** Stellar address of the token contract */
  token: string;
  /** Funding goal in stroops */
  goal: number;
  /** Campaign deadline as Unix timestamp (seconds) */
  deadline: number;
  /** Minimum contribution amount */
  minContribution: number;
  /** Network the contract was deployed to */
  network: SupportedNetwork;
  /** Absolute or relative path to the compiled WASM file */
  wasmPath: string;
  /** SHA-256 hex hash of the WASM binary at deploy time */
  wasmHash: string;
  /** Unix timestamp (ms) when this entry was written */
  createdAt: number;
  /** Unix timestamp (ms) after which this entry is stale */
  expiresAt: number;
}

/**
 * @notice Input required to store a new deploy entry
 */
export interface ScriptDeployInput {
  contractId: string;
  creator: string;
  token: string;
  goal: number;
  deadline: number;
  minContribution: number;
  network: SupportedNetwork;
  wasmPath: string;
  wasmHash: string;
  /** Optional custom TTL; defaults to SCRIPT_CACHE_TTL_MS */
  ttlMs?: number;
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/**
 * @title WasmCacheError
 * @notice Custom error class for all WASM cache operations
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
 * @notice Static helpers for validating cache keys, values, and script inputs
 */
export class WasmCacheValidator {
  /**
   * @notice Validates a cache key
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
   * @notice Validates a build hash string (must be non-empty hex)
   * @throws WasmCacheError if the hash is invalid
   */
  static validateHash(hash: string): void {
    if (!hash || !/^[a-fA-F0-9]+$/.test(hash)) {
      throw new WasmCacheError(
        `Invalid build hash "${hash}". Hash must be a non-empty hexadecimal string.`
      );
    }
  }

  /**
   * @notice Validates a Stellar contract ID (C + 55 base32 chars)
   * @throws WasmCacheError if the format is invalid
   */
  static validateContractId(contractId: string): void {
    if (!contractId || !CONTRACT_ID_REGEX.test(contractId)) {
      throw new WasmCacheError(
        `Invalid contract ID "${contractId}". Must be a 56-character Stellar contract address starting with "C".`
      );
    }
  }

  /**
   * @notice Validates a Stellar account address (G + 55 base32 chars)
   * @throws WasmCacheError if the format is invalid
   */
  static validateStellarAddress(address: string, fieldName = 'address'): void {
    if (!address || !STELLAR_ADDRESS_REGEX.test(address)) {
      throw new WasmCacheError(
        `Invalid Stellar ${fieldName} "${address}". Must be a 56-character address starting with "G".`
      );
    }
  }

  /**
   * @notice Validates a WASM file path
   * @throws WasmCacheError if the path does not end in .wasm
   */
  static validateWasmPath(wasmPath: string): void {
    if (!wasmPath || !WASM_PATH_REGEX.test(wasmPath)) {
      throw new WasmCacheError(
        `Invalid WASM path "${wasmPath}". Must be a relative or absolute path ending in ".wasm".`
      );
    }
  }

  /**
   * @notice Validates a network name against the supported list
   * @throws WasmCacheError if the network is not supported
   */
  static validateNetwork(network: string): void {
    if (!SUPPORTED_NETWORKS.includes(network as SupportedNetwork)) {
      throw new WasmCacheError(
        `Unsupported network "${network}". Must be one of: ${SUPPORTED_NETWORKS.join(', ')}.`
      );
    }
  }

  /**
   * @notice Validates a full ScriptDeployInput object
   * @throws WasmCacheError on the first invalid field found
   */
  static validateDeployInput(input: ScriptDeployInput): void {
    WasmCacheValidator.validateContractId(input.contractId);
    WasmCacheValidator.validateStellarAddress(input.creator, 'creator');
    WasmCacheValidator.validateStellarAddress(input.token, 'token');
    WasmCacheValidator.validateNetwork(input.network);
    WasmCacheValidator.validateWasmPath(input.wasmPath);
    WasmCacheValidator.validateHash(input.wasmHash);

    if (!Number.isInteger(input.goal) || input.goal <= 0) {
      throw new WasmCacheError(`goal must be a positive integer, got: ${input.goal}`);
    }
    if (!Number.isInteger(input.deadline) || input.deadline <= 0) {
      throw new WasmCacheError(
        `deadline must be a positive Unix timestamp, got: ${input.deadline}`
      );
    }
    if (!Number.isInteger(input.minContribution) || input.minContribution <= 0) {
      throw new WasmCacheError(
        `minContribution must be a positive integer, got: ${input.minContribution}`
      );
    }
  }
}

// ---------------------------------------------------------------------------
// General-purpose WASM artifact cache
// ---------------------------------------------------------------------------

/**
 * @title WasmBuildCache
 * @notice Manages caching of WASM build artifacts for the frontend UI.
 *
 * @dev Uses an in-memory Map as the backing store so it works in both browser
 *      and Node/test environments without requiring Web Storage APIs.
 */
export class WasmBuildCache {
  private _store: Map<string, WasmCacheEntry>;

  constructor() {
    this._store = new Map();
  }

  private _buildKey(key: string): string {
    return `${CACHE_KEY_PREFIX}${key}`;
  }

  /**
   * @notice Stores a WASM build artifact in the cache
   * @throws WasmCacheError on validation failure
   */
  set(key: string, value: string, hash: string, opts: WasmCacheSetOptions = {}): void {
    WasmCacheValidator.validateKey(key);
    WasmCacheValidator.validateValue(value);
    WasmCacheValidator.validateHash(hash);

    const now = Date.now();
    const ttl = opts.ttlMs ?? DEFAULT_CACHE_TTL_MS;

    this._store.set(this._buildKey(key), {
      hash,
      value,
      createdAt: now,
      expiresAt: now + ttl,
      label: opts.label,
    });

    // Ensure we don't exceed the configured maximum entry count in long
    // running processes. Evict oldest entries (by createdAt) until we're
    // within the limit. This prevents unbounded memory growth and makes
    // cache behaviour deterministic under adversarial key churn.
    if (this._store.size > MAX_CACHE_ENTRIES) {
      // remove oldest entries until size <= MAX_CACHE_ENTRIES
      const entries = Array.from(this._store.entries());
      // Each entry: [key, WasmCacheEntry]
      entries.sort((a, b) => a[1].createdAt - b[1].createdAt);
      let idx = 0;
      while (this._store.size > MAX_CACHE_ENTRIES && idx < entries.length) {
        this._store.delete(entries[idx][0]);
        idx++;
      }
    }
  }

  /**
   * @notice Retrieves a cached artifact; returns null if missing or expired
   * @throws WasmCacheError if the key is invalid
   */
  get(key: string): WasmCacheEntry | null {
    WasmCacheValidator.validateKey(key);

    const entry = this._store.get(this._buildKey(key));
    if (!entry) return null;

    if (Date.now() > entry.expiresAt) {
      this._store.delete(this._buildKey(key));
      return null;
    }

    return entry;
  }

  /** @notice Returns true if a valid (non-expired) entry exists */
  has(key: string): boolean {
    return this.get(key) !== null;
  }

  /** @notice Removes a single entry */
  delete(key: string): void {
    WasmCacheValidator.validateKey(key);
    this._store.delete(this._buildKey(key));
  }

  /** @notice Removes all entries */
  clear(): void {
    this._store.clear();
  }

  /**
   * @notice Evicts all expired entries
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

  /** @notice Returns cache statistics */
  getStats(): WasmCacheStats {
    const now = Date.now();
    let expired = 0;
    const keys: string[] = [];

    for (const [k, entry] of this._store.entries()) {
      keys.push(k.replace(CACHE_KEY_PREFIX, ''));
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
   * @notice Returns true if the entry exists, is not expired, and hash matches
   */
  isValid(key: string, hash: string): boolean {
    WasmCacheValidator.validateKey(key);
    WasmCacheValidator.validateHash(hash);
    const entry = this.get(key);
    return entry !== null && entry.hash === hash;
  }
}

// ---------------------------------------------------------------------------
// Script-layer deploy cache
// ---------------------------------------------------------------------------

/**
 * @title ScriptBuildCache
 * @notice Specialised cache for deployment script outputs.
 *
 * @dev Models the deploy.sh → interact.sh workflow:
 *   1. After `deploy.sh` builds the WASM, deploys to a network, and
 *      initialises the campaign, the resulting contract ID and deploy
 *      parameters are stored here so subsequent script runs can skip
 *      redundant work.
 *   2. `interact.sh` calls `getDeployEntry` to retrieve the cached contract
 *      ID and verify the WASM hash before invoking on-chain actions.
 *
 * Cache keys are scoped by `<network>.<contractId>` to prevent cross-network
 * collisions.
 */
export class ScriptBuildCache {
  private _store: Map<string, ScriptDeployEntry>;

  constructor() {
    this._store = new Map();
  }

  private _buildKey(network: string, contractId: string): string {
    return `${SCRIPT_CACHE_KEY_PREFIX}${network}.${contractId}`;
  }

  /**
   * @notice Stores a deploy entry after a successful `deploy.sh` run
   * @param input Validated deploy parameters and results
   * @throws WasmCacheError on validation failure
   */
  setDeployEntry(input: ScriptDeployInput): void {
    WasmCacheValidator.validateDeployInput(input);

    const now = Date.now();
    const ttl = input.ttlMs ?? SCRIPT_CACHE_TTL_MS;

    this._store.set(this._buildKey(input.network, input.contractId), {
      contractId: input.contractId,
      creator: input.creator,
      token: input.token,
      goal: input.goal,
      deadline: input.deadline,
      minContribution: input.minContribution,
      network: input.network,
      wasmPath: input.wasmPath,
      wasmHash: input.wasmHash,
      createdAt: now,
      expiresAt: now + ttl,
    });
  }

  /**
   * @notice Retrieves a deploy entry; returns null if missing or expired
   * @throws WasmCacheError on invalid inputs
   */
  getDeployEntry(network: string, contractId: string): ScriptDeployEntry | null {
    WasmCacheValidator.validateNetwork(network);
    WasmCacheValidator.validateContractId(contractId);

    const entry = this._store.get(this._buildKey(network, contractId));
    if (!entry) return null;

    if (Date.now() > entry.expiresAt) {
      this._store.delete(this._buildKey(network, contractId));
      return null;
    }

    return entry;
  }

  /**
   * @notice Returns true if a valid, non-expired deploy entry exists
   */
  hasDeployEntry(network: string, contractId: string): boolean {
    return this.getDeployEntry(network, contractId) !== null;
  }

  /**
   * @notice Returns true if the cached WASM hash matches the provided hash
   * @dev Use before `interact.sh` to confirm the on-chain binary is current
   */
  isWasmHashValid(network: string, contractId: string, wasmHash: string): boolean {
    WasmCacheValidator.validateHash(wasmHash);
    const entry = this.getDeployEntry(network, contractId);
    return entry !== null && entry.wasmHash === wasmHash;
  }

  /**
   * @notice Removes a single deploy entry
   */
  deleteDeployEntry(network: string, contractId: string): void {
    WasmCacheValidator.validateNetwork(network);
    WasmCacheValidator.validateContractId(contractId);
    this._store.delete(this._buildKey(network, contractId));
  }

  /** @notice Removes all deploy entries */
  clear(): void {
    this._store.clear();
  }

  /**
   * @notice Evicts all expired deploy entries
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

  /** @notice Returns cache statistics */
  getStats(): WasmCacheStats {
    const now = Date.now();
    let expired = 0;
    const keys: string[] = [];

    for (const [k, entry] of this._store.entries()) {
      keys.push(k.replace(SCRIPT_CACHE_KEY_PREFIX, ''));
      if (now > entry.expiresAt) expired++;
    }

    return {
      totalEntries: this._store.size,
      expiredEntries: expired,
      validEntries: this._store.size - expired,
      keys,
    };
  }
}

// ---------------------------------------------------------------------------
// Singletons
// ---------------------------------------------------------------------------

/**
 * @notice Module-level singleton for general WASM artifact caching
 */
export const wasmBuildCache = new WasmBuildCache();

/**
 * @notice Module-level singleton for script deploy caching
 */
export const scriptBuildCache = new ScriptBuildCache();

export default WasmBuildCache;
 * WASM Build Pipeline Constants for Stellar Raise Contracts
 * Extracted from Cargo.toml [profile.release], .cargo/config.toml, and build scripts.
 * 
 * Usage: Import and use in Node scripts for cargo invocations, CI, or testing.
 * Ensures consistent WASM optimization for caching and reproducibility.
 * 
 * NatSpec-style:
 * @notice Constants for WASM build pipeline to improve testing readability and caching.
 * @dev Optimized for Soroban contracts (crowdfund, factory).
 * @security No runtime risks; compile-time config.
 */

export interface WasmProfileConfig {
  opt_level: 'z' | 's' | '0' | '1' | '2' | '3';
  overflow_checks: boolean;
  debug: 0 | 1 | 2;
  strip: 'none' | 'symbols' | 'all';
  debug_assertions: boolean;
  panic: 'unwind' | 'abort';
  codegen_units: number;
  lto: boolean | 'thin' | 'fat';
}

export interface WasmRustflags {
  stack_size: number;
  target_features: string[];
}

export const WASM_BUILD_CONSTANTS = {
  /**
   * Target triple for Soroban WASM builds.
   */
  target: 'wasm32-unknown-unknown' as const,

  /**
   * Release profile configuration (from Cargo.toml).
   * Used for production/testing WASM optimization.
   */
  profile: {
    opt_level: 'z' as const,
    overflow_checks: true,
    debug: 0,
    strip: 'symbols' as const,
    debug_assertions: false,
    panic: 'abort' as const,
    codegen_units: 1,
    lto: true as const,
  } satisfies WasmProfileConfig,

  /**
   * Rustflags for wasm32 target (from .cargo/config.toml).
   */
  rustflags: [
    '-C',
    'link-arg=-zstack-size=65536',
    '-C',
    'target-feature=-reference-types,-multivalue',
  ] as const,

  /**
   * Default network for deploy/testing.
   */
  network: 'testnet' as const,

  /**
   * Cargo command flags for release WASM build.
   */
  cargo_flags: [
    '--target',
    'wasm32-unknown-unknown',
    '--release',
  ] as const,
} as const;

export type WasmBuildConstants = typeof WASM_BUILD_CONSTANTS;

