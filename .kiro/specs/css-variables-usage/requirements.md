# Requirements Document

## Introduction

This feature delivers targeted updates to the CSS variables usage utility (`css_variables_usage.tsx`) in the Stellar Raise crowdfunding DApp frontend. The goals are:

1. **Gas efficiency** — reduce unnecessary re-computation of CSS variable lookups by introducing memoization and batch-read optimizations, minimizing redundant `getComputedStyle` calls.
2. **UX improvements** — add a reactive hook (`useCssVariableReactive`) that re-reads variables on theme changes, and extend the allowed variable set with documentation-specific tokens already referenced in the codebase.
3. **Security hardening** — tighten the dangerous-pattern regex and add logging bounds so that blocked injection attempts are observable without leaking sensitive data.
4. **Test coverage ≥ 95%** — comprehensive tests covering all new paths, edge cases, and security assumptions.
5. **Documentation** — NatSpec-style comments on all public APIs and an updated `css_variables_usage.md`.

The implementation lives entirely in the frontend layer (`frontend/utils/css_variables_usage.tsx`) and its companion test and documentation files. No Rust/Soroban contract changes are required.

---

## Glossary

- **CSS_Variables_Utility**: The module at `frontend/utils/css_variables_usage.tsx` that provides secure, validated access to CSS custom properties.
- **CssVariablesUsage**: The main class exported by the CSS_Variables_Utility.
- **CssVariableValidator**: The static validator class that enforces the whitelist and value-safety rules.
- **ALLOWED_CSS_VARIABLES**: The compile-time constant array of permitted CSS variable names derived from the `THEME` object.
- **THEME**: The design-token map exported by the CSS_Variables_Utility.
- **Docs_Token**: A CSS variable prefixed with `--color-docs-`, `--font-docs-`, or `--space-docs-` used exclusively by documentation components.
- **Memoization_Cache**: An in-memory `Map` keyed by `(element, variableName)` that stores the last-read CSS variable value to avoid redundant `getComputedStyle` calls.
- **Reactive_Hook**: The `useCssVariableReactive` React hook that subscribes to a `MutationObserver` on the target element and re-reads the variable when inline styles change.
- **Injection_Attempt**: Any call to `CssVariableValidator.isValidValue` or `CssVariableValidator.isValidVariableName` that triggers the dangerous-pattern guard.
- **Logging_Bounds**: A structured, sanitized log entry emitted (via `console.warn`) when an Injection_Attempt is detected, containing only the pattern category and a truncated, redacted representation of the offending input — never the raw value.
- **SSR**: Server-Side Rendering context where `window` and `document` are unavailable.

---

## Requirements

### Requirement 1: Docs Token Whitelist Extension

**User Story:** As a documentation component author, I want to use `--color-docs-*`, `--font-docs-*`, and `--space-docs-*` CSS variables through the validated utility, so that documentation pages share the same security guarantees as the rest of the DApp.

#### Acceptance Criteria

1. THE CSS_Variables_Utility SHALL include `--color-docs-bg`, `--color-docs-text`, `--color-docs-link`, `--color-docs-accent`, `--font-docs-code`, and `--space-docs-content` in `ALLOWED_CSS_VARIABLES`.
2. WHEN `CssVariableValidator.isValidVariableName` is called with any Docs_Token listed in criterion 1, THE CssVariableValidator SHALL return `true` without throwing.
3. WHEN `CssVariableValidator.isValidVariableName` is called with a Docs_Token not in the whitelist (e.g., `--color-docs-unknown`), THE CssVariableValidator SHALL throw a `CssVariablesError`.
4. THE CSS_Variables_Utility SHALL export a `DOCS_THEME` constant object mapping semantic names to the six Docs_Token variable names.
5. THE CSS_Variables_Utility SHALL derive `ALLOWED_CSS_VARIABLES` from the union of `THEME` and `DOCS_THEME` values, maintaining a single source of truth.

---

### Requirement 2: Memoized Variable Reads for Gas Efficiency

**User Story:** As a frontend developer, I want CSS variable reads to be memoized per element so that repeated calls within the same render cycle do not trigger redundant `getComputedStyle` invocations, reducing layout-thrashing and improving rendering performance.

#### Acceptance Criteria

1. THE CssVariablesUsage SHALL maintain a Memoization_Cache that stores the last-read value for each `(element, variableName)` pair.
2. WHEN `CssVariablesUsage.get` is called for a variable that is already in the Memoization_Cache for the same element, THE CssVariablesUsage SHALL return the cached value without calling `getComputedStyle`.
3. WHEN `CssVariablesUsage.set` is called for a variable, THE CssVariablesUsage SHALL invalidate the Memoization_Cache entry for that `(element, variableName)` pair.
4. WHEN `CssVariablesUsage.remove` is called for a variable, THE CssVariablesUsage SHALL invalidate the Memoization_Cache entry for that `(element, variableName)` pair.
5. THE CssVariablesUsage SHALL expose a `clearCache(): void` method that empties the entire Memoization_Cache for the instance.
6. WHEN `CssVariablesUsage.getMultiple` is called, THE CssVariablesUsage SHALL use the Memoization_Cache for each variable in the batch, calling `getComputedStyle` at most once per unique element per batch.
7. IF the Memoization_Cache grows beyond 200 entries for a single instance, THEN THE CssVariablesUsage SHALL evict the oldest 50 entries (LRU-style) to bound memory usage.

---

### Requirement 3: Reactive CSS Variable Hook

**User Story:** As a React component author, I want a hook that automatically re-renders my component when a CSS variable value changes at runtime (e.g., on theme switch), so that the UI stays in sync without manual refresh.

#### Acceptance Criteria

1. THE CSS_Variables_Utility SHALL export a `useCssVariableReactive(variableName: string, fallback?: string): string` hook.
2. WHEN `useCssVariableReactive` is called in a browser context, THE Reactive_Hook SHALL read the current value of the CSS variable from `document.documentElement` and return it.
3. WHEN the inline style of `document.documentElement` changes (detected via `MutationObserver` on the `style` attribute), THE Reactive_Hook SHALL re-read the variable and trigger a React state update if the value has changed.
4. WHEN `useCssVariableReactive` is called in an SSR context (window is undefined), THE Reactive_Hook SHALL return the fallback value without attempting DOM access.
5. WHEN the component using `useCssVariableReactive` unmounts, THE Reactive_Hook SHALL disconnect the `MutationObserver` to prevent memory leaks.
6. IF `variableName` is not in `ALLOWED_CSS_VARIABLES`, THEN THE Reactive_Hook SHALL throw a `CssVariablesError` before mounting the observer.

---

### Requirement 4: Logging Bounds for Injection Attempts

**User Story:** As a security engineer, I want blocked CSS injection attempts to emit a structured, sanitized warning log so that I can detect and investigate attack patterns without exposing raw malicious input in logs.

#### Acceptance Criteria

1. WHEN `CssVariableValidator.isValidValue` detects a dangerous pattern, THE CssVariableValidator SHALL emit a `console.warn` log entry containing the pattern category (e.g., `"url-injection"`, `"js-protocol"`, `"css-expression"`, `"import-injection"`, `"data-url"`) and a redacted, truncated representation of the input (maximum 40 characters, with the matched segment replaced by `[REDACTED]`).
2. WHEN `CssVariableValidator.isValidVariableName` detects a variable name not in the whitelist, THE CssVariableValidator SHALL emit a `console.warn` log entry containing the string `"unknown-variable"` and the first 40 characters of the variable name.
3. THE CssVariableValidator SHALL NOT include the full raw value of a blocked input in any log entry.
4. THE CSS_Variables_Utility SHALL export a `setLoggingEnabled(enabled: boolean): void` function that allows callers to suppress logging (e.g., during tests).
5. WHEN `setLoggingEnabled(false)` has been called, THE CssVariableValidator SHALL not emit any `console.warn` entries for blocked inputs.
6. THE CSS_Variables_Utility SHALL default to logging enabled (`true`) on module load.

---

### Requirement 5: Comprehensive Test Coverage

**User Story:** As a developer maintaining the CSS variables utility, I want a test suite with at least 95% coverage across all branches, so that regressions are caught before they reach production.

#### Acceptance Criteria

1. THE test suite (`css_variables_usage.test.tsx`) SHALL achieve a minimum of 95% statement, branch, function, and line coverage as reported by Jest with `--coverage`.
2. THE test suite SHALL include tests for all six Docs_Token variables added in Requirement 1.
3. THE test suite SHALL include tests verifying that the Memoization_Cache is populated on first read and returns cached values on subsequent reads (Requirement 2).
4. THE test suite SHALL include tests verifying that `set` and `remove` invalidate the Memoization_Cache (Requirement 2).
5. THE test suite SHALL include tests verifying that `clearCache` empties the cache (Requirement 2).
6. THE test suite SHALL include tests verifying that `useCssVariableReactive` returns the correct value in a browser-like environment and the fallback in SSR (Requirement 3).
7. THE test suite SHALL include tests verifying that `useCssVariableReactive` disconnects its `MutationObserver` on unmount (Requirement 3).
8. THE test suite SHALL include tests verifying that each of the five injection-pattern categories triggers a `console.warn` with the correct category label (Requirement 4).
9. THE test suite SHALL include tests verifying that `setLoggingEnabled(false)` suppresses `console.warn` output (Requirement 4).
10. FOR ALL valid `AllowedCssVariable` values `v`, calling `cssVar(v)` SHALL return a string of the form `var(v)` (round-trip property).
11. FOR ALL valid `AllowedCssVariable` values `v`, calling `CssVariableValidator.isValidVariableName(v)` SHALL return `true` (invariant: whitelist is self-consistent).

---

### Requirement 6: Documentation and NatSpec Comments

**User Story:** As a contributor to the Stellar Raise project, I want all public APIs in the CSS variables utility to have NatSpec-style comments and an updated markdown documentation file, so that I can understand the purpose, parameters, and security assumptions of each function without reading the implementation.

#### Acceptance Criteria

1. THE CSS_Variables_Utility SHALL include NatSpec-style `@notice`, `@param`, `@returns`, and `@dev` comments on every exported function, class, and constant.
2. THE CSS_Variables_Utility SHALL include a `@custom:security` tag on any function that enforces a security invariant (validation, sanitization, or logging).
3. THE documentation file (`css_variables_usage.md`) SHALL document the `DOCS_THEME` constant, the `useCssVariableReactive` hook, the `clearCache` method, and the `setLoggingEnabled` function.
4. THE documentation file SHALL include a "Security Assumptions" section listing all attack vectors blocked by the utility and the mechanism used to block each one.
5. THE documentation file SHALL include a "Performance Notes" section describing the Memoization_Cache strategy and its memory bounds.
