# Requirements Document

## Introduction

This feature delivers responsive styling updates to the Stellar Raise frontend header and navigation components. The goal is to improve CI/CD pipeline integration and developer experience by establishing clear, testable CSS standards for the header area across all breakpoints (mobile < 768px, tablet 768–1024px, desktop > 1024px). The work covers the page-level header element (brand identity bar), its interaction with the existing BottomNav (mobile) and Sidebar (tablet/desktop) navigation components, and the tooling/documentation needed to keep the system maintainable.

## Glossary

- **Header**: The top-of-page `<header>` element containing the brand logo, app name, and optional contextual actions (e.g., wallet connect button).
- **BottomNav**: The fixed bottom navigation component rendered on viewports narrower than 768px (`frontend/components/navigation/BottomNav.css`).
- **Sidebar**: The fixed left-side navigation component rendered on viewports 768px and wider (`frontend/components/navigation/Sidebar.css`).
- **Design_System**: The set of CSS custom properties, breakpoints, and utility classes defined in `frontend/styles/responsive.css` and `frontend/styles/utilities.css`.
- **CI_Pipeline**: The automated build, lint, and test workflow defined in `.github/` that runs on every pull request.
- **Linter**: The CSS/HTML static analysis tool integrated into the CI_Pipeline.
- **Snapshot_Test**: A visual regression test that captures a rendered screenshot at a defined viewport and compares it against a stored baseline image.
- **Accessibility_Audit**: An automated check (e.g., axe-core) that validates WCAG 2.1 AA compliance.
- **Touch_Target**: An interactive element that must meet the 44×44 px minimum size defined in `--touch-target-min`.
- **Safe_Area**: Device-specific insets exposed via `env(safe-area-inset-*)` for notched/rounded-corner screens.
This feature introduces a responsive Header component for the Stellar Raise crowdfunding dApp frontend. The Header provides a top-of-page navigation bar that is visible on mobile viewports (< 768px) where the Sidebar is hidden, and optionally on all breakpoints as a page-level context bar. It must integrate cleanly with the existing design system (responsive.css, utilities.css, BEM naming conventions), meet accessibility standards, and be structured to support CI/CD pipelines and developer experience goals: easy to review, well-documented, and covered by automated tests.

## Glossary

- **Header**: The new `<header>` HTML component defined in `frontend/components/header/Header.html` and styled in `frontend/components/header/Header.css`.
- **Design_System**: The existing CSS design tokens and utility classes defined in `frontend/styles/responsive.css` and `frontend/styles/utilities.css`.
- **Breakpoint_Mobile**: Viewport widths less than 768px, as defined by `--breakpoint-mobile` in the Design_System.
- **Breakpoint_Tablet**: Viewport widths between 768px and 1024px, as defined by `--breakpoint-tablet` in the Design_System.
- **Breakpoint_Desktop**: Viewport widths greater than 1024px.
- **Touch_Target**: An interactive element meeting the 44×44px minimum size requirement per WCAG 2.5.5.
- **Safe_Area_Inset**: Device-specific padding applied via `env(safe-area-inset-*)` CSS functions to avoid overlap with notches and rounded corners.
- **BEM**: Block Element Modifier CSS naming convention used throughout the existing component library.
- **CI_Pipeline**: The automated build, lint, and test workflow triggered on pull requests.
- **Reduced_Motion**: The `prefers-reduced-motion: reduce` media query preference indicating the user has requested minimal animation.

---

## Requirements

### Requirement 1: Responsive Header Component

**User Story:** As a user, I want a header that adapts its layout and content to my device's screen size, so that the brand identity and key actions are always visible and usable.

#### Acceptance Criteria

1. THE Header SHALL render a logo mark and the "Stellar Raise" brand name on all viewports.
2. WHEN the viewport width is less than 768px, THE Header SHALL display in a compact single-row layout with a height between 48px and 64px.
3. WHEN the viewport width is between 768px and 1023px, THE Header SHALL display in a full-row layout with a height between 56px and 72px and SHALL be hidden when the Sidebar is visible, to avoid duplicate brand presentation.
4. WHEN the viewport width is 1024px or greater, THE Header SHALL display in a full-row layout with a height between 64px and 80px and SHALL be hidden when the Sidebar is visible.
5. THE Header SHALL use only CSS custom properties defined in `frontend/styles/responsive.css` for colors, spacing, typography, and z-index values.
6. WHILE the page is scrolled beyond 0px, THE Header SHALL remain fixed at the top of the viewport using `position: fixed` and `z-index: var(--z-fixed)`.
7. THE Header SHALL apply `padding-top: var(--safe-area-inset-top)` to account for Safe_Area insets on notched devices.

---

### Requirement 2: Header and Navigation Co-existence

**User Story:** As a user, I want the header and navigation components to coexist without overlapping page content, so that I can read and interact with all content without obstruction.

#### Acceptance Criteria

1. WHEN the viewport width is less than 768px, THE Header SHALL be visible and THE BottomNav SHALL be visible, and the main content area SHALL have `padding-top` equal to the Header height plus `var(--safe-area-inset-top)` and `padding-bottom` equal to 72px plus `var(--safe-area-inset-bottom)`.
2. WHEN the viewport width is 768px or greater, THE Sidebar SHALL be visible and THE Header SHALL be hidden, and the main content area SHALL have `margin-left` equal to the Sidebar width (240px on tablet, 280px on desktop).
3. THE Design_System SHALL expose a CSS custom property `--header-height` with a value appropriate for each breakpoint so that layout offset calculations reference a single source of truth.
4. IF the Header and BottomNav are both rendered simultaneously on a viewport narrower than 768px, THEN THE Header SHALL not overlap the BottomNav and THE BottomNav SHALL not overlap the Header.

---

### Requirement 3: Design System Token Compliance

**User Story:** As a developer, I want all header styles to use the established design tokens, so that visual consistency is maintained and future theme changes propagate automatically.

#### Acceptance Criteria

1. THE Header SHALL use `var(--color-neutral-100)` as its background color and `var(--color-deep-navy)` as its primary text color.
2. THE Header SHALL use `var(--font-family-primary)` for all text and `var(--font-size-lg)` for the brand name.
3. THE Header SHALL use `var(--transition-fast)` for any interactive state transitions (e.g., shadow on scroll).
4. THE Header SHALL use `var(--shadow-sm)` at rest and `var(--shadow-md)` when the page is scrolled beyond 0px.
5. THE Design_System SHALL define `--header-height-mobile`, `--header-height-tablet`, and `--header-height-desktop` custom properties in `frontend/styles/responsive.css`.
6. IF a CSS rule in the Header stylesheet references a hard-coded color, spacing, or font value that has an equivalent Design_System token, THEN THE Linter SHALL report a violation and THE CI_Pipeline SHALL fail.

---

### Requirement 4: Accessibility Compliance

**User Story:** As a user relying on assistive technology or keyboard navigation, I want the header to be fully accessible, so that I can navigate the application without barriers.

#### Acceptance Criteria

1. THE Header SHALL include a `<header>` landmark element with `role="banner"`.
2. THE Header SHALL include a skip-navigation link as its first focusable child, with visible focus styling, that moves keyboard focus to the main content area when activated.
3. ALL interactive elements within the Header SHALL meet the Touch_Target minimum of 44×44 px.
4. ALL text within the Header SHALL meet a color contrast ratio of at least 4.5:1 against the background color (WCAG 2.1 AA, criterion 1.4.3).
5. THE Header SHALL expose a visible focus indicator on all interactive elements using `outline: 2px solid var(--color-primary-blue); outline-offset: 2px`.
6. WHEN the `prefers-reduced-motion` media feature is set to `reduce`, THE Header SHALL apply transition durations of 0.01ms to all animated properties.
7. THE Accessibility_Audit SHALL report zero violations against WCAG 2.1 AA criteria for the Header component.

---

### Requirement 5: CI/CD Integration

**User Story:** As a developer, I want header styling changes to be automatically validated in the CI pipeline, so that regressions are caught before merging.

#### Acceptance Criteria

1. THE CI_Pipeline SHALL execute a CSS lint step that validates all files in `frontend/components/navigation/` and `frontend/styles/` against the project's linting rules on every pull request.
2. THE CI_Pipeline SHALL execute an HTML validation step that checks all HTML files in `frontend/components/navigation/` for well-formed markup on every pull request.
3. THE CI_Pipeline SHALL execute the Accessibility_Audit against the Header component at the 375px, 768px, and 1280px viewports on every pull request.
4. THE CI_Pipeline SHALL execute Snapshot_Tests for the Header component at the 375px, 768px, and 1280px viewports on every pull request.
5. WHEN a Snapshot_Test detects a visual difference greater than 0.1% of changed pixels, THE CI_Pipeline SHALL fail and SHALL surface the diff image as a build artifact.
6. IF any CI_Pipeline step fails, THEN THE CI_Pipeline SHALL block the pull request from merging and SHALL report the specific failing step and error message.

---

### Requirement 6: Developer Experience and Documentation

**User Story:** As a developer, I want clear documentation and tooling for the header component, so that I can implement, test, and review changes efficiently.

#### Acceptance Criteria

1. THE Header component SHALL be documented in `frontend/docs/RESPONSIVE_DESIGN_GUIDE.md` with usage examples, breakpoint behavior, and a list of all CSS custom properties it consumes.
2. THE Header component SHALL include inline CSS comments that identify each breakpoint block and reference the corresponding Design_System token names.
3. THE TESTING_GUIDE SHALL be updated in `frontend/docs/TESTING_GUIDE.md` to include Header-specific test cases covering all three breakpoints, keyboard navigation, and safe area inset behavior.
4. THE Header HTML file SHALL include a self-contained demo page at `frontend/components/navigation/Header.html` that renders the component in isolation with all required stylesheet dependencies.
5. WHERE a developer adds a new interactive element to the Header, THE Design_System SHALL provide a utility class (e.g., `.touch-target`) that enforces the 44×44 px Touch_Target minimum without requiring custom CSS.
6. THE Header component files SHALL follow the existing naming convention: `Header.css` and `Header.html` under `frontend/components/navigation/`.
### Requirement 1: Header Component Structure

**User Story:** As a developer, I want a well-structured Header HTML component, so that I can integrate it into pages consistently and review it easily in pull requests.

#### Acceptance Criteria

1. THE Header SHALL be implemented as a `<header>` element with the BEM block class `site-header` in `frontend/components/header/Header.html`.
2. THE Header SHALL include a logo area, a page title or brand name slot, and a right-side action area within a single flex row.
3. THE Header SHALL reference only `../../styles/responsive.css` and `./Header.css` as stylesheet dependencies, with no inline styles.
4. THE Header SHALL include a `role="banner"` attribute and an `aria-label` describing its purpose.
5. THE Header SHALL include a skip-navigation link as the first focusable child element, pointing to the main content landmark.

---

### Requirement 2: Responsive Layout Across Breakpoints

**User Story:** As a user, I want the Header to adapt its layout across mobile, tablet, and desktop viewports, so that navigation remains usable on any device.

#### Acceptance Criteria

1. WHILE the viewport width is less than 768px, THE Header SHALL be visible and positioned as a fixed bar at the top of the viewport.
2. WHILE the viewport width is 768px or greater, THE Header SHALL remain visible and co-exist with the Sidebar without overlapping the Sidebar's fixed left position.
3. WHEN the viewport width crosses the 768px breakpoint, THE Header SHALL reflow its layout without causing a Cumulative Layout Shift score above 0.1.
4. THE Header SHALL use only Design_System CSS custom properties for all color, spacing, typography, and z-index values.
5. THE Header SHALL apply `z-index: var(--z-fixed)` so it layers correctly above page content and below modals.

---

### Requirement 3: Safe Area and Notch Support

**User Story:** As a user on a notched or rounded-corner device, I want the Header to respect device safe areas, so that content is not obscured by hardware cutouts.

#### Acceptance Criteria

1. THE Header SHALL apply `padding-top: calc(var(--space-2) + var(--safe-area-inset-top))` to its top padding on all viewports.
2. THE Header SHALL apply `padding-left: calc(var(--space-4) + var(--safe-area-inset-left))` and `padding-right: calc(var(--space-4) + var(--safe-area-inset-right))` to its horizontal padding.
3. WHEN `env(safe-area-inset-top)` resolves to zero, THE Header SHALL fall back to `var(--space-2)` top padding without visual regression.

---

### Requirement 4: Touch Target Compliance

**User Story:** As a mobile user, I want all interactive elements in the Header to be large enough to tap accurately, so that I can navigate without mis-taps.

#### Acceptance Criteria

1. THE Header SHALL ensure every interactive child element (links, buttons, icon buttons) has a minimum rendered height of 44px and minimum rendered width of 44px.
2. WHEN an interactive element's visual size is smaller than 44×44px, THE Header SHALL expand its Touch_Target using padding or a `::after` pseudo-element technique consistent with the `.touch-target-expand` pattern in the Design_System.
3. THE Header SHALL maintain a minimum spacing of 8px between adjacent Touch_Target areas.

---

### Requirement 5: Accessibility

**User Story:** As a user relying on assistive technology, I want the Header to be fully accessible, so that I can navigate the application using a screen reader or keyboard.

#### Acceptance Criteria

1. THE Header SHALL expose all interactive elements in the DOM tab order with a logical sequence from left to right.
2. WHEN a user navigates to an interactive Header element using the keyboard, THE Header SHALL display a visible focus indicator using `outline: 2px solid var(--color-primary-blue)` with `outline-offset: 2px`.
3. THE Header SHALL mark all decorative SVG icons with `aria-hidden="true"`.
4. THE Header SHALL provide an `aria-label` on every icon-only button that describes the button's action.
5. WHEN the Header contains a notification badge, THE Header SHALL include an `aria-label` on the badge element that states the count and context (e.g., "3 new notifications").

---

### Requirement 6: Reduced Motion Support

**User Story:** As a user who has enabled reduced motion in their OS settings, I want Header animations to be suppressed, so that I am not affected by motion that could cause discomfort.

#### Acceptance Criteria

1. WHILE the `prefers-reduced-motion: reduce` media query is active, THE Header SHALL set all transition durations to 0.01ms.
2. WHILE the `prefers-reduced-motion: reduce` media query is active, THE Header SHALL set all animation durations to 0.01ms.
3. THE Header SHALL not rely on animation or transition to convey state changes; state changes SHALL also be communicated through color or text.

---

### Requirement 7: Design System Token Compliance

**User Story:** As a developer, I want the Header CSS to use only established design tokens, so that future theme changes propagate automatically and the component is easy to review.

#### Acceptance Criteria

1. THE Header SHALL use `var(--color-*)` tokens for all color values and SHALL NOT contain hardcoded hex, rgb, or hsl color values.
2. THE Header SHALL use `var(--space-*)` tokens for all margin and padding values and SHALL NOT contain hardcoded pixel or rem spacing values outside of safe-area calculations.
3. THE Header SHALL use `var(--font-size-*)` tokens for all font-size declarations.
4. THE Header SHALL use `var(--transition-fast)` or `var(--transition-base)` for all CSS transitions.
5. IF a required design token does not exist in the Design_System, THEN THE Header SHALL define the token as a local CSS custom property within the `site-header` block scope and document it with an inline comment.

---

### Requirement 8: CI/CD Integration and Developer Experience

**User Story:** As a developer, I want the Header component to include automated tests and linting-friendly code, so that the CI_Pipeline can validate correctness on every pull request.

#### Acceptance Criteria

1. THE Header SHALL be accompanied by a test file `frontend/components/header/Header.test.html` that documents manual browser test cases covering all breakpoints, touch targets, and accessibility checks defined in this document.
2. THE Header SHALL be accompanied by a JavaScript test script `frontend/components/header/header.test.js` that programmatically verifies Touch_Target sizes for all interactive elements using the pattern established in `frontend/docs/TESTING_GUIDE.md`.
3. THE Header CSS file SHALL contain no duplicate selector blocks, as verified by CSS linting.
4. THE Header HTML file SHALL pass HTML validation with zero errors when checked against the W3C HTML validator rules.
5. THE Header SHALL be documented in `frontend/docs/RESPONSIVE_DESIGN_GUIDE.md` with a usage example, breakpoint behavior table, and accessibility notes.
6. THE Header SHALL be listed in `frontend/README.md` under the Components section with a minimal usage snippet.

---

### Requirement 9: Integration with Existing Navigation Components

**User Story:** As a developer, I want the Header to integrate with the existing BottomNav and Sidebar components without layout conflicts, so that the full navigation system works cohesively.

#### Acceptance Criteria

1. WHILE the viewport width is less than 768px, THE Header SHALL co-exist with the BottomNav by ensuring the main content area accounts for both the Header height at the top and the BottomNav height at the bottom.
2. WHILE the viewport width is 768px or greater, THE Header SHALL co-exist with the Sidebar by not overlapping the Sidebar's 240px (tablet) or 280px (desktop) left offset.
3. THE Header SHALL expose a CSS custom property `--header-height` set to the Header's rendered height so that other components can reference it for layout offset calculations.
4. WHEN the `has-header` class is applied to the `<main>` element, THE Header SHALL cause the main content area to receive a `padding-top` equal to `var(--header-height)`.
5. THE Header component file SHALL include an HTML comment referencing the BottomNav and Sidebar components it is designed to work alongside.
