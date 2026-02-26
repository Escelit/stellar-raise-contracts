/**
 * Formats a Unix timestamp into a human-readable date string.
 * @param {number} timestamp - Unix timestamp in seconds
 * @param {string} locale - Locale string (default: "en-US")
 * @returns {string} Formatted date string e.g. "Feb 22, 2026"
 */
export const formatDate = (timestamp, locale = "en-US") => {
  if (!timestamp && timestamp !== 0) return "—";
  const date = new Date(Number(timestamp) * 1000);
  return date.toLocaleDateString(locale, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
};

/**
 * Formats a Unix timestamp into a full date and time string.
 * @param {number} timestamp - Unix timestamp in seconds
 * @param {string} locale - Locale string (default: "en-US")
 * @returns {string} Formatted datetime string e.g. "Feb 22, 2026, 10:30 AM"
 */
export const formatDateTime = (timestamp, locale = "en-US") => {
  if (!timestamp && timestamp !== 0) return "—";
  const date = new Date(Number(timestamp) * 1000);
  return date.toLocaleString(locale, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
};

/**
 * Returns a relative time string from a Unix timestamp.
 * @param {number} timestamp - Unix timestamp in seconds
 * @returns {string} Relative time e.g. "3 days left", "Ended 2 days ago"
 */
export const formatRelativeTime = (timestamp) => {
  if (!timestamp) return "—";
  const now = Math.floor(Date.now() / 1000);
  const diff = Number(timestamp) - now;
  const absDiff = Math.abs(diff);
  
  const minutes = Math.floor(absDiff / 60);
  const hours = Math.floor(absDiff / 3600);
  const days = Math.floor(absDiff / 86400);

  if (diff > 0) {
    if (days > 0) return `${days} day${days !== 1 ? "s" : ""} left`;
    if (hours > 0) return `${hours} hour${hours !== 1 ? "s" : ""} left`;
    return `${minutes} minute${minutes !== 1 ? "s" : ""} left`;
  } else {
    if (days > 0) return `Ended ${days} day${days !== 1 ? "s" : ""} ago`;
    if (hours > 0) return `Ended ${hours} hour${hours !== 1 ? "s" : ""} ago`;
    return `Ended ${minutes} minute${minutes !== 1 ? "s" : ""} ago`;
  }
};

/**
 * Checks whether a campaign deadline has passed.
 * @param {number} timestamp - Unix timestamp in seconds
 * @returns {boolean} True if deadline has passed
 */
export const isExpired = (timestamp) => {
  if (!timestamp) return false;
  return Math.floor(Date.now() / 1000) > Number(timestamp);
};