import {
  stroopsToXLM,
  xlmToStroops,
  formatCurrency,
  formatCompact,
  formatProgress,
} from "./currencyFormatter";

describe("stroopsToXLM", () => {
  test("converts 1 stroop to correct XLM decimal", () => {
    expect(stroopsToXLM(1)).toBe(0.0000001);
  });

  test("converts 10,000,000 stroops to 1 XLM", () => {
    expect(stroopsToXLM(10_000_000)).toBe(1);
  });

  test("converts 5,000,000 stroops to 0.5 XLM", () => {
    expect(stroopsToXLM(5_000_000)).toBe(0.5);
  });

  test("converts large value of 1 billion XLM in stroops", () => {
    expect(stroopsToXLM(10_000_000_000_000_000)).toBe(1_000_000_000);
  });

  test("returns 0 for null input", () => {
    expect(stroopsToXLM(null)).toBe(0);
  });

  test("returns 0 for undefined input", () => {
    expect(stroopsToXLM(undefined)).toBe(0);
  });

  test("handles zero stroops correctly", () => {
    expect(stroopsToXLM(0)).toBe(0);
  });
});

describe("xlmToStroops", () => {
  test("converts 1 XLM to 10,000,000 stroops", () => {
    expect(xlmToStroops(1)).toBe(10_000_000);
  });

  test("converts 0.5 XLM to 5,000,000 stroops", () => {
    expect(xlmToStroops(0.5)).toBe(5_000_000);
  });

  test("converts 0.0000001 XLM to 1 stroop", () => {
    expect(xlmToStroops(0.0000001)).toBe(1);
  });

  test("returns 0 for null input", () => {
    expect(xlmToStroops(null)).toBe(0);
  });

  test("handles zero XLM correctly", () => {
    expect(xlmToStroops(0)).toBe(0);
  });
});

describe("formatCurrency", () => {
  test("formats 1 stroop without dropping decimals", () => {
    const result = formatCurrency(1, 7);
    expect(result).toBe("0.0000001 XLM");
  });

  test("formats 10,000,000 stroops as 1.00 XLM", () => {
    expect(formatCurrency(10_000_000)).toBe("1.00 XLM");
  });

  test("formats large value of 1 billion XLM correctly", () => {
    const result = formatCurrency(10_000_000_000_000_000);
    expect(result).toContain("1,000,000,000.00 XLM");
  });

  test("formats with comma separators for thousands", () => {
    const result = formatCurrency(12_500_000_000);
    expect(result).toBe("1,250.00 XLM");
  });

  test("respects custom decimal places", () => {
    const result = formatCurrency(10_000_000, 4);
    expect(result).toBe("1.0000 XLM");
  });

  test("respects custom symbol", () => {
    const result = formatCurrency(10_000_000, 2, "USDC");
    expect(result).toBe("1.00 USDC");
  });

  test("returns zero fallback for null input", () => {
    expect(formatCurrency(null)).toBe("0.00 XLM");
  });

  test("handles zero stroops correctly", () => {
    expect(formatCurrency(0)).toBe("0.00 XLM");
  });
});

describe("formatCompact", () => {
  test("formats millions correctly", () => {
    const result = formatCompact(15_000_000_000_000);
    expect(result).toBe("1.50M XLM");
  });

  test("formats thousands correctly", () => {
    const result = formatCompact(10_000_000_000);
    expect(result).toBe("1.00K XLM");
  });

  test("formats small values without suffix", () => {
    const result = formatCompact(10_000_000);
    expect(result).toBe("1.00 XLM");
  });

  test("returns zero fallback for null input", () => {
    expect(formatCompact(null)).toBe("0 XLM");
  });
});

describe("formatProgress", () => {
  test("calculates 50% progress correctly", () => {
    expect(formatProgress(5_000_000, 10_000_000)).toBe("50.00%");
  });

  test("calculates 100% when goal is met", () => {
    expect(formatProgress(10_000_000, 10_000_000)).toBe("100.00%");
  });

  test("caps at 100% when raised exceeds goal", () => {
    expect(formatProgress(20_000_000, 10_000_000)).toBe("100.00%");
  });

  test("returns 0% for zero raised", () => {
    expect(formatProgress(0, 10_000_000)).toBe("0.00%");
  });

  test("returns 0% for zero goal", () => {
    expect(formatProgress(5_000_000, 0)).toBe("0.00%");
  });

  test("returns 0% for null goal", () => {
    expect(formatProgress(5_000_000, null)).toBe("0.00%");
  });
});
