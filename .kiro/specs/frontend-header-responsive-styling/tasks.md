# Implementation Plan: Frontend Header Responsive Styling

## Overview

Implement the `Header` component as a mobile-only fixed page-level header, integrate it with the existing design system tokens, update documentation, and wire up CI pipeline steps for lint, HTML validation, accessibility audit, snapshot tests, and property tests.

## Tasks

- [ ] 1. Add header design tokens to `frontend/styles/responsive.css`
  - Add `--header-height-mobile: 48px`, `--header-height-tablet: 56px`, `--header-height-desktop: 64px` to `:root`
  - Add `--header-height` alias that resolves to the correct per-breakpoint value via media queries
  - Add `.has-header` layout offset class: `padding-top: calc(var(--header-height) + var(--safe-area-inset-top))` on mobile, reset to `0` at ≥ 768px
  - _Requirements: 2.3, 3.5_

- [ ] 2. Implement `frontend/components/navigation/Header.css`
  - [ ] 2.1 Write the base `.site-header` styles
    - `position: fixed; top: 0; left: 0; right: 0; z-index: var(--z-fixed)`
    - Height driven by `--header-height`; background `var(--color-neutral-100)`; `box-shadow: var(--shadow-sm)`; `transition: box-shadow var(--transition-fast)`
    - `padding-top: var(--safe-area-inset-top)`; horizontal padding `var(--space-4)`
    - Add `display: none` at `min-width: 768px` (header hidden when sidebar is present)
    - Include inline comments identifying each breakpoint block and token names
    - _Requirements: 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 3.1, 3.2, 3.3, 3.4, 6.2_

  - [ ]* 2.2 Write property test for design token exclusivity (Property 4)
    - **Property 4: Design token exclusivity**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6**
    - Parse `Header.css` and assert every color, spacing, font, shadow, and z-index value uses `var(--...)` rather than a hard-coded literal

  - [ ] 2.3 Write `.site-header__skip-link` styles
    - Visually hidden by default (`.sr-only` pattern); visible on `:focus-visible` with `outline: 2px solid var(--color-primary-blue); outline-offset: 2px`
    - _Requirements: 4.2, 4.5_

  - [ ] 2.4 Write `.site-header__inner`, `.site-header__brand`, `.site-header__brand-name`, `.site-header__actions` styles
    - `__inner`: flex row, `align-items: center; justify-content: space-between`; full height
    - `__brand`: flex row, `gap: var(--space-3); align-items: center`
    - `__brand-name`: `font-family: var(--font-family-primary); font-size: var(--font-size-lg); color: var(--color-deep-navy); font-weight: 700`
    - `__actions`: flex row, `gap: var(--space-2); align-items: center; margin-left: auto`
    - _Requirements: 1.1, 3.1, 3.2_

  - [ ] 2.5 Write `.site-header--scrolled` modifier and reduced-motion override
    - `.site-header--scrolled { box-shadow: var(--shadow-md); }`
    - `@media (prefers-reduced-motion: reduce)` block sets `transition-duration: 0.01ms` on `.site-header`
    - _Requirements: 3.3, 3.4, 4.6_

  - [ ]* 2.6 Write property test for header height within spec (Property 1)
    - **Property 1: Header height is within spec at every breakpoint**
    - **Validates: Requirements 1.2, 1.3, 1.4**
    - Use fast-check + Playwright: `fc.integer({ min: 320, max: 1920 })` → set viewport → assert computed height within breakpoint range

  - [ ]* 2.7 Write property test for header/sidebar mutual exclusivity (Property 2)
    - **Property 2: Header visibility is mutually exclusive with Sidebar**
    - **Validates: Requirements 1.3, 1.4, 2.2**
    - Use fast-check + Playwright: random viewport width → assert exactly one of `.site-header` / `.sidebar` is visible

  - [ ]* 2.8 Write property test for reduced-motion suppression (Property 7)
    - **Property 7: Reduced-motion transitions are suppressed**
    - **Validates: Requirements 4.6**
    - Simulate `prefers-reduced-motion: reduce` → assert `transitionDuration` ≤ 0.01ms on `.site-header`

- [ ] 3. Checkpoint — Ensure all CSS is valid and tokens resolve correctly
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 4. Create `frontend/components/navigation/Header.html` demo page
  - [ ] 4.1 Write the self-contained demo HTML file
    - `<meta viewport>` with `viewport-fit=cover`; link `../../styles/responsive.css` then `Header.css`
    - Include `<header class="site-header" role="banner">` with skip link, `__inner`, `__brand` (SVG logo + brand name), and empty `__actions` slot
    - Include `<main id="main-content" class="has-header has-bottom-nav">` with sample content
    - Include inline `<script>` for scroll shadow toggle (passive listener toggling `.site-header--scrolled`)
    - _Requirements: 1.1, 4.1, 4.2, 6.4_

  - [ ]* 4.2 Write property test for main content not obscured (Property 3)
    - **Property 3: Main content is never obscured by fixed navigation**
    - **Validates: Requirements 2.1, 2.4**
    - Use fast-check + Playwright: `fc.integer({ min: 320, max: 767 })` → assert `padding-top` of `main.has-header` ≥ computed header height and `padding-bottom` ≥ 72px

  - [ ]* 4.3 Write property test for touch target compliance (Property 5)
    - **Property 5: Touch target compliance for all interactive elements**
    - **Validates: Requirements 4.3**
    - Use fast-check + Playwright: `fc.constantFrom(375, 390, 430)` → assert all `.site-header a, .site-header button` have `width ≥ 44` and `height ≥ 44`

  - [ ]* 4.4 Write property test for focus indicator presence (Property 6)
    - **Property 6: Focus indicator present on all interactive elements**
    - **Validates: Requirements 4.5**
    - Use fast-check + Playwright: `fc.constantFrom(375, 768, 1280)` → focus each interactive element → assert `outlineWidth === '2px'` and outline color contains `0, 102, 255`

  - [ ]* 4.5 Write example test for skip link (Property 8)
    - **Property 8: Skip link is the first focusable element**
    - **Validates: Requirements 4.2**
    - Tab from outside header → assert first focused element matches `.site-header__skip-link` → activate → assert `document.activeElement.id === 'main-content'`

- [ ] 5. Checkpoint — Verify Header.html renders correctly at 375px, 768px, and 1280px
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Update `frontend/docs/RESPONSIVE_DESIGN_GUIDE.md`
  - Add a "Header" section under "Navigation Patterns" documenting:
    - Breakpoint behavior table (mobile visible / tablet+desktop hidden)
    - HTML usage example with `.has-header` on `<main>`
    - Full list of CSS custom properties consumed by the component
    - Note on `.has-header` + `.has-sidebar` co-existence edge case
  - Update the "File Structure" section to include `Header.css` and `Header.html`
  - _Requirements: 6.1_

- [ ] 7. Update `frontend/docs/TESTING_GUIDE.md`
  - Add a "Header Component" subsection under "Navigation Testing" with test steps and expected results for:
    - Breakpoint rendering at 375px, 768px, 1280px (visibility, height, shadow)
    - Skip link keyboard activation
    - Scroll shadow toggle
    - Safe area inset simulation
    - Sidebar co-existence at 768px
    - Accessibility audit (axe-core, zero violations)
  - _Requirements: 6.3_

- [ ] 8. Add CI pipeline steps in `.github/`
  - [ ] 8.1 Add CSS lint step
    - Validate `frontend/components/navigation/Header.css` and `frontend/styles/responsive.css`; fail on hard-coded values that have design token equivalents
    - _Requirements: 3.6, 5.1_

  - [ ] 8.2 Add HTML validation step
    - Validate all HTML files in `frontend/components/navigation/` for well-formed markup
    - _Requirements: 5.2_

  - [ ] 8.3 Add accessibility audit step
    - Run axe-core against `Header.html` at 375px, 768px, and 1280px viewports; fail on any WCAG 2.1 AA violation
    - _Requirements: 4.7, 5.3_

  - [ ] 8.4 Add snapshot test step
    - Capture screenshots of `Header.html` at 375px, 768px, and 1280px; fail if pixel diff > 0.1%; surface diff image as build artifact
    - _Requirements: 5.4, 5.5_

  - [ ] 8.5 Add property test step
    - Run fast-check property suite (Properties 1–8); minimum 100 iterations per property; fail pipeline on any counterexample
    - _Requirements: 5.6_

- [ ] 9. Final checkpoint — Ensure all tests pass and CI steps are wired correctly
Implement the `site-header` component as a responsive, accessible, BEM-structured HTML/CSS component with a vanilla JS touch-target test script. Tasks build incrementally from file scaffolding through full integration with BottomNav and Sidebar, ending with documentation updates.

## Tasks

- [ ] 1. Scaffold component files and base HTML structure
  - Create `frontend/components/header/` directory with `Header.html`, `Header.css`, `Header.test.html`, and `header.test.js`
  - Write `Header.html` with `<header role="banner" aria-label="Site header" class="site-header">`, skip-navigation link as first focusable child, three BEM child regions (`site-header__logo`, `site-header__title`, `site-header__actions`), decorative SVGs with `aria-hidden="true"`, icon-only buttons with `aria-label`, and notification badge with `aria-label="N new notifications"`
  - Add HTML comment referencing BottomNav and Sidebar co-existence
  - Link only `../../styles/responsive.css` and `./Header.css` as stylesheets; no inline styles
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.3, 5.4, 5.5, 9.5_

- [ ] 2. Implement core CSS — block, elements, and design token compliance
  - [ ] 2.1 Write `Header.css` block (`.site-header`) with `position: fixed`, `top: 0`, `z-index: var(--z-fixed)`, background `var(--color-neutral-100)`, bottom border `var(--color-neutral-300)`, and safe-area padding using `calc(var(--space-2) + env(safe-area-inset-top, 0))`, `calc(var(--space-4) + env(safe-area-inset-left, 0))`, `calc(var(--space-4) + env(safe-area-inset-right, 0))`
    - Define `--header-height: 56px` as a local custom property on `.site-header`
    - Use only `var(--color-*)`, `var(--space-*)`, `var(--font-size-*)`, `var(--transition-fast)`, `var(--radius-*)`, `var(--z-fixed)` tokens; no hardcoded hex, rgb, hsl, px, or rem literals outside safe-area `calc()` expressions
    - _Requirements: 2.4, 2.5, 3.1, 3.2, 3.3, 7.1, 7.2, 7.3, 7.4, 9.3_

  - [ ]* 2.2 Write property test for design token exclusivity (Property 3)
    - **Property 3: Design token exclusivity**
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.4**
    - Use `fc.constantFrom(...cssDeclarations)` to sample declarations from parsed `Header.css` and assert no hardcoded color/spacing/font-size/transition literals

  - [ ] 2.3 Write BEM element rules: `__skip-link` (visually hidden until focused), `__logo`, `__brand` (`var(--color-deep-navy)`, `var(--font-size-lg)`), `__title`, `__actions` (flex row, `gap: var(--space-2)`), `__icon-btn` (min 44×44px via padding, `var(--radius-md)`, `var(--transition-fast)` hover), `__badge` (`var(--color-error-red)` background, `var(--font-size-xs)`, `var(--radius-full)`)
    - Add `__icon-btn--active` modifier using `var(--color-primary-blue)`
    - _Requirements: 1.2, 4.1, 4.2, 4.3, 5.1_

  - [ ]* 2.4 Write property test for touch target minimum size (Property 1)
    - **Property 1: Touch target minimum size**
    - **Validates: Requirements 4.1, 4.2**
    - Use `fc.integer({ min: 320, max: 1920 })` to generate viewport widths; render header and assert all interactive elements have `getBoundingClientRect()` width ≥ 44 and height ≥ 44

  - [ ]* 2.5 Write property test for touch target spacing (Property 2)
    - **Property 2: Touch target spacing**
    - **Validates: Requirements 4.3**
    - Use `fc.array(fc.record({...}))` to generate adjacent button pairs and assert gap between bounding rectangles ≥ 8px

- [ ] 3. Implement responsive overrides and layout offset
  - [ ] 3.1 Add `@media (min-width: 768px)` block: update `--header-height` to `64px`, offset header left edge by `240px` (tablet sidebar width) so header does not overlap sidebar
    - Add `@media (min-width: 1024px)` block: offset left edge by `280px` (desktop sidebar width)
    - Add `.has-header` rule: `padding-top: var(--header-height)`; on mobile also account for BottomNav bottom padding
    - _Requirements: 2.1, 2.2, 9.1, 9.2, 9.3, 9.4_

  - [ ]* 3.2 Write property test for header visibility and positioning across breakpoints (Property 8)
    - **Property 8: Header visibility and positioning across breakpoints**
    - **Validates: Requirements 2.1, 2.2, 9.2**
    - Use `fc.integer({ min: 320, max: 1920 })` to generate viewport widths; verify `position: fixed`, `top: 0`, `display !== none` on mobile; verify left offset ≥ 240px on tablet/desktop

  - [ ]* 3.3 Write property test for layout offset correctness (Property 5)
    - **Property 5: Layout offset correctness**
    - **Validates: Requirements 9.1, 9.4**
    - Use `fc.integer({ min: 40, max: 120 })` to generate `--header-height` values; verify `.has-header` computed `padding-top` equals the token value

- [ ] 4. Implement accessibility styles and reduced motion
  - [ ] 4.1 Add focus indicator rule for all interactive elements inside `.site-header`: `outline: 2px solid var(--color-primary-blue); outline-offset: 2px` on `:focus-visible`
    - Add `@media (prefers-reduced-motion: reduce)` block setting all `transition-duration` and `animation-duration` within `.site-header` to `0.01ms`
    - _Requirements: 5.2, 6.1, 6.2, 6.3_

  - [ ]* 4.2 Write property test for focus indicator on interactive elements (Property 9)
    - **Property 9: Focus indicator on interactive elements**
    - **Validates: Requirements 5.2**
    - Use `fc.constantFrom(...interactiveElements)` to enumerate interactive elements; programmatically focus each and assert computed `outline` is `2px solid var(--color-primary-blue)` with `outline-offset: 2px`

  - [ ]* 4.3 Write property test for reduced motion suppression (Property 6)
    - **Property 6: Reduced motion suppression**
    - **Validates: Requirements 6.1, 6.2**
    - Use `fc.boolean()` to toggle `prefers-reduced-motion` media query state; verify all transition/animation durations within `.site-header` resolve to ≤ 0.01ms

- [ ] 5. Implement safe-area padding and badge accessibility
  - [ ]* 5.1 Write property test for safe-area padding correctness (Property 4)
    - **Property 4: Safe-area padding correctness**
    - **Validates: Requirements 3.1, 3.2, 3.3**
    - Use `fc.nat({ max: 60 })` mapped to env mock to generate inset values including 0; verify computed `padding-top` = `var(--space-2)` + inset, `padding-left/right` = `var(--space-4)` + respective inset

  - [ ]* 5.2 Write property test for notification badge accessibility label (Property 7)
    - **Property 7: Notification badge accessibility label**
    - **Validates: Requirements 5.5**
    - Use `fc.integer({ min: 0, max: 999 })` to generate notification counts; verify badge `aria-label` contains the count string and the word "notifications"

- [ ] 6. Implement automated touch-target test script
  - Write `frontend/components/header/header.test.js` as a vanilla JS script (no framework) that queries all interactive elements within `.site-header`, measures `getBoundingClientRect()`, asserts `width >= 44` and `height >= 44`, logs pass/fail with element references, and exits non-zero on any failure
  - Mirror the pattern from `frontend/docs/TESTING_GUIDE.md`
  - _Requirements: 8.2_

- [ ] 7. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 8. Write manual browser test cases
  - Write `frontend/components/header/Header.test.html` documenting manual test cases for: visual rendering at 375px, 768px, 1280px; skip-link visibility and focus; active state indicator; badge with and without count; Sidebar co-existence at tablet/desktop; BottomNav co-existence at mobile; reduced motion transition suppression; keyboard tab order through all interactive elements
  - _Requirements: 8.1_

  - [ ]* 8.1 Write property test for decorative SVG aria-hidden (Property 10)
    - **Property 10: Decorative SVG aria-hidden**
    - **Validates: Requirements 5.3**
    - Use `fc.constantFrom(...svgElements)` to enumerate SVG elements inside `.site-header`; assert each decorative SVG carries `aria-hidden="true"`

  - [ ]* 8.2 Write property test for icon-only button aria-label (Property 11)
    - **Property 11: Icon-only button aria-label**
    - **Validates: Requirements 5.4**
    - Use `fc.constantFrom(...iconButtons)` to enumerate icon-only buttons; assert each carries a non-empty `aria-label`

  - [ ]* 8.3 Write property test for no duplicate CSS selectors (Property 12)
    - **Property 12: No duplicate CSS selectors**
    - **Validates: Requirements 8.3**
    - Parse all selector strings from `Header.css`; use `fc.constantFrom(...selectors)` and assert each selector string is unique across the stylesheet

- [ ] 9. Update documentation
  - Add a `### Header (All Viewports)` section to `frontend/docs/RESPONSIVE_DESIGN_GUIDE.md` with a usage example, breakpoint behavior table (mobile fixed top / tablet+desktop offset from sidebar), and accessibility notes
  - Add a `#### Header (All Viewports)` entry under the Components section in `frontend/README.md` with a minimal usage snippet showing `<header class="site-header">` and `<main class="has-header">`
  - _Requirements: 8.5, 8.6_

- [ ] 10. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Each task references specific requirements for traceability
- Property tests use fast-check + Playwright (headless browser with CSS support)
- The `.has-header` class is only meaningful on mobile viewports; on tablet/desktop the header is hidden and the offset resets to 0
- Snapshot baselines are created on first CI run; subsequent runs diff against them
- Property tests use fast-check; each test runs a minimum of 100 iterations
- Tag format for property tests: `Feature: frontend-header-responsive-styling, Property N: <property_text>`
- `header.test.js` must exit non-zero on any failing assertion for CI integration
- No new global design tokens are introduced; `--header-height` is scoped to `.site-header`
