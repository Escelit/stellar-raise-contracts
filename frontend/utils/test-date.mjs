import { formatDate, formatDateTime, formatRelativeTime, isExpired } from './frontend/utils/dateFormatter.js';

const testTs = 1740182400; // Feb 22, 2026
console.log('Date:', formatDate(testTs));
console.log('DateTime:', formatDateTime(testTs));
console.log('Relative:', formatRelativeTime(testTs));
console.log('Expired:', isExpired(testTs));