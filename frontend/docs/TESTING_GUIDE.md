# Responsive Design Testing Guide

## Overview

This document provides comprehensive testing procedures for the Stellar Raise responsive design system. Follow these guidelines to ensure consistent quality across all devices and browsers.

---

## Quick Start Testing Checklist

### Essential Tests (Must Complete)

- [ ] Mobile viewport (375px) - iPhone SE
- [ ] Tablet viewport (768px) - iPad
- [ ] Desktop viewport (1280px+)
- [ ] Touch target sizes (all interactive elements ≥ 44x44px)
- [ ] Keyboard navigation (tab through all interactive elements)
- [ ] Screen reader (test with VoiceOver or NVDA)
- [ ] Color contrast (use browser DevTools)
- [ ] Orientation changes (portrait ↔ landscape)

---

## Device Testing Matrix

### Mobile Devices

#### iPhone Testing

| Device | Width | Height | Notes |
|--------|-------|--------|-------|
| iPhone SE | 375px | 667px | Smallest modern iPhone |
| iPhone 12/13/14 | 390px | 844px | Standard size |
| iPhone 14 Pro | 393px | 852px | Dynamic Island |
| iPhone 14 Pro Max | 430px | 932px | Largest iPhone |

**Test Cases:**
- Bottom navigation visibility and spacing
- Safe area insets (notch/Dynamic Island)
- Full-screen modals
- Form input sizing
- Touch target accessibility

#### Android Testing

| Device | Width | Height | Notes |
|--------|-------|--------|-------|
| Samsung Galaxy S21 | 360px | 800px | Common Android size |
| Google Pixel 5 | 393px | 851px | Stock Android |
| Samsung Galaxy S21 Ultra | 412px | 915px | Large Android |

**Test Cases:**
- Bottom navigation behavior
- Material Design compatibility
- Keyboard behavior with forms
- Back button handling

### Tablet Devices

| Device | Width | Height | Notes |
|--------|-------|--------|-------|
| iPad (9th gen) | 768px | 1024px | Standard iPad |
| iPad Air | 820px | 1180px | Larger iPad |
| iPad Pro 11" | 834px | 1194px | Pro model |
| iPad Pro 12.9" | 1024px | 1366px | Largest iPad |

**Test Cases:**
- Sidebar navigation appearance
- Modal centering and sizing
- Grid layout (8 columns)
- Table display (not cards)
- Form layout optimization

### Desktop

| Viewport | Notes |
|----------|-------|
| 1024px | Minimum desktop |
| 1280px | Standard laptop |
| 1440px | Large laptop |
| 1920px | Full HD monitor |
| 2560px | 2K monitor |

**Test Cases:**
- Sidebar navigation (280px width)
- 12-column grid layout
- Maximum container width (1280px)
- Hover states on interactive elements
- Modal sizing and centering

---

## Browser Testing

### Required Browsers

#### Mobile Browsers

- **iOS Safari** (iOS 13+)
  - Default browser on iPhone/iPad
  - Test safe area insets
  - Test form autofill
  - Test touch interactions

- **Chrome Mobile** (Android)
  - Most common Android browser
  - Test viewport behavior
  - Test form validation
  - Test PWA features

#### Desktop Browsers

- **Chrome** (latest 2 versions)
  - Primary development browser
  - Test all features
  - Use DevTools device emulation

- **Firefox** (latest 2 versions)
  - Test CSS Grid compatibility
  - Test flexbox behavior
  - Test accessibility features

- **Safari** (macOS, latest 2 versions)
  - Test WebKit-specific features
  - Test form styling
  - Test animations

- **Edge** (latest 2 versions)
  - Test Chromium compatibility
  - Test Windows-specific features

---

## Functional Testing

### Navigation Testing

#### Header Component (Mobile < 768px)

**Test Steps:**
1. Open `Header.html` at 375px viewport
2. Verify `.site-header` is visible and fixed at top
3. Measure computed height — must be 48px (`--header-height-mobile`)
4. Verify `box-shadow` equals `var(--shadow-sm)` at `scrollY = 0`
5. Scroll page to `scrollY = 1`; assert `.site-header--scrolled` class is present and shadow equals `var(--shadow-md)`
6. Resize to 768px; assert `.site-header` has `display: none`
7. Resize to 1280px; assert `.site-header` has `display: none`
8. Press Tab; assert first focused element is `.site-header__skip-link`
9. Press Enter on skip link; assert `document.activeElement.id === 'main-content'`
10. Simulate `env(safe-area-inset-top) = 44px`; assert header `padding-top` is `44px`
11. Run axe-core at 375px, 768px, 1280px; assert zero WCAG 2.1 AA violations

**Expected Results:**
- Header visible and fixed on mobile; hidden on tablet/desktop
- Height within spec: [48px, 64px] mobile, [56px, 72px] tablet, [64px, 80px] desktop
- Shadow elevates on scroll; resets when back at top
- Skip link is first focusable element and navigates to `#main-content`
- Safe area inset applied correctly on notched devices
- Zero accessibility violations
#### Site Header (Mobile < 768px)

**Test Steps:**
1. Open `frontend/components/navigation/Header.html` at 375 px viewport
2. Verify `.site-header` is visible and fixed at the top
3. Measure computed height — must be between 48 px and 64 px
4. Verify `box-shadow` equals `var(--shadow-sm)` at `scrollY = 0`
5. Scroll page to `scrollY = 1`; verify `.site-header--scrolled` class is present and `box-shadow` equals `var(--shadow-md)`
6. Resize to 768 px; verify `.site-header` has `display: none`
7. Resize to 1280 px; verify `.site-header` has `display: none` and `.sidebar` is visible

**Expected Results:**
- Header visible and fixed on mobile; hidden on tablet/desktop
- Height within spec at each breakpoint
- Shadow elevates on scroll; resets when scrolled back to top
- Sidebar and header are never simultaneously visible

**Skip Link Keyboard Test:**
1. Tab from outside the header
2. Verify first focused element matches `.site-header__skip-link`
3. Press Enter; verify `document.activeElement.id === 'main-content'`

**Safe Area Inset Test:**
1. Simulate `env(safe-area-inset-top) = 44px` (Chrome DevTools → Sensors)
2. Verify header `padding-top` is `44px`
3. Verify `.has-header` `padding-top` equals `header-height + 44px`

**Accessibility Audit:**
1. Run axe-core against `Header.html` at 375 px, 768 px, and 1280 px
2. Assert zero WCAG 2.1 AA violations

**Reduced Motion Test:**
1. Enable `prefers-reduced-motion: reduce` in DevTools
2. Verify `.site-header` `transition-duration` ≤ 0.01 ms

**Touch Target Test:**
```javascript
// Run in browser console at 375px viewport
document.querySelectorAll('.site-header a, .site-header button').forEach(el => {
  const rect = el.getBoundingClientRect();
  if (rect.width < 44 || rect.height < 44) {
    console.warn('Touch target too small:', el, rect);
  }
});
// Expected: no warnings
```

---

#### Bottom Navigation (Mobile)

**Test Steps:**
1. Open site on mobile viewport (< 768px)
2. Verify bottom nav is visible and fixed
3. Tap each navigation item
4. Verify active state indicator
5. Check notification badge visibility
6. Scroll page content
7. Verify bottom nav remains fixed
8. Check safe area inset spacing

**Expected Results:**
- Bottom nav always visible at bottom
- Active item shows blue bar at top
- Touch targets ≥ 44x44px
- No overlap with content
- Safe area respected on notched devices

#### Sidebar Navigation (Tablet/Desktop)

**Test Steps:**
1. Open site on tablet viewport (≥ 768px)
2. Verify sidebar is visible on left
3. Click each navigation item
4. Verify active state styling
5. Check security indicator visibility
6. Verify version number display
7. Scroll main content
8. Verify sidebar remains fixed

**Expected Results:**
- Sidebar visible at 240px (tablet) or 280px (desktop)
- Active item has blue left border
- Main content offset correctly
- Security status always visible
- Smooth transitions between states

### Modal Testing

#### Mobile Modal

**Test Steps:**
1. Open modal on mobile viewport
2. Verify full-screen coverage
3. Test slide-up animation
4. Scroll modal content
5. Tap close button
6. Tap backdrop (should close)
7. Test with keyboard open
8. Check safe area insets

**Expected Results:**
- Modal covers entire viewport
- Smooth slide-up animation
- Body scroll disabled
- Close button accessible
- Backdrop dismisses modal
- Content scrollable within modal

#### Tablet/Desktop Modal

**Test Steps:**
1. Open modal on tablet/desktop
2. Verify centered positioning
3. Test scale-in animation
4. Check backdrop blur effect
5. Click outside modal (should close)
6. Press Escape key (should close)
7. Tab through form elements
8. Verify max-width constraints

**Expected Results:**
- Modal centered on screen
- Max 90vw × 90vh size
- Backdrop has blur effect
- Click outside closes modal
- Keyboard navigation works
- Focus trapped in modal

### Form Testing

#### Input Fields

**Test Steps:**
1. Focus each input field
2. Verify focus indicator visibility
3. Type text in each field
4. Test autofill functionality
5. Trigger validation errors
6. Check error message display
7. Test with mobile keyboard
8. Verify input height ≥ 44px

**Expected Results:**
- Clear focus indicators (2px blue outline)
- Font size ≥ 16px (prevents iOS zoom)
- Error states clearly visible
- Error messages accessible
- Touch targets adequate
- Keyboard doesn't obscure inputs

#### Toggle Switches

**Test Steps:**
1. Tap/click toggle switch
2. Verify state change animation
3. Test keyboard activation (Space)
4. Check focus indicator
5. Verify touch target size
6. Test disabled state

**Expected Results:**
- Smooth toggle animation
- Clear on/off states
- Keyboard accessible
- Touch target ≥ 44x44px
- Disabled state visually distinct

#### Buttons

**Test Steps:**
1. Tap/click each button variant
2. Verify hover states (desktop)
3. Check active/pressed states
4. Test focus indicators
5. Verify disabled states
6. Measure touch target sizes
7. Test with keyboard (Enter/Space)

**Expected Results:**
- All buttons ≥ 44px height
- Clear hover feedback
- Visible focus indicators
- Disabled buttons not clickable
- Keyboard activation works

### Table/Data Display Testing

#### Responsive Tables

**Test Steps:**
1. View table on desktop (≥ 768px)
2. Verify standard table layout
3. Resize to mobile (< 768px)
4. Verify card transformation
5. Check data label visibility
6. Test horizontal scroll (if needed)
7. Verify row hover states

**Expected Results:**
- Desktop: Standard table with headers
- Mobile: Card layout with labels
- All data visible and readable
- No horizontal overflow
- Touch-friendly spacing

#### Horizontal Scroll Lists

**Test Steps:**
1. View on mobile viewport
2. Swipe horizontally
3. Verify momentum scrolling
4. Check scroll indicators
5. Resize to tablet/desktop
6. Verify grid transformation

**Expected Results:**
- Mobile: Smooth horizontal scroll
- Visible scroll indicators
- Touch momentum works
- Tablet/Desktop: Grid layout (no scroll)
- Items properly sized

---

## Accessibility Testing

### Keyboard Navigation

**Test Steps:**
1. Tab through all interactive elements
2. Verify logical tab order
3. Test Shift+Tab (reverse)
4. Activate elements with Enter/Space
5. Test Escape key (close modals)
6. Navigate forms with Tab
7. Test arrow keys in custom components

**Expected Results:**
- Logical tab order maintained
- All interactive elements reachable
- Clear focus indicators
- Keyboard shortcuts work
- No keyboard traps

### Screen Reader Testing

#### VoiceOver (iOS/macOS)

**Activation:**
- iOS: Settings > Accessibility > VoiceOver
- macOS: System Preferences > Accessibility > VoiceOver

**Test Steps:**
1. Enable VoiceOver
2. Navigate through page
3. Verify element announcements
4. Test form labels
5. Check button descriptions
6. Verify heading hierarchy
7. Test modal announcements
8. Check notification badges

**Expected Results:**
- All elements properly announced
- Form labels associated correctly
- Buttons have clear descriptions
- Headings in logical order
- ARIA labels present where needed
- Status changes announced

#### NVDA (Windows)

**Activation:**
- Download from nvaccess.org
- Launch NVDA

**Test Steps:**
1. Navigate with arrow keys
2. Test heading navigation (H key)
3. Test link navigation (K key)
4. Test form navigation (F key)
5. Verify table navigation
6. Check landmark regions

**Expected Results:**
- Smooth navigation
- Proper element identification
- Form fields clearly labeled
- Tables properly structured
- Landmarks defined

### Color Contrast Testing

**Tools:**
- Chrome DevTools (Lighthouse)
- WebAIM Contrast Checker
- axe DevTools extension

**Test Steps:**
1. Run Lighthouse accessibility audit
2. Check all text contrast ratios
3. Verify button contrast
4. Test focus indicator contrast
5. Check disabled state contrast
6. Test error message contrast

**Required Ratios:**
- Normal text: 4.5:1 minimum
- Large text (18px+): 3:1 minimum
- UI components: 3:1 minimum

### Touch Target Testing

**Test Steps:**
1. Measure all interactive elements
2. Use browser DevTools
3. Check buttons, links, inputs
4. Verify toggle switches
5. Test navigation items
6. Check icon buttons

**Requirements:**
- Minimum size: 44x44px
- Adequate spacing between targets
- Visual size may be smaller (use padding)

**Measurement Tool:**
```javascript
// Run in browser console
document.querySelectorAll('button, a, input').forEach(el => {
  const rect = el.getBoundingClientRect();
  if (rect.width < 44 || rect.height < 44) {
    console.warn('Touch target too small:', el, rect);
  }
});
```

---

## Performance Testing

### Layout Shift (CLS)

**Test Steps:**
1. Open Chrome DevTools
2. Run Lighthouse performance audit
3. Check Cumulative Layout Shift score
4. Identify shifting elements
5. Fix layout shift issues

**Target:** CLS < 0.1

### Animation Performance

**Test Steps:**
1. Open Performance tab in DevTools
2. Record page interactions
3. Check frame rate (should be 60fps)
4. Identify janky animations
5. Optimize using CSS transforms

**Tools:**
```css
/* Enable performance monitoring */
* {
  outline: 1px solid red;
}
```

### Touch Response Time

**Test Steps:**
1. Tap buttons on mobile device
2. Measure response time
3. Check for delays
4. Test during scroll
5. Verify no 300ms delay

**Target:** < 100ms response time

---

## Visual Regression Testing

### Manual Visual Testing

**Test Steps:**
1. Take screenshots at each breakpoint
2. Compare with design mockups
3. Check spacing and alignment
4. Verify typography scaling
5. Test color accuracy
6. Check border radius consistency

### Automated Visual Testing (Optional)

**Tools:**
- Percy
- Chromatic
- BackstopJS

---

## Orientation Testing

### Portrait to Landscape

**Test Steps:**
1. Open site in portrait mode
2. Rotate device to landscape
3. Verify layout adaptation
4. Check navigation behavior
5. Test modal display
6. Verify form usability

**Expected Results:**
- Smooth transition
- No layout breaks
- Content remains accessible
- Navigation adapts appropriately

### Landscape to Portrait

**Test Steps:**
1. Open site in landscape mode
2. Rotate device to portrait
3. Verify layout reflow
4. Check bottom nav appearance
5. Test modal transformation

---

## Cross-Browser Issues

### Known Issues & Fixes

#### iOS Safari

**Issue:** Input zoom on focus  
**Fix:** Use font-size ≥ 16px

**Issue:** Safe area insets  
**Fix:** Use `env(safe-area-inset-*)`

**Issue:** 100vh includes address bar  
**Fix:** Use `100dvh` or JavaScript calculation

#### Firefox

**Issue:** Flexbox gap not supported (older versions)  
**Fix:** Use margin fallback

#### Edge

**Issue:** CSS Grid gaps (older versions)  
**Fix:** Use margin fallback

---

## Testing Tools

### Browser DevTools

- **Chrome DevTools**
  - Device emulation
  - Lighthouse audits
  - Accessibility tree
  - Performance profiling

- **Firefox DevTools**
  - Responsive design mode
  - Accessibility inspector
  - CSS Grid inspector

### Extensions

- **axe DevTools** - Accessibility testing
- **WAVE** - Web accessibility evaluation
- **Lighthouse** - Performance and accessibility
- **ColorZilla** - Color picker and contrast checker

### Online Tools

- **WebAIM Contrast Checker** - https://webaim.org/resources/contrastchecker/
- **Responsive Design Checker** - https://responsivedesignchecker.com/
- **BrowserStack** - Real device testing
- **LambdaTest** - Cross-browser testing

---

## Bug Reporting Template

```markdown
### Bug Description
[Clear description of the issue]

### Steps to Reproduce
1. [First step]
2. [Second step]
3. [Third step]

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happens]

### Environment
- Device: [e.g., iPhone 14 Pro]
- OS: [e.g., iOS 16.5]
- Browser: [e.g., Safari 16.5]
- Viewport: [e.g., 393x852]
- Orientation: [Portrait/Landscape]

### Screenshots
[Attach screenshots if applicable]

### Severity
- [ ] Critical (blocks functionality)
- [ ] High (major usability issue)
- [ ] Medium (minor issue)
- [ ] Low (cosmetic)
```

---

## Continuous Testing

### Pre-Deployment Checklist

- [ ] All device viewports tested
- [ ] All browsers tested
- [ ] Accessibility audit passed
- [ ] Performance metrics acceptable
- [ ] Visual regression tests passed
- [ ] No console errors
- [ ] Touch targets verified
- [ ] Keyboard navigation works
- [ ] Screen reader compatible

### Post-Deployment Monitoring

- Monitor real user metrics (RUM)
- Track Core Web Vitals
- Monitor error rates
- Collect user feedback
- Review analytics for device usage

---

## Resources

- [Chrome DevTools Documentation](https://developer.chrome.com/docs/devtools/)
- [WebAIM Accessibility Resources](https://webaim.org/resources/)
- [MDN Web Docs - Responsive Design](https://developer.mozilla.org/en-US/docs/Learn/CSS/CSS_layout/Responsive_Design)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

---

**Last Updated**: February 20, 2026  
**Version**: 1.0.0
