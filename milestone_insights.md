# Campaign Milestone Celebration Insights

## Overview

This module provides **pure, testable insight computation** and a small **React panel** for milestone celebration flows in the crowdfunding frontend. It turns raw campaign numbers into:

- Percent funded and **next celebration threshold** (25%, 50%, 75%, 100%)
- **Achieved** thresholds for badges or timelines
- Optional **funding velocity** and rough **ETA** (when history + timestamps are available)
- A **chart-friendly series** (normalized 0–100) for bars or sparklines

All user-facing strings derived from API data are **sanitized** before use.

## Files

| File | Role |
|------|------|
| `milestone_insights.tsx` | Types, `MilestoneInsightsEngine`, `computeCampaignMilestoneInsights`, `formatCompactAmount`, `buildSparklinePolylinePoints`, `MilestoneInsightsPanel` |
| `milestone_insights.css` | Layout, metric grid, threshold rail, sparkline, insight cards; respects `prefers-reduced-motion` |
| `milestone_insights.test.tsx` | Unit and component tests |
| `milestone_insights.md` | This document |

The home page (`frontend/pages/index.tsx`) imports `MilestoneInsightsPanel` with demo campaign data so the dashboard is visible in the app shell.

## API

### `computeCampaignMilestoneInsights(input: CampaignProgressInput): CampaignMilestoneInsightsResult`

Main entry for hooks, stores, or SSR: pass normalized numeric fields and optional history; receive display-safe `displayTitle` and structured `insights`.

### `MilestoneInsightsEngine`

| Method | Purpose |
|--------|---------|
| `sanitizeDisplayText` | Strip control chars and angle-bracket fragments; bound length (see `MAX_DISPLAY_STRING_LENGTH`) |
| `clampNonNegative` | Coerce invalid numbers to `0` |
| `isSafeCampaignId` | Validate opaque ids for use in DOM keys (`[a-zA-Z0-9_-]{1,64}`) |

### `formatCompactAmount` / `buildSparklinePolylinePoints`

- `formatCompactAmount(value, suffix?)` — compact labels for metric tiles (e.g. `12.5k`, `2.5M`).
- `buildSparklinePolylinePoints(series)` — SVG `points` string for `viewBox="0 0 100 100"` (numeric output only).

### `MilestoneInsightsPanel`

Props: `input`, optional `className`, optional `testId` (default `milestone-insights-panel`), optional `showDetailedViz` (default `true`).

Renders:

- Summary (sanitized title, percent, next milestone label)
- **Metrics** grid: raised, goal, backers, ETA (estimate), pace — when `showDetailedViz` is true
- Accessible **progressbar** with **threshold rail** (25 / 50 / 75 / 100 markers; achieved ticks highlighted)
- **SVG sparkline** from `chartSeries` when history exists and `showDetailedViz` is true
- **Insights** list with severity styling (`info` / `success` / `warning`); plain text only (no HTML injection)

## Data model

### `CampaignProgressInput`

- `raisedAmount` / `goalAmount`: same unit (e.g. smallest on-chain unit)
- `historyRaisedTotals`: non-decreasing, oldest first
- `historyTimestampsMs`: optional; must match history length; used for velocity; if omitted, spacing is assumed **one day per step** (documented limitation for demos)

## Security assumptions and notes

1. **XSS**: The panel uses React text nodes only — no `dangerouslySetInnerHTML`. Titles are passed through `sanitizeDisplayText`. Prefer keeping **celebration copy in code or CMS**, not raw user HTML.
2. **DoS / UI abuse**: String length is capped; numeric clamps prevent `Infinity` from breaking layout math.
3. **IDs**: Use `isSafeCampaignId` before concatenating external ids into keys if the format is not already controlled.
4. **Velocity / ETA**: Heuristic only — not a financial guarantee; copy in insights states that estimates are approximate.
5. **Privacy**: Do not log raw `historyRaisedTotals` in production without policy; treat as campaign analytics.

## Testing

From the repository root:

```bash
npx jest milestone_insights.test.tsx --coverage --collectCoverageFrom=milestone_insights.tsx
```

The suite covers sanitization, numeric edge cases, velocity branches (including invalid timestamps), goal-edge cases, and component rendering/accessibility attributes.

## Integration tips

- Wire `input` from your Soroban/indexer layer after converting ledger amounts to numbers.
- Map `insights` to toasts or a “celebration feed”; map `chartSeries` to your chart library (values are already 0–100).
- For **reduced motion**, wrap animations in the parent; this module only renders static bars.

## Version

Documented alongside `@version 1.0.0` in `milestone_insights.tsx`.
