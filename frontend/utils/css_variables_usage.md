# CSS Variables Usage "Contract"

The `CSSVariablesUsage` utility acts as the single source of truth for design tokens used across the Stellar Raise platform's frontend. It creates a robust link between the CSS variables defined in `utilities.css` and the logic used in our components and tests.

## Rationale

Design tokens like colors, spacing, and typography are often defined as CSS variables. However, using these variables directly in TypeScript/JavaScript can be flaky if names change or variables go missing. By defining a "contract" for these variables:
- **Consistency**: All components use the same approved platform colors and spacing.
- **Reliability**: Any breaking change in a CSS variable name can be detected at the compilation or test level.
- **Readability**: Using `DESIGN_TOKENS.COLORS.PRIMARY_BLUE` is more readable and less error-prone than hardcoding hex codes.

## Features

- **Queryable Design Tokens**: Access colors, spacing, fonts, and radii directly in your TypeScript logic.
- **CSS Variable Generators**: Use `CSSVariablesContract.getVar()` to safely generate `var()` strings for inline styles or styled-components.
- **Unit Validation**: Use `isApprovedColor()` to validate if a dynamic color string is within the platform's approved design palette.
- **Pixel Conversion**: Handles `0.25rem` to `4px` conversions for logic that requires absolute measurements.

## Usage Example

```tsx
import { CSSVariablesContract, DESIGN_TOKENS } from '../utils/css_variables_usage';

const MyComponent = () => (
  <div style={{ padding: CSSVariablesContract.getVar('SPACING', 'SPACE_4') }}>
    <h1 style={{ color: DESIGN_TOKENS.COLORS.PRIMARY_BLUE }}>
      Welcome to Stellar Raise
    </h1>
  </div>
);
```

## Security Assumptions

1. **Approved Design Only**: The `isApprovedColor` method protects against injecting unapproved colors into components that might have security-critical branding (like "Verified Campaign" badges).
2. **Immutable Constants**: All `DESIGN_TOKENS` are marked as `as const` to prevent mutation at runtime.
# CSS Variables Usage Documentation

The `CSSVariablesUsage` utility acts as the single source of truth for design tokens used across the Stellar Raise platform's frontend. It creates a robust link between the CSS variables defined in `utilities.css` and the logic used in our components and tests.

## Rationale

Design tokens like colors, spacing, and typography are often defined as CSS variables. However, using these variables directly in TypeScript/JavaScript can be flaky if names change or variables go missing. By defining a "contract" for these variables:
- **Consistency**: All components use the same approved platform colors and spacing.
- **Reliability**: Any breaking change in a CSS variable name can be detected at the compilation or test level.
- **Readability**: Using `DESIGN_TOKENS.COLORS.PRIMARY_BLUE` is more readable and less error-prone than hardcoding hex codes.

## Features

- **Queryable Design Tokens**: Access colors, spacing, fonts, and radii directly in your TypeScript logic.
- **CSS Variable Generators**: Use `CSSVariablesContract.getVar()` to safely generate `var()` strings for inline styles or styled-components.
- **Unit Validation**: Use `isApprovedColor()` to validate if a dynamic color string is within the platform's approved design palette.
- **Pixel Conversion**: Handles `0.25rem` to `4px` conversions for logic that requires absolute measurements.

## Usage Example

```tsx
import { CSSVariablesContract, DESIGN_TOKENS } from '../utils/css_variables_usage';

const MyComponent = () => (
  <div style={{ padding: CSSVariablesContract.getVar('SPACING', 'SPACE_4') }}>
    <h1 style={{ color: DESIGN_TOKENS.COLORS.PRIMARY_BLUE }}>
      Welcome to Stellar Raise
    </h1>
  </div>
);
```

## Security Assumptions

1.  **Whitelist Enforcement**: All variables in `THEME` are automatically whitelisted.
2.  **Code Completion**: IDEs provide autocomplete for all available style tokens.
3.  **Documentation**: Every variable in `THEME` includes NatSpec comments explaining its intended use.

## Security Considerations

### Why Secure CSS Variable Handling?

CSS variables, while powerful, can be a vector for security vulnerabilities if not handled properly:

1. **CSS Injection Attacks**: Malicious values injected into CSS can lead to:
   - `expression()` attacks in Internet Explorer
   - `url()` injection for loading malicious resources
   - `javascript:` pseudo-protocol in some contexts

2. **Data Exfiltration**: CSS can be used to exfiltrate sensitive data via:
   - Attribute selectors targeting sensitive patterns
   - `background-image` with encoded data URLs

3. **Denial of Service**: Complex CSS expressions can cause browser crashes

### Our Security Measures

1. **Whitelist Validation**: Only predefined, approved CSS variables can be accessed
2. **Value Sanitization**: Dangerous patterns are blocked before values are applied
3. **Type Safety**: TypeScript types prevent invalid usage at compile time
4. **Fallback Values**: Safe defaults prevent undefined variable issues

## API Reference

### Constants

- **`THEME`**: The main object containing all design tokens.
- **`COLORS`**: Shortcut to `THEME.colors`.
- **`SPACING`**: Shortcut to `THEME.spacing`.
- **`TYPOGRAPHY`**: Shortcut to `THEME.typography`.
- **`LAYOUT`**: Shortcut to `THEME.layout`.
- **`Z_INDEX`**: Shortcut to `THEME.zIndex`.
- **`EFFECTS`**: Shortcut to `THEME.effects`.
- **`SAFE_AREA`**: Shortcut to `THEME.safeArea`.
- **`DOCS`**: Shortcut to `THEME.docs`.

### Classes

#### `CssVariablesUsage`

Main utility class for CSS variable operations.

```typescript
import { CssVariablesUsage } from './css_variables_usage';

const cssVars = new CssVariablesUsage();
```

**Constructor**

```typescript
constructor(element?: HTMLElement)
```

Creates a new instance. Defaults to `document.documentElement` if no element provided.

**Methods**

##### `get(variableName: string, fallback?: string): string`

Gets a CSS variable value.

```typescript
const color = cssVars.get('--color-primary-blue');
const fontSize = cssVars.get('--font-size-base', '1rem');
```

##### `set(variableName: string, value: string): void`

Sets a CSS variable value.

```typescript
cssVars.set('--color-primary-blue', '#0066FF');
```

##### `remove(variableName: string): void`

Removes a CSS variable.

```typescript
cssVars.remove('--color-primary-blue');
```

##### `has(variableName: string): boolean`

Checks if a variable is defined.

```typescript
if (cssVars.has('--color-primary-blue')) {
  // Variable exists
}
```

##### `getMultiple(variableNames: string[], fallback?: string): CssVariablesMap`

Gets multiple variables at once.

```typescript
const values = cssVars.getMultiple(['--color-primary', '--space-4']);
```

##### `setMultiple(variables: CssVariablesMap): void`

Sets multiple variables at once.

```typescript
cssVars.setMultiple({
  '--color-primary': '#0066FF',
  '--space-4': '1rem',
});
```

#### `CssVariableValidator`

Static validator class for CSS variable operations.

```typescript
import { CssVariableValidator } from './css_variables_usage';
```

**Methods**

##### `isValidVariableName(variableName: string): boolean`

Validates that a variable name is in the allowed list.

```typescript
CssVariableValidator.isValidVariableName('--color-primary'); // true
CssVariableValidator.isValidVariableName('--custom'); // throws CssVariablesError
```

##### `isValidValue(value: string): boolean`

Validates that a CSS value is safe.

```typescript
CssVariableValidator.isValidValue('#0066FF'); // true
CssVariableValidator.isValidValue('url(https://evil.com)'); // throws
```

##### `getAllowedVariables(): readonly string[]`

Returns all allowed CSS variable names.

```typescript
const allowed = CssVariableValidator.getAllowedVariables();
```

### Helper Functions

#### `cssVar(variableName: string, fallback?: string): string`

Creates a CSS `var()` expression safely.

```typescript
const cssExpression = cssVar('color-primary-blue', '#ffffff');
// Returns: 'var(--color-primary-blue, #ffffff)'
```

#### `cssCalc(expression: string): string`

Creates a CSS `calc()` expression safely.

```typescript
const calcExpression = cssCalc('100% - var(--space-4)');
// Returns: 'calc(100% - var(--space-4))'
```

### Types

```typescript
type AllowedCssVariable = 
  | '--breakpoint-mobile'
  | '--breakpoint-tablet'
  | '--color-primary-blue'
  // ... and 55+ more predefined variables

type CssVariablesMap = Partial<Record<AllowedCssVariable, string>>;
```

### Exceptions

#### `CssVariablesError`

Thrown when invalid CSS variable names or values are detected.

```typescript
try {
  CssVariableValidator.isValidVariableName('--invalid');
} catch (e) {
  if (e instanceof CssVariablesError) {
    console.error(e.message);
  }
}
```

## Allowed CSS Variables

### Colors

| Variable | Default Value | Description |
|----------|---------------|-------------|
| `--color-primary-blue` | `#0066FF` | Primary brand color |
| `--color-deep-navy` | `#0A1929` | Deep navy accent |
| `--color-success-green` | `#00C853` | Success states |
| `--color-error-red` | `#FF3B30` | Error states |
| `--color-warning-orange` | `#FF9500` | Warning states |
| `--color-neutral-100` | `#FFFFFF` | Lightest neutral |
| `--color-neutral-200` | `#F5F7FA` | Light neutral |
| `--color-neutral-300` | `#E4E7EB` | Medium neutral |
| `--color-neutral-700` | `#374151` | Dark neutral |
| `--color-neutral-900` | `#111827` | Darkest neutral |

### Typography

| Variable | Description |
|----------|-------------|
| `--font-family-primary` | Primary font family |
| `--font-size-xs` | Extra small text |
| `--font-size-sm` | Small text |
| `--font-size-base` | Base text size |
| `--font-size-lg` | Large text |
| `--font-size-xl` | Extra large text |
| `--font-size-2xl` | 2x large text |
| `--font-size-3xl` | 3x large text |

### Spacing

| Variable | Value | Equivalent |
|----------|-------|-----------|
| `--space-1` | `0.25rem` | 4px |
| `--space-2` | `0.5rem` | 8px |
| `--space-3` | `0.75rem` | 12px |
| `--space-4` | `1rem` | 16px |
| `--space-5` | `1.25rem` | 20px |
| `--space-6` | `1.5rem` | 24px |
| `--space-8` | `2rem` | 32px |
| `--space-10` | `2.5rem` | 40px |
| `--space-12` | `3rem` | 48px |
| `--space-16` | `4rem` | 64px |

### Z-Index Scale

| Variable | Value | Use Case |
|----------|-------|----------|
| `--z-base` | 1 | Base layer |
| `--z-dropdown` | 100 | Dropdowns |
| `--z-sticky` | 200 | Sticky headers |
| `--z-fixed` | 300 | Fixed elements |
| `--z-modal-backdrop` | 400 | Modal overlays |
| `--z-modal` | 500 | Modal content |
| `--z-toast` | 600 | Toast notifications |

### Transitions

| Variable | Value |
|----------|-------|
| `--transition-fast` | `150ms ease-in-out` |
| `--transition-base` | `250ms ease-in-out` |
| `--transition-slow` | `350ms ease-in-out` |

### Border Radius

| Variable | Value |
|----------|-------|
| `--radius-sm` | `0.25rem` |
| `--radius-md` | `0.5rem` |
| `--radius-lg` | `0.75rem` |
| `--radius-xl` | `1rem` |
| `--radius-full` | `9999px` |

### Shadows

| Variable | Description |
|----------|-------------|
| `--shadow-sm` | Small shadow |
| `--shadow-md` | Medium shadow |
| `--shadow-lg` | Large shadow |
| `--shadow-xl` | Extra large shadow |

### Safe Area Insets

| Variable | Description |
|----------|-------------|
| `--safe-area-inset-top` | Top safe area |
| `--safe-area-inset-right` | Right safe area |
| `--safe-area-inset-bottom` | Bottom safe area |
| `--safe-area-inset-left` | Left safe area |

### Documentation

| Variable | Description |
|----------|-------------|
| `--color-docs-bg` | Documentation background color |
| `--color-docs-text` | Documentation text color |
| `--font-docs-code` | Documentation code font family |

## Usage Examples

### Basic Usage

```typescript
import { CssVariablesUsage, THEME } from './css_variables_usage';

const cssVars = new CssVariablesUsage();

// Get a value using the THEME object for type safety
const primaryColor = cssVars.get(THEME.colors.primary);

// Set a value
cssVars.set(THEME.colors.primary, '#ff0000');

// Check if defined
if (cssVars.has(THEME.colors.primary)) {
  // ...
}
```

### React Integration

```typescript
import { useCssVariable, THEME } from './css_variables_usage';

function MyComponent() {
  const primaryColor = useCssVariable(THEME.colors.primary, '#0066FF');
  
  return (
    <div style={{ color: primaryColor }}>
      Styled with CSS variable
    </div>
  );
}
```

### Safe CSS Expression Creation

```typescript
import { cssVar, cssCalc } from './css_variables_usage';

// Safe var() creation
const expression = cssVar('space-4', '1rem');

// Safe calc() creation
const calcExpression = cssCalc('100% - var(--space-4) * 2');

// Use in style
element.style.width = calcExpression;
```

## Security Notes

### What Is Blocked

The following patterns are blocked to prevent security vulnerabilities:

1. **URL() Functions**: `url()`, `url('https://...')`, `url("data:...")`
2. **JavaScript URLs**: `javascript:alert()`
3. **CSS Expressions**: `expression(alert())` (IE compatibility)
4. **@import with URLs**: `@import url(...)`
5. **Data URLs**: `data:text/css,...`

### Best Practices

1. **Always use the utility**: Don't access CSS variables directly
2. **Provide fallbacks**: Always include fallback values when appropriate
3. **Use TypeScript**: Leverage type safety for variable names
4. **Validate early**: Check variable existence before use

## Testing

Run tests with:

```bash
npm test -- --testPathPatterns="css_variables_usage" --coverage
```

### Test Coverage

Current test coverage: **98.57%**
Current test coverage: **95%+** with caching and typed cssVar tests.

| Metric | Coverage |
|--------|----------|
| Statements | 98.57% |
| Branches | 86.95% |
| Functions | 100% |
| Lines | 98.57% |

### Test Structure

- **CssVariableValidator**: Tests for validation logic
- **CssVariablesUsage**: Tests for get/set/remove operations
- **Helper Functions**: Tests for cssVar() and cssCalc() utilities
- **Security Edge Cases**: Tests for attack vector prevention
- **setMultiple**: Tests for batch operations
- **useCssVariable**: Tests for React hook functionality

## Maintenance

### Adding New CSS Variables

1. Add the variable to `ALLOWED_CSS_VARIABLES` in `css_variables_usage.tsx`
2. Define the variable in `frontend/styles/responsive.css`
3. Add tests for the new variable
4. Update this documentation

### Updating Security Rules

If new attack vectors are discovered:

1. Update `DANGEROUS_CSS_PATTERN` regex in `css_variables_usage.tsx`
2. Add corresponding test cases
3. Document the new security measure

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.1.0 | 2026-03-23 | Enhanced test coverage, added setMultiple tests, updated documentation |
| 1.0.0 | 2026-03-23 | Initial implementation with whitelist validation |

## License

Part of the Stellar Raise project - see LICENSE file for details.
1. **Approved Design Only**: The `isApprovedColor` method protects against injecting unapproved colors into components that might have security-critical branding (like "Verified Campaign" badges).
2. **Immutable Constants**: All `DESIGN_TOKENS` are marked as `as const` to prevent mutation at runtime.
