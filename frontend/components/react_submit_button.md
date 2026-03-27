# ReactSubmitButton

A typed React submit button with a strict state machine, safe label handling, double-submit prevention, and ARIA accessibility semantics.

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

---

## Usage

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
