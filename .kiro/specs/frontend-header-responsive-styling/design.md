# Design Document: Frontend Header Responsive Styling

## Overview

This document describes the technical design for the `Header` component — a responsive, fixed page-level header for the Stellar Raise frontend. The header renders the brand logo and "Stellar Raise" name on mobile viewports (< 768px) where the `BottomNav` handles primary navigation. On tablet and desktop (≥ 768px), the `Sidebar` already provides brand identity, so the header is hidden to avoid duplication.

The design integrates with the existing responsive design system (`frontend/styles/responsive.css`, `frontend/styles/utilities.css`) and follows the same patterns established by `BottomNav` and `Sidebar`.

### Key Design Decisions

- **Mobile-only visibility**: The header is visible only on viewports < 768px. This mirrors how `BottomNav` hides at ≥ 768px and `Sidebar` appears at ≥ 768px — the two components together cover all breakpoints without overlap.
- **Single source of truth for height**: A `--header-height` custom property (and per-breakpoint variants) is added to `responsive.css` so layout offset calculations never hard-code pixel values.
- **No JavaScript required**: All responsive behavior is achieved with CSS media queries and custom properties, keeping the component purely declarative.
- **Skip-navigation link**: Included as the first focusable child for keyboard/screen-reader users, following WCAG 2.4.1.

---

## Architecture

The header fits into the existing component architecture as a peer of `BottomNav` and `Sidebar`:

```
frontend/
├── styles/
│   └── responsive.css          ← add --header-height-* tokens here
├── components/
│   └── navigation/
│       ├── BottomNav.css / .html   (existing)
│       ├── Sidebar.css / .html     (existing)
│       ├── Header.css              (new)
│       └── Header.html             (new demo page)
└── docs/
    ├── RESPONSIVE_DESIGN_GUIDE.md  (update)
    └── TESTING_GUIDE.md            (update)
```

### Breakpoint Behavior

```
< 768px   (mobile)   → Header visible, BottomNav visible, Sidebar hidden
≥ 768px   (tablet)   → Header hidden, Sidebar visible, BottomNav hidden
≥ 1024px  (desktop)  → Header hidden, Sidebar visible (wider), BottomNav hidden
```

```mermaid
stateDiagram-v2
    [*] --> Mobile : viewport < 768px
    [*] --> Tablet : 768px ≤ viewport < 1024px
    [*] --> Desktop : viewport ≥ 1024px

    Mobile : Header ✓  BottomNav ✓  Sidebar ✗
    Tablet : Header ✗  BottomNav ✗  Sidebar ✓ (240px)
    Desktop : Header ✗  BottomNav ✗  Sidebar ✓ (280px)
```

### Layout Offset Model

On mobile, the main content area must be offset for both the fixed header (top) and the fixed bottom nav (bottom):

```
┌─────────────────────────────────┐
│  Header (fixed, top)            │  ← --header-height-mobile (48–64px)
├─────────────────────────────────┤
│                                 │
│  Main content                   │  ← padding-top: var(--header-height)
│                                 │       + safe-area-inset-top
│                                 │  ← padding-bottom: 72px
│                                 │       + safe-area-inset-bottom
├─────────────────────────────────┤
│  BottomNav (fixed, bottom)      │  ← 72px
└─────────────────────────────────┘
```

---

## Components and Interfaces

### Header HTML Structure

```html
<header class="site-header" role="banner">
  <!-- Skip navigation (first focusable child) -->
  <a href="#main-content" class="site-header__skip-link">Skip to main content</a>

  <div class="site-header__inner">
    <!-- Brand identity -->
    <div class="site-header__brand">
      <svg class="site-header__logo" width="32" height="32" viewBox="0 0 32 32"
           fill="none" aria-hidden="true">
        <circle cx="16" cy="16" r="16" fill="var(--color-primary-blue)"/>
        <path d="M16 8L20 16L16 24L12 16L16 8Z" fill="white"/>
      </svg>
      <span class="site-header__brand-name">Stellar Raise</span>
    </div>

    <!-- Optional contextual actions slot -->
    <div class="site-header__actions">
      <!-- e.g. wallet connect button -->
    </div>
  </div>
</header>
```

### CSS Class API

| Class | Purpose |
|---|---|
| `.site-header` | Root element — fixed positioning, height, background, shadow |
| `.site-header__skip-link` | Visually hidden skip link, visible on focus |
| `.site-header__inner` | Flex row container for brand + actions |
| `.site-header__brand` | Flex row: logo + brand name |
| `.site-header__logo` | SVG logo mark |
| `.site-header__brand-name` | "Stellar Raise" text |
| `.site-header__actions` | Right-aligned slot for contextual actions |
| `.has-header` | Applied to `<main>` — adds `padding-top` equal to header height |

### Design Token Interface

The following tokens are consumed by `Header.css` (all defined in `responsive.css`):

| Token | Usage |
|---|---|
| `--header-height-mobile` | Height at < 768px (48px) |
| `--header-height-tablet` | Height at 768–1023px (56px) |
| `--header-height-desktop` | Height at ≥ 1024px (64px) |
| `--header-height` | Alias set per breakpoint for layout offsets |
| `--color-neutral-100` | Header background |
| `--color-deep-navy` | Brand name text color |
| `--color-primary-blue` | Focus indicator, logo fill |
| `--font-family-primary` | All header text |
| `--font-size-lg` | Brand name font size |
| `--shadow-sm` | Resting shadow |
| `--shadow-md` | Scrolled shadow |
| `--transition-fast` | Shadow transition on scroll |
| `--z-fixed` | z-index (same layer as BottomNav/Sidebar) |
| `--safe-area-inset-top` | Top padding for notched devices |
| `--space-4` | Horizontal padding |
| `--touch-target-min` | Minimum 44px for interactive elements |

---

## Data Models

The header is a purely presentational CSS/HTML component with no runtime data model. The relevant "data" is the set of CSS custom properties that parameterize its appearance.

### New CSS Custom Properties (added to `responsive.css`)

```css
:root {
  /* Header height tokens */
  --header-height-mobile:  48px;
  --header-height-tablet:  56px;
  --header-height-desktop: 64px;

  /* Alias — updated per breakpoint via media queries */
  --header-height: var(--header-height-mobile);
}

@media (min-width: 768px) {
  :root {
    --header-height: var(--header-height-tablet);
  }
}

@media (min-width: 1024px) {
  :root {
    --header-height: var(--header-height-desktop);
  }
}
```

### Layout Offset Classes (added to `responsive.css` or `Header.css`)

```css
/* Applied to <main> when header is present */
.has-header {
  padding-top: calc(var(--header-height) + var(--safe-area-inset-top));
}

/* On tablet+, header is hidden so offset is removed */
@media (min-width: 768px) {
  .has-header {
    padding-top: 0;
  }
}
```

### Scroll State (JavaScript-assisted, optional)

The shadow elevation change on scroll (Requirement 3.4) can be driven by a small inline script or a CSS scroll-driven animation. The recommended approach is a single `scroll` event listener that toggles a `.site-header--scrolled` class on the `<header>` element:

```js
// Minimal scroll shadow toggle
const header = document.querySelector('.site-header');
window.addEventListener('scroll', () => {
  header.classList.toggle('site-header--scrolled', window.scrollY > 0);
}, { passive: true });
```

```css
.site-header {
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--transition-fast);
}
.site-header--scrolled {
  box-shadow: var(--shadow-md);
}
```

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Header height is within spec at every breakpoint

*For any* rendered viewport width, the computed height of `.site-header` should fall within the range specified for that breakpoint: [48px, 64px] for mobile (< 768px), [56px, 72px] for tablet (768–1023px), and [64px, 80px] for desktop (≥ 1024px).

**Validates: Requirements 1.2, 1.3, 1.4**

---

### Property 2: Header visibility is mutually exclusive with Sidebar

*For any* viewport width ≥ 768px, `.site-header` should have `display: none` (or equivalent computed style that removes it from layout), and `.sidebar` should be visible. *For any* viewport width < 768px, `.site-header` should be visible and `.sidebar` should not be rendered.

**Validates: Requirements 1.3, 1.4, 2.2**

---

### Property 3: Main content is never obscured by fixed navigation

*For any* viewport width < 768px, the `padding-top` of the main content area should be greater than or equal to the computed height of `.site-header`, and the `padding-bottom` should be greater than or equal to 72px (the BottomNav height). This ensures no content is hidden behind either fixed element.

**Validates: Requirements 2.1, 2.4**

---

### Property 4: Design token exclusivity

*For any* CSS rule in `Header.css`, every color, spacing, font, shadow, and z-index value should reference a CSS custom property (i.e., use `var(--...)`) rather than a hard-coded literal. The linter should report a violation for any hard-coded value that has a design system equivalent.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**

---

### Property 5: Touch target compliance for all interactive elements

*For any* interactive element (link, button) within `.site-header`, its computed `min-height` and `min-width` should both be ≥ 44px.

**Validates: Requirements 4.3**

---

### Property 6: Focus indicator present on all interactive elements

*For any* interactive element within `.site-header` that receives keyboard focus, the element should display a visible focus indicator with `outline: 2px solid var(--color-primary-blue)` and `outline-offset: 2px`.

**Validates: Requirements 4.5**

---

### Property 7: Reduced-motion transitions are suppressed

*For any* animated property in `.site-header`, when `prefers-reduced-motion: reduce` is active, the transition duration should be ≤ 0.01ms.

**Validates: Requirements 4.6**

---

### Property 8: Skip link is the first focusable element

*For any* rendered Header, the first element reached by pressing Tab from outside the header should be the skip-navigation link, and activating it should move focus to `#main-content`.

**Validates: Requirements 4.2**

---

## Error Handling

### Missing Design Tokens

If a CSS custom property referenced by `Header.css` is not defined (e.g., `responsive.css` is not loaded), the browser falls back to the initial value for that property. To mitigate:
- `Header.html` explicitly links both `../../styles/responsive.css` and `Header.css` in the correct order.
- The CI lint step validates that all `var(--...)` references in `Header.css` are declared in `responsive.css`.

### Safe Area Insets Not Supported

On browsers that do not support `env(safe-area-inset-top)`, the expression `calc(var(--space-4) + env(safe-area-inset-top, 0))` falls back to `var(--space-4)` (16px) via the fallback argument, which is acceptable.

### Snapshot Test Baseline Missing

On first run, the CI snapshot step will create baseline images rather than diff against them. The pipeline should be configured to commit these baselines and only fail on subsequent runs when a diff exceeds 0.1%.

### Sidebar/Header Both Visible (Edge Case)

If a developer accidentally applies both `.has-header` and `.has-sidebar` to the same `<main>` element, the header's `padding-top` and the sidebar's `margin-left` will both apply. This is not harmful but is redundant on tablet/desktop. Documentation should note that `.has-header` is only meaningful on mobile viewports.

---

## Testing Strategy

### Dual Testing Approach

Both unit/example tests and property-based tests are required. Unit tests cover specific examples and edge cases; property tests verify universal invariants across generated inputs.

### Unit / Example Tests

These are manual or automated browser-based checks at specific viewports:

- **Breakpoint rendering**: Load `Header.html` at 375px, 768px, and 1280px; assert header visibility, height, and shadow state.
- **Skip link**: Tab to the skip link, press Enter, assert `document.activeElement` is `#main-content`.
- **Scroll shadow**: Scroll page to `scrollY = 1`; assert `.site-header--scrolled` class is present and `box-shadow` equals `var(--shadow-md)`.
- **Safe area**: Simulate `env(safe-area-inset-top) = 44px`; assert header `padding-top` is `44px`.
- **Sidebar co-existence**: At 768px, assert `.site-header` is not in the layout flow and `.sidebar` is visible.
- **Accessibility audit**: Run axe-core against `Header.html` at 375px, 768px, 1280px; assert zero violations.

### Property-Based Tests

Property-based tests use a PBT library appropriate for the target language. Since this is a CSS/HTML component, tests are written in JavaScript using **fast-check** (a TypeScript/JavaScript PBT library) combined with a headless browser (Playwright or jsdom with CSS support).

Each property test runs a minimum of **100 iterations** with randomly generated inputs (viewport widths, scroll positions, content lengths).

**Tag format**: `Feature: frontend-header-responsive-styling, Property {N}: {property_text}`

#### Property Test Specifications

**Property 1 — Header height within spec**
```
// Feature: frontend-header-responsive-styling, Property 1: Header height is within spec at every breakpoint
// For any viewport width, assert computed header height is within the breakpoint range.
fc.property(
  fc.integer({ min: 320, max: 1920 }),  // random viewport width
  (width) => {
    setViewportWidth(width);
    const height = getComputedHeight('.site-header');
    if (width < 768)  return height >= 48 && height <= 64;
    if (width < 1024) return height >= 56 && height <= 72;
    return height >= 64 && height <= 80;
  }
)
```

**Property 2 — Header/Sidebar mutual exclusivity**
```
// Feature: frontend-header-responsive-styling, Property 2: Header visibility is mutually exclusive with Sidebar
fc.property(
  fc.integer({ min: 320, max: 1920 }),
  (width) => {
    setViewportWidth(width);
    const headerVisible = isVisible('.site-header');
    const sidebarVisible = isVisible('.sidebar');
    return headerVisible !== sidebarVisible; // exactly one is visible
  }
)
```

**Property 3 — Content not obscured**
```
// Feature: frontend-header-responsive-styling, Property 3: Main content is never obscured by fixed navigation
fc.property(
  fc.integer({ min: 320, max: 767 }),  // mobile only
  (width) => {
    setViewportWidth(width);
    const paddingTop = getComputedPaddingTop('main.has-header');
    const paddingBottom = getComputedPaddingBottom('main.has-header');
    const headerHeight = getComputedHeight('.site-header');
    return paddingTop >= headerHeight && paddingBottom >= 72;
  }
)
```

**Property 5 — Touch target compliance**
```
// Feature: frontend-header-responsive-styling, Property 5: Touch target compliance for all interactive elements
fc.property(
  fc.constantFrom(375, 390, 430),  // representative mobile widths
  (width) => {
    setViewportWidth(width);
    const interactives = queryAll('.site-header a, .site-header button');
    return interactives.every(el => {
      const rect = el.getBoundingClientRect();
      return rect.width >= 44 && rect.height >= 44;
    });
  }
)
```

**Property 6 — Focus indicator present**
```
// Feature: frontend-header-responsive-styling, Property 6: Focus indicator present on all interactive elements
fc.property(
  fc.constantFrom(375, 768, 1280),
  (width) => {
    setViewportWidth(width);
    const interactives = queryAll('.site-header a, .site-header button');
    return interactives.every(el => {
      el.focus();
      const style = getComputedStyle(el);
      return style.outlineWidth === '2px' && style.outlineColor.includes('0, 102, 255');
    });
  }
)
```

**Property 7 — Reduced-motion suppression**
```
// Feature: frontend-header-responsive-styling, Property 7: Reduced-motion transitions are suppressed
// Simulate prefers-reduced-motion: reduce, then assert all transition durations ≤ 0.01ms
fc.property(
  fc.integer({ min: 320, max: 1920 }),
  (width) => {
    setViewportWidth(width);
    setMediaFeature('prefers-reduced-motion', 'reduce');
    const header = query('.site-header');
    const duration = parseFloat(getComputedStyle(header).transitionDuration) * 1000;
    return duration <= 0.01;
  }
)
```

**Property 8 — Skip link is first focusable**
```
// Feature: frontend-header-responsive-styling, Property 8: Skip link is the first focusable element
// Example test (single specific case, not randomized)
const firstFocusable = getFirstFocusableInHeader();
assert(firstFocusable.matches('.site-header__skip-link'));
activateLink(firstFocusable);
assert(document.activeElement.id === 'main-content');
```

### CI/CD Integration

The following steps are added to the CI pipeline (`.github/`):

1. **CSS lint** — validate `frontend/components/navigation/Header.css` and `frontend/styles/responsive.css` against project lint rules; fail on hard-coded values with design token equivalents.
2. **HTML validation** — validate `frontend/components/navigation/Header.html` for well-formed markup.
3. **Accessibility audit** — run axe-core against `Header.html` at 375px, 768px, 1280px; fail on any WCAG 2.1 AA violation.
4. **Snapshot tests** — capture screenshots at 375px, 768px, 1280px; fail if pixel diff > 0.1%.
5. **Property tests** — run fast-check property suite; minimum 100 iterations per property.

All steps must pass before a pull request can be merged.
