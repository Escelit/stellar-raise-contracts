/**
 * @title Campaign Milestone Celebration Insights
 * @notice Computes safe, display-ready insights and visualization data for crowdfunding milestone UI
 * @dev Pure functions and a small React panel; no unsafe HTML. Use {@link MilestoneInsightsEngine} for validation helpers.
 * @author Stellar Raise Team
 * @version 1.0.0
 */

import React, { useMemo } from 'react';

import './milestone_insights.css';

/** @notice Hard cap for user-origin strings shown in the UI (DoS / layout abuse mitigation) */
export const MAX_DISPLAY_STRING_LENGTH = 200;

/** @notice Standard funding thresholds used for celebration milestones (percent of goal) */
export const CELEBRATION_THRESHOLDS = [25, 50, 75, 100] as const;

/**
 * @notice Raw campaign inputs from API or chain indexer (numbers may be unvalidated)
 * @dev All string fields should pass through {@link MilestoneInsightsEngine.sanitizeDisplayText} before display
 */
export interface CampaignProgressInput {
  /** @notice Opaque campaign identifier (not rendered by default; available for keys) */
  campaignId: string;
  /** @notice Human-readable title; may contain unsafe characters until sanitized */
  campaignTitle: string;
  /** @notice Amount raised in smallest display unit (e.g. stroops or cents) */
  raisedAmount: number;
  /** @notice Funding goal in the same unit as raisedAmount */
  goalAmount: number;
  /** @notice Number of distinct contributors */
  contributorCount: number;
  /** @notice Monotonic non-decreasing series of raised totals, oldest first */
  historyRaisedTotals: number[];
  /** @notice Optional parallel timestamps (ms since epoch); same length as historyRaisedTotals when provided */
  historyTimestampsMs?: number[];
}

/**
 * @notice A single insight row for lists, toasts, or dashboards
 * @dev Headlines and details are expected to be plain text only
 */
export interface MilestoneCelebrationInsight {
  id: string;
  severity: 'info' | 'success' | 'warning';
  headline: string;
  detail: string;
}

/**
 * @notice One point for bar/sparkline-style visualization (0–100 domain)
 */
export interface MilestoneVisualizationPoint {
  label: string;
  value: number;
}

/**
 * @notice Full result for frontend data layers and the {@link MilestoneInsightsPanel} component
 */
export interface CampaignMilestoneInsightsResult {
  percentFunded: number;
  nextThresholdPercent: number | null;
  nextThresholdLabel: string | null;
  achievedThresholds: number[];
  velocityPerDay: number | null;
  estimatedDaysToGoal: number | null;
  insights: MilestoneCelebrationInsight[];
  chartSeries: MilestoneVisualizationPoint[];
  displayTitle: string;
  isGoalReached: boolean;
}

/**
 * @title MilestoneInsightsEngine
 * @notice Static helpers for input hygiene and numeric safety
 * @dev Security-oriented: bounded strings, finite numbers, no HTML interpretation
 */
export class MilestoneInsightsEngine {
  /**
   * @notice Reduces user-controlled text to plain, bounded content safe for React text nodes
   * @dev Strips `<>`-style markup fragments and control characters; truncates to {@link MAX_DISPLAY_STRING_LENGTH}
   * @param input Raw user or API string
   * @returns Sanitized plain string
   */
  static sanitizeDisplayText(input: string): string {
    if (input == null || typeof input !== 'string') {
      return '';
    }
    const withoutControls = input.replace(/[\u0000-\u001F\u007F]/g, '');
    const noAngle = withoutControls.replace(/<[^>]*>/g, '');
    const collapsed = noAngle.replace(/\s+/g, ' ').trim();
    return collapsed.slice(0, MAX_DISPLAY_STRING_LENGTH);
  }

  /**
   * @notice Coerces a value to a finite non-negative number
   * @param n Arbitrary numeric input
   * @returns Finite number ≥ 0, or 0 if invalid
   */
  static clampNonNegative(n: number): number {
    if (typeof n !== 'number' || !Number.isFinite(n) || n < 0) {
      return 0;
    }
    return n;
  }

  /**
   * @notice Validates identifier used in DOM keys (alphanumeric, dash, underscore only)
   * @param id Candidate id string
   * @returns True if safe for use as a stable key suffix
   */
  static isSafeCampaignId(id: string): boolean {
    if (!id || id.length > 64) {
      return false;
    }
    return /^[a-zA-Z0-9_-]+$/.test(id);
  }
}

function percentOfGoal(raised: number, goal: number): number {
  const r = MilestoneInsightsEngine.clampNonNegative(raised);
  const g = MilestoneInsightsEngine.clampNonNegative(goal);
  if (g <= 0) {
    return 0;
  }
  return Math.min(100, (r / g) * 100);
}

function computeVelocityPerDay(
  history: number[],
  timestampsMs?: number[]
): number | null {
  if (!history || history.length < 2) {
    return null;
  }
  const first = history[0];
  const last = history[history.length - 1];
  if (!Number.isFinite(first) || !Number.isFinite(last) || last < first) {
    return null;
  }
  const deltaRaised = last - first;
  if (deltaRaised <= 0) {
    return null;
  }

  let spanMs: number;
  if (timestampsMs && timestampsMs.length === history.length) {
    const t0 = timestampsMs[0];
    const t1 = timestampsMs[timestampsMs.length - 1];
    if (!Number.isFinite(t0) || !Number.isFinite(t1) || t1 <= t0) {
      return null;
    }
    spanMs = t1 - t0;
  } else {
    spanMs = (history.length - 1) * 24 * 60 * 60 * 1000;
  }

  const spanDays = spanMs / (24 * 60 * 60 * 1000);
  if (spanDays <= 0) {
    return null;
  }
  return deltaRaised / spanDays;
}

function nextThreshold(currentPercent: number): number | null {
  for (const t of CELEBRATION_THRESHOLDS) {
    if (currentPercent < t) {
      return t;
    }
  }
  return null;
}

function achievedThresholds(currentPercent: number): number[] {
  return CELEBRATION_THRESHOLDS.filter((t) => currentPercent >= t);
}

/**
 * @notice Formats large amounts for compact dashboard labels (plain text, no HTML)
 * @param value Non-negative amount in campaign units
 * @param suffix Optional suffix appended without space (e.g. unit symbol)
 */
export function formatCompactAmount(value: number, suffix = ''): string {
  const v = MilestoneInsightsEngine.clampNonNegative(value);
  if (v >= 1_000_000_000) {
    return `${(v / 1_000_000_000).toFixed(v >= 10_000_000_000 ? 0 : 1)}B${suffix}`;
  }
  if (v >= 1_000_000) {
    return `${(v / 1_000_000).toFixed(v >= 10_000_000 ? 0 : 1)}M${suffix}`;
  }
  if (v >= 10_000) {
    return `${(v / 1_000).toFixed(v >= 100_000 ? 0 : 1)}k${suffix}`;
  }
  return `${Math.round(v)}${suffix}`;
}

/**
 * @notice Builds an SVG `points` string for a sparkline (viewBox 0 0 100 100)
 * @dev Coordinates are numeric only — safe to pass to SVG attributes
 * @param series Normalized 0–100 values (e.g. from {@link computeCampaignMilestoneInsights})
 * @returns Space-separated "x,y" pairs, or null if empty
 */
export function buildSparklinePolylinePoints(
  series: MilestoneVisualizationPoint[]
): string | null {
  if (!series.length) {
    return null;
  }
  const n = series.length;
  return series
    .map((pt, i) => {
      const x = n === 1 ? 50 : (i / (n - 1)) * 100;
      const y = 100 - Math.min(100, Math.max(0, pt.value));
      return `${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(' ');
}

/**
 * @notice Derives celebration insights and chart-ready series from campaign progress
 * @dev Does not mutate input. All user strings sanitized in the result.
 * @param input See {@link CampaignProgressInput}
 * @returns Structured data for UI binding
 */
export function computeCampaignMilestoneInsights(
  input: CampaignProgressInput
): CampaignMilestoneInsightsResult {
  const displayTitle = MilestoneInsightsEngine.sanitizeDisplayText(
    input.campaignTitle ?? ''
  );
  const raised = MilestoneInsightsEngine.clampNonNegative(input.raisedAmount);
  const goal = MilestoneInsightsEngine.clampNonNegative(input.goalAmount);
  const contributors = Math.floor(
    MilestoneInsightsEngine.clampNonNegative(input.contributorCount)
  );

  const history = Array.isArray(input.historyRaisedTotals)
    ? input.historyRaisedTotals.map((x) => MilestoneInsightsEngine.clampNonNegative(x))
    : [];

  const percentFunded = percentOfGoal(raised, goal);
  const isGoalReached = goal > 0 && raised >= goal;
  const next = nextThreshold(percentFunded);
  const achieved = achievedThresholds(percentFunded);

  const velocity = computeVelocityPerDay(history, input.historyTimestampsMs);
  let estimatedDaysToGoal: number | null = null;
  if (
    velocity != null &&
    velocity > 0 &&
    goal > raised &&
    !isGoalReached
  ) {
    estimatedDaysToGoal = (goal - raised) / velocity;
  }

  const insights: MilestoneCelebrationInsight[] = [];

  if (isGoalReached) {
    insights.push({
      id: 'goal-complete',
      severity: 'success',
      headline: 'Funding goal reached',
      detail: `${displayTitle || 'This campaign'} has met its target. Time to celebrate with backers.`,
    });
  } else if (goal > 0 && next != null) {
    insights.push({
      id: 'next-milestone',
      severity: 'info',
      headline: `Next celebration at ${next}%`,
      detail: `You are at ${percentFunded.toFixed(1)}% funded. Crossing ${next}% unlocks the next milestone moment in the UI.`,
    });
  }

  if (contributors === 1) {
    insights.push({
      id: 'first-backer',
      severity: 'success',
      headline: 'First contributor milestone',
      detail: 'Highlight this moment in the timeline — early backers drive momentum.',
    });
  } else if (contributors >= 10) {
    insights.push({
      id: 'community',
      severity: 'info',
      headline: 'Strong contributor base',
      detail: `${contributors} backers — good moment for a community shout-out.`,
    });
  }

  if (velocity != null && velocity > 0 && !isGoalReached && estimatedDaysToGoal != null) {
    const daysRounded = Math.ceil(estimatedDaysToGoal);
    insights.push({
      id: 'velocity',
      severity: 'info',
      headline: 'Funding velocity',
      detail: `At the recent pace, reaching the goal is roughly ${daysRounded} day(s) away (estimate only).`,
    });
  }

  if (goal <= 0) {
    insights.push({
      id: 'no-goal',
      severity: 'warning',
      headline: 'Goal not configured',
      detail: 'Set a valid funding goal to enable percent-based milestone insights.',
    });
  }

  const chartSeries: MilestoneVisualizationPoint[] = history.map((total, i) => ({
    label: `T${i + 1}`,
    value: goal > 0 ? Math.min(100, (total / goal) * 100) : 0,
  }));

  return {
    percentFunded,
    nextThresholdPercent: next,
    nextThresholdLabel:
      next != null ? `${next}% funding milestone` : null,
    achievedThresholds: achieved,
    velocityPerDay: velocity,
    estimatedDaysToGoal,
    insights,
    chartSeries,
    displayTitle,
    isGoalReached,
  };
}

export interface MilestoneInsightsPanelProps {
  /** @notice Campaign progress payload */
  input: CampaignProgressInput;
  /** @notice Optional className for layout integration */
  className?: string;
  /** @notice Root test id */
  testId?: string;
  /**
   * @notice When false, hides metrics row, threshold rail, and sparkline (compact copy-only mode)
   * @dev Defaults to true for full dashboard visualization
   */
  showDetailedViz?: boolean;
}

/**
 * @title MilestoneInsightsPanel
 * @notice Lightweight visualization + insight list for milestone celebration screens
 * @dev Renders only React text nodes and numeric formatting — no `dangerouslySetInnerHTML`
 */
export const MilestoneInsightsPanel: React.FC<MilestoneInsightsPanelProps> = ({
  input,
  className,
  testId = 'milestone-insights-panel',
  showDetailedViz = true,
}) => {
  const data = useMemo(() => computeCampaignMilestoneInsights(input), [input]);
  const sparklinePoints = useMemo(
    () => buildSparklinePolylinePoints(data.chartSeries),
    [data.chartSeries]
  );

  const raised = MilestoneInsightsEngine.clampNonNegative(input.raisedAmount);
  const goal = MilestoneInsightsEngine.clampNonNegative(input.goalAmount);
  const backers = Math.floor(
    MilestoneInsightsEngine.clampNonNegative(input.contributorCount)
  );

  const barAria = `Funding progress ${data.percentFunded.toFixed(1)} percent`;
  const etaLabel =
    data.estimatedDaysToGoal != null
      ? `${Math.ceil(data.estimatedDaysToGoal)} d`
      : '—';
  const paceLabel =
    data.velocityPerDay != null
      ? `${formatCompactAmount(data.velocityPerDay)}/d`
      : '—';

  const rootClass = ['milestone-insights', className].filter(Boolean).join(' ');

  return (
    <div className={rootClass} data-testid={testId}>
      <h2 className="milestone-insights__title">Milestone celebration insights</h2>

      <section aria-label="Campaign summary">
        <p data-testid="insight-title">
          <strong>{data.displayTitle || 'Campaign'}</strong>
        </p>
        <p data-testid="insight-percent">
          {data.percentFunded.toFixed(1)}% funded
          {data.isGoalReached ? ' — goal reached' : ''}
        </p>
        {data.nextThresholdLabel && !data.isGoalReached && (
          <p data-testid="insight-next">{data.nextThresholdLabel}</p>
        )}
      </section>

      {showDetailedViz && (
        <div className="milestone-insights__metrics" aria-label="Funding metrics">
          <div className="milestone-insights__metric">
            <span className="milestone-insights__metric-label">Raised</span>
            <span className="milestone-insights__metric-value" data-testid="metric-raised">
              {formatCompactAmount(raised)}
            </span>
          </div>
          <div className="milestone-insights__metric">
            <span className="milestone-insights__metric-label">Goal</span>
            <span className="milestone-insights__metric-value" data-testid="metric-goal">
              {formatCompactAmount(goal)}
            </span>
          </div>
          <div className="milestone-insights__metric">
            <span className="milestone-insights__metric-label">Backers</span>
            <span className="milestone-insights__metric-value" data-testid="metric-backers">
              {backers}
            </span>
          </div>
          <div className="milestone-insights__metric">
            <span className="milestone-insights__metric-label">ETA (est.)</span>
            <span className="milestone-insights__metric-value" data-testid="metric-eta">
              {etaLabel}
            </span>
          </div>
          <div className="milestone-insights__metric">
            <span className="milestone-insights__metric-label">Pace</span>
            <span className="milestone-insights__metric-value" data-testid="metric-pace">
              {paceLabel}
            </span>
          </div>
        </div>
      )}

      <div className="milestone-insights__progress-wrap">
        <div
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={Math.round(data.percentFunded)}
          aria-label={barAria}
          data-testid="funding-progress-bar"
          className="milestone-insights__bar"
        >
          <div
            data-testid="funding-progress-fill"
            className={
              data.isGoalReached
                ? 'milestone-insights__bar-fill milestone-insights__bar-fill--done'
                : 'milestone-insights__bar-fill milestone-insights__bar-fill--active'
            }
            style={{ width: `${Math.min(100, data.percentFunded)}%` }}
          />
        </div>

        {showDetailedViz && (
          <div className="milestone-insights__rail" data-testid="threshold-rail">
            {CELEBRATION_THRESHOLDS.map((t) => (
              <React.Fragment key={t}>
                <span
                  className={
                    data.achievedThresholds.includes(t)
                      ? 'milestone-insights__rail-tick milestone-insights__rail-tick--achieved'
                      : 'milestone-insights__rail-tick'
                  }
                  style={{ left: `${t}%` }}
                />
                <span className="milestone-insights__rail-label" style={{ left: `${t}%` }}>
                  {t}%
                </span>
              </React.Fragment>
            ))}
          </div>
        )}
      </div>

      {showDetailedViz && sparklinePoints && (
        <section
          className="milestone-insights__sparkline-wrap"
          aria-label="Funding trend"
        >
          <h3 className="milestone-insights__sparkline-title">Funding trend</h3>
          <svg
            className="milestone-insights__sparkline"
            viewBox="0 0 100 100"
            preserveAspectRatio="none"
            data-testid="funding-sparkline"
            aria-hidden="true"
          >
            <polyline
              fill="none"
              stroke="currentColor"
              strokeWidth={2.5}
              strokeLinejoin="round"
              strokeLinecap="round"
              vectorEffect="non-scaling-stroke"
              points={sparklinePoints}
            />
          </svg>
        </section>
      )}

      <section aria-label="Insights">
        <h3 className="milestone-insights__insights-title">Insights</h3>
        <ul className="milestone-insights__list" data-testid="insight-list">
          {data.insights.map((row) => (
            <li
              key={row.id}
              data-testid={`insight-${row.id}`}
              className={`milestone-insights__insight milestone-insights__insight--${row.severity}`}
            >
              <span className="milestone-insights__insight-headline">{row.headline}</span>
              <div>{row.detail}</div>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
};

export default MilestoneInsightsPanel;
