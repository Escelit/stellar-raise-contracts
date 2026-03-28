# Celebration Compatibility

## Overview

`CelebrationCompatibility` is a cross-environment milestone celebration overlay for the Stellar Raise crowdfunding dApp. It renders a themed modal when a campaign milestone is reached and is designed for maximum compatibility across React 17/18, SSR/Next.js, Jest + jsdom, and browsers without CSS custom-property support.

---

## Files

| File | Purpose |
|---|---|
| `frontend/components/celebration_compatibility.tsx` | Component implementation |
| `frontend/components/celebration_compatibility.test.tsx` | Comprehensive test suite |
| `docs/celebration_compatibility.md` | This document |

---

## Props

| Prop | Type | Required | Default | Description |
|---|---|---|---|---|
| `milestoneLabel` | `string` | Yes | — | Human-readable label, e.g. `"50% Funded"`. Rendered as plain text. |
| `milestonePercent` | `number` | Yes | — | Numeric percentage (0–100) used to derive the default tier. |
| `campaignName` | `string` | Yes | — | Campaign display name. Rendered as plain text. |
| `onDismiss` | `() => void` | Yes | — | Called when the overlay is dismissed. |
| `theme` | `"light" \| "dark" \| "auto"` | No | `"auto"` | Visual theme. `"auto"` reads `prefers-color-scheme`. |
| `tier` | `MilestoneTier` | No | derived | Overrides the auto-derived tier. |
| `autoDismissMs` | `number` | No | `0` | Auto-dismiss after N ms. `0` or negative = never. |
| `className` | `string` | No | `""` | Extra CSS class names forwarded to the root element. |

---

## Milestone Tiers

Tiers are derived automatically from `milestonePercent` when `tier` is not provided:

| Range | Tier | Badge | Accent |
|---|---|---|---|
| < 25% | `bronze` | 🥉 | `#CD7F32` |
| 25–49% | `silver` | 🥈 | `#A8A9AD` |
| 50–74% | `gold` | 🥇 | `#FFD700` |
| ≥ 75% | `platinum` | 🏆 | `#E5E4E2` |

---

## Exported Helpers

These pure functions are exported for independent unit testing:

```ts
deriveMilestoneTier(percent: number): MilestoneTier
tierEmoji(tier: MilestoneTier): string
tierAccentColor(tier: MilestoneTier): string
resolveTheme(theme: CelebrationTheme): 'light' | 'dark'
```

`resolveTheme('auto')` reads `window.matchMedia('(prefers-color-scheme: dark)')` and falls back to `"light"` when `window` is unavailable (SSR).

---

## Usage

```tsx
import { CelebrationCompatibility } from './components/celebration_compatibility';

function CampaignPage() {
  const [showCelebration, setShowCelebration] = useState(false);

  return (
    <>
      {showCelebration && (
        <CelebrationCompatibility
          milestoneLabel="50% Funded"
          milestonePercent={50}
          campaignName="Clean Water Initiative"
          theme="auto"
          autoDismissMs={8000}
          onDismiss={() => setShowCelebration(false)}
        />
      )}
    </>
  );
}
```

---

## Dismissal Behaviour

The overlay can be dismissed three ways:

1. Clicking the "Continue" button
2. Clicking the backdrop
3. Pressing the `Escape` key

All three paths call `onDismiss` exactly once and unmount the overlay.

---

## Security Notes

- `milestoneLabel` and `campaignName` are rendered as plain text nodes — no `dangerouslySetInnerHTML` is used, eliminating XSS risk.
- `onDismiss` is a caller-supplied callback with no internal side-effects beyond local state transitions.
- No network requests are made inside this component.
- `autoDismissMs` is validated at runtime; non-positive values disable the timer.

---

## Accessibility

- Root element carries `role="dialog"` and `aria-modal="true"`.
- `aria-label` is derived from `milestoneLabel` for screen-reader context.
- Dismiss button receives focus on mount (keyboard-first UX).
- Escape key dismisses the overlay (WCAG 2.1 SC 2.1.2).
- Backdrop click dismisses the overlay for pointer users.

---

## Running Tests

```bash
cd frontend
npm test -- --run --testPathPattern=celebration_compatibility
```
