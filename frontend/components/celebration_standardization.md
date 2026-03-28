# Celebration Standardization Component

## Overview

The `CelebrationStandardization` component provides a standardized, accessible, and visually consistent way to celebrate campaign milestones in the Stellar Raise crowdfund platform. It includes confetti animations, progress indicators, and full accessibility support.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Milestone Types](#milestone-types)
- [Security Considerations](#security-considerations)
- [Accessibility](#accessibility)
- [Performance](#performance)
- [Testing](#testing)
- [Examples](#examples)

---

## Features

### Core Features

- **Standardized Celebrations**: Consistent visual feedback for all campaign milestones
- **8 Milestone Types**: Pre-configured celebrations for every stage of a campaign
- **Confetti Animations**: Celebratory confetti effects with fall physics
- **Progress Indicators**: Visual progress bars with percentage display
- **Responsive Design**: Adapts to different screen sizes
- **Dark Mode Support**: Uses CSS variables for theme compatibility

### Animation Features

- **Entrance Animations**: Smooth scale and fade-in effects
- **Confetti Physics**: Realistic falling confetti with rotation
- **Pulse Effects**: Glowing pulse on milestone icons
- **Reduced Motion Support**: Respects `prefers-reduced-motion` user preference

### Accessibility Features

- **ARIA Live Regions**: Announces celebrations to screen readers
- **Keyboard Navigation**: Fully navigable with keyboard
- **Focus Management**: Proper focus handling
- **Color Contrast**: Meets WCAG 2.1 AA standards
- **Semantic HTML**: Uses proper heading and landmark elements

---

## Installation

The component is part of the frontend components library. No additional installation is required beyond the existing dependencies.

```tsx
import CelebrationStandardization, { MilestoneType } from "@/components/celebration_standardization";
```

---

## Usage

### Basic Usage

```tsx
import CelebrationStandardization, { MilestoneType } from "@/components/celebration_standardization";

function App() {
  const [showCelebration, setShowCelebration] = React.useState(false);

  return (
    <div>
      <button onClick={() => setShowCelebration(true)}>
        Reach 50%!
      </button>
      
      {showCelebration && (
        <CelebrationStandardization
          milestoneType={MilestoneType.MILESTONE_50_PERCENT}
          isVisible={showCelebration}
          onAnimationComplete={() => setShowCelebration(false)}
        />
      )}
    </div>
  );
}
```

### With Campaign Progress

```tsx
<CelebrationStandardization
  milestoneType={MilestoneType.MILESTONE_75_PERCENT}
  campaignName="Eco-Friendly Tech Project"
  progressPercentage={75}
  raisedAmount="$75,000"
  targetAmount="$100,000"
  isVisible={true}
  onAnimationComplete={handleAnimationEnd}
/>
```

### Multiple Milestones Flow

```tsx
const milestoneFlow = [
  MilestoneType.CAMPAIGN_CREATED,
  MilestoneType.FIRST_CONTRIBUTION,
  MilestoneType.MILESTONE_25_PERCENT,
  MilestoneType.MILESTONE_50_PERCENT,
  MilestoneType.MILESTONE_75_PERCENT,
  MilestoneType.MILESTONE_100_PERCENT,
  MilestoneType.CAMPAIGN_SUCCESS,
];

function CampaignMilestones() {
  const [currentMilestoneIndex, setCurrentMilestoneIndex] = useState(0);

  return (
    <CelebrationStandardization
      milestoneType={milestoneFlow[currentMilestoneIndex]}
      campaignName="My Campaign"
      progressPercentage={(currentMilestoneIndex + 1) * 12.5}
      isVisible={currentMilestoneIndex < milestoneFlow.length}
    />
  );
}
```

---

## API Reference

### Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `milestoneType` | `MilestoneType` | Yes | - | Type of milestone to celebrate |
| `isVisible` | `boolean` | No | `false` | Whether the celebration is visible |
| `campaignName` | `string` | No | `undefined` | Campaign name for context |
| `progressPercentage` | `number` | No | `undefined` | Funding progress (0-100+) |
| `raisedAmount` | `string` | No | `undefined` | Current raised amount |
| `targetAmount` | `string` | No | `undefined` | Campaign target amount |
| `onAnimationComplete` | `() => void` | No | `undefined` | Callback when animation finishes |
| `className` | `string` | No | `""` | Custom CSS class |
| `testId` | `string` | No | `undefined` | Test identifier |
| `ariaLabel` | `string` | No | Config title | Custom accessibility label |

### Exported Constants

#### `MilestoneType`

Enum representing all available milestone types:

```tsx
enum MilestoneType {
  CAMPAIGN_CREATED = "campaign_created",
  FIRST_CONTRIBUTION = "first_contribution",
  MILESTONE_25_PERCENT = "milestone_25_percent",
  MILESTONE_50_PERCENT = "milestone_50_percent",
  MILESTONE_75_PERCENT = "milestone_75_percent",
  MILESTONE_100_PERCENT = "milestone_100_percent",
  CAMPAIGN_SUCCESS = "campaign_success",
  STRETCH_GOAL_REACHED = "stretch_goal_reached",
}
```

#### `MilestoneConfigRegistry`

Pre-configured settings for each milestone type:

```tsx
interface MilestoneConfig {
  title: string;
  description: string;
  icon: string;
  primaryColor: string;
  secondaryColor: string;
  animationDuration: number;
  showConfetti: boolean;
  playSound: boolean;
  priority: number;
}
```

#### `CelebrationCSSVariables`

CSS custom properties for theming:

```tsx
const CelebrationCSSVariables = {
  "--celebration-primary": "var(--color-primary, #6366f1)",
  "--celebration-secondary": "var(--color-secondary, #8b5cf6)",
  "--celebration-success": "var(--color-success, #10b981)",
  // ... more variables
};
```

---

## Milestone Types

### Campaign Created
- **Trigger**: When a new campaign is created
- **Priority**: 1
- **Icon**: Campaign flag
- **Color**: Indigo (#6366f1)

### First Contribution
- **Trigger**: When the first backer contributes
- **Priority**: 2
- **Icon**: Star
- **Color**: Amber (#f59e0b)

### 25% Funded
- **Trigger**: Quarter of the goal reached
- **Priority**: 3
- **Icon**: Chart
- **Color**: Emerald (#10b981)

### 50% Funded
- **Trigger**: Halfway to the goal
- **Priority**: 4
- **Icon**: Fire
- **Color**: Red (#ef4444)

### 75% Funded
- **Trigger**: Three-quarters funded
- **Priority**: 5
- **Icon**: Rocket
- **Color**: Blue (#3b82f6)

### 100% Funded
- **Trigger**: Goal reached
- **Priority**: 6
- **Icon**: Target
- **Color**: Emerald (#10b981)

### Campaign Success
- **Trigger**: Campaign ends successfully
- **Priority**: 7
- **Icon**: Trophy
- **Color**: Gold (#fbbf24)

### Stretch Goal
- **Trigger**: Exceeded the goal
- **Priority**: 8
- **Icon**: Sparkles
- **Color**: Purple (#8b5cf6)

---

## Security Considerations

### XSS Prevention

The component implements multiple layers of protection against XSS attacks:

1. **Text Sanitization**: User-provided content (campaign names, amounts) is sanitized using `textContent` before rendering
2. **Predefined Icons**: Only predefined SVG icons from the `MilestoneIcons` registry are rendered
3. **Style Injection**: CSS variables are read-only constants, preventing style injection
4. **Class Name Validation**: Custom class names are validated

### Content Security

```tsx
// User input is sanitized before rendering
const sanitizeText = (text: string): string => {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
};
```

### Animation Security

- Animations use CSS transforms for GPU acceleration
- No JavaScript-based animation libraries required
- No user-controlled animation parameters

---

## Accessibility

### ARIA Support

The component provides comprehensive ARIA support:

- `role="alert"`: Announces celebrations to screen readers
- `aria-live="polite"`: Non-intrusive announcements
- `aria-atomic="true"`: Announces entire content change
- `aria-hidden`: Hides decorative elements from assistive technology

### Keyboard Navigation

- Tab navigation through interactive elements
- Escape key to dismiss (when applicable)
- Focus trapping within modal celebrations

### Reduced Motion

Respects user's motion preferences:

```css
@media (prefers-reduced-motion: reduce) {
  .confetti-particle,
  .milestone-icon-wrapper > div {
    animation: none !important;
  }
}
```

### Color Contrast

All text meets WCAG 2.1 AA contrast requirements:
- Primary text: #111827 (dark gray)
- Secondary text: #6b7280 (medium gray)
- Background: Transparent with semi-transparent overlays

---

## Performance

### Optimization Techniques

1. **CSS Transforms**: Uses `transform` for GPU-accelerated animations
2. **Will-Change**: Strategically applied for animation elements
3. **Memoization**: Particle configurations are memoized
4. **Lazy Loading**: Animations only start when visible

### Bundle Impact

- **Gzipped Size**: ~3KB
- **Dependencies**: None (uses inline SVG)
- **Code Splitting**: Component can be lazy-loaded

### Animation Performance

| Animation | Duration | GPU Usage |
|-----------|----------|-----------|
| Entrance | 300ms | Yes |
| Confetti | 3000ms | Yes |
| Pulse | 2000ms | Yes |
| Exit | 300ms | Yes |

---

## Testing

### Test Coverage

The test suite covers:

- **MilestoneConfigRegistry**: All milestone configurations
- **Component Rendering**: All prop combinations
- **Animation States**: Enter, visible, exit transitions
- **Accessibility**: ARIA attributes, keyboard support
- **Security**: XSS prevention, sanitization
- **Edge Cases**: 0%, 100%+, long text, rapid toggling

### Running Tests

```bash
# Run all tests
npm test

# Run with coverage
npm test -- --coverage

# Run specific test file
npm test -- celebration_standardization.test.tsx
```

### Test Examples

```tsx
// Basic rendering
renderCelebration();
expect(screen.getByText(/Campaign Created/i)).toBeInTheDocument();

// Security - XSS prevention
const maliciousName = '<script>alert("XSS")</script>';
renderCelebration({ campaignName: maliciousName });
expect(screen.getByText(maliciousName)).toBeInTheDocument();

// Accessibility
expect(getContainer()).toHaveAttribute("role", "alert");

// Animation states
rerender({ isVisible: true });
await waitFor(() => {
  expect(getContainer()).toBeInTheDocument();
});
```

---

## Examples

### Campaign Progress Celebration

```tsx
import React, { useState } from "react";
import CelebrationStandardization, { MilestoneType } from "./celebration_standardization";

function CampaignDashboard({ campaign }) {
  const [celebrationType, setCelebrationType] = useState(null);
  const [showCelebration, setShowCelebration] = useState(false);

  const handleContribution = (amount) => {
    const newProgress = (campaign.raised + amount) / campaign.target * 100;
    
    if (newProgress >= 100 && campaign.raised < campaign.target) {
      setCelebrationType(MilestoneType.MILESTONE_100_PERCENT);
      setShowCelebration(true);
    }
  };

  return (
    <div>
      {/* Campaign content */}
      
      <CelebrationStandardization
        milestoneType={celebrationType}
        campaignName={campaign.name}
        progressPercentage={campaign.progress}
        raisedAmount={formatCurrency(campaign.raised)}
        targetAmount={formatCurrency(campaign.target)}
        isVisible={showCelebration}
        onAnimationComplete={() => setShowCelebration(false)}
      />
    </div>
  );
}
```

### Success Campaign Celebration

```tsx
function CampaignCompletion({ campaign }) {
  const [showSuccess, setShowSuccess] = useState(false);

  useEffect(() => {
    if (campaign.status === "successful") {
      setShowSuccess(true);
    }
  }, [campaign.status]);

  return (
    <CelebrationStandardization
      milestoneType={MilestoneType.CAMPAIGN_SUCCESS}
      campaignName={campaign.name}
      progressPercentage={100}
      raisedAmount={formatCurrency(campaign.raised)}
      targetAmount={formatCurrency(campaign.target)}
      isVisible={showSuccess}
      onAnimationComplete={() => setShowSuccess(false)}
      className="campaign-success-celebration"
    />
  );
}
```

### Stretch Goal Achievement

```tsx
function StretchGoalAlert({ campaign }) {
  const [showStretch, setShowStretch] = useState(false);

  useEffect(() => {
    if (campaign.raised > campaign.target) {
      setShowStretch(true);
    }
  }, [campaign.raised, campaign.target]);

  return (
    <CelebrationStandardization
      milestoneType={MilestoneType.STRETCH_GOAL_REACHED}
      campaignName={campaign.name}
      progressPercentage={Math.round((campaign.raised / campaign.target) * 100)}
      raisedAmount={formatCurrency(campaign.raised)}
      targetAmount={formatCurrency(campaign.target)}
      isVisible={showStretch}
      onAnimationComplete={() => setShowStretch(false)}
    />
  );
}
```

---

## Version History

### v1.0.0 (2026-03-27)
- Initial release
- 8 milestone types implemented
- Confetti animations
- Full accessibility support
- XSS protection
- Reduced motion support

---

## Contributing

When adding new milestone types:

1. Add the type to `MilestoneType` enum
2. Add configuration to `MilestoneConfigRegistry`
3. Add SVG icon to `MilestoneIcons` object
4. Add tests for the new milestone
5. Update this documentation

---

## License

Part of the Stellar Raise contracts project. See LICENSE file for details.
