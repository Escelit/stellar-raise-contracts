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
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Each task references specific requirements for traceability
- Property tests use fast-check + Playwright (headless browser with CSS support)
- The `.has-header` class is only meaningful on mobile viewports; on tablet/desktop the header is hidden and the offset resets to 0
- Snapshot baselines are created on first CI run; subsequent runs diff against them
