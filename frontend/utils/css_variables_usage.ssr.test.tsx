/**
 * @jest-environment node
 */
import { useCssVariable } from './css_variables_usage';

describe('useCssVariable hook (SSR boundaries)', () => {
  it('should safely return fallback when window is undefined', () => {
    // In explicit Node test environment, window is cleanly undefined.
    expect(typeof window).toBe('undefined');
    
    // Testing the SSR hook boundaries
    expect(useCssVariable('--color-primary-blue', '#123456')).toBe('#123456');
    expect(useCssVariable('--color-primary-blue')).toBe('');
  });
});
