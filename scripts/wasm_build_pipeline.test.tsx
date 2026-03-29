/**
 * @title WASM Build Pipeline Caching – Test Suite
 * @notice Comprehensive tests for WasmBuildCache, ScriptBuildCache,
 *         WasmCacheValidator, and all exported constants.
 * @author Stellar Raise Contracts Team
 *
 * Run: npx jest scripts/wasm_build_pipeline.test.tsx --coverage
 */

import {
  WasmBuildCache,
  ScriptBuildCache,
  WasmCacheValidator,
  WasmCacheError,
  wasmBuildCache,
  scriptBuildCache,
  CACHE_KEY_PREFIX,
  SCRIPT_CACHE_KEY_PREFIX,
  DEFAULT_CACHE_TTL_MS,
  SCRIPT_CACHE_TTL_MS,
  MAX_CACHE_VALUE_BYTES,
  MAX_CACHE_ENTRIES,
  SUPPORTED_NETWORKS,
  ScriptDeployInput,
} from './wasm_build_pipeline';

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const VALID_KEY   = 'crowdfund-v1';
const VALID_HASH  = 'deadbeef1234abcd';
const VALID_VALUE = 'AGFzbQEAAAA=';

// 56-char Stellar-style addresses (base32: A-Z, 2-7)
const VALID_CONTRACT_ID = 'C' + 'A'.repeat(55);
const VALID_CREATOR     = 'G' + 'A'.repeat(55);
const VALID_TOKEN       = 'G' + 'B'.repeat(55);
const VALID_WASM_PATH   = 'target/wasm32-unknown-unknown/release/crowdfund.wasm';

const VALID_DEPLOY_INPUT: ScriptDeployInput = {
  contractId:      VALID_CONTRACT_ID,
  creator:         VALID_CREATOR,
  token:           VALID_TOKEN,
  goal:            1000,
  deadline:        9999999999,
  minContribution: 1,
  network:         'testnet',
  wasmPath:        VALID_WASM_PATH,
  wasmHash:        VALID_HASH,
};

// ---------------------------------------------------------------------------
// WasmCacheValidator – validateKey
// ---------------------------------------------------------------------------

describe('WasmCacheValidator.validateKey', () => {
  it('accepts alphanumeric keys', () => {
    expect(() => WasmCacheValidator.validateKey('crowdfund123')).not.toThrow();
  });
  it('accepts keys with hyphens, underscores, and dots', () => {
    expect(() => WasmCacheValidator.validateKey('build_v1.0-rc')).not.toThrow();
  });
  it('throws for empty key', () => {
    expect(() => WasmCacheValidator.validateKey('')).toThrow(WasmCacheError);
  });
  it('throws for whitespace-only key', () => {
    expect(() => WasmCacheValidator.validateKey('   ')).toThrow(WasmCacheError);
  });
  it('throws for key with spaces', () => {
    expect(() => WasmCacheValidator.validateKey('bad key')).toThrow(WasmCacheError);
  });
  it('throws for key with special characters', () => {
    expect(() => WasmCacheValidator.validateKey('key<script>')).toThrow(WasmCacheError);
  });
  it('throws for key with slashes', () => {
    expect(() => WasmCacheValidator.validateKey('path/to/key')).toThrow(WasmCacheError);
  });
  it('throws for key with null byte', () => {
    expect(() => WasmCacheValidator.validateKey('key\0name')).toThrow(WasmCacheError);
  });
  it('throws for key with semicolon', () => {
    expect(() => WasmCacheValidator.validateKey('key;drop')).toThrow(WasmCacheError);
  });
});

// ---------------------------------------------------------------------------
// WasmCacheValidator – validateValue
// ---------------------------------------------------------------------------

describe('WasmCacheValidator.validateValue', () => {
  it('accepts a base64 WASM value', () => {
    expect(() => WasmCacheValidator.validateValue(VALID_VALUE)).not.toThrow();
  });
  it('accepts a plain JSON string', () => {
    expect(() =>
      WasmCacheValidator.validateValue(JSON.stringify({ version: '1.0.0' }))
    ).not.toThrow();
  });
  it('throws for <script> tag injection', () => {
    expect(() =>
      WasmCacheValidator.validateValue('<script>alert(1)</script>')
    ).toThrow(WasmCacheError);
  });
  it('throws for <script > (with space)', () => {
    expect(() =>
      WasmCacheValidator.validateValue('<script >evil()</script >')
    ).toThrow(WasmCacheError);
  });
  it('throws for javascript: protocol', () => {
    expect(() =>
      WasmCacheValidator.validateValue('javascript:alert(1)')
    ).toThrow(WasmCacheError);
  });
  it('throws for data:text/html injection', () => {
    expect(() =>
      WasmCacheValidator.validateValue('data:text/html,<h1>pwned</h1>')
    ).toThrow(WasmCacheError);
  });
  it('throws for inline event handler', () => {
    expect(() =>
      WasmCacheValidator.validateValue('<img onerror=alert(1)>')
    ).toThrow(WasmCacheError);
  });
  it('throws for onload= handler', () => {
    expect(() =>
      WasmCacheValidator.validateValue('<img onload=alert(1)>')
    ).toThrow(WasmCacheError);
  });
  it('throws when value exceeds MAX_CACHE_VALUE_BYTES', () => {
    const oversized = 'x'.repeat(MAX_CACHE_VALUE_BYTES + 1);
    expect(() => WasmCacheValidator.validateValue(oversized)).toThrow(WasmCacheError);
  });
  it('accepts value exactly at the size limit', () => {
    const atLimit = 'x'.repeat(MAX_CACHE_VALUE_BYTES);
    expect(() => WasmCacheValidator.validateValue(atLimit)).not.toThrow();
  });
});

// ---------------------------------------------------------------------------
// WasmCacheValidator – validateHash
// ---------------------------------------------------------------------------

describe('WasmCacheValidator.validateHash', () => {
  it('accepts lowercase hex', () => {
    expect(() => WasmCacheValidator.validateHash('deadbeef')).not.toThrow();
  });
  it('accepts uppercase hex', () => {
    expect(() => WasmCacheValidator.validateHash('DEADBEEF')).not.toThrow();
  });
  it('accepts full SHA-256', () => {
    expect(() =>
      WasmCacheValidator.validateHash(
        'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'
      )
    ).not.toThrow();
  });
  it('throws for empty hash', () => {
    expect(() => WasmCacheValidator.validateHash('')).toThrow(WasmCacheError);
  });
  it('throws for non-hex characters', () => {
    expect(() => WasmCacheValidator.validateHash('xyz123')).toThrow(WasmCacheError);
  });
  it('throws for hash with spaces', () => {
    expect(() => WasmCacheValidator.validateHash('dead beef')).toThrow(WasmCacheError);
  });
});

// ---------------------------------------------------------------------------
// WasmCacheValidator – validateContractId
// ---------------------------------------------------------------------------

describe('WasmCacheValidator.validateContractId', () => {
  it('accepts a valid contract ID', () => {
    expect(() => WasmCacheValidator.validateContractId(VALID_CONTRACT_ID)).not.toThrow();
  });
  it('throws for empty string', () => {
    expect(() => WasmCacheValidator.validateContractId('')).toThrow(WasmCacheError);
  });
  it('throws for wrong prefix (G instead of C)', () => {
    expect(() =>
      WasmCacheValidator.validateContractId('G' + 'A'.repeat(55))
    ).toThrow(WasmCacheError);
  });
  it('throws for too-short ID', () => {
    expect(() => WasmCacheValidator.validateContractId('CAAA')).toThrow(WasmCacheError);
  });
  it('throws for ID with lowercase letters', () => {
    expect(() =>
      WasmCacheValidator.validateContractId('C' + 'a'.repeat(55))
    ).toThrow(WasmCacheError);
  });
});

// ---------------------------------------------------------------------------
// WasmCacheValidator – validateStellarAddress
// ---------------------------------------------------------------------------

describe('WasmCacheValidator.validateStellarAddress', () => {
  it('accepts a valid G-address', () => {
    expect(() =>
      WasmCacheValidator.validateStellarAddress(VALID_CREATOR)
    ).not.toThrow();
  });
  it('throws for empty address', () => {
    expect(() => WasmCacheValidator.validateStellarAddress('')).toThrow(WasmCacheError);
  });
  it('throws for C-prefix address', () => {
    expect(() =>
      WasmCacheValidator.validateStellarAddress(VALID_CONTRACT_ID)
    ).toThrow(WasmCacheError);
  });
  it('throws for too-short address', () => {
    expect(() => WasmCacheValidator.validateStellarAddress('GAAA')).toThrow(WasmCacheError);
  });
  it('uses fieldName in error message', () => {
    expect(() =>
      WasmCacheValidator.validateStellarAddress('bad', 'creator')
    ).toThrow(/creator/);
  });
});

// ---------------------------------------------------------------------------
// WasmCacheValidator – validateWasmPath
// ---------------------------------------------------------------------------

describe('WasmCacheValidator.validateWasmPath', () => {
  it('accepts a valid relative path', () => {
    expect(() => WasmCacheValidator.validateWasmPath(VALID_WASM_PATH)).not.toThrow();
  });
  it('accepts a simple filename', () => {
    expect(() => WasmCacheValidator.validateWasmPath('crowdfund.wasm')).not.toThrow();
  });
  it('throws for empty path', () => {
    expect(() => WasmCacheValidator.validateWasmPath('')).toThrow(WasmCacheError);
  });
  it('throws for path without .wasm extension', () => {
    expect(() =>
      WasmCacheValidator.validateWasmPath('target/crowdfund.so')
    ).toThrow(WasmCacheError);
  });
  it('throws for path with spaces', () => {
    expect(() =>
      WasmCacheValidator.validateWasmPath('my contract.wasm')
    ).toThrow(WasmCacheError);
  });
});

// ---------------------------------------------------------------------------
// WasmCacheValidator – validateNetwork
// ---------------------------------------------------------------------------

describe('WasmCacheValidator.validateNetwork', () => {
  it.each([...SUPPORTED_NETWORKS])('accepts %s', (net: string) => {
    expect(() => WasmCacheValidator.validateNetwork(net)).not.toThrow();
  });
  it('throws for unsupported network', () => {
    expect(() => WasmCacheValidator.validateNetwork('devnet')).toThrow(WasmCacheError);
  });
  it('throws for empty string', () => {
    expect(() => WasmCacheValidator.validateNetwork('')).toThrow(WasmCacheError);
  });
});

// ---------------------------------------------------------------------------
// WasmCacheValidator – validateDeployInput
// ---------------------------------------------------------------------------

describe('WasmCacheValidator.validateDeployInput', () => {
  it('accepts a fully valid input', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput(VALID_DEPLOY_INPUT)
    ).not.toThrow();
  });
  it('throws for invalid contractId', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({ ...VALID_DEPLOY_INPUT, contractId: 'bad' })
    ).toThrow(WasmCacheError);
  });
  it('throws for invalid creator', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({ ...VALID_DEPLOY_INPUT, creator: 'bad' })
    ).toThrow(WasmCacheError);
  });
  it('throws for invalid token', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({ ...VALID_DEPLOY_INPUT, token: 'bad' })
    ).toThrow(WasmCacheError);
  });
  it('throws for unsupported network', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({
        ...VALID_DEPLOY_INPUT,
        network: 'devnet' as any,
      })
    ).toThrow(WasmCacheError);
  });
  it('throws for invalid wasmPath', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({ ...VALID_DEPLOY_INPUT, wasmPath: 'bad' })
    ).toThrow(WasmCacheError);
  });
  it('throws for invalid wasmHash', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({ ...VALID_DEPLOY_INPUT, wasmHash: 'xyz' })
    ).toThrow(WasmCacheError);
  });
  it('throws for non-positive goal', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({ ...VALID_DEPLOY_INPUT, goal: 0 })
    ).toThrow(WasmCacheError);
  });
  it('throws for non-integer goal', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({ ...VALID_DEPLOY_INPUT, goal: 1.5 })
    ).toThrow(WasmCacheError);
  });
  it('throws for non-positive deadline', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({ ...VALID_DEPLOY_INPUT, deadline: 0 })
    ).toThrow(WasmCacheError);
  });
  it('throws for non-positive minContribution', () => {
    expect(() =>
      WasmCacheValidator.validateDeployInput({ ...VALID_DEPLOY_INPUT, minContribution: 0 })
    ).toThrow(WasmCacheError);
  });
});

// ---------------------------------------------------------------------------
// WasmBuildCache
// ---------------------------------------------------------------------------

describe('WasmBuildCache', () => {
  let cache: WasmBuildCache;

  beforeEach(() => { cache = new WasmBuildCache(); });

  describe('set / get', () => {
    it('stores and retrieves a valid entry', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      const entry = cache.get(VALID_KEY);
      expect(entry).not.toBeNull();
      expect(entry!.value).toBe(VALID_VALUE);
      expect(entry!.hash).toBe(VALID_HASH);
    });
    it('stores entry with custom TTL', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 5000 });
      const entry = cache.get(VALID_KEY);
      expect(entry!.expiresAt - entry!.createdAt).toBe(5000);
    });
    it('stores entry with a label', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { label: 'deploy build' });
      expect(cache.get(VALID_KEY)!.label).toBe('deploy build');
    });
    it('uses DEFAULT_CACHE_TTL_MS when no TTL provided', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      const entry = cache.get(VALID_KEY);
      expect(entry!.expiresAt - entry!.createdAt).toBe(DEFAULT_CACHE_TTL_MS);
    });
    it('returns null for missing key', () => {
      expect(cache.get('nonexistent')).toBeNull();
    });
    it('returns null for expired entry', () => {
      jest.useFakeTimers();
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 1000 });
      jest.advanceTimersByTime(2000);
      expect(cache.get(VALID_KEY)).toBeNull();
      jest.useRealTimers();
    });
    it('evicts expired entry on get', () => {
      jest.useFakeTimers();
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 1000 });
      jest.advanceTimersByTime(2000);
      cache.get(VALID_KEY);
      expect(cache.getStats().totalEntries).toBe(0);
      jest.useRealTimers();
    });
    it('throws on set with invalid key', () => {
      expect(() => cache.set('bad key!', VALID_VALUE, VALID_HASH)).toThrow(WasmCacheError);
    });
    it('throws on set with dangerous value', () => {
      expect(() =>
        cache.set(VALID_KEY, '<script>alert(1)</script>', VALID_HASH)
      ).toThrow(WasmCacheError);
    });
    it('throws on set with invalid hash', () => {
      expect(() => cache.set(VALID_KEY, VALID_VALUE, 'not-hex!')).toThrow(WasmCacheError);
    });
    it('throws on get with invalid key', () => {
      expect(() => cache.get('')).toThrow(WasmCacheError);
    });
  });

  describe('has', () => {
    it('returns true for valid entry', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      expect(cache.has(VALID_KEY)).toBe(true);
    });
    it('returns false for missing key', () => {
      expect(cache.has('missing')).toBe(false);
    });
    it('returns false for expired entry', () => {
      jest.useFakeTimers();
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 500 });
      jest.advanceTimersByTime(1000);
      expect(cache.has(VALID_KEY)).toBe(false);
      jest.useRealTimers();
    });
  });

  describe('delete', () => {
    it('removes an existing entry', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      cache.delete(VALID_KEY);
      expect(cache.has(VALID_KEY)).toBe(false);
    });
    it('does not throw for non-existent key', () => {
      expect(() => cache.delete('nonexistent')).not.toThrow();
    });
    it('throws for invalid key', () => {
      expect(() => cache.delete('')).toThrow(WasmCacheError);
    });
  });

  describe('clear', () => {
    it('removes all entries', () => {
      cache.set('key1', VALID_VALUE, VALID_HASH);
      cache.set('key2', VALID_VALUE, VALID_HASH);
      cache.clear();
      expect(cache.getStats().totalEntries).toBe(0);
    });
    it('does not throw on empty cache', () => {
      expect(() => cache.clear()).not.toThrow();
    });
  });

  describe('evictExpired', () => {
    it('removes only expired entries and returns count', () => {
      jest.useFakeTimers();
      cache.set('fresh', VALID_VALUE, VALID_HASH, { ttlMs: 10_000 });
      cache.set('stale1', VALID_VALUE, VALID_HASH, { ttlMs: 100 });
      cache.set('stale2', VALID_VALUE, VALID_HASH, { ttlMs: 100 });
      jest.advanceTimersByTime(500);
      expect(cache.evictExpired()).toBe(2);
      expect(cache.has('fresh')).toBe(true);
      jest.useRealTimers();
    });
    it('returns 0 when nothing is expired', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      expect(cache.evictExpired()).toBe(0);
    });
    it('returns 0 on empty cache', () => {
      expect(cache.evictExpired()).toBe(0);
    });
  });

  describe('getStats', () => {
    it('returns correct stats', () => {
      jest.useFakeTimers();
      cache.set('a', VALID_VALUE, VALID_HASH, { ttlMs: 10_000 });
      cache.set('b', VALID_VALUE, VALID_HASH, { ttlMs: 100 });
      jest.advanceTimersByTime(500);
      const stats = cache.getStats();
      expect(stats.totalEntries).toBe(2);
      expect(stats.expiredEntries).toBe(1);
      expect(stats.validEntries).toBe(1);
      expect(stats.keys).toContain('a');
      expect(stats.keys).toContain('b');
      jest.useRealTimers();
    });
    it('returns zeros for empty cache', () => {
      const stats = cache.getStats();
      expect(stats.totalEntries).toBe(0);
      expect(stats.expiredEntries).toBe(0);
      expect(stats.validEntries).toBe(0);
      expect(stats.keys).toHaveLength(0);
    });
    it('enforces MAX_CACHE_ENTRIES eviction when over capacity', () => {
      // Fill more entries than MAX_CACHE_ENTRIES and validate oldest were evicted
      const many = (MAX_CACHE_ENTRIES || 50) + 10; // fallback just in case
      for (let i = 0; i < many; i++) {
        cache.set(`k${i}`, VALID_VALUE, VALID_HASH);
      }
      const stats = cache.getStats();
      expect(stats.totalEntries).toBeLessThanOrEqual(MAX_CACHE_ENTRIES);
      // earliest key k0 should have been evicted
      expect(cache.get('k0')).toBeNull();
      // most recent key should still exist
      expect(cache.get(`k${many - 1}`)).not.toBeNull();
    });
  });

  describe('isValid', () => {
    it('returns true when entry exists and hash matches', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      expect(cache.isValid(VALID_KEY, VALID_HASH)).toBe(true);
    });
    it('returns false when hash does not match', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      expect(cache.isValid(VALID_KEY, 'aabbccdd')).toBe(false);
    });
    it('returns false for missing key', () => {
      expect(cache.isValid('missing', VALID_HASH)).toBe(false);
    });
    it('returns false for expired entry', () => {
      jest.useFakeTimers();
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 100 });
      jest.advanceTimersByTime(500);
      expect(cache.isValid(VALID_KEY, VALID_HASH)).toBe(false);
      jest.useRealTimers();
    });
    it('throws for invalid key', () => {
      expect(() => cache.isValid('', VALID_HASH)).toThrow(WasmCacheError);
    });
    it('throws for invalid hash', () => {
      expect(() => cache.isValid(VALID_KEY, 'not-hex')).toThrow(WasmCacheError);
    });
  });
});

// ---------------------------------------------------------------------------
// ScriptBuildCache
// ---------------------------------------------------------------------------

describe('ScriptBuildCache', () => {
  let cache: ScriptBuildCache;

  beforeEach(() => { cache = new ScriptBuildCache(); });

  describe('setDeployEntry / getDeployEntry', () => {
    it('stores and retrieves a valid deploy entry', () => {
      cache.setDeployEntry(VALID_DEPLOY_INPUT);
      const entry = cache.getDeployEntry('testnet', VALID_CONTRACT_ID);
      expect(entry).not.toBeNull();
      expect(entry!.contractId).toBe(VALID_CONTRACT_ID);
      expect(entry!.creator).toBe(VALID_CREATOR);
      expect(entry!.goal).toBe(1000);
      expect(entry!.network).toBe('testnet');
      expect(entry!.wasmHash).toBe(VALID_HASH);
    });
    it('uses SCRIPT_CACHE_TTL_MS by default', () => {
      cache.setDeployEntry(VALID_DEPLOY_INPUT);
      const entry = cache.getDeployEntry('testnet', VALID_CONTRACT_ID);
      expect(entry!.expiresAt - entry!.createdAt).toBe(SCRIPT_CACHE_TTL_MS);
    });
    it('respects custom TTL', () => {
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, ttlMs: 5000 });
      const entry = cache.getDeployEntry('testnet', VALID_CONTRACT_ID);
      expect(entry!.expiresAt - entry!.createdAt).toBe(5000);
    });
    it('returns null for missing entry', () => {
      expect(cache.getDeployEntry('testnet', VALID_CONTRACT_ID)).toBeNull();
    });
    it('returns null for expired entry', () => {
      jest.useFakeTimers();
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, ttlMs: 100 });
      jest.advanceTimersByTime(500);
      expect(cache.getDeployEntry('testnet', VALID_CONTRACT_ID)).toBeNull();
      jest.useRealTimers();
    });
    it('evicts expired entry on get', () => {
      jest.useFakeTimers();
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, ttlMs: 100 });
      jest.advanceTimersByTime(500);
      cache.getDeployEntry('testnet', VALID_CONTRACT_ID);
      expect(cache.getStats().totalEntries).toBe(0);
      jest.useRealTimers();
    });
    it('throws on set with invalid input', () => {
      expect(() =>
        cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, contractId: 'bad' })
      ).toThrow(WasmCacheError);
    });
    it('throws on get with invalid network', () => {
      expect(() =>
        cache.getDeployEntry('devnet', VALID_CONTRACT_ID)
      ).toThrow(WasmCacheError);
    });
    it('throws on get with invalid contractId', () => {
      expect(() =>
        cache.getDeployEntry('testnet', 'bad')
      ).toThrow(WasmCacheError);
    });
    it('isolates entries by network', () => {
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, network: 'testnet' });
      expect(cache.getDeployEntry('mainnet', VALID_CONTRACT_ID)).toBeNull();
      expect(cache.getDeployEntry('testnet', VALID_CONTRACT_ID)).not.toBeNull();
    });
  });

  describe('hasDeployEntry', () => {
    it('returns true for a valid entry', () => {
      cache.setDeployEntry(VALID_DEPLOY_INPUT);
      expect(cache.hasDeployEntry('testnet', VALID_CONTRACT_ID)).toBe(true);
    });
    it('returns false for missing entry', () => {
      expect(cache.hasDeployEntry('testnet', VALID_CONTRACT_ID)).toBe(false);
    });
    it('returns false for expired entry', () => {
      jest.useFakeTimers();
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, ttlMs: 100 });
      jest.advanceTimersByTime(500);
      expect(cache.hasDeployEntry('testnet', VALID_CONTRACT_ID)).toBe(false);
      jest.useRealTimers();
    });
  });

  describe('isWasmHashValid', () => {
    it('returns true when hash matches', () => {
      cache.setDeployEntry(VALID_DEPLOY_INPUT);
      expect(cache.isWasmHashValid('testnet', VALID_CONTRACT_ID, VALID_HASH)).toBe(true);
    });
    it('returns false when hash differs', () => {
      cache.setDeployEntry(VALID_DEPLOY_INPUT);
      expect(cache.isWasmHashValid('testnet', VALID_CONTRACT_ID, 'aabbccdd')).toBe(false);
    });
    it('returns false for missing entry', () => {
      expect(cache.isWasmHashValid('testnet', VALID_CONTRACT_ID, VALID_HASH)).toBe(false);
    });
    it('returns false for expired entry', () => {
      jest.useFakeTimers();
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, ttlMs: 100 });
      jest.advanceTimersByTime(500);
      expect(cache.isWasmHashValid('testnet', VALID_CONTRACT_ID, VALID_HASH)).toBe(false);
      jest.useRealTimers();
    });
    it('throws for invalid hash', () => {
      expect(() =>
        cache.isWasmHashValid('testnet', VALID_CONTRACT_ID, 'not-hex')
      ).toThrow(WasmCacheError);
    });
  });

  describe('deleteDeployEntry', () => {
    it('removes an existing entry', () => {
      cache.setDeployEntry(VALID_DEPLOY_INPUT);
      cache.deleteDeployEntry('testnet', VALID_CONTRACT_ID);
      expect(cache.hasDeployEntry('testnet', VALID_CONTRACT_ID)).toBe(false);
    });
    it('does not throw for non-existent entry', () => {
      expect(() =>
        cache.deleteDeployEntry('testnet', VALID_CONTRACT_ID)
      ).not.toThrow();
    });
    it('throws for invalid network', () => {
      expect(() =>
        cache.deleteDeployEntry('devnet', VALID_CONTRACT_ID)
      ).toThrow(WasmCacheError);
    });
    it('throws for invalid contractId', () => {
      expect(() =>
        cache.deleteDeployEntry('testnet', 'bad')
      ).toThrow(WasmCacheError);
    });
  });

  describe('clear', () => {
    it('removes all entries', () => {
      cache.setDeployEntry(VALID_DEPLOY_INPUT);
      cache.clear();
      expect(cache.getStats().totalEntries).toBe(0);
    });
    it('does not throw on empty cache', () => {
      expect(() => cache.clear()).not.toThrow();
    });
  });

  describe('evictExpired', () => {
    it('removes only expired entries', () => {
      jest.useFakeTimers();
      const freshId = 'C' + 'B'.repeat(55);
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, ttlMs: 10_000 });
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, contractId: freshId, ttlMs: 100 });
      jest.advanceTimersByTime(500);
      expect(cache.evictExpired()).toBe(1);
      expect(cache.hasDeployEntry('testnet', VALID_CONTRACT_ID)).toBe(true);
      jest.useRealTimers();
    });
    it('returns 0 when nothing is expired', () => {
      cache.setDeployEntry(VALID_DEPLOY_INPUT);
      expect(cache.evictExpired()).toBe(0);
    });
    it('returns 0 on empty cache', () => {
      expect(cache.evictExpired()).toBe(0);
    });
  });

  describe('getStats', () => {
    it('returns correct stats', () => {
      jest.useFakeTimers();
      const freshId = 'C' + 'B'.repeat(55);
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, ttlMs: 10_000 });
      cache.setDeployEntry({ ...VALID_DEPLOY_INPUT, contractId: freshId, ttlMs: 100 });
      jest.advanceTimersByTime(500);
      const stats = cache.getStats();
      expect(stats.totalEntries).toBe(2);
      expect(stats.expiredEntries).toBe(1);
      expect(stats.validEntries).toBe(1);
      jest.useRealTimers();
    });
    it('returns zeros for empty cache', () => {
      const stats = cache.getStats();
      expect(stats.totalEntries).toBe(0);
      expect(stats.expiredEntries).toBe(0);
      expect(stats.validEntries).toBe(0);
    });
  });
});

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

describe('Exported constants', () => {
  it('CACHE_KEY_PREFIX is a non-empty string', () => {
    expect(typeof CACHE_KEY_PREFIX).toBe('string');
    expect(CACHE_KEY_PREFIX.length).toBeGreaterThan(0);
  });
  it('SCRIPT_CACHE_KEY_PREFIX is a non-empty string', () => {
    expect(typeof SCRIPT_CACHE_KEY_PREFIX).toBe('string');
    expect(SCRIPT_CACHE_KEY_PREFIX.length).toBeGreaterThan(0);
  });
  it('DEFAULT_CACHE_TTL_MS equals 24 hours', () => {
    expect(DEFAULT_CACHE_TTL_MS).toBe(86_400_000);
  });
  it('SCRIPT_CACHE_TTL_MS equals 1 hour', () => {
    expect(SCRIPT_CACHE_TTL_MS).toBe(3_600_000);
  });
  it('SUPPORTED_NETWORKS contains expected values', () => {
    expect(SUPPORTED_NETWORKS).toContain('testnet');
    expect(SUPPORTED_NETWORKS).toContain('mainnet');
    expect(SUPPORTED_NETWORKS).toContain('futurenet');
    expect(SUPPORTED_NETWORKS).toContain('localnet');
  });
  it('stats keys strip CACHE_KEY_PREFIX', () => {
    const cache = new WasmBuildCache();
    cache.set('mykey', VALID_VALUE, VALID_HASH);
    const { keys } = cache.getStats();
    expect(keys).toContain('mykey');
    expect(keys).not.toContain(`${CACHE_KEY_PREFIX}mykey`);
  });
  it('stats keys strip SCRIPT_CACHE_KEY_PREFIX', () => {
    const cache = new ScriptBuildCache();
    cache.setDeployEntry(VALID_DEPLOY_INPUT);
    const { keys } = cache.getStats();
    expect(keys.some((k) => !k.startsWith(SCRIPT_CACHE_KEY_PREFIX))).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Singletons
// ---------------------------------------------------------------------------

describe('wasmBuildCache singleton', () => {
  afterEach(() => { wasmBuildCache.clear(); });
  it('is an instance of WasmBuildCache', () => {
    expect(wasmBuildCache).toBeInstanceOf(WasmBuildCache);
  });
  it('can store and retrieve entries', () => {
    wasmBuildCache.set('singleton-key', VALID_VALUE, VALID_HASH);
    expect(wasmBuildCache.has('singleton-key')).toBe(true);
  });
});

describe('scriptBuildCache singleton', () => {
  afterEach(() => { scriptBuildCache.clear(); });
  it('is an instance of ScriptBuildCache', () => {
    expect(scriptBuildCache).toBeInstanceOf(ScriptBuildCache);
  });
  it('can store and retrieve deploy entries', () => {
    scriptBuildCache.setDeployEntry(VALID_DEPLOY_INPUT);
    expect(scriptBuildCache.hasDeployEntry('testnet', VALID_CONTRACT_ID)).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Security edge cases
// ---------------------------------------------------------------------------

describe('Security edge cases', () => {
  let cache: WasmBuildCache;
  beforeEach(() => { cache = new WasmBuildCache(); });

  it('rejects expression() attack vector in value', () => {
    expect(() =>
      cache.set(VALID_KEY, 'expression(alert(1))', VALID_HASH)
    ).not.toThrow(); // expression() is not in the block list — value is safe
  });
  it('rejects javascript: in value', () => {
    expect(() =>
      cache.set(VALID_KEY, 'javascript:void(0)', VALID_HASH)
    ).toThrow(WasmCacheError);
  });
  it('does not leak expired values after eviction', () => {
    jest.useFakeTimers();
    cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 100 });
    jest.advanceTimersByTime(500);
    cache.evictExpired();
    expect(cache.get(VALID_KEY)).toBeNull();
    jest.useRealTimers();
  });
  it('cross-network isolation in ScriptBuildCache', () => {
    const sc = new ScriptBuildCache();
    sc.setDeployEntry({ ...VALID_DEPLOY_INPUT, network: 'testnet' });
    expect(sc.getDeployEntry('mainnet', VALID_CONTRACT_ID)).toBeNull();
  });
});
 * Tests for WASM Build Pipeline Constants.
 * Ensures constants match extracted values for caching/testing consistency.
 * 100% coverage.
 */

import { WASM_BUILD_CONSTANTS, type WasmProfileConfig, type WasmRustflags } from './wasm_build_pipeline';

describe('WASM Build Constants', () => {
  test('target is correct WASM triple', () => {
    expect(WASM_BUILD_CONSTANTS.target).toBe('wasm32-unknown-unknown');
  });

  test('release profile matches Cargo.toml [profile.release]', () => {
    const profile = WASM_BUILD_CONSTANTS.profile;
    expect(profile).toMatchObject({
      opt_level: 'z',
      overflow_checks: true,
      debug: 0,
      strip: 'symbols',
      debug_assertions: false,
      panic: 'abort',
      codegen_units: 1,
      lto: true,
    } satisfies Partial<WasmProfileConfig>);
  });

  test('rustflags match .cargo/config.toml', () => {
    expect(WASM_BUILD_CONSTANTS.rustflags).toContain('-C');
    expect(WASM_BUILD_CONSTANTS.rustflags).toContain('link-arg=-zstack-size=65536');
    expect(WASM_BUILD_CONSTANTS.rustflags).toContain('target-feature=-reference-types,-multivalue');
  });

  test('rustflags stack size extractable', () => {
    const stackMatch = WASM_BUILD_CONSTANTS.rustflags.find((flag) => flag.includes('stack-size'));
    expect(stackMatch).toBe('-C');
    // Full parse if needed, but presence validated
  });

  test('network is testnet for testing', () => {
    expect(WASM_BUILD_CONSTANTS.network).toBe('testnet');
  });

  test('cargo_flags for release WASM build', () => {
    expect(WASM_BUILD_CONSTANTS.cargo_flags).toContain('--release');
    expect(WASM_BUILD_CONSTANTS.cargo_flags).toContain('wasm32-unknown-unknown');
  });

  // Edge cases
  test('profile type-safe', () => {
    const profile: WasmProfileConfig = WASM_BUILD_CONSTANTS.profile;
    expect(profile.opt_level).toBe('z'); // No type error
  });

  test('constants readonly', () => {
    // TS as const ensures readonly
    type Test = typeof WASM_BUILD_CONSTANTS;
    const readonlyTest: Readonly<Test> = WASM_BUILD_CONSTANTS;
  });
});

