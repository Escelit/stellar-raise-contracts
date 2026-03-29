/**
 * Number of stroops in one XLM.
 * 1 XLM = 10,000,000 stroops
 */
const STROOPS_PER_XLM = 10_000_000;

/**
 * Converts stroops to XLM.
 * @param {number} stroops - Amount in stroops
 * @returns {number} Amount in XLM
 * @example
 * stroopsToXLM(10000000) // 1
 * stroopsToXLM(5000000) // 0.5
 */
export const stroopsToXLM = (stroops) => {
  if (!stroops && stroops !== 0) return 0;
  return stroops / STROOPS_PER_XLM;
};

/**
 * Converts XLM to stroops.
 * @param {number} xlm - Amount in XLM
 * @returns {number} Amount in stroops
 * @example
 * xlmToStroops(1) // 10000000
 * xlmToStroops(0.5) // 5000000
 */
export const xlmToStroops = (xlm) => {
  if (!xlm && xlm !== 0) return 0;
  return Math.round(xlm * STROOPS_PER_XLM);
};

/**
 * Formats a stroops amount into a human-readable XLM string
 * with thousands commas and up to 7 decimal places.
 * @param {number} stroops - Amount in stroops
 * @param {number} decimals - Decimal places to display (default: 2)
 * @param {string} symbol - Currency symbol to append (default: "XLM")
 * @returns {string} Formatted currency string e.g. "1,250.50 XLM"
 * @example
 * formatCurrency(12500000000) // "1,250.00 XLM"
 * formatCurrency(5000000, 4) // "0.5000 XLM"
 */
export const formatCurrency = (stroops, decimals = 2, symbol = "XLM") => {
  if (!stroops && stroops !== 0) return `0.00 ${symbol}`;

  const xlm = stroopsToXLM(stroops);

  return `${xlm.toLocaleString("en-US", {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  })} ${symbol}`;
};

/**
 * Formats a stroops amount into a compact short form.
 * e.g. 1,500,000 XLM → "1.5M XLM", 1,000 XLM → "1K XLM"
 * @param {number} stroops - Amount in stroops
 * @param {string} symbol - Currency symbol (default: "XLM")
 * @returns {string} Compact formatted string
 * @example
 * formatCompact(15000000000000) // "1.5M XLM"
 * formatCompact(10000000000) // "1K XLM"
 */
export const formatCompact = (stroops, symbol = "XLM") => {
  if (!stroops && stroops !== 0) return `0 ${symbol}`;

  const xlm = stroopsToXLM(stroops);

  if (xlm >= 1_000_000) {
    return `${(xlm / 1_000_000).toFixed(2)}M ${symbol}`;
  }
  if (xlm >= 1_000) {
    return `${(xlm / 1_000).toFixed(2)}K ${symbol}`;
  }

  return `${xlm.toFixed(2)} ${symbol}`;
};

/**
 * Formats a progress percentage from stroops raised vs goal.
 * @param {number} raised - Amount raised in stroops
 * @param {number} goal - Goal amount in stroops
 * @returns {string} Percentage string e.g. "73.50%"
 * @example
 * formatProgress(7350000, 10000000) // "73.50%"
 */
export const formatProgress = (raised, goal) => {
  if (!goal || goal === 0) return "0.00%";
  const percent = Math.min((raised / goal) * 100, 100);
  return `${percent.toFixed(2)}%`;
};
