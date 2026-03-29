# Documentation for CSS Variables Usage Refactoring

## Overview
The CSS variables usage system correctly manages design tokens within `stellar-raise-contracts`. In Issue #311, the architecture was explicitly refactored and expanded specifically for the Documentation framework to offer a clean developer-experience when integrating custom design systems for technical pages.

## Hooks Refactoring
The original generic hook (`useCssVariable`) has been augmented with an explicitly tailored `useDocsCssVariable`.

### `useDocsCssVariable(variableName: string, fallback?: string): string`
This React hook allows you to cleanly resolve documentation-specific UI variables directly from your Documentation components without cross-contaminating state logic with complex JS evaluation.

**Usage:**
```tsx
import { useDocsCssVariable } from '../utils/css_variables_usage';

export const DocumentationCodeSnippets = () => {
    const codeBackground = useDocsCssVariable('--color-docs-bg', '#FAFAFA');
    const codeAccent = useDocsCssVariable('--color-docs-accent', '#FF0000');

    return (
        <pre style={{ backgroundColor: codeBackground, borderLeft: `2px solid ${codeAccent}`}}>
            <code>...</code>
        </pre>
    );
};
```

## Supported Variables
The underlying validator (`CssVariableValidator`) natively tracks allowed documentation tokens representing the UX guidelines. The new keys are:
- `--color-docs-bg`
- `--color-docs-text`
- `--color-docs-link`
- `--color-docs-accent`
- `--font-docs-code`
- `--space-docs-content`

## Security Assumptions
1. **CSS Injection Prevention**: Accessing tokens leverages the central `CssVariableValidator` which completely denies expressions yielding strings such as `url()` or `javascript:`, mitigating arbitrary executions mapping directly to rendered tokens.
2. **Whitelist Policy**: Using `useDocsCssVariable` checks input tokens explicitly against `ALLOWED_CSS_VARIABLES`, ensuring no runtime anomalies from typos.
3. **Efficiency**: Native CSS Variable compilation defers heavily to browser native methods (`getComputedStyle`), offering strict client-performance boosts over JS tracking systems.
