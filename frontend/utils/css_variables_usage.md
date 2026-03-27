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
