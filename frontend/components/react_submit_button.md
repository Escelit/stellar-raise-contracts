# ReactSubmitButton

A typed React submit button with a strict state machine, safe label handling, double-submit prevention, and ARIA accessibility semantics.
# SubmitButton Component
# React Submit Button Component States

Addresses [GitHub Issue #359](https://github.com/Crowdfunding-DApp/stellar-raise-contracts/issues/359).

A reusable, accessible React submit button with a strict state machine, safe label handling, and double-submit prevention for crowdfunding transaction flows.
# React Submit Button — Dependencies

Documents the runtime dependencies, peer requirements, dev toolchain, and upgrade guidance for `react_submit_button.tsx`.

---

## Runtime Dependencies

| File | Purpose |
|------|---------|
| `react_submit_button.tsx` | Component and pure helper exports |
| `react_submit_button.test.tsx` | Test suite (≥ 95% coverage) |
| `react_submit_button.md` | This document |

---

## States

| State        | Description                                      | Clickable |
|--------------|--------------------------------------------------|-----------|
| `idle`       | Default — ready to submit                        | ✅        |
| `submitting` | Async action in-flight; blocks interaction       | ❌        |
| `success`    | Action confirmed                                 | ✅        |
| `error`      | Action failed; user can retry                    | ✅        |
| `disabled`   | Externally locked (deadline passed, goal met…)   | ❌        |

### Allowed transitions

```
idle        → submitting | disabled
submitting  → success | error | disabled
success     → idle | disabled
error       → idle | submitting | disabled
disabled    → idle
```

Same-state updates are always allowed (idempotent).

---

## Props

| Prop                | Type                                              | Default      | Description                                              |
|---------------------|---------------------------------------------------|--------------|----------------------------------------------------------|
| `state`             | `SubmitButtonState`                               | —            | Current button state (required)                          |
| `previousState`     | `SubmitButtonState`                               | `undefined`  | Previous state for strict transition validation          |
| `strictTransitions` | `boolean`                                         | `true`       | Falls back to `previousState` on invalid transitions     |
| `labels`            | `SubmitButtonLabels`                              | `undefined`  | Per-state label overrides                                |
| `onClick`           | `(e: MouseEvent) => void \| Promise<void>`        | `undefined`  | Click handler; blocked while submitting/disabled         |
| `className`         | `string`                                          | `undefined`  | Additional CSS class                                     |
| `id`                | `string`                                          | `undefined`  | HTML `id` attribute                                      |
| `type`              | `"button" \| "submit" \| "reset"`                 | `"button"`   | HTML button type                                         |
| `disabled`          | `boolean`                                         | `undefined`  | External disabled override                               |

---

## Default labels

| State        | Label             |
|--------------|-------------------|
| `idle`       | `Submit`          |
| `submitting` | `Submitting...`   |
| `success`    | `Submitted`       |
| `error`      | `Try Again`       |
| `disabled`   | `Submit Disabled` |
The button moves through a deterministic state machine:
## State Machine

```
idle ──────────────────► submitting ──► success ──► idle
  │                          │                       │
  └──► disabled ◄────────────┘◄──── error ◄──────────┘
                                       │
                                       └──► submitting (retry)
```

| State | Visual | Interaction | `disabled` | `aria-busy` |
|-------|--------|-------------|------------|-------------|
| `idle` | Indigo | Clickable | No | No |
| `submitting` | Light indigo | Blocked | Yes | Yes |
| `success` | Green | Blocked | No | No |
| `error` | Red | Clickable (retry) | No | No |
| `disabled` | Grey | Blocked | Yes | No |
The component has **zero production dependencies beyond React itself**.

| Package | Version range | Role | Justification |
|---------|--------------|------|---------------|
| `react` | `^19.0.0` | Peer — JSX, hooks (`useState`) | Required for component rendering |
| `react-dom` | `^19.0.0` | Peer — DOM rendering | Required by the consuming app; not imported directly |

No third-party UI libraries, animation libraries, or utility packages are imported. This keeps the bundle contribution of this component to a minimum and eliminates transitive dependency risk.

---

## Dev / Test Dependencies

| Package | Version range | Role |
|---------|--------------|------|
| `@testing-library/react` | `^16.0.0` | `render`, `screen`, `fireEvent`, `act` |
| `@testing-library/jest-dom` | `^6.0.0` | Extended matchers (used via `jest.setup.ts`) |
| `@testing-library/user-event` | `^14.0.0` | Available for future interaction tests |
| `jest` | `^30.0.0` | Test runner |
| `jest-environment-jsdom` | `^30.0.0` | DOM environment for React tests |
| `ts-jest` | `^29.0.0` | TypeScript transform for Jest |
| `typescript` | `^5.0.0` | Type checking |
| `@types/react` | `^19.0.0` | React type definitions |
| `@types/jest` | `^30.0.0` | Jest type definitions |

All dev dependencies are declared in the workspace `package.json` and are not shipped to production.

---

## Peer Dependency Requirements

```tsx
import ReactSubmitButton from "./react_submit_button";

// Basic
<ReactSubmitButton state="idle" onClick={handleSubmit} />

// With custom labels
<ReactSubmitButton
  state={txState}
  previousState={prevTxState}
  labels={{ idle: "Fund Campaign", submitting: "Funding...", success: "Funded!" }}
  onClick={handleContribute}
/>

// Externally disabled (e.g. campaign deadline passed)
<ReactSubmitButton state="disabled" labels={{ disabled: "Campaign Ended" }} />
import SubmitButton from "../components/react_submit_button";
import ReactSubmitButton from "../components/react_submit_button";

// Basic
<ReactSubmitButton state={txState} onClick={handleSubmit} />

// With all options
<ReactSubmitButton
  state={txState}
  previousState={prevState}
  strictTransitions
  labels={{ idle: "Fund Campaign", submitting: "Funding..." }}
  onClick={handleContribute}
  type="submit"
  id="contribute-btn"
  className="my-btn"
/>
### React 19

The component uses:

- `useState` — local in-flight submission guard
- `React.MouseEvent<HTMLButtonElement>` — typed click handler
- `React.CSSProperties` — inline style typing
- `react-jsx` transform — no explicit `React` import needed at call sites

React 18 is also compatible. The only React 19 feature used is the `react-jsx` automatic JSX transform, which was introduced in React 17. **Minimum supported React version: 17**.

### TypeScript 4.7+

The component uses:

- Const type parameters (TypeScript 5.0) — not used; compatible with TS 4.7+
- Template literal types — not used
- Strict union types and `Record<K, V>` — available since TS 4.1

**Minimum supported TypeScript version: 4.7**.

---

## What the Component Does NOT Depend On

| Excluded dependency | Reason |
|--------------------|--------|
| `classnames` / `clsx` | Class logic is a single optional prop passthrough |
| `styled-components` / `emotion` | Styles are inline `React.CSSProperties` constants |
| `framer-motion` / `react-spring` | Transitions use CSS `transition` property only |
| `react-hook-form` | Component is form-library agnostic |
| `zustand` / `redux` | State is fully controlled by the parent via the `state` prop |
| `lodash` | No utility functions needed |
| `uuid` | No ID generation needed |

Keeping the dependency surface minimal reduces:
- Bundle size impact on the consuming app
- Supply-chain attack surface
- Version conflict risk in monorepos

---

## Dependency Graph

```
react_submit_button.tsx
└── react          (peer, runtime)
    └── react-dom  (peer, runtime — consuming app only)

react_submit_button.test.tsx
├── react                        (peer)
├── @testing-library/react       (dev)
├── @testing-library/jest-dom    (dev, via jest.setup.ts)
└── react_submit_button.tsx      (local)
```

---

## Exported helpers

All pure functions are exported for independent unit testing.

| Function                              | Purpose                                                        |
|---------------------------------------|----------------------------------------------------------------|
| `normalizeSubmitButtonLabel`          | Sanitizes a label: strips control chars, truncates to 80 chars |
| `resolveSubmitButtonLabel`            | Returns the safe label for a given state                       |
| `isValidSubmitButtonStateTransition`  | Validates a `from → to` state transition                       |
| `resolveSafeSubmitButtonState`        | Enforces strict transitions, falls back to `previousState`     |
| `isSubmitButtonInteractionBlocked`    | Returns `true` when clicks must be suppressed                  |
| `isSubmitButtonBusy`                  | Returns `true` when `aria-busy` should be set                  |
| `ALLOWED_TRANSITIONS`                 | Transition map (shared by component and tests)                 |

---

## Security assumptions

- **No `dangerouslySetInnerHTML`** — labels are rendered as React text nodes only.
- **Label sanitization** — control characters (`U+0000–U+001F`, `U+007F`) are stripped; labels are truncated to 80 characters to prevent layout abuse.
- **Double-submit prevention** — an internal `isLocallySubmitting` flag blocks re-entry while an async `onClick` is in-flight, preventing duplicate blockchain transactions.
- **Hardcoded styles** — all CSS values are compile-time constants; no dynamic style injection from user input.
- **Input validation is the caller's responsibility** — the component surfaces state only; it never submits data itself.

---

## Accessibility

- `aria-live="polite"` — state label changes are announced to screen readers.
- `aria-busy` — set to `true` while submitting.
- `aria-label` — always set to the resolved, sanitized label.
- `disabled` — set on the HTML element when interaction is blocked, preventing keyboard activation.

---

## Tests

```
frontend/components/react_submit_button.test.tsx
```

51 tests covering:
- Label normalization and sanitization edge cases
- Default and custom label resolution per state
- State transition validation (allowed, blocked, idempotent)
- Strict transition enforcement and fallback
- Interaction blocking (submitting, disabled, external flag, local in-flight)
- `aria-busy` / `aria-live` / `aria-label` attributes
- Click handler: idle, error (retry), blocked states, async, rejected promise
- Rendering: element type, `data-state`, `type`, `className`, `id`
## Props
## Version Pinning Policy

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `state` | `SubmitButtonState` | required | Current button state |
| `previousState` | `SubmitButtonState` | — | Used for strict transition validation |
| `strictTransitions` | `boolean` | `true` | Rejects invalid state jumps |
| `labels` | `SubmitButtonLabels` | — | Per-state label overrides |
| `onClick` | `(e) => void \| Promise<void>` | — | Click handler; blocked while submitting/disabled |
| `className` | `string` | — | Additional CSS class |
| `id` | `string` | — | HTML id attribute |
| `type` | `"button" \| "submit" \| "reset"` | `"button"` | HTML button type |
| `disabled` | `boolean` | — | External disabled override |

---

## Exported Helpers

All pure functions are exported for direct unit testing:

| Function | Description |
|----------|-------------|
| `normalizeSubmitButtonLabel(candidate, fallback)` | Sanitizes a label: rejects non-strings, strips control chars, normalizes whitespace, truncates to 80 chars |
| `resolveSubmitButtonLabel(state, labels?)` | Returns a safe label for the given state |
| `isValidSubmitButtonStateTransition(from, to)` | Returns true when the transition is in `ALLOWED_TRANSITIONS` |
| `resolveSafeSubmitButtonState(state, prev?, strict?)` | Falls back to `prev` when transition is invalid in strict mode |
| `isSubmitButtonInteractionBlocked(state, disabled?, locallySubmitting?)` | True when clicks must be suppressed |
| `isSubmitButtonBusy(state, locallySubmitting?)` | True when `aria-busy` should be set |
| `ALLOWED_TRANSITIONS` | Exported transition map (shared by component and tests) |
The workspace `package.json` uses caret ranges (`^`) for all dependencies. This allows non-breaking minor and patch updates while preventing automatic major version upgrades.

For production deployments, pin exact versions in `package-lock.json` (already committed) and run `npm ci` rather than `npm install` to guarantee reproducible installs.

---

## Upgrading React

### Double-submit prevention
`onClick` is suppressed in `submitting`, `disabled`, and locally-in-flight states. This prevents duplicate blockchain transactions when a user clicks rapidly while a transaction is pending.

### No HTML injection
Labels are rendered as React text nodes. `dangerouslySetInnerHTML` is never used. Hostile markup-like strings (e.g. `<img onerror=...>`) are inert plain text.

### No dynamic CSS injection
All background colours and cursors are sourced from the `STATE_STYLES` constant. No user-supplied strings are interpolated into CSS values.

### Label normalization
- Non-string values are rejected and replaced with defaults.
- Control characters (`U+0000–U+001F`, `U+007F`) are stripped.
- Labels are capped at 80 characters to prevent layout abuse.

### Strict transition enforcement
When `strictTransitions` is enabled (default), invalid state jumps fall back to `previousState`, preventing race-condition-driven UI inconsistencies.
### React 17 → 18

No changes required. The component does not use `ReactDOM.render` (removed in React 18) or any deprecated lifecycle methods.

### React 18 → 19

No changes required. The component does not use:
- `ReactDOM.render` (removed in 18)
- `act` from `react-dom/test-utils` (moved to `react` in 19 — tests already import from `@testing-library/react`)
- Concurrent features that changed API between 18 and 19

### Upgrading `@testing-library/react`

The test suite uses `render`, `screen`, `fireEvent`, and `act` — all stable APIs present since `@testing-library/react` v13. No breaking changes are expected through v16.

---

## Security Assumptions Related to Dependencies

1. **No `dangerouslySetInnerHTML`** — the component never uses this API, so XSS via label content is not possible regardless of React version.
2. **No dynamic `import()`** — the component is statically bundled; no code-splitting attack surface.
3. **No network calls** — the component is purely presentational; it emits events to the parent and renders state. No fetch, axios, or WebSocket dependency.
4. **Inline styles only** — no CSS-in-JS runtime that could be exploited via style injection. All colour values are hardcoded constants in `STATE_STYLES`.

---

## NatSpec-style Reference

### `ReactSubmitButton`
- **@notice** Accessible submit button with a strict state machine and safe label handling.
- **@param** `state` — Current button state (required).
- **@param** `previousState` — Used to validate transitions in strict mode.
- **@param** `strictTransitions` — When true, invalid transitions fall back to `previousState`. Default: `true`.
- **@param** `labels` — Optional per-state label overrides; values are normalized before use.
- **@param** `onClick` — Async-safe handler; blocked while submitting or disabled.
- **@security** Clicks are suppressed in non-interactive states (double-submit protection).
- **@security** Labels are sanitized to prevent blank CTA states and layout abuse.

### `normalizeSubmitButtonLabel`
- **@notice** Sanitizes a candidate label value.
- **@security** Strips control characters; truncates to 80 chars; rejects non-strings and empty values.

### `ALLOWED_TRANSITIONS`
- **@notice** Exported transition map shared by the component and test suite.
- **@dev** Single source of truth for valid state movements.
### Dependency contract
- **@notice** `react_submit_button.tsx` has one runtime peer dependency: `react ≥ 17`.
- **@dev** All other imports are local types and constants — no third-party runtime packages.
- **@security** Zero third-party runtime dependencies eliminates transitive supply-chain risk for this component.

---

## Running Tests

```bash
# Run with coverage
npm test -- --testPathPattern=react_submit_button --coverage

# Watch mode
npm run test:watch -- --testPathPattern=react_submit_button
# Run with coverage report
npx jest --testPathPatterns=react_submit_button --coverage

# Watch mode during development
npm run test:watch -- --testPathPatterns=react_submit_button
```

### Latest test output

- `normalizeSubmitButtonLabel` — non-string rejection, whitespace, control chars, truncation, XSS strings
- `resolveSubmitButtonLabel` — defaults, custom overrides, fallbacks, truncation
- `isValidSubmitButtonStateTransition` — all allowed paths, same-state, blocked paths
- `resolveSafeSubmitButtonState` — strict/non-strict, missing previousState
- `isSubmitButtonInteractionBlocked` — all blocking conditions
- `isSubmitButtonBusy` — submitting and local in-flight
- Component rendering — element, labels, data-state, type, className, id
- Component disabled behavior — all blocking states
- Component accessibility — aria-live, aria-busy, aria-label
- Component click handling — idle, error, blocked states, async, rejected promise
- Strict transition enforcement — invalid/valid transitions, strict off
```
Tests:    51 passed, 51 total
Coverage: 97.67% statements | 96.87% branches | 100% functions | 100% lines
```
# ReactSubmitButton

A typed React submit button with a strict state machine, safe label handling,
double-submit prevention, and ARIA accessibility — designed for Stellar/Soroban
transaction flows.

## States

| State        | Description                                              | Interaction |
| :----------- | :------------------------------------------------------- | :---------- |
| `idle`       | Default; ready to submit                                 | Clickable   |
| `submitting` | Transaction in-flight; blocks duplicate submissions      | Blocked     |
| `success`    | Transaction confirmed                                    | Blocked     |
| `error`      | Transaction failed; user may retry                       | Clickable   |
| `disabled`   | Externally disabled (deadline passed, goal met, etc.)    | Blocked     |

### Allowed Transitions

```
idle        → submitting | disabled
submitting  → success | error | disabled
success     → idle | disabled
error       → idle | submitting | disabled
disabled    → idle
```

Invalid transitions in strict mode fall back to `previousState`.

## Props

| Prop                | Type                        | Default      | Description                                      |
| :------------------ | :-------------------------- | :----------- | :----------------------------------------------- |
| `state`             | `SubmitButtonState`         | —            | Current button state (required)                  |
| `previousState`     | `SubmitButtonState`         | `undefined`  | Used for strict transition validation             |
| `strictTransitions` | `boolean`                   | `true`       | Enforce the allowed-transition map                |
| `labels`            | `SubmitButtonLabels`        | `undefined`  | Per-state label overrides                         |
| `onClick`           | `(e) => void \| Promise`    | `undefined`  | Async-safe handler; blocked while submitting      |
| `type`              | `"button" \| "submit" \| "reset"` | `"button"` | HTML button type                           |
| `disabled`          | `boolean`                   | `undefined`  | External disabled override                        |
| `className`         | `string`                    | `undefined`  | Additional CSS class                              |
| `id`                | `string`                    | `undefined`  | HTML id attribute                                 |

## Usage

```tsx
import ReactSubmitButton from "./react_submit_button";

// Basic
<ReactSubmitButton state="idle" onClick={handleContribute} />

// With state machine enforcement
<ReactSubmitButton
  state={txState}          // e.g. "submitting"
  previousState={prevState} // e.g. "idle"
  strictTransitions
  labels={{ idle: "Fund Campaign", submitting: "Funding…", success: "Funded!" }}
  onClick={handleContribute}
/>
```

## Exported Helpers

All pure helpers are exported for independent unit testing:

| Export                                  | Purpose                                              |
| :-------------------------------------- | :--------------------------------------------------- |
| `normalizeSubmitButtonLabel`            | Sanitize/truncate a label candidate                  |
| `resolveSubmitButtonLabel`              | Pick the correct label for a given state             |
| `isValidSubmitButtonStateTransition`    | Check if a state transition is allowed               |
| `resolveSafeSubmitButtonState`          | Apply strict-mode fallback logic                     |
| `isSubmitButtonInteractionBlocked`      | Determine if clicks should be suppressed             |
| `isSubmitButtonBusy`                    | Determine if `aria-busy` should be `true`            |
| `ALLOWED_TRANSITIONS`                   | The canonical transition map (shared with tests)     |

## Security Assumptions

- **No `dangerouslySetInnerHTML`** — labels are rendered as React text nodes only.
- **Label sanitization** — control characters are stripped; labels are truncated at
  80 characters to prevent layout abuse.
- **Double-submit prevention** — the component maintains a local `isLocallySubmitting`
  flag that blocks re-entry while an async `onClick` is in-flight, preventing
  duplicate Stellar transactions.
- **Hardcoded styles** — all colours are compile-time constants; no user input
  reaches CSS properties.
- **Caller responsibility** — input validation (amounts, addresses) must be
  performed by the parent before calling `onClick`.

## Testing

```bash
# Run the full test suite
npm test -- --testPathPattern=react_submit_button

# With coverage
npm run test:coverage -- --testPathPattern=react_submit_button
```

Coverage target: ≥ 95% (branches, lines, functions, statements).

The test suite covers:

- `normalizeSubmitButtonLabel` — non-string inputs, empty/whitespace, control
  characters, 80-char boundary, truncation, XSS-like strings
- `resolveSubmitButtonLabel` — all five states, custom overrides, fallback logic
- `isValidSubmitButtonStateTransition` — every allowed edge, every blocked edge,
  same-state idempotency
- `resolveSafeSubmitButtonState` — strict/non-strict modes, missing previousState
- `isSubmitButtonInteractionBlocked` — all blocking conditions
- `isSubmitButtonBusy` — submitting state, local flag
- Component rendering — all states, props, ARIA attributes
- Click handling — idle/error fire, submitting/disabled/success block,
  async handler, rejected promise, no-onClick guard
- Strict transition enforcement — invalid fallback, valid pass-through, non-strict
