# Stellar Raise - Frontend Responsive Design System

A comprehensive, mobile-first responsive design framework for the Stellar Raise crowdfunding dApp. Built to maintain the "Modern" and "Trusted" brand identity across all devices.

## 🚀 Quick Start

### Include Core Styles

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
  <meta name="theme-color" content="#0066FF">
  
  <!-- Core Responsive System -->
  <link rel="stylesheet" href="styles/responsive.css">
  <link rel="stylesheet" href="styles/utilities.css">
  
  <!-- Component Styles (as needed) -->
  <link rel="stylesheet" href="components/navigation/BottomNav.css">
  <link rel="stylesheet" href="components/navigation/Sidebar.css">
  <link rel="stylesheet" href="components/forms/Forms.css">
  <link rel="stylesheet" href="components/modals/Modal.css">
  <link rel="stylesheet" href="components/tables/ResponsiveTable.css">
  
  <!-- Space Grotesk Font -->
  <link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&display=swap" rel="stylesheet">
</head>
<body>
  <!-- Your content -->
</body>
</html>
```

## 📱 Breakpoints

| Breakpoint | Range | Target Devices | Primary Use Case |
|------------|-------|----------------|------------------|
| **Mobile** | < 768px | Phones | Quick contributions, activity checks |
| **Tablet** | 768-1024px | iPads, tablets | Project lists, detailed dashboards |
| **Desktop** | > 1024px | Laptops, monitors | Full management interface |

## 🎨 Design Tokens

### Colors

```css
--color-primary-blue:   #0066FF  /* Primary actions */
--color-deep-navy:      #0A1929  /* Headings, text */
--color-success-green:  #00C853  /* Success states */
--color-error-red:      #FF3B30  /* Errors */
--color-warning-orange: #FF9500  /* Warnings */
```

### Typography

**Font Family:** Space Grotesk

```css
--font-size-xs:   clamp(0.75rem, 0.7rem + 0.25vw, 0.875rem)
--font-size-sm:   clamp(0.875rem, 0.825rem + 0.25vw, 1rem)
--font-size-base: clamp(1rem, 0.95rem + 0.25vw, 1.125rem)
--font-size-lg:   clamp(1.125rem, 1.05rem + 0.375vw, 1.5rem)
--font-size-xl:   clamp(1.25rem, 1.15rem + 0.5vw, 1.875rem)
--font-size-2xl:  clamp(1.5rem, 1.35rem + 0.75vw, 2.25rem)
--font-size-3xl:  clamp(1.875rem, 1.65rem + 1.125vw, 3rem)
```

### Spacing

```css
--space-1:  0.25rem  /* 4px */
--space-2:  0.5rem   /* 8px */
--space-3:  0.75rem  /* 12px */
--space-4:  1rem     /* 16px */
--space-6:  1.5rem   /* 24px */
--space-8:  2rem     /* 32px */
--space-12: 3rem     /* 48px */
```

## 🧩 Components

### Navigation

#### Bottom Navigation (Mobile)
```html
<nav class="bottom-nav">
  <div class="bottom-nav__container">
    <a href="/home" class="bottom-nav__item bottom-nav__item--active">
      <svg class="bottom-nav__icon">...</svg>
      <span class="bottom-nav__label">Home</span>
    </a>
  </div>
</nav>
```

#### Sidebar (Tablet/Desktop)
```html
<aside class="sidebar">
  <div class="sidebar__header">...</div>
  <nav class="sidebar__nav">...</nav>
  <div class="sidebar__footer">...</div>
</aside>
```

### Modals

```html
<div class="modal modal--open">
  <div class="modal__backdrop"></div>
  <div class="modal__content">
    <header class="modal__header">
      <h2 class="modal__title">Title</h2>
      <button class="modal__close">×</button>
    </header>
    <div class="modal__body">Content</div>
    <footer class="modal__footer">
      <button class="btn btn--secondary">Cancel</button>
      <button class="btn btn--primary">Confirm</button>
    </footer>
  </div>
</div>
```

### Forms

```html
<form class="form">
  <div class="form__group">
    <label for="email" class="form__label">Email</label>
    <input type="email" id="email" class="form__input" />
  </div>
  
  <div class="form__toggle">
    <span class="form__toggle-label">Enable 2FA</span>
    <label class="toggle-switch">
      <input type="checkbox" />
      <span class="toggle-slider"></span>
    </label>
  </div>
  
  <button type="submit" class="btn btn--primary btn--full">Submit</button>
</form>
```

### Buttons

```html
<button class="btn btn--primary">Primary</button>
<button class="btn btn--secondary">Secondary</button>
<button class="btn btn--danger">Danger</button>
<button class="btn btn--outline">Outline</button>
```

### Responsive Tables

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
      <td data-label="Status">
        <span class="status-badge status-badge--success">Success</span>
      </td>
    </tr>
  </tbody>
</table>
```

## 📐 Grid System

```html
<div class="container">
  <div class="grid">
    <!-- Full width on mobile, 4 cols on tablet, 3 cols on desktop -->
    <div class="col-span-full col-span-md-4 col-span-lg-3">
      Content
    </div>
  </div>
</div>
```

## 🎯 Touch Targets

All interactive elements meet **44x44px minimum** size requirement (WCAG 2.5.5).

```css
button,
a,
input[type="checkbox"],
input[type="radio"] {
  min-height: 44px;
  min-width: 44px;
}
```

## ♿ Accessibility

### Focus Indicators

```css
:focus-visible {
  outline: 2px solid var(--color-primary-blue);
  outline-offset: 2px;
}
```

### Screen Reader Support

```html
<!-- Proper ARIA labels -->
<button aria-label="Close modal">×</button>

<!-- Current page indication -->
<a href="/home" aria-current="page">Home</a>

<!-- Hidden decorative icons -->
<svg aria-hidden="true">...</svg>
```

### Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

## 📱 Safe Area Insets

Support for notched devices:

```html
<div class="safe-area-top">Content</div>
<div class="safe-area-bottom">Content</div>
```

```css
padding-top: calc(var(--space-4) + env(safe-area-inset-top));
```

## 🛠️ Utility Classes

### Display
```html
<div class="hidden md:block">Visible on tablet+</div>
<div class="block md:hidden">Visible on mobile only</div>
```

### Flexbox
```html
<div class="flex items-center justify-between gap-4">
  <span>Left</span>
  <span>Right</span>
</div>
```

### Spacing
```html
<div class="p-4 mb-6">Padding 16px, margin-bottom 24px</div>
<div class="px-6 py-8">Padding x-axis 24px, y-axis 32px</div>
```

### Typography
```html
<h1 class="text-3xl font-bold text-navy">Heading</h1>
<p class="text-base text-neutral">Body text</p>
```

## 📂 File Structure

```
frontend/
├── styles/
│   ├── responsive.css      # Core design system
│   └── utilities.css       # Utility classes
├── components/
│   ├── navigation/
│   │   ├── BottomNav.css
│   │   ├── BottomNav.html
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
│   ├── RESPONSIVE_DESIGN_GUIDE.md
│   └── TESTING_GUIDE.md
├── index.html              # Example implementation
└── README.md               # This file
```

## 🧪 Testing

### Browser DevTools Testing

```javascript
// Check touch target sizes
document.querySelectorAll('button, a, input').forEach(el => {
  const rect = el.getBoundingClientRect();
  if (rect.width < 44 || rect.height < 44) {
    console.warn('Touch target too small:', el, rect);
  }
});
```

### Viewport Testing

- Mobile: 375px (iPhone SE)
- Tablet: 768px (iPad)
- Desktop: 1280px (Standard laptop)

### Accessibility Testing

- Lighthouse audit (Chrome DevTools)
- axe DevTools extension
- Screen reader testing (VoiceOver, NVDA)
- Keyboard navigation

See [TESTING_GUIDE.md](docs/TESTING_GUIDE.md) for comprehensive testing procedures.

## 📖 Documentation

- **[Responsive Design Guide](docs/RESPONSIVE_DESIGN_GUIDE.md)** - Complete design system documentation
- **[Testing Guide](docs/TESTING_GUIDE.md)** - Testing procedures and checklists

## 🎯 Key Features

✅ Mobile-first responsive design  
✅ 44x44px minimum touch targets  
✅ WCAG 2.1 AA compliant  
✅ Safe area inset support  
✅ Reduced motion support  
✅ Screen reader optimized  
✅ Keyboard navigation  
✅ Fluid typography  
✅ Flexible grid system  
✅ Comprehensive component library  

## 🔧 Browser Support

- iOS Safari 13+
- Chrome (mobile & desktop)
- Firefox (mobile & desktop)
- Edge (latest 2 versions)
- Safari (macOS, latest 2 versions)

## 📝 Best Practices

### Mobile-First CSS

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
```

### Touch-Friendly Interactions

- Avoid hover-only interactions
- Provide visual feedback for all interactions
- Use adequate spacing between touch targets
- Test on real devices

### Performance

- Use CSS transforms for animations
- Minimize layout shifts
- Optimize images for mobile
- Lazy load heavy components

## 🤝 Contributing

When adding new components:

1. Follow mobile-first approach
2. Ensure 44x44px touch targets
3. Add proper ARIA labels
4. Test across all breakpoints
5. Verify accessibility compliance
6. Document in style guide

## 📄 License

MIT License - See [LICENSE](../LICENSE) file for details

---

**Version**: 1.0.0  
**Last Updated**: February 20, 2026  
**Maintained by**: Stellar Raise Development Team
