/**
 * CSS Variables Usage "Contract"
 * 
 * This file acts as the single source of truth for design tokens used across the 
 * Stellar Raise frontend and their corresponding logical bounds in the smart contracts.
 * 
 * @module CSSVariablesUsage
 */

/**
 * Design Token Constants
 * These should match the values defined in utilities.css and utils.css
 */
export const DESIGN_TOKENS = {
  COLORS: {
    PRIMARY_BLUE: '#4f46e5',
    DEEP_NAVY: '#1e293b',
    SUCCESS_GREEN: '#10b981',
    ERROR_RED: '#ef4444',
    WARNING_ORANGE: '#f59e0b',
    NEUTRAL_100: '#f9fafb',
    NEUTRAL_200: '#f3f4f6',
    NEUTRAL_300: '#e5e7eb',
    NEUTRAL_700: '#374151',
  },
  SPACING: {
    SPACE_1: '0.25rem',
    SPACE_2: '0.5rem',
    SPACE_3: '0.75rem',
    SPACE_4: '1rem',
    SPACE_5: '1.25rem',
    SPACE_6: '1.5rem',
    SPACE_8: '2rem',
    SPACE_10: '2.5rem',
    SPACE_12: '3rem',
  },
  FONTS: {
    XS: '0.75rem',
    SM: '0.875rem',
    BASE: '1rem',
    LG: '1.125rem',
    XL: '1.25rem',
    '2XL': '1.5rem',
    '3XL': '1.875rem',
  },
  RADIUS: {
    SM: '0.125rem',
    MD: '0.375rem',
    LG: '0.5rem',
    XL: '0.75rem',
    FULL: '9999px',
  }
} as const;

/**
 * CSS Variable Contract Class
 * Provides helper methods to ensure UI consistency and reliability.
 */
export class CSSVariablesContract {
  /**
   * Returns a CSS variable string for use in inline styles or styled-components.
   * @param category The token category (colors, spacing, fonts, radius)
   * @param key The specific token key
   */
  static getVar(category: keyof typeof DESIGN_TOKENS, key: string): string {
    const formattedKey = key.toLowerCase().replace(/_/g, '-');
    return `var(--${category.toLowerCase().slice(0, -1)}-${formattedKey})`;
  }

  /**
   * Validates if a hex color is part of the approved platform palette.
   * @param hex The color hex code to validate.
   */
  static isApprovedColor(hex: string): boolean {
    return Object.values(DESIGN_TOKENS.COLORS).includes(hex.toLowerCase() as any);
  }

  /**
   * Returns the absolute pixel value for a spacing token (assuming 16px base rem).
   * @param key The spacing key
   */
  static getSpacingPx(key: keyof typeof DESIGN_TOKENS.SPACING): number {
    const remStr = DESIGN_TOKENS.SPACING[key];
    return parseFloat(remStr) * 16;
 * @title CSS Variables Usage Utility
 * @notice Secure utility for CSS custom properties (CSS variables) handling
 * @dev Provides type-safe access to design tokens with validation and sanitization
 * @author Stellar Raise Security Team
 * @notice SECURITY: All CSS variable access must go through this utility to prevent
 *         CSS injection attacks and ensure variable name validation.
 */

/**
 * Design Token Constants
 * These should match the values defined in utilities.css and utils.css
 */
export const DESIGN_TOKENS = {
  COLORS: {
    PRIMARY_BLUE: '#4f46e5',
    DEEP_NAVY: '#1e293b',
    SUCCESS_GREEN: '#10b981',
    ERROR_RED: '#ef4444',
    WARNING_ORANGE: '#f59e0b',
    NEUTRAL_100: '#f9fafb',
    NEUTRAL_200: '#f3f4f6',
    NEUTRAL_300: '#e5e7eb',
    NEUTRAL_700: '#374151',
  },
  SPACING: {
    SPACE_1: '0.25rem',
    SPACE_2: '0.5rem',
    SPACE_3: '0.75rem',
    SPACE_4: '1rem',
    SPACE_5: '1.25rem',
    SPACE_6: '1.5rem',
    SPACE_8: '2rem',
    SPACE_10: '2.5rem',
    SPACE_12: '3rem',
  },
  FONTS: {
    XS: '0.75rem',
    SM: '0.875rem',
    BASE: '1rem',
    LG: '1.125rem',
    XL: '1.25rem',
    '2XL': '1.5rem',
    '3XL': '1.875rem',
  },
  RADIUS: {
    SM: '0.125rem',
    MD: '0.375rem',
    LG: '0.5rem',
    XL: '0.75rem',
    FULL: '9999px',
  }
} as const;

/**
 * CSS Variable Contract Class
 * Provides helper methods to ensure UI consistency and reliability.
 */
export class CSSVariablesContract {
  /**
   * Returns a CSS variable string for use in inline styles or styled-components.
   * @param category The token category (colors, spacing, fonts, radius)
   * @param key The specific token key
   */
  static getVar(category: keyof typeof DESIGN_TOKENS, key: string): string {
    const formattedKey = key.toLowerCase().replace(/_/g, '-');
    return `var(--${category.toLowerCase().slice(0, -1)}-${formattedKey})`;
  }

/**
 * @notice Predefined list of allowed CSS variable names
 * @dev Derived from the THEME object to ensure single source of truth
 */
export const ALLOWED_CSS_VARIABLES = [
  // Breakpoints
  '--breakpoint-mobile',
  '--breakpoint-tablet',

  // Brand Colors
  '--color-primary-blue',
  '--color-deep-navy',
  '--color-success-green',
  '--color-error-red',
  '--color-warning-orange',
  '--color-neutral-100',
  '--color-neutral-200',
  '--color-neutral-300',
  '--color-neutral-700',
  '--color-neutral-900',

  // Typography
  '--font-family-primary',
  '--font-size-xs',
  '--font-size-sm',
  '--font-size-base',
  '--font-size-lg',
  '--font-size-xl',
  '--font-size-2xl',
  '--font-size-3xl',

  // Spacing
  '--space-1',
  '--space-2',
  '--space-3',
  '--space-4',
  '--space-5',
  '--space-6',
  '--space-8',
  '--space-10',
  '--space-12',
  '--space-16',

  // Touch Targets
  '--touch-target-min',

  // Grid System
  '--grid-columns-mobile',
  '--grid-columns-tablet',
  '--grid-columns-desktop',
  '--grid-gutter-mobile',
  '--grid-gutter-tablet',
  '--grid-gutter-desktop',

  // Container
  '--container-mobile',
  '--container-tablet',
  '--container-desktop',

  // Z-index Scale
  '--z-base',
  '--z-dropdown',
  '--z-sticky',
  '--z-fixed',
  '--z-modal-backdrop',
  '--z-modal',
  '--z-toast',

  // Transitions
  '--transition-fast',
  '--transition-base',
  '--transition-slow',

  // Border Radius
  '--radius-sm',
  '--radius-md',
  '--radius-lg',
  '--radius-xl',
  '--radius-full',

  // Shadows
  '--shadow-sm',
  '--shadow-md',
  '--shadow-lg',
  '--shadow-xl',

  // Safe Area Insets
  '--safe-area-inset-top',
  '--safe-area-inset-right',
  '--safe-area-inset-bottom',
  '--safe-area-inset-left',

  // Documentation
  '--color-docs-bg',
  '--color-docs-text',
  '--color-docs-link',
  '--color-docs-accent',
  '--font-docs-code',
  '--space-docs-content',
  ...Object.values(THEME.colors),
  ...Object.values(THEME.spacing),
  ...Object.values(THEME.typography),
  ...Object.values(THEME.layout),
  ...Object.values(THEME.zIndex),
  ...Object.values(THEME.effects),
  ...Object.values(THEME.safeArea),
  ...Object.values(THEME.docs),
] as const;
  /**
   * Validates if a hex color is part of the approved platform palette.
   * @param hex The color hex code to validate.
   */
  static isApprovedColor(hex: string): boolean {
    return Object.values(DESIGN_TOKENS.COLORS).includes(hex.toLowerCase() as any);
  }

  /**
   * Returns the absolute pixel value for a spacing token (assuming 16px base rem).
   * @param key The spacing key
   */
  static getSpacingPx(key: keyof typeof DESIGN_TOKENS.SPACING): number {
    const remStr = DESIGN_TOKENS.SPACING[key];
    return parseFloat(remStr) * 16;
  }
}

/**
 * @title CssVariableValidator
 * @notice Static validator class for CSS variable operations
 */
export class CssVariableValidator {
  /**
   * @notice Validates if a CSS variable name is allowed
   * @param variableName The variable name to validate
   * @returns True if the variable is valid and allowed
   * @throws CssVariablesError if validation fails
   */
  static isValidVariableName(variableName: string): boolean {
    // Check format
    if (!CSS_VAR_NAME_REGEX.test(variableName)) {
      throw new CssVariablesError(
        `Invalid CSS variable name format: "${variableName}". Must start with "--" and contain only alphanumeric characters, hyphens, and underscores.`
      );
    }

    // Check against allowed list
    if (!ALLOWED_CSS_VARIABLES.includes(variableName as AllowedCssVariable)) {
      throw new CssVariablesError(
        `CSS variable "${variableName}" is not in the allowed list. Use getAllowedVariables() to see available variables.`
      );
    }

    return true;
  }

  /**
   * @notice Validates a CSS value for potential security issues
   * @param value The CSS value to validate
   * @returns True if the value is safe
   * @throws CssVariablesError if dangerous patterns are detected
   */
  static isValidValue(value: string): boolean {
    if (DANGEROUS_CSS_PATTERN.test(value)) {
      throw new CssVariablesError(
        `Potentially dangerous CSS value detected. URL(), expression(), and @import are not allowed for security reasons.`
      );
    }
    return true;
  }

  /**
   * @notice Returns the list of all allowed CSS variables
   * @returns Readonly array of allowed variable names
   */
  static getAllowedVariables(): readonly string[] {
    return ALLOWED_CSS_VARIABLES;
  }
}

/**
 * @title CssVariablesUsage
 * @notice Main utility class for secure CSS variable operations
 */
export class CssVariablesUsage {
  private element: HTMLElement;
  private _cache = new Map<string, string>();

  /**
   * @notice Creates a new CssVariablesUsage instance
   * @param element The HTML element to operate on (defaults to document.documentElement)
   */
  constructor(element?: HTMLElement) {
    this.element = element || document.documentElement;
  }

  /**
   * @notice Invalidates the internal cache
   */
  invalidateCache(): void {
    this._cache.clear();
  }

  /**
   * @notice Gets a CSS variable value securely
   * @param variableName The name of the CSS variable (with or without -- prefix)
   * @param fallback Optional fallback value if variable is not defined
   * @returns The computed value of the CSS variable
   * @throws CssVariablesError if variable name is invalid
   */
  get(variableName: string, fallback?: string): string {
    const normalizedName = this.normalizeVariableName(variableName);
    CssVariableValidator.isValidVariableName(normalizedName);

    if (this._cache.has(normalizedName)) {
      return this._cache.get(normalizedName)!;
    }

    const computedStyle = getComputedStyle(this.element);
    const value = computedStyle.getPropertyValue(normalizedName).trim();
    this._cache.set(normalizedName, value);

    // Normalize variable name
    const normalizedName = this.normalizeVariableName(variableName);
    CssVariableValidator.isValidVariableName(normalizedName);

    if (this._cache.has(normalizedName)) {
      return this._cache.get(normalizedName)!;
    }

    const computedStyle = getComputedStyle(this.element);
    const value = computedStyle.getPropertyValue(normalizedName).trim();
    this._cache.set(normalizedName, value);

    return value || fallback || '';
  }

  /**
   * @notice Sets a CSS variable value securely
   * @param variableName The name of the CSS variable
   * @param value The value to set
   * @throws CssVariablesError if variable name or value is invalid
   */
  set(variableName: string, value: string): void {
    const normalizedName = this.normalizeVariableName(variableName);

    CssVariableValidator.isValidVariableName(normalizedName);
    CssVariableValidator.isValidValue(value);

    this.element.style.setProperty(normalizedName, value);
    this.invalidateCache();
    // Normalize variable name
    const normalizedName = this.normalizeVariableName(variableName);

    CssVariableValidator.isValidVariableName(normalizedName);
    CssVariableValidator.isValidValue(value);

    this.element.style.setProperty(normalizedName, value);
    this.invalidateCache();
  }

  /**
   * @notice Removes a CSS variable
   * @param variableName The name of the CSS variable to remove
   * @throws CssVariablesError if variable name is invalid
   */
  remove(variableName: string): void {
    const normalizedName = this.normalizeVariableName(variableName);

    CssVariableValidator.isValidVariableName(normalizedName);

    this.element.style.removeProperty(normalizedName);
    this.invalidateCache();
    // Normalize variable name
    const normalizedName = this.normalizeVariableName(variableName);

    CssVariableValidator.isValidVariableName(normalizedName);

    this.element.style.removeProperty(normalizedName);
    this.invalidateCache();
  }

  /**
   * @notice Gets multiple CSS variables at once
   * @param variableNames Array of variable names to retrieve
   * @param fallback Optional fallback value for undefined variables
   * @returns Object mapping variable names to their values
   * @throws CssVariablesError if any variable name is invalid
   */
  getMultiple(variableNames: string[], fallback?: string): CssVariablesMap {
    const result: CssVariablesMap = {};

    for (const name of variableNames) {
      const normalizedName = this.normalizeVariableName(name);
      CssVariableValidator.isValidVariableName(normalizedName);
      result[normalizedName as AllowedCssVariable] = this.get(name, fallback);
    }

    return result;
  }

  /**
   * @notice Sets multiple CSS variables at once
   * @param variables Object mapping variable names to values
   * @throws CssVariablesError if any variable name or value is invalid
   */
  setMultiple(variables: CssVariablesMap): void {
    for (const [name, value] of Object.entries(variables)) {
      if (value !== undefined) {
        this.set(name, value);
      }
    }
  }

  /**
   * @notice Checks if a CSS variable is defined
   * @param variableName The name of the CSS variable
   * @returns True if the variable is defined
   * @throws CssVariablesError if variable name is invalid
   */
  has(variableName: string): boolean {
    const normalizedName = this.normalizeVariableName(variableName);
    CssVariableValidator.isValidVariableName(normalizedName);
    return this.get(normalizedName) !== '';
  }

  /**
   * @notice Normalizes a CSS variable name
   * @dev Ensures the variable name has the -- prefix
   * @param variableName The variable name to normalize
   * @returns Normalized variable name with -- prefix
   */
  private normalizeVariableName(variableName: string): string {
    const trimmed = variableName.trim();
    return trimmed.startsWith('--') ? trimmed : `--${trimmed}`;
  }
}

/**
 * @title CSS Variable Hooks for React
 * @notice React hooks for CSS variable operations
 */

/**
 * @notice Gets a CSS variable value as a React hook
 * @param variableName The name of the CSS variable
 * @param fallback Optional fallback value
 * @returns The computed value of the CSS variable
 */
export function useCssVariable(variableName: string, fallback?: string): string {
  if (typeof window === 'undefined') {
    return fallback || '';
  }

  const cssVars = new CssVariablesUsage();
  return cssVars.get(variableName, fallback);
}

/**
 * @title Helper Functions
 */

/**
 * @notice Creates a CSS var() expression safely
 * @param variableName The CSS variable name
 * @param fallback Optional fallback value
 * @returns A formatted CSS var() expression
 */
export type VarExpression = string;

export function cssVar<V extends AllowedCssVariable>(variableName: V, fallback?: string): VarExpression {
  const trimmed = (variableName as string).trim() as V;
  const normalizedName = trimmed.startsWith('--') ? trimmed : `--${trimmed}` as V;
export function cssVar(variableName: string, fallback?: string): string {
  // Validate the variable name
  const normalizedName = variableName.trim().startsWith('--')
    ? variableName.trim()
    : `--${variableName.trim()}`;
  const normalizedName = variableName.startsWith('--') ? variableName : `--${variableName}` as V;
  
  CssVariableValidator.isValidVariableName(normalizedName);

  if (fallback !== undefined) {
    return `var(${normalizedName}, ${fallback})` as VarExpression;
  }
  return `var(${normalizedName})` as VarExpression;
    return `var(${normalizedName}, ${fallback})`;
  }
  return `var(${normalizedName})` as VarExpression;
}

/**
 * @notice Creates a CSS calc() expression with CSS variables
 * @param expression The calc expression
 * @returns The formatted calc expression
 */
export function cssCalc(expression: string): string {
  // Basic validation - ensure no dangerous patterns
  CssVariableValidator.isValidValue(expression);
  return `calc(${expression})`;
}

/**
 * @notice Gets a CSS variable value specifically tailored for documentation components.
 * @dev NatSpec: This React hook wrapper simplifies extracting documentation-specific 
 * design tokens. It guarantees that documentation styling variables are securely validated, 
 * maintaining architectural separation and reducing component clutter.
 * @custom:security Variables fetched via this hook still pass through the core validator 
 * preventing rogue values or untested variables from breaking documentation layout.
 * @param variableName The documentation CSS variable name to fetch.
 * @param fallback Optional fallback value if the documentation token is undefined.
 * @returns The computed styling value.
 */
export function useDocsCssVariable(variableName: string, fallback?: string): string {
  return useCssVariable(variableName, fallback);
}

// SSR fallback map for server-side rendering (partial - extend as needed)
export const SSR_FALLBACKS: Partial<Record<AllowedCssVariable, string>> = {
  '--color-primary-blue': '#0066FF',
  '--space-4': '1rem',
  '--font-size-base': 'clamp(1rem, 0.95rem + 0.25vw, 1.125rem)',
  // Add more as needed from responsive.css
  '--color-neutral-100': '#FFFFFF',
  '--color-neutral-900': '#111827',
  '--transition-base': '250ms ease-in-out',
  '--radius-md': '0.5rem',
  '--shadow-md': '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
} as const;

// Default export for convenience
export default CssVariablesUsage;

// Default export for convenience
export default CssVariablesUsage;

