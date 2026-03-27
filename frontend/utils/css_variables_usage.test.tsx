import { DESIGN_TOKENS, CSSVariablesContract } from './css_variables_usage';

describe('CSSVariablesUsage Contract Tests', () => {
  it('should return correct CSS variable strings from getVar()', () => {
    expect(CSSVariablesContract.getVar('COLORS', 'PRIMARY_BLUE')).toBe('var(--color-primary-blue)');
    expect(CSSVariablesContract.getVar('SPACING', 'SPACE_4')).toBe('var(--spacing-space-4)');
  });

  it('should validate approved platform colors', () => {
    // Approved color
    expect(CSSVariablesContract.isApprovedColor('#4f46e5')).toBe(true);
    // Unapproved color
    expect(CSSVariablesContract.isApprovedColor('#ff00ff')).toBe(false);
  });

  it('should return correct pixel values for spacing tokens', () => {
    // 1rem = 16px
    expect(CSSVariablesContract.getSpacingPx('SPACE_4')).toBe(16);
    // 0.25rem = 4px
    expect(CSSVariablesContract.getSpacingPx('SPACE_1')).toBe(4);
    // 3rem = 48px
    expect(CSSVariablesContract.getSpacingPx('SPACE_12')).toBe(48);
  });

  it('should contain all expected DESIGN_TOKENS categories', () => {
    expect(DESIGN_TOKENS).toHaveProperty('COLORS');
    expect(DESIGN_TOKENS).toHaveProperty('SPACING');
    expect(DESIGN_TOKENS).toHaveProperty('FONTS');
    expect(DESIGN_TOKENS).toHaveProperty('RADIUS');
  });

  it('should match the values defined in utilities.css', () => {
    // Sanity check against provided CSS variables like --color-primary-blue
    expect(DESIGN_TOKENS.COLORS.PRIMARY_BLUE).toBe('#4f46e5');
    expect(DESIGN_TOKENS.COLORS.SUCCESS_GREEN).toBe('#10b981');
  });
});
