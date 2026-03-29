# celebration_scalability

Scalable milestone celebration system for the Stellar Raise crowdfunding dApp.

## Overview

`CelebrationScalability` supports an arbitrary number of custom milestones (capped at 100), queues multiple simultaneous crossings, and drains them one at a time to avoid UI flooding. Designed for campaigns with dense milestone schedules.

## Security Assumptions

| # | Assumption |
|---|-----------|
| 1 | No `dangerouslySetInnerHTML` — all content rendered as React text nodes. |
| 2 | All milestone labels are sanitized before render. |
| 3 | `currentPercent` is clamped to [0, 100]. |
| 4 | Auto-dismiss timer is cleared on unmount to prevent memory leaks. |
| 5 | Milestone list is deduplicated by `id` to prevent double-celebrations. |

## Scalability Design

| Concern | Approach |
|---------|----------|
| Large milestone lists | `prepareMilestones` sorts + deduplicates once; capped at `MAX_MILESTONES=100` |
| Multiple simultaneous crossings | Enqueued and drained one at a time |
| Deduplication | O(1) `Set<string>` lookup by milestone `id` |
| Re-render cost | `prepareMilestones` called at render time, not inside effects |

## Exported API

### Constants

| Name | Value | Description |
|------|-------|-------------|
| `DEFAULT_AUTO_DISMISS_MS` | `5000` | Default auto-dismiss delay |
| `MAX_LABEL_LENGTH` | `80` | Max chars for milestone labels |
| `MAX_MILESTONES` | `100` | Max milestones per campaign |

### Types

```ts
interface ScalableMilestone {
  id: string;              // unique identifier for deduplication
  thresholdPercent: number; // 0–100
  label: string;           // human-readable label
}
```

### Pure helpers

| Function | Description |
|----------|-------------|
| `clampPercent(value)` | Clamps to [0, 100]; returns 0 for NaN/non-number |
| `sanitizeMilestoneLabel(input)` | Strips control chars, collapses whitespace, truncates to MAX_LABEL_LENGTH |
| `isValidMilestone(m)` | Type guard — validates id, thresholdPercent, label |
| `findCrossedMilestones(sorted, percent, celebrated)` | Returns uncelebrated milestones crossed by percent |
| `prepareMilestones(raw)` | Filters invalid, deduplicates by id, sorts ascending, caps at MAX_MILESTONES |

### Component props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `currentPercent` | `number` | required | Current funding % (clamped internally) |
| `milestones` | `ScalableMilestone[]` | required | Milestone definitions |
| `onCelebrate` | `(milestone) => void` | — | Called when a milestone celebration begins |
| `onDismiss` | `(milestone) => void` | — | Called when overlay is dismissed |
| `autoDismissMs` | `number` | `5000` | Auto-dismiss delay; `0` disables |

## Usage

```tsx
const milestones: ScalableMilestone[] = [
  { id: "ms-25",  thresholdPercent: 25,  label: "25% Funded" },
  { id: "ms-50",  thresholdPercent: 50,  label: "Halfway There" },
  { id: "ms-75",  thresholdPercent: 75,  label: "75% Funded" },
  { id: "ms-100", thresholdPercent: 100, label: "Goal Reached!" },
];

<CelebrationScalability
  currentPercent={fundingPercent}
  milestones={milestones}
  onCelebrate={(m) => analytics.track("milestone_reached", { id: m.id })}
/>
```

## Accessibility

- `role="status"` + `aria-live="polite"` for screen-reader announcements.
- Dismiss button has `aria-label="Dismiss celebration"`.
- Decorative icons carry `aria-hidden="true"`.

## Test Coverage

`celebration_scalability.test.tsx` covers:
- All pure helpers (happy path + edge cases).
- `isValidMilestone` boundary conditions.
- `findCrossedMilestones` with various celebrated sets.
- `prepareMilestones` sorting, deduplication, filtering, and cap.
- Component renders nothing below first threshold.
- Milestone label display and sanitization.
- Queue remaining count (singular/plural).
- Queue drains correctly after dismiss.
- Manual dismiss hides overlay and calls `onDismiss`.
- Auto-dismiss fires after `autoDismissMs`.
- `autoDismissMs=0` disables auto-dismiss.
- `onCelebrate` callback fires on milestone cross.
- Deduplication: same milestone not re-triggered.
- Accessibility attributes.
- All exported constants.
