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
