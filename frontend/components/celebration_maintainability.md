# celebration_maintainability

## Overview

`celebration_maintainability` is a lightweight frontend component that
surfaces campaign milestone progress and maintainability signals in a
maintainable, accessible UI panel.

It is designed to improve frontend maintainability by separating milestone
presentation, summary generation, and action handling into small reusable
helpers.

---

## Security and Accessibility

- No `dangerouslySetInnerHTML` is used.
- All milestone labels are normalized and truncated to prevent layout issues.
- The component uses `role="status"` and `aria-live="polite"` for dynamic
  announcement of maintainability updates.
- Interactive controls use semantic `button` markup with clear labels.

---

## Public API

| Export | Description |
|---|---|
| `CelebrationMaintainability` | Main component rendering maintainability status and milestone list. |
| `clampPercent` | Clamps progress values to the [0, 100] range. |
| `formatMilestoneLabel` | Normalizes and truncates milestone labels for display. |
| `getNextPendingMilestone` | Returns the nearest pending milestone by target percent. |
| `buildMaintainabilitySummary` | Builds a human-friendly maintainability summary string. |
| `Milestone`, `MilestoneStatus` | Types used by component props. |

---

## Example

```tsx
import CelebrationMaintainability from "./celebration_maintainability";

<CelebrationMaintainability
  campaignName="Stellar Raise"
  currentPercent={48}
  milestones={[
    { id: "m1", label: "25% Funded", targetPercent: 25, status: "celebrated" },
    { id: "m2", label: "50% Funded", targetPercent: 50, status: "pending" },
  ]}
  onReview={() => console.log("review milestone maintainability")}
/>
```

---

## Maintainability Guidance

- `MilestoneStatus` values are intentionally conservative: `pending`, `reached`, `celebrated`, and `failed`.
- The summary text is recomputed only when `milestones` or `currentPercent` change.
- The component supports auto-dismiss behavior with `autoDismissMs`.
