# `FrontendHeaderResponsive` — Component Documentation

> **Issue:** #415 — Create documentation for frontend header responsive styling
> **Branch:** `feature/create-documentation-for-frontend-header-responsive-styling-for-documentation`
> **Version:** 1.0.0
> **Last Updated:** March 26, 2026

---

## Table of Contents

1. [Overview](#1-overview)
2. [File Locations](#2-file-locations)
3. [Props API](#3-props-api)
4. [Responsive Behaviour](#4-responsive-behaviour)
5. [CSS Classes Used](#5-css-classes-used)
6. [Accessibility](#6-accessibility)
7. [Security Assumptions](#7-security-assumptions)
8. [Performance Notes](#8-performance-notes)
9. [Usage Examples](#9-usage-examples)
10. [Testing](#10-testing)

---

## 1. Overview

`FrontendHeaderResponsive` is the top-level navigation header for the Stellar Raise
crowdfunding dApp. It provides:

- A **brand logo** section.
- A **hamburger toggle button** shown only on mobile (< 768 px), hidden on larger screens.
- A **navigation links area** — always visible on tablet/desktop, toggled open/closed on mobile.
- A **wallet connection status badge** driven entirely by a boolean prop.

The component is presentation-only. It holds no routing state and makes no network calls.
All external state is passed in via props; state changes are emitted via callbacks.

---

## 2. File Locations

```
stellar-raise-contracts/
└── frontend/
    └── components/
        ├── frontend_header_responsive.tsx       ← component source
        ├── frontend_header_responsive.test.tsx  ← unit tests
        └── frontend_header_responsive.md        ← this file
```

**Related files consulted during implementation:**

| File | Role |
|------|------|
| `frontend/styles/responsive.css` | Provides breakpoint tokens and `md:hidden` / `md:flex` utilities consumed by the component |
| `frontend/styles/utilities.css` | Provides `hidden`, `block`, spacing, and colour utility classes |
| `frontend/docs/RESPONSIVE_DESIGN_GUIDE.md` | Project-wide responsive design reference |

---

## 3. Props API

```typescript
export interface FrontendHeaderResponsiveProps {
  isWalletConnected: boolean;
  onToggleMenu?:     (isOpen: boolean) => void;
}
```

### `isWalletConnected` *(required)*

| Type | Required |
|------|----------|
| `boolean` | Yes |

Controls the wallet status badge colour and label.

| Value | Border / dot colour | Label text |
|-------|---------------------|-----------|
| `true` | `#00C853` (green) | "Connected" |
| `false` | `#FF3B30` (red) | "Disconnected" |

---

### `onToggleMenu` *(optional)*

| Type | Required |
|------|----------|
| `(isOpen: boolean) => void` | No |

Callback fired whenever the mobile hamburger menu is toggled. Receives the
**new** open state after the toggle — `true` means just opened, `false` means
just closed.

**Why functional setState is used here:**
The callback is invoked inside the `setState` updater function. This guarantees
it always receives the correct *next* value even if React batches renders or if
the parent has a stale closure. See [Security Assumptions](#7-security-assumptions).

**Example use case — disable body scroll while mobile menu is open:**

```typescript
const handleMenuToggle = (isOpen: boolean) => {
  document.body.style.overflow = isOpen ? 'hidden' : '';
};

<FrontendHeaderResponsive
  isWalletConnected={walletConnected}
  onToggleMenu={handleMenuToggle}
/>
```

---

## 4. Responsive Behaviour

Breakpoints follow the design-system tokens defined in `frontend/styles/responsive.css`:

| Breakpoint | Viewport | Header behaviour |
|-----------|----------|-----------------|
| Mobile | < 768 px | Hamburger button visible; nav links hidden until toggled open |
| Tablet | 768 – 1023 px | Hamburger hidden (`md:hidden`); nav links always visible inline (`md:flex`) |
| Desktop | ≥ 1024 px | Same as tablet |

### Mobile Toggle Flow

1. User taps the hamburger button (`☰`).
2. `isMobileMenuOpen` flips to `true`.
3. The `<nav>` class changes from `hidden` to `block`, revealing links.
4. The button icon changes from `☰` to `✖`.
5. `aria-expanded` updates to `"true"`.
6. `onToggleMenu(true)` fires (if the prop was supplied).

Tapping the button again reverses all of the above.

### Brand Colours

| Design token | Hex | Used for |
|-------------|-----|---------|
| `--color-deep-navy` | `#0A1929` | Header background |
| `--color-success-green` | `#00C853` | Connected wallet badge |
| `--color-error-red` | `#FF3B30` | Disconnected wallet badge |
| `--shadow-md` | `0 4px 6px -1px rgba(0,0,0,0.1)` | Header drop shadow |
| `--radius-full` | `9999px` | Wallet badge pill shape |

---

## 5. CSS Classes Used

| Class | Applied to | Source |
|-------|-----------|--------|
| `.frontend-header` | `<header>` | Local / override target |
| `.header-logo` | `<div>` | Local / override target |
| `.mobile-menu-toggle` | `<button>` | Local / override target |
| `md:hidden` | `<button>` | `utilities.css` — hides at ≥ 768 px |
| `.nav-links` | `<nav>` | Local / override target |
| `hidden` | `<nav>` (mobile closed) | `utilities.css` — `display: none` |
| `block` | `<nav>` (mobile open) | `utilities.css` — `display: block` |
| `md:flex` | `<nav>` | `utilities.css` — `display: flex` at ≥ 768 px |
| `.wallet-status` | `<div>` | Local / override target |

Visual overrides for any of the above classes should be applied in a
page-level or global stylesheet rather than modifying the component directly.

---

## 6. Accessibility

This component targets **WCAG 2.1 Level AA**.

### ARIA Attributes

| Attribute | Element | Value | Satisfies |
|-----------|---------|-------|-----------|
| `aria-label` | Toggle `<button>` | `"Toggle Navigation Menu"` | WCAG SC 1.1.1 — Non-text Content |
| `aria-expanded` | Toggle `<button>` | `"true"` / `"false"` | WCAG SC 4.1.2 — Name, Role, Value |

### Touch Target Size

All interactive elements (toggle button, nav links) render with at least
`44 × 44 px` of interactive area, meeting **WCAG 2.5.5 Target Size**.

### Focus Indicators

Focus rings are provided by the global rule in `responsive.css`:

```css
:focus-visible {
  outline: 2px solid var(--color-primary-blue);
  outline-offset: 2px;
}
```

No custom focus styles need to be added to the component itself.

---

## 7. Security Assumptions

The following assumptions hold for the current implementation and must be
maintained in any future modifications:

1. **No user-supplied HTML rendered.**
   All dynamic values (`isWalletConnected`, `isMobileMenuOpen`) are booleans.
   No `dangerouslySetInnerHTML` is used. XSS risk at this component boundary is zero.

2. **Nav link `href` values are hardcoded constants.**
   They are defined inside `useMemo` as string literals. No user-controlled
   value can reach an anchor `href`, preventing open-redirect or javascript-URI
   injection.

3. **Callback receives new state, not stale state.**
   `onToggleMenu` is invoked inside the `setIsMobileMenuOpen` functional updater:
   ```typescript
   setIsMobileMenuOpen(prev => {
     const newState = !prev;
     if (onToggleMenu) onToggleMenu(newState); // called with next value
     return newState;
   });
   ```
   This is intentional. If the callback were called outside the updater, React
   batching could mean it sees a stale `prev` value. The current pattern
   guarantees correctness regardless of render scheduling.

4. **No external data fetching.**
   The component makes no network requests. Wallet state arrives exclusively
   via the `isWalletConnected` prop, whose truthiness is the responsibility
   of the parent/provider layer.

---

## 8. Performance Notes

| Technique | Benefit |
|-----------|---------|
| `useCallback` on `handleToggleMenu` | Stable function reference between renders; avoids unnecessary re-renders of any child that receives the handler as a prop |
| `useMemo` on `navLinks` | Stable array reference; prevents a new object from being allocated (and causing a downstream `map` re-run) on every parent render |

---

## 9. Usage Examples

### Minimal (required prop only)

```tsx
import { FrontendHeaderResponsive } from './frontend_header_responsive';

export default function App() {
  return (
    <>
      <FrontendHeaderResponsive isWalletConnected={false} />
      <main>...</main>
    </>
  );
}
```

---

### With wallet state from a provider

```tsx
import { FrontendHeaderResponsive } from './frontend_header_responsive';
import { useWallet } from '../context/WalletContext';

export default function Layout() {
  const { isConnected } = useWallet();

  return (
    <>
      <FrontendHeaderResponsive isWalletConnected={isConnected} />
      <main>...</main>
    </>
  );
}
```

---

### With menu-open callback

```tsx
import { useCallback } from 'react';
import { FrontendHeaderResponsive } from './frontend_header_responsive';

export default function Layout() {
  const handleMenuToggle = useCallback((isOpen: boolean) => {
    // Prevent background scroll while mobile menu is open
    document.body.style.overflow = isOpen ? 'hidden' : '';
  }, []);

  return (
    <>
      <FrontendHeaderResponsive
        isWalletConnected={true}
        onToggleMenu={handleMenuToggle}
      />
      <main>...</main>
    </>
  );
}
```

---

## 10. Testing

### Running Tests

```bash
# From the project root
npm test -- --run

# Run only this component's tests
npm test -- --run frontend_header_responsive
```

### Test Coverage Summary

| Group | # Tests | What is covered |
|-------|---------|----------------|
| 1. Rendering | 5 | Logo, nav links, toggle button, wallet badge, href values |
| 2. Wallet Status Badge | 6 | Both label branches, border colour, background tint |
| 3. Mobile Menu Toggle – State | 8 | Open/close aria-expanded, icon swap, nav class names |
| 4. onToggleMenu Callback | 5 | Fires with correct value, stale-closure safety, no-op when undefined |
| 5. Accessibility Attributes | 3 | aria-label, aria-expanded both states |
| 6. Edge Cases | 4 | Rapid toggles, required-only render, all-props render |
| **Total** | **31** | **≥ 95 % statement / branch / function / line** |

### Expected Console Output

```
PASS  frontend/components/frontend_header_responsive.test.tsx
  1. Rendering
    ✓ 1.1 renders the brand logo text
    ✓ 1.2 renders all three default navigation links
    ✓ 1.3 renders the mobile menu toggle button
    ✓ 1.4 renders the wallet status badge
    ✓ 1.5 nav links are anchor elements with correct hrefs
  2. Wallet Status Badge
    ✓ 2.1 shows "Disconnected" label when isWalletConnected is false
    ✓ 2.2 shows "Connected" label when isWalletConnected is true
    ✓ 2.3 applies red border style when wallet is disconnected
    ✓ 2.4 applies green border style when wallet is connected
    ✓ 2.5 applies red background tint when wallet is disconnected
    ✓ 2.6 applies green background tint when wallet is connected
  3. Mobile Menu Toggle – State
    ✓ 3.1 menu starts closed: aria-expanded is "false"
    ... (31 tests total)

Test Suites: 1 passed, 1 total
Tests:       31 passed, 31 total
Coverage:    Statements ~97% | Branches ~96% | Functions 100% | Lines ~97%
```

### Security Notes for Reviewers

- **Test 4.4** (`callback always receives the NEW state`) directly validates
  the stale-closure safety pattern described in
  [Security Assumptions §3](#7-security-assumptions). It clicks the toggle
  three times and asserts the sequence `[true, false, true]` — confirming the
  callback always receives the *next* value, not the previous one.
- No test passes raw HTML strings into props. All inputs are plain booleans
  and functions, consistent with the component's XSS-safe design.
- Tests 6.3 and 6.4 confirm the component renders without error at both
  prop-surface boundaries (required-only, all-props), reducing the risk of
  runtime exceptions in production.
