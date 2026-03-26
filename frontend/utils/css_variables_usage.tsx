/**
 * @title CSS Variables Usage Utility
 * @notice Secure utility for CSS custom properties (CSS variables) handling
 * @dev Provides type-safe access to design tokens with validation and sanitization
 * @author Stellar Raise Security Team
 * @notice SECURITY: All CSS variable access must go through this utility to prevent
 *         CSS injection attacks and ensure variable name validation.
 */

/**
 * @notice Design tokens mapped to CSS variables for the Stellar Raise DApp
 * @dev This object provides a central reference for all standard UI styles
 */
export const THEME = {
  colors: {
    /** @notice Primary brand color used for primary actions and highlights */
    primary: '--color-primary-blue',
    /** @notice Deep navy color used for headings and dark backgrounds */
    navy: '--color-deep-navy',
    /** @notice Semantic color for success states and positive indicators */
    success: '--color-success-green',
    /** @notice Semantic color for error states, destructive actions, and alerts */
    error: '--color-error-red',
    /** @notice Semantic color for warning states and cautionary messages */
    warning: '--color-warning-orange',
    /** @notice Lightest neutral color, used for backgrounds and text on dark areas */
    neutral100: '--color-neutral-100',
    /** @notice Very light neutral color for subtle backgrounds */
    neutral200: '--color-neutral-200',
    /** @notice Light neutral color for borders and dividers */
    neutral300: '--color-neutral-300',
    /** @notice Medium neutral color for secondary text and icons */
    neutral700: '--color-neutral-700',
    /** @notice Darkest neutral color for primary body text */
    neutral900: '--color-neutral-900',
  },
  spacing: {
    /** @notice 4px spacing unit */
    space1: '--space-1',
    /** @notice 8px spacing unit */
    space2: '--space-2',
    /** @notice 12px spacing unit */
    space3: '--space-3',
    /** @notice 16px spacing unit - Standard base padding/margin */
    space4: '--space-4',
    /** @notice 20px spacing unit */
    space5: '--space-5',
    /** @notice 24px spacing unit */
    space6: '--space-6',
    /** @notice 32px spacing unit */
    space8: '--space-8',
    /** @notice 40px spacing unit */
    space10: '--space-10',
    /** @notice 48px spacing unit */
    space12: '--space-12',
    /** @notice 64px spacing unit */
    space16: '--space-16',
  },
  typography: {
    /** @notice Main brand font family (Space Grotesk) */
    familyPrimary: '--font-family-primary',
    /** @notice Extra small font size (approx 12px) */
    sizeXs: '--font-size-xs',
    /** @notice Small font size (approx 14px) */
    sizeSm: '--font-size-sm',
    /** @notice Base font size (approx 16px) */
    sizeBase: '--font-size-base',
    /** @notice Large font size (approx 18px-24px) */
    sizeLg: '--font-size-lg',
    /** @notice Extra large font size (approx 20px-30px) */
    sizeXl: '--font-size-xl',
    /** @notice 2x large font size (approx 24px-36px) */
    size2xl: '--font-size-2xl',
    /** @notice 3x large font size (approx 30px-48px) */
    size3xl: '--font-size-3xl',
  },
  layout: {
    /** @notice Mobile breakpoint (768px) */
    breakpointMobile: '--breakpoint-mobile',
    /** @notice Tablet breakpoint (1024px) */
    breakpointTablet: '--breakpoint-tablet',
    /** @notice Minimum touch target size for accessibility (44px) */
    touchTargetMin: '--touch-target-min',
    /** @notice Mobile grid column count (4) */
    gridColumnsMobile: '--grid-columns-mobile',
    /** @notice Tablet grid column count (8) */
    gridColumnsTablet: '--grid-columns-tablet',
    /** @notice Desktop grid column count (12) */
    gridColumnsDesktop: '--grid-columns-desktop',
    /** @notice Mobile grid gutter size */
    gridGutterMobile: '--grid-gutter-mobile',
    /** @notice Tablet grid gutter size */
    gridGutterTablet: '--grid-gutter-tablet',
    /** @notice Desktop grid gutter size */
    gridGutterDesktop: '--grid-gutter-desktop',
    /** @notice Mobile container max width */
    containerMobile: '--container-mobile',
    /** @notice Tablet container max width */
    containerTablet: '--container-tablet',
    /** @notice Desktop container max width */
    containerDesktop: '--container-desktop',
  },
  zIndex: {
    /** @notice Base z-index layer (1) */
    base: '--z-base',
    /** @notice Dropdown menus and popovers (100) */
    dropdown: '--z-dropdown',
    /** @notice Sticky elements like headers (200) */
    sticky: '--z-sticky',
    /** @notice Fixed position elements (300) */
    fixed: '--z-fixed',
    /** @notice Backdrop for modals (400) */
    modalBackdrop: '--z-modal-backdrop',
    /** @notice Modal content (500) */
    modal: '--z-modal',
    /** @notice Toast notifications and alerts (600) */
    toast: '--z-toast',
  },
  effects: {
    /** @notice Fast transition duration (150ms) */
    transitionFast: '--transition-fast',
    /** @notice Standard base transition duration (250ms) */
    transitionBase: '--transition-base',
    /** @notice Slow transition duration (350ms) */
    transitionSlow: '--transition-slow',
    /** @notice Small border radius (4px) */
    radiusSm: '--radius-sm',
    /** @notice Medium border radius (8px) */
    radiusMd: '--radius-md',
    /** @notice Large border radius (12px) */
    radiusLg: '--radius-lg',
    /** @notice Extra large border radius (16px) */
    radiusXl: '--radius-xl',
    /** @notice Fully rounded radius for circles/pills */
    radiusFull: '--radius-full',
    /** @notice Small elevation shadow */
    shadowSm: '--shadow-sm',
    /** @notice Medium elevation shadow */
    shadowMd: '--shadow-md',
    /** @notice Large elevation shadow */
    shadowLg: '--shadow-lg',
    /** @notice Extra large elevation shadow */
    shadowXl: '--shadow-xl',
  },
  safeArea: {
    /** @notice Top safe area inset for notched devices */
    top: '--safe-area-inset-top',
    /** @notice Right safe area inset for notched devices */
    right: '--safe-area-inset-right',
    /** @notice Bottom safe area inset for notched devices */
    bottom: '--safe-area-inset-bottom',
    /** @notice Left safe area inset for notched devices */
    left: '--safe-area-inset-left',
  },
  docs: {
    /** @notice Documentation background color */
    bg: '--color-docs-bg',
    /** @notice Documentation text color */
    text: '--color-docs-text',
    /** @notice Documentation code font family */
    codeFont: '--font-docs-code',
  },
} as const;

/**
 * @notice Individual constants for grouped theme categories
 */
export const COLORS = THEME.colors;
export const SPACING = THEME.spacing;
export const TYPOGRAPHY = THEME.typography;
export const LAYOUT = THEME.layout;
export const Z_INDEX = THEME.zIndex;
export const EFFECTS = THEME.effects;
export const SAFE_AREA = THEME.safeArea;
export const DOCS = THEME.docs;

/**
 * @notice Predefined list of allowed CSS variable names
 * @dev Derived from the THEME object to ensure single source of truth
 */
export const ALLOWED_CSS_VARIABLES = [
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
 * @notice Type for allowed CSS variable names
 */
export type AllowedCssVariable = typeof ALLOWED_CSS_VARIABLES[number];

/**
 * @notice Regular expression for validating CSS variable names
 * @dev Ensures variable names start with -- and contain only valid characters
 */
const CSS_VAR_NAME_REGEX = /^--[a-zA-Z][a-zA-Z0-9-_]*$/;

/**
 * @notice Regular expression for detecting potentially malicious CSS values
 * @dev Blocks URL() references and expression() which could be used for attacks
 */
const DANGEROUS_CSS_PATTERN = /(?:url\s*\(|expression\s*|javascript:|data:text\/css|@import)/i;

/**
 * @notice Type for CSS variable value map
 */
export type CssVariablesMap = Partial<Record<AllowedCssVariable, string>>;

/**
 * @title CssVariablesError
 * @notice Custom error class for CSS variable related errors
 */
export class CssVariablesError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CssVariablesError';
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
  
  CssVariableValidator.isValidVariableName(normalizedName);

  if (fallback !== undefined) {
    return `var(${normalizedName}, ${fallback})` as VarExpression;
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

