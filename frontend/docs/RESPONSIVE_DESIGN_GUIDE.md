# Stellar Raise - Responsive Design System Guide

## Overview

This guide documents the responsive design framework for the Stellar Raise crowdfunding dApp. The system maintains the "Modern" and "Trusted" brand identity across all screen sizes while ensuring optimal usability on mobile, tablet, and desktop devices.

## Table of Contents

1. [Breakpoint System](#breakpoint-system)
2. [Grid System](#grid-system)
3. [Navigation Patterns](#navigation-patterns)
4. [Modal Patterns](#modal-patterns)
5. [Form Design](#form-design)
6. [Data Display](#data-display)
7. [Touch Targets](#touch-targets)
8. [Typography](#typography)
9. [Accessibility](#accessibility)
10. [Testing Checklist](#testing-checklist)

---

## Breakpoint System

### Defined Breakpoints

```css
Mobile:   < 768px   (Primary focus for quick contributions)
Tablet:   768-1024px (Enhanced layout for project lists)
Desktop:  > 1024px   (Full-screen management interface)
```

### CSS Custom Properties

```css
--breakpoint-mobile: 768px;
--breakpoint-tablet: 1024px;
```

### Usage in Media Queries

```css
/* Mobile-first approach */
.element {
  /* Mobile styles (default) */
}

@media (min-width: 768px) {
  .element {
    /* Tablet styles */
  }
}

@media (min-width: 1024px) {
  .element {
    /* Desktop styles */
  }
}
```

---

## Grid System

### Container Widths

- **Mobile**: 100% width with 16px gutters
- **Tablet**: 768px max-width with 24px gutters
- **Desktop**: 1280px max-width with 32px gutters

### Grid Columns

- **Mobile**: 4 columns
- **Tablet**: 8 columns
- **Desktop**: 12 columns

### Example Usage

```html
<div class="container">
  <div class="grid">
    <div class="col-span-full col-span-md-4 col-span-lg-3">
      <!-- Content spans full width on mobile, 4 cols on tablet, 3 cols on desktop -->
    </div>
  </div>
</div>
```

---

## Navigation Patterns

### Bottom Navigation (Mobile < 768px)

**Location**: Fixed at bottom of screen  
**Items**: Home, Savings, Activity, Profile  
**Features**:
- 44x44px minimum touch targets
- Active state indicator (blue bar at top)
- Badge support for notifications
- Safe area inset support for notched devices

**Implementation**:
```html
<nav class="bottom-nav">
  <div class="bottom-nav__container">
    <a href="/home" class="bottom-nav__item bottom-nav__item--active">
      <!-- Icon and label -->
    </a>
  </div>
</nav>
```

**Content Adjustment**:
```html
<main class="has-bottom-nav">
  <!-- Content automatically padded for bottom nav -->
</main>
```

### Site Header (Mobile < 768px)

**Location**: Fixed at top of screen  
**Height**: 48px (mobile), hidden on tablet/desktop  
**Features**:
- Brand logo + "Stellar Raise" wordmark on all mobile viewports
- Skip-navigation link as first focusable child (WCAG 2.4.1)
- Elevated shadow (`--shadow-md`) when page is scrolled beyond 0 px
- Safe-area inset support for notched devices
- Contextual actions slot (e.g. wallet-connect button)

**Breakpoint behaviour**:

| Viewport | Header | BottomNav | Sidebar |
|---|---|---|---|
| < 768 px (mobile) | ✓ Visible | ✓ Visible | ✗ Hidden |
| 768–1023 px (tablet) | ✗ Hidden | ✗ Hidden | ✓ 240 px |
| ≥ 1024 px (desktop) | ✗ Hidden | ✗ Hidden | ✓ 280 px |

**Implementation**:
```html
<!-- Link stylesheets in this order -->
<link rel="stylesheet" href="styles/responsive.css" />
<link rel="stylesheet" href="components/navigation/Header.css" />

<header class="site-header" role="banner">
  <a href="#main-content" class="site-header__skip-link">Skip to main content</a>
  <div class="site-header__inner">
    <a href="/" class="site-header__brand" aria-label="Stellar Raise — home">
      <svg class="site-header__logo" aria-hidden="true"><!-- logo --></svg>
      <span class="site-header__brand-name">Stellar Raise</span>
    </a>
    <div class="site-header__actions">
      <!-- e.g. wallet-connect button -->
    </div>
  </div>
</header>

<!-- Offset main content so it is never obscured by the fixed header -->
<main id="main-content" class="has-header has-bottom-nav" tabindex="-1">
  <!-- page content -->
</main>
```

**Scroll shadow toggle** (inline script):
```js
const header = document.querySelector('.site-header');
window.addEventListener('scroll', () => {
  header.classList.toggle('site-header--scrolled', window.scrollY > 0);
}, { passive: true });
```

**CSS custom properties consumed by `Header.css`**:

| Token | Usage |
|---|---|
| `--header-height-mobile` | Height at < 768 px (48 px) |
| `--header-height-tablet` | Height at 768–1023 px (56 px) |
| `--header-height-desktop` | Height at ≥ 1024 px (64 px) |
| `--header-height` | Per-breakpoint alias for layout offsets |
| `--color-neutral-100` | Header background |
| `--color-deep-navy` | Brand name text colour |
| `--color-primary-blue` | Focus indicator, logo fill |
| `--font-family-primary` | All header text |
| `--font-size-lg` | Brand name font size |
| `--shadow-sm` | Resting shadow |
| `--shadow-md` | Scrolled shadow |
| `--transition-fast` | Shadow transition on scroll |
| `--z-fixed` | z-index (same layer as BottomNav/Sidebar) |
| `--safe-area-inset-top` | Top padding for notched devices |
| `--space-4` | Horizontal padding |
| `--touch-target-min` | Minimum 44 px for interactive elements |

**Co-existence edge case — `.has-header` + `.has-sidebar`**:  
If both classes are applied to the same `<main>` element, the header's `padding-top` and the sidebar's `margin-left` will both apply. This is harmless but redundant on tablet/desktop because the header is hidden and `.has-header` resets `padding-top` to `0` at ≥ 768 px. No corrective action is needed.

---

### Sidebar Navigation (Tablet & Desktop ≥ 768px)

**Location**: Fixed on left side  
**Width**: 240px (tablet), 280px (desktop)  
**Features**:
- Persistent visibility
- Security status indicator
- Version information
- Notification badges

**Implementation**:
```html
<aside class="sidebar">
  <!-- Sidebar content -->
</aside>

<main class="has-sidebar">
  <!-- Content automatically offset for sidebar -->
</main>
```

---

### Header (Mobile < 768px)

**Location**: Fixed at top of screen  
**Height**: 48px (mobile), token-driven via `--header-height`  
**Visibility**: Mobile only — hidden at ≥ 768px (sidebar provides brand identity)

**Breakpoint behaviour**:

| Viewport | Header | BottomNav | Sidebar |
|----------|--------|-----------|---------|
| < 768px (mobile) | ✓ visible | ✓ visible | ✗ hidden |
| 768–1023px (tablet) | ✗ hidden | ✗ hidden | ✓ 240px |
| ≥ 1024px (desktop) | ✗ hidden | ✗ hidden | ✓ 280px |

**CSS custom properties consumed**:

| Token | Usage |
|-------|-------|
| `--header-height-mobile` | Height at < 768px (48px) |
| `--header-height-tablet` | Height at 768–1023px (56px) |
| `--header-height-desktop` | Height at ≥ 1024px (64px) |
| `--header-height` | Per-breakpoint alias for layout offsets |
| `--color-neutral-100` | Header background |
| `--color-deep-navy` | Brand name text |
| `--color-primary-blue` | Focus indicator, logo fill |
| `--font-family-primary` | All header text |
| `--font-size-lg` | Brand name font size |
| `--shadow-sm` | Resting shadow |
| `--shadow-md` | Scrolled shadow (`.site-header--scrolled`) |
| `--transition-fast` | Shadow transition |
| `--z-fixed` | z-index (same layer as BottomNav/Sidebar) |
| `--safe-area-inset-top` | Top padding for notched devices |
| `--space-4` | Horizontal padding |

**Implementation**:
```html
<header class="site-header" role="banner">
  <a href="#main-content" class="site-header__skip-link">Skip to main content</a>
  <div class="site-header__inner">
    <a href="/" class="site-header__brand" aria-label="Stellar Raise — home">
      <svg class="site-header__logo" aria-hidden="true" focusable="false"><!-- logo --></svg>
      <span class="site-header__brand-name">Stellar Raise</span>
    </a>
    <div class="site-header__actions"><!-- wallet button etc. --></div>
  </div>
</header>

<!-- Main content offset on mobile -->
<main id="main-content" class="has-header has-bottom-nav">
  <!-- content -->
</main>
```

**Content adjustment**:
```html
<!-- Mobile: offset for both header (top) and BottomNav (bottom) -->
<main class="has-header has-bottom-nav">...</main>

<!-- Tablet+: sidebar layout, no header offset needed -->
<main class="has-sidebar">...</main>
```

> **Edge case — `.has-header` + `.has-sidebar` on the same element**: Both classes can coexist without harm. On tablet/desktop `.has-header` resets `padding-top` to `0`, so only the sidebar `margin-left` applies. On mobile `.has-sidebar` has no effect (sidebar is hidden). No conflict occurs.

**Scroll shadow**: A small passive `scroll` listener toggles `.site-header--scrolled` to elevate the shadow from `--shadow-sm` to `--shadow-md`. See `Header.html` for the reference implementation.

---

## Modal Patterns

### Mobile Modal (< 768px)

**Behavior**: Full-screen takeover  
**Animation**: Slide up from bottom  
**Features**:
- Full viewport coverage
- Safe area inset support
- Scroll within modal body

### Tablet/Desktop Modal (≥ 768px)

**Behavior**: Centered with backdrop  
**Size**: 
- Tablet: 480px min-width
- Desktop: 600px min-width
- Max: 90vw x 90vh

**Animation**: Scale in with fade  
**Features**:
- Rounded corners (16px radius)
- Backdrop blur effect
- Click outside to close

### Modal Variants

#### Auth Modal
```html
<div class="modal modal--auth">
  <!-- Optimized for sign-in/sign-up forms -->
</div>
```

#### Confirmation Modal
```html
<div class="modal modal--confirm">
  <!-- Optimized for action confirmations -->
</div>
```

### Implementation

```html
<div class="modal modal--open">
  <div class="modal__backdrop"></div>
  <div class="modal__content">
    <header class="modal__header">
      <h2 class="modal__title">Title</h2>
      <button class="modal__close">×</button>
    </header>
    <div class="modal__body">
      <!-- Content -->
    </div>
    <footer class="modal__footer">
      <button class="btn btn--secondary">Cancel</button>
      <button class="btn btn--primary">Confirm</button>
    </footer>
  </div>
</div>
```

---

## Form Design

### Single-Column Layout

All forms use a single-column layout across all breakpoints for consistency and touch optimization.

### Input Specifications

- **Minimum height**: 44px (touch target compliance)
- **Padding**: 12px 16px
- **Font size**: 16px (prevents iOS zoom on focus)
- **Border**: 2px solid with color transitions

### Form Components

#### Text Input
```html
<div class="form__group">
  <label for="email" class="form__label">Email Address</label>
  <input type="email" id="email" class="form__input" />
</div>
```

#### Input with Prefix/Suffix
```html
<div class="form__input-group">
  <span class="form__input-prefix">XLM</span>
  <input type="number" class="form__input" />
</div>
```

#### Toggle Switch
```html
<div class="form__toggle">
  <span class="form__toggle-label">Enable 2FA</span>
  <label class="toggle-switch">
    <input type="checkbox" />
    <span class="toggle-slider"></span>
  </label>
</div>
```

### Button Specifications

- **Minimum height**: 44px
- **Padding**: 12px 24px
- **Font weight**: 600
- **Border radius**: 8px

#### Button Variants
```html
<button class="btn btn--primary">Primary Action</button>
<button class="btn btn--secondary">Secondary Action</button>
<button class="btn btn--danger">Danger Zone</button>
<button class="btn btn--outline">Outline Style</button>
```

---

## Data Display

### Responsive Tables

Tables transform into card layouts on mobile devices to prevent horizontal scrolling and data clipping.

#### Desktop View (≥ 768px)
Standard table with hover states

#### Mobile View (< 768px)
Each row becomes a card with label-value pairs

```html
<table class="responsive-table">
  <thead>
    <tr>
      <th>Date</th>
      <th>Amount</th>
      <th>Status</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td data-label="Date">2026-02-20</td>
      <td data-label="Amount">500 XLM</td>
      <td data-label="Status">Success</td>
    </tr>
  </tbody>
</table>
```

### Transaction Cards

Optimized card layout for financial data display.

```html
<article class="transaction-card">
  <div class="transaction-card__header">
    <div class="transaction-card__type">Campaign Contribution</div>
    <div class="transaction-card__amount transaction-card__amount--negative">
      -500 XLM
    </div>
  </div>
  <div class="transaction-card__details">
    <!-- Detail rows -->
  </div>
</article>
```

### Horizontal Scroll Lists

For campaign lists and similar content on mobile.

```html
<div class="scroll-list">
  <article class="scroll-list__item">
    <!-- Campaign card -->
  </article>
  <!-- More items -->
</div>
```

**Behavior**:
- Mobile: Horizontal scroll with touch momentum
- Tablet/Desktop: Grid layout (no scroll)

---

## Touch Targets

### Minimum Size Requirements

All interactive elements must meet **44x44px minimum** size (WCAG 2.5.5 compliance).

### Elements Covered

- Buttons
- Links
- Checkboxes
- Radio buttons
- Toggle switches
- Icon buttons
- Navigation items

### Implementation

```css
button,
a,
input[type="checkbox"],
input[type="radio"] {
  min-height: var(--touch-target-min); /* 44px */
  min-width: var(--touch-target-min);
}
```

### Touch Target Expansion

For visually small elements that need larger touch areas:

```html
<button class="touch-target-expand">
  <!-- Visual content -->
</button>
```

---

## Typography

### Font Family

**Space Grotesk** - Maintains technical, modern feel across all devices

```css
font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
```

### Fluid Typography Scale

Uses `clamp()` for responsive scaling without media queries:

```css
--font-size-xs:   clamp(0.75rem, 0.7rem + 0.25vw, 0.875rem);
--font-size-sm:   clamp(0.875rem, 0.825rem + 0.25vw, 1rem);
--font-size-base: clamp(1rem, 0.95rem + 0.25vw, 1.125rem);
--font-size-lg:   clamp(1.125rem, 1.05rem + 0.375vw, 1.5rem);
--font-size-xl:   clamp(1.25rem, 1.15rem + 0.5vw, 1.875rem);
--font-size-2xl:  clamp(1.5rem, 1.35rem + 0.75vw, 2.25rem);
--font-size-3xl:  clamp(1.875rem, 1.65rem + 1.125vw, 3rem);
```

### Usage

```html
<h1 class="text-3xl">Large Heading</h1>
<p class="text-base">Body text</p>
<span class="text-sm">Small text</span>
```

### Readability Requirements

- Minimum font size: 14px (0.875rem)
- Line height: 1.6 for body text
- Text must remain readable when zoomed to 200%

---

## Accessibility

### WCAG 2.1 AA Compliance

#### Color Contrast

All text must meet minimum contrast ratios:
- Normal text: 4.5:1
- Large text (18px+): 3:1
- UI components: 3:1

#### Focus Indicators

All interactive elements have visible focus states:

```css
:focus-visible {
  outline: 2px solid var(--color-primary-blue);
  outline-offset: 2px;
}
```

#### Keyboard Navigation

- All interactive elements are keyboard accessible
- Logical tab order maintained
- Skip links provided where appropriate

#### Screen Reader Support

```html
<!-- Proper ARIA labels -->
<button aria-label="Close modal">×</button>

<!-- Current page indication -->
<a href="/home" aria-current="page">Home</a>

<!-- Hidden decorative icons -->
<svg aria-hidden="true">...</svg>

<!-- Notification badges -->
<span class="badge" aria-label="3 new notifications">3</span>
```

### Reduced Motion

Respects user preferences for reduced motion:

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

### Safe Area Insets

Support for devices with notches and rounded corners:

```css
.safe-area-top {
  padding-top: calc(var(--space-4) + env(safe-area-inset-top));
}
```

---

## Brand Consistency

### Color Palette

```css
--color-primary-blue:   #0066FF  /* Primary actions, links */
--color-deep-navy:      #0A1929  /* Headings, primary text */
--color-success-green:  #00C853  /* Success states, positive values */
--color-error-red:      #FF3B30  /* Errors, negative values */
--color-warning-orange: #FF9500  /* Warnings, pending states */
```

### Security Indicators

The "Secured with End-to-End Encryption" status and version info must remain visible at all breakpoints:

- **Mobile**: In bottom nav footer or header
- **Tablet/Desktop**: In sidebar footer

```html
<div class="sidebar__security">
  <svg><!-- Shield icon --></svg>
  <span>Secured with E2E Encryption</span>
</div>
<div class="sidebar__version">v1.0.0</div>
```

---

## Testing Checklist

### Device Testing

- [ ] iPhone SE (375px width)
- [ ] iPhone 12/13/14 (390px width)
- [ ] iPhone 14 Pro Max (430px width)
- [ ] iPad (768px width)
- [ ] iPad Pro (1024px width)
- [ ] Desktop (1280px+ width)

### Browser Testing

- [ ] Chrome (mobile & desktop)
- [ ] Safari (iOS & macOS)
- [ ] Firefox (mobile & desktop)
- [ ] Edge (desktop)

### Orientation Testing

- [ ] Portrait mode (mobile & tablet)
- [ ] Landscape mode (mobile & tablet)
- [ ] Orientation change transitions

### Interaction Testing

- [ ] Touch interactions (tap, swipe, scroll)
- [ ] Keyboard navigation
- [ ] Mouse interactions (hover, click)
- [ ] Form input with mobile keyboards
- [ ] Autofill functionality

### Accessibility Testing

- [ ] VoiceOver (iOS)
- [ ] TalkBack (Android)
- [ ] NVDA (Windows)
- [ ] Color contrast validation
- [ ] Focus indicator visibility
- [ ] 200% zoom readability

### Performance Testing

- [ ] Layout shift metrics (CLS)
- [ ] Animation performance (60fps)
- [ ] Touch response time
- [ ] Modal open/close performance
- [ ] Scroll performance

### Visual Testing

- [ ] Safe area insets (notched devices)
- [ ] Status bar overlap
- [ ] Bottom nav spacing
- [ ] Modal backdrop blur
- [ ] Typography scaling
- [ ] Button touch targets
- [ ] Form input sizing

---

## Implementation Notes

### Viewport Meta Tag

Required in all HTML pages:

```html
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
```

### Mobile-First CSS

Always write mobile styles first, then enhance for larger screens:

```css
/* ✅ Correct */
.element {
  font-size: 14px; /* Mobile */
}

@media (min-width: 768px) {
  .element {
    font-size: 16px; /* Tablet+ */
  }
}

/* ❌ Incorrect */
.element {
  font-size: 16px; /* Desktop */
}

@media (max-width: 767px) {
  .element {
    font-size: 14px; /* Mobile */
  }
}
```

### Performance Optimization

- Use CSS transforms for animations (GPU-accelerated)
- Minimize layout shifts during responsive transitions
- Lazy load images and heavy components
- Use `will-change` sparingly for animated elements

---

## Component Library

### File Structure

```
frontend/
├── styles/
│   └── responsive.css          # Core design system (includes --header-height-* tokens)
├── components/
│   ├── navigation/
│   │   ├── BottomNav.css
│   │   ├── BottomNav.html
│   │   ├── Header.css          # Site-level header (mobile only)
│   │   ├── Header.css          # Site header (mobile-only fixed header)
│   │   ├── Header.html         # Self-contained demo page
│   │   ├── Sidebar.css
│   │   └── Sidebar.html
│   ├── modals/
│   │   ├── Modal.css
│   │   └── Modal.html
│   ├── forms/
│   │   └── Forms.css
│   └── tables/
│       ├── ResponsiveTable.css
│       └── ResponsiveTable.html
├── docs/
│   └── RESPONSIVE_DESIGN_GUIDE.md
└── index.html                  # Example implementation
```

### Usage

1. Include core responsive.css in all pages
2. Include component-specific CSS as needed
3. Follow HTML patterns from example files
4. Use utility classes for spacing and layout

---

## Support & Maintenance

### Browser Support

- **Modern browsers**: Full support
- **iOS Safari**: 13+
- **Android Chrome**: 80+
- **Desktop browsers**: Last 2 versions

### Known Issues

None at this time.

### Future Enhancements

- Dark mode support
- Additional breakpoint for large desktops (1440px+)
- Component library in React/Vue
- Storybook documentation

---

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [MDN Responsive Design](https://developer.mozilla.org/en-US/docs/Learn/CSS/CSS_layout/Responsive_Design)
- [Space Grotesk Font](https://fonts.google.com/specimen/Space+Grotesk)
- [CSS Grid Guide](https://css-tricks.com/snippets/css/complete-guide-grid/)

---

**Last Updated**: February 20, 2026  
**Version**: 1.0.0  
**Maintained by**: Stellar Raise Development Team
