import {
  formatDate,
  formatDateTime,
  formatRelativeTime,
  isExpired,
} from "./dateFormatter";

// Fixed reference timestamp: Feb 22, 2026 00:00:00 UTC
const FIXED_TIMESTAMP = 1771718400;
const FIXED_TIMESTAMP = 1740182400;

describe("formatDate", () => {
  test("formats a valid Unix timestamp to MMM DD YYYY", () => {
    const result = formatDate(FIXED_TIMESTAMP);
    expect(result).toMatch(/Feb\s+\d{1,2},\s+2026/);
  });

  test("returns dash for null input", () => {
    expect(formatDate(null)).toBe("—");
  });

  test("returns dash for undefined input", () => {
    expect(formatDate(undefined)).toBe("—");
  });

  test("handles zero timestamp without crashing", () => {
    const result = formatDate(0);
    expect(typeof result).toBe("string");
    expect(result.length).toBeGreaterThan(0);
  });

  test("formats a far future timestamp correctly", () => {
    const future = 2000000000; // May 18, 2033
    const result = formatDate(future);
    expect(result).toMatch(/2033/);
  });
});

describe("formatDateTime", () => {
  test("formats a valid timestamp with date and time", () => {
    const result = formatDateTime(FIXED_TIMESTAMP);
    expect(result).toMatch(/2026/);
    expect(result).toMatch(/AM|PM/);
  });

  test("returns dash for null input", () => {
    expect(formatDateTime(null)).toBe("—");
  });

  test("returns dash for undefined input", () => {
    expect(formatDateTime(undefined)).toBe("—");
  });

  test("handles zero timestamp without crashing", () => {
    const result = formatDateTime(0);
    expect(typeof result).toBe("string");
  });
});

describe("formatRelativeTime", () => {
  test("returns days left for future timestamp", () => {
    const future = Math.floor(Date.now() / 1000) + 86400 * 5;
    const result = formatRelativeTime(future);
    expect(result).toMatch(/days? left/);
  });

  test("returns hours left for near future timestamp", () => {
    const future = Math.floor(Date.now() / 1000) + 3600 * 3;
    const result = formatRelativeTime(future);
    expect(result).toMatch(/hours? left/);
  });

  test("returns ended days ago for past timestamp", () => {
    const past = Math.floor(Date.now() / 1000) - 86400 * 3;
    const result = formatRelativeTime(past);
    expect(result).toMatch(/Ended \d+ days? ago/);
  });

  test("returns ended hours ago for recent past timestamp", () => {
    const past = Math.floor(Date.now() / 1000) - 3600 * 2;
    const result = formatRelativeTime(past);
    expect(result).toMatch(/Ended \d+ hours? ago/);
  });

  test("returns dash for null input", () => {
    expect(formatRelativeTime(null)).toBe("—");
  });
});

describe("isExpired", () => {
  test("returns true for past timestamp", () => {
    const past = Math.floor(Date.now() / 1000) - 1000;
    expect(isExpired(past)).toBe(true);
  });

  test("returns false for future timestamp", () => {
    const future = Math.floor(Date.now() / 1000) + 86400;
    expect(isExpired(future)).toBe(false);
  });

  test("returns false for null input", () => {
    expect(isExpired(null)).toBe(false);
  });

  test("returns false for undefined input", () => {
    expect(isExpired(undefined)).toBe(false);
  });

  test("returns true for zero timestamp", () => {
    expect(isExpired(0)).toBe(true);
  });
});
