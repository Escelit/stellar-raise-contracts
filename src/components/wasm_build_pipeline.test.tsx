/**
 * @title WASM Build Pipeline Caching – Test Suite
 * @notice Comprehensive tests covering happy paths, edge cases, and security
 * @author Stellar Raise Contracts Team
 *
 * Run with:  npx jest src/components/wasm_build_pipeline.test.tsx --coverage
 */

import {
  WasmBuildCache,
  WasmCacheValidator,
  WasmCacheError,
  wasmBuildCache,
  CACHE_KEY_PREFIX,
  DEFAULT_CACHE_TTL_MS,
  MAX_CACHE_VALUE_BYTES,
} from './wasm_build_pipeline';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const VALID_KEY = 'crowdfund-v1';
const VALID_HASH = 'deadbeef1234abcd';
const VALID_VALUE = 'AGFzbQEAAAA='; // minimal base64-encoded WASM stub

// ---------------------------------------------------------------------------
// WasmCacheValidator
// ---------------------------------------------------------------------------

describe('WasmCacheValidator', () => {
  // --- validateKey ---
  describe('validateKey', () => {
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
  });

  // --- validateValue ---
  describe('validateValue', () => {
    it('accepts a normal base64 WASM value', () => {
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

    it('throws for inline event handler injection', () => {
      expect(() =>
        WasmCacheValidator.validateValue('<img onerror=alert(1)>')
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

  // --- validateHash ---
  describe('validateHash', () => {
    it('accepts a valid lowercase hex hash', () => {
      expect(() => WasmCacheValidator.validateHash('deadbeef')).not.toThrow();
    });

    it('accepts a valid uppercase hex hash', () => {
      expect(() => WasmCacheValidator.validateHash('DEADBEEF')).not.toThrow();
    });

    it('accepts a full SHA-256 hex string', () => {
      expect(() =>
        WasmCacheValidator.validateHash(
          'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855'
        )
      ).not.toThrow();
    });

    it('throws for empty hash', () => {
      expect(() => WasmCacheValidator.validateHash('')).toThrow(WasmCacheError);
    });

    it('throws for hash with non-hex characters', () => {
      expect(() => WasmCacheValidator.validateHash('xyz123')).toThrow(WasmCacheError);
    });

    it('throws for hash with spaces', () => {
      expect(() => WasmCacheValidator.validateHash('dead beef')).toThrow(WasmCacheError);
    });
  });
});

// ---------------------------------------------------------------------------
// WasmBuildCache – core operations
// ---------------------------------------------------------------------------

describe('WasmBuildCache', () => {
  let cache: WasmBuildCache;

  beforeEach(() => {
    cache = new WasmBuildCache();
  });

  // --- set / get ---
  describe('set and get', () => {
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
      expect(entry).not.toBeNull();
      expect(entry!.expiresAt - entry!.createdAt).toBe(5000);
    });

    it('stores entry with a label', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { label: 'crowdfund contract' });
      const entry = cache.get(VALID_KEY);
      expect(entry!.label).toBe('crowdfund contract');
    });

    it('uses DEFAULT_CACHE_TTL_MS when no TTL is provided', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      const entry = cache.get(VALID_KEY);
      expect(entry!.expiresAt - entry!.createdAt).toBe(DEFAULT_CACHE_TTL_MS);
    });

    it('returns null for a missing key', () => {
      expect(cache.get('nonexistent')).toBeNull();
    });

    it('returns null for an expired entry', () => {
      jest.useFakeTimers();
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 1000 });
      jest.advanceTimersByTime(2000);
      expect(cache.get(VALID_KEY)).toBeNull();
      jest.useRealTimers();
    });

    it('evicts the expired entry on get', () => {
      jest.useFakeTimers();
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 1000 });
      jest.advanceTimersByTime(2000);
      cache.get(VALID_KEY); // triggers eviction
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

  // --- has ---
  describe('has', () => {
    it('returns true for a valid, non-expired entry', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      expect(cache.has(VALID_KEY)).toBe(true);
    });

    it('returns false for a missing key', () => {
      expect(cache.has('missing')).toBe(false);
    });

    it('returns false for an expired entry', () => {
      jest.useFakeTimers();
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 500 });
      jest.advanceTimersByTime(1000);
      expect(cache.has(VALID_KEY)).toBe(false);
      jest.useRealTimers();
    });
  });

  // --- delete ---
  describe('delete', () => {
    it('removes an existing entry', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      cache.delete(VALID_KEY);
      expect(cache.has(VALID_KEY)).toBe(false);
    });

    it('does not throw when deleting a non-existent key', () => {
      expect(() => cache.delete('nonexistent')).not.toThrow();
    });

    it('throws for invalid key', () => {
      expect(() => cache.delete('')).toThrow(WasmCacheError);
    });
  });

  // --- clear ---
  describe('clear', () => {
    it('removes all entries', () => {
      cache.set('key1', VALID_VALUE, VALID_HASH);
      cache.set('key2', VALID_VALUE, VALID_HASH);
      cache.clear();
      expect(cache.getStats().totalEntries).toBe(0);
    });

    it('does not throw on an already-empty cache', () => {
      expect(() => cache.clear()).not.toThrow();
    });
  });

  // --- evictExpired ---
  describe('evictExpired', () => {
    it('removes only expired entries and returns the count', () => {
      jest.useFakeTimers();
      cache.set('fresh', VALID_VALUE, VALID_HASH, { ttlMs: 10_000 });
      cache.set('stale1', VALID_VALUE, VALID_HASH, { ttlMs: 100 });
      cache.set('stale2', VALID_VALUE, VALID_HASH, { ttlMs: 100 });
      jest.advanceTimersByTime(500);
      const removed = cache.evictExpired();
      expect(removed).toBe(2);
      expect(cache.has('fresh')).toBe(true);
      jest.useRealTimers();
    });

    it('returns 0 when no entries are expired', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      expect(cache.evictExpired()).toBe(0);
    });

    it('returns 0 on an empty cache', () => {
      expect(cache.evictExpired()).toBe(0);
    });
  });

  // --- getStats ---
  describe('getStats', () => {
    it('returns correct stats for a populated cache', () => {
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

    it('returns zero counts for an empty cache', () => {
      const stats = cache.getStats();
      expect(stats.totalEntries).toBe(0);
      expect(stats.expiredEntries).toBe(0);
      expect(stats.validEntries).toBe(0);
      expect(stats.keys).toHaveLength(0);
    });
  });

  // --- isValid ---
  describe('isValid', () => {
    it('returns true when entry exists and hash matches', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      expect(cache.isValid(VALID_KEY, VALID_HASH)).toBe(true);
    });

    it('returns false when hash does not match', () => {
      cache.set(VALID_KEY, VALID_VALUE, VALID_HASH);
      expect(cache.isValid(VALID_KEY, 'aabbccdd')).toBe(false);
    });

    it('returns false for a missing key', () => {
      expect(cache.isValid('missing', VALID_HASH)).toBe(false);
    });

    it('returns false for an expired entry', () => {
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
// CACHE_KEY_PREFIX namespacing
// ---------------------------------------------------------------------------

describe('CACHE_KEY_PREFIX', () => {
  it('is a non-empty string', () => {
    expect(typeof CACHE_KEY_PREFIX).toBe('string');
    expect(CACHE_KEY_PREFIX.length).toBeGreaterThan(0);
  });

  it('is used to namespace keys (stats keys strip the prefix)', () => {
    const cache = new WasmBuildCache();
    cache.set('mykey', VALID_VALUE, VALID_HASH);
    const { keys } = cache.getStats();
    expect(keys).toContain('mykey');
    expect(keys).not.toContain(`${CACHE_KEY_PREFIX}mykey`);
  });
});

// ---------------------------------------------------------------------------
// Singleton export
// ---------------------------------------------------------------------------

describe('wasmBuildCache singleton', () => {
  afterEach(() => {
    wasmBuildCache.clear();
  });

  it('is an instance of WasmBuildCache', () => {
    expect(wasmBuildCache).toBeInstanceOf(WasmBuildCache);
  });

  it('can store and retrieve entries', () => {
    wasmBuildCache.set('singleton-key', VALID_VALUE, VALID_HASH);
    expect(wasmBuildCache.has('singleton-key')).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Security edge cases
// ---------------------------------------------------------------------------

describe('Security edge cases', () => {
  let cache: WasmBuildCache;

  beforeEach(() => {
    cache = new WasmBuildCache();
  });

  it('rejects <script > (with space) injection in value', () => {
    expect(() =>
      cache.set(VALID_KEY, '<script >evil()</script >', VALID_HASH)
    ).toThrow(WasmCacheError);
  });

  it('rejects onload= event handler injection', () => {
    expect(() =>
      cache.set(VALID_KEY, '<img onload=alert(1)>', VALID_HASH)
    ).toThrow(WasmCacheError);
  });

  it('rejects key with null byte', () => {
    expect(() => cache.set('key\0name', VALID_VALUE, VALID_HASH)).toThrow(WasmCacheError);
  });

  it('rejects key with semicolon', () => {
    expect(() => cache.set('key;drop', VALID_VALUE, VALID_HASH)).toThrow(WasmCacheError);
  });

  it('does not leak expired values after eviction', () => {
    jest.useFakeTimers();
    cache.set(VALID_KEY, VALID_VALUE, VALID_HASH, { ttlMs: 100 });
    jest.advanceTimersByTime(500);
    cache.evictExpired();
    expect(cache.get(VALID_KEY)).toBeNull();
    jest.useRealTimers();
  });
});
