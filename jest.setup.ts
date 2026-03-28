import '@testing-library/jest-dom';

// Polyfill TextEncoder/TextDecoder for jsdom environment
// Polyfill TextEncoder and TextDecoder for jsdom
import { TextEncoder, TextDecoder } from 'util';
Object.assign(global, { TextEncoder, TextDecoder });