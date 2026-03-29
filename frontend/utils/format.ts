/**
 * Shortens a cryptocurrency address or hash for display.
 * e.g. "GABCDEFGHIJKLMNOP...1234"
 * @param address - The full address string to shorten
 * @param startChars - Number of characters to show at the start (default: 4)
 * @param endChars - Number of characters to show at the end (default: 4)
 * @returns Shortened address string with ellipsis in the middle
 */
export const shortenAddress = (
  address: string,
  startChars: number = 4,
  endChars: number = 4
): string => {
  if (!address) return "";
  if (address.length <= startChars + endChars) return address;
  return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
};

/**
 * Shortens a transaction hash for display.
 * e.g. "abc1...ef89"
 * @param hash - The full transaction hash string
 * @returns Shortened hash string with ellipsis in the middle
 */
export const shortenHash = (hash: string): string => {
  return shortenAddress(hash, 6, 4);
};

/**
 * Formats a large number with commas for readability.
 * e.g. 1000000 → "1,000,000"
 * @param value - The number to format
 * @returns Formatted number string
 */
export const formatNumber = (value: number): string => {
  return value.toLocaleString();
};
