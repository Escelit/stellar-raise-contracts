# Milestone Fireworks

> **Component:** `frontend/components/milestone_fireworks.tsx`
> **Tests:** `frontend/components/milestone_fireworks.test.tsx`
> **Issue:** create-campaign-milestone-celebration-fireworks-for-frontend-ui

---

## Overview

`MilestoneFireworks` renders an animated canvas-based fireworks celebration
overlay when a crowdfunding campaign crosses a funding milestone (25 %, 50 %,
75 %, 100 %). It fires rocket bursts of coloured particles, shows a dismissible
banner with the milestone heading, and announces the event to screen readers
via an accessible live region.

---

## Security Assumptions

1. No `dangerouslySetInnerHTML` — all content is rendered as React text nodes.
2. All user-supplied strings (`campaignName`) are passed through
   `sanitizeFireworksLabel` before render, stripping control characters and
   truncating to `MAX_FIREWORKS_NAME_LENGTH` (60).
3. Progress values are clamped to `[0, 100]` by `clampFireworksProgress` —
   out-of-range values cannot cause layout abuse.
4. Canvas drawing uses only hardcoded colour palettes (`MILESTONE_COLORS`) —
   no user-controlled CSS values reach the canvas context.
5. All `requestAnimationFrame` handles and `setTimeout` timers are cancelled
   on unmount to prevent memory leaks and post-unmount state updates.
6. `onMilestone` and `onDismiss` callbacks are guarded by a `mountedRef`
   check so they are never called after the component unmounts.

---

## Props

| Prop             | Type                              | Default  | Description                                     |
| ---------------- | --------------------------------- | -------- | ----------------------------------------------- |
| `currentPercent` | `number`                          | required | Funding percentage (0–100). Clamped internally. |
| `campaignName`   | `string`                          | —        | Optional campaign name shown in the banner.     |
| `onMilestone`    | `(t: FireworksMilestone) => void` | —        | Called when a new milestone fires.              |
| `onDismiss`      | `(t: FireworksMilestone) => void` | —        | Called when the overlay is dismissed.           |
| `autoDismissMs`  | `number`                          | `6000`   | Auto-dismiss delay in ms. `0` disables.         |

---

## Exported Pure Helpers

| Function                                     | Purpose                                       |
| -------------------------------------------- | --------------------------------------------- |
| `clampFireworksProgress(value)`              | Clamps progress to `[0, 100]`                 |
| `sanitizeFireworksLabel(input, maxLength)`   | Strips control chars, truncates               |
| `resolveFireworksMilestone(pct, celebrated)` | Returns next uncelebrated threshold           |
| `getFireworksContent(threshold)`             | Returns heading + subtitle for a threshold    |
| `createBurst(x, y, colors, count, now)`      | Generates particle burst array                |
| `stepParticles(particles, now)`              | Advances particles one frame (gravity + fade) |
| `drawParticles(ctx, particles, w, h)`        | Draws particles onto a canvas context         |

---

## Constants

| Constant                       | Value               | Purpose                      |
| ------------------------------ | ------------------- | ---------------------------- |
| `FIREWORKS_MILESTONES`         | `[25, 50, 75, 100]` | Milestone thresholds         |
| `DEFAULT_FIREWORKS_DISMISS_MS` | `6000`              | Default auto-dismiss delay   |
| `MAX_FIREWORKS_NAME_LENGTH`    | `60`                | Max campaign name chars      |
| `PARTICLES_PER_BURST`          | `48`                | Particles per rocket burst   |
| `ROCKETS_PER_TRIGGER`          | `3`                 | Simultaneous rocket launches |
| `PARTICLE_LIFETIME_MS`         | `1200`              | Particle fade duration       |
| `CANVAS_WIDTH`                 | `400`               | Canvas logical width (px)    |
| `CANVAS_HEIGHT`                | `220`               | Canvas logical height (px)   |

---

## Usage

```tsx
import MilestoneFireworks from "@/components/milestone_fireworks";

<MilestoneFireworks
  currentPercent={progressPercent}
  campaignName={campaign.title}
  onMilestone={(t) => console.log(`Milestone: ${t}%`)}
  onDismiss={(t) => analytics.track("fireworks_dismissed", { threshold: t })}
  autoDismissMs={6000}
/>;
```

---

## Accessibility

- `role="status"` + `aria-live="polite"` — screen readers announce the milestone.
- `aria-label` on the overlay carries the heading text.
- Canvas is `aria-hidden="true"` — purely decorative.
- Dismiss button has `aria-label="Dismiss fireworks celebration"`.

---

## Running the Tests

```bash
# Run milestone fireworks tests only
npx jest milestone_fireworks --coverage

# Run all frontend tests
npm test
```

Expected: all tests pass, ≥ 95 % coverage.
