/**
 * @title Milestone Celebration Insights Tests
 * @notice Covers sanitization, insight computation, visualization data, and the panel component
 * @dev Mocks none required; uses jsdom + Testing Library
 * @author Stellar Raise Team
 * @version 1.0.0
 */

import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import {
  MilestoneInsightsPanel,
  MilestoneInsightsEngine,
  computeCampaignMilestoneInsights,
  formatCompactAmount,
  buildSparklinePolylinePoints,
  CELEBRATION_THRESHOLDS,
  MAX_DISPLAY_STRING_LENGTH,
  type CampaignProgressInput,
} from './milestone_insights';

const baseInput = (): CampaignProgressInput => ({
  campaignId: 'camp_1',
  campaignTitle: 'Lunar Garden',
  raisedAmount: 250,
  goalAmount: 1000,
  contributorCount: 3,
  historyRaisedTotals: [0, 100, 250],
});

describe('MilestoneInsightsEngine.sanitizeDisplayText', () => {
  it('returns empty string for nullish or non-string', () => {
    expect(MilestoneInsightsEngine.sanitizeDisplayText(null as unknown as string)).toBe('');
    expect(MilestoneInsightsEngine.sanitizeDisplayText(undefined as unknown as string)).toBe('');
    expect(MilestoneInsightsEngine.sanitizeDisplayText(42 as unknown as string)).toBe('');
  });

  it('strips angle-bracket segments and control characters', () => {
    expect(
      MilestoneInsightsEngine.sanitizeDisplayText('Hi<script>alert(1)</script>There')
    ).toBe('HiThere');
    expect(MilestoneInsightsEngine.sanitizeDisplayText('A\u0000B\u007FC')).toBe('ABC');
  });

  it('collapses whitespace and trims', () => {
    expect(MilestoneInsightsEngine.sanitizeDisplayText('  a   b  ')).toBe('a b');
  });

  it('truncates to MAX_DISPLAY_STRING_LENGTH', () => {
    const long = 'x'.repeat(MAX_DISPLAY_STRING_LENGTH + 50);
    expect(MilestoneInsightsEngine.sanitizeDisplayText(long).length).toBe(
      MAX_DISPLAY_STRING_LENGTH
    );
  });
});

describe('MilestoneInsightsEngine.clampNonNegative', () => {
  it('returns 0 for NaN, negative, or non-finite', () => {
    expect(MilestoneInsightsEngine.clampNonNegative(NaN)).toBe(0);
    expect(MilestoneInsightsEngine.clampNonNegative(-1)).toBe(0);
    expect(MilestoneInsightsEngine.clampNonNegative(Infinity)).toBe(0);
    expect(MilestoneInsightsEngine.clampNonNegative(Number.NEGATIVE_INFINITY)).toBe(0);
  });

  it('returns value for valid numbers', () => {
    expect(MilestoneInsightsEngine.clampNonNegative(0)).toBe(0);
    expect(MilestoneInsightsEngine.clampNonNegative(42.5)).toBe(42.5);
  });
});

describe('MilestoneInsightsEngine.isSafeCampaignId', () => {
  it('accepts alphanumeric, dash, underscore within length', () => {
    expect(MilestoneInsightsEngine.isSafeCampaignId('abc-1_x')).toBe(true);
    expect(MilestoneInsightsEngine.isSafeCampaignId('a'.repeat(64))).toBe(true);
  });

  it('rejects empty, too long, or unsafe characters', () => {
    expect(MilestoneInsightsEngine.isSafeCampaignId('')).toBe(false);
    expect(MilestoneInsightsEngine.isSafeCampaignId('a'.repeat(65))).toBe(false);
    expect(MilestoneInsightsEngine.isSafeCampaignId('../x')).toBe(false);
    expect(MilestoneInsightsEngine.isSafeCampaignId('x y')).toBe(false);
  });
});

describe('formatCompactAmount', () => {
  it('formats small integers without suffix', () => {
    expect(formatCompactAmount(0)).toBe('0');
    expect(formatCompactAmount(9999)).toBe('9999');
  });

  it('uses k/M/B compact forms', () => {
    expect(formatCompactAmount(12_500)).toMatch(/^12\.5k$/);
    expect(formatCompactAmount(150_000)).toBe('150k');
    expect(formatCompactAmount(2_500_000)).toMatch(/^2\.5M$/);
    expect(formatCompactAmount(12_000_000_000)).toMatch(/^12B$/);
  });

  it('appends suffix when provided', () => {
    expect(formatCompactAmount(1000, ' ꜩ')).toContain('ꜩ');
  });
});

describe('buildSparklinePolylinePoints', () => {
  it('returns null for empty series', () => {
    expect(buildSparklinePolylinePoints([])).toBeNull();
  });

  it('centers a single point horizontally', () => {
    expect(buildSparklinePolylinePoints([{ label: 'a', value: 40 }])).toBe('50.00,60.00');
  });

  it('spreads multiple points across the viewBox width', () => {
    const s = buildSparklinePolylinePoints([
      { label: 'a', value: 0 },
      { label: 'b', value: 100 },
    ]);
    expect(s).toContain('0.00,100.00');
    expect(s).toContain('100.00,0.00');
  });
});

describe('computeCampaignMilestoneInsights', () => {
  it('computes percent funded and next threshold', () => {
    const r = computeCampaignMilestoneInsights(baseInput());
    expect(r.percentFunded).toBe(25);
    expect(r.nextThresholdPercent).toBe(50);
    expect(r.achievedThresholds).toEqual([25]);
    expect(r.isGoalReached).toBe(false);
  });

  it('caps percent at 100 when raised exceeds goal', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      raisedAmount: 2000,
      goalAmount: 1000,
    });
    expect(r.percentFunded).toBe(100);
    expect(r.isGoalReached).toBe(true);
    expect(r.nextThresholdPercent).toBeNull();
    expect(r.insights.some((i) => i.id === 'goal-complete')).toBe(true);
  });

  it('treats goal zero as no percent milestones and emits warning', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      goalAmount: 0,
      raisedAmount: 500,
    });
    expect(r.percentFunded).toBe(0);
    expect(r.nextThresholdPercent).toBeNull();
    expect(r.insights.some((i) => i.id === 'no-goal')).toBe(true);
  });

  it('sanitizes campaign title in result', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      campaignTitle: 'Evil<img src=x onerror=alert(1)>',
    });
    expect(r.displayTitle).not.toMatch(/</);
    expect(r.displayTitle).toContain('Evil');
  });

  it('floors contributor count and adds first-backer insight', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      contributorCount: 1,
    });
    expect(r.insights.some((i) => i.id === 'first-backer')).toBe(true);
  });

  it('adds community insight for 10+ backers', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      contributorCount: 10,
    });
    expect(r.insights.some((i) => i.id === 'community')).toBe(true);
  });

  it('computes velocity and ETA from aligned timestamps', () => {
    const t0 = Date.UTC(2026, 0, 1);
    const t1 = Date.UTC(2026, 0, 11);
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      raisedAmount: 500,
      goalAmount: 1000,
      historyRaisedTotals: [0, 500],
      historyTimestampsMs: [t0, t1],
    });
    expect(r.velocityPerDay).toBe(50);
    expect(r.estimatedDaysToGoal).toBe(10);
    expect(r.insights.some((i) => i.id === 'velocity')).toBe(true);
  });

  it('returns null velocity when timestamps are invalid or non-increasing', () => {
    expect(
      computeCampaignMilestoneInsights({
        ...baseInput(),
        historyRaisedTotals: [0, 100],
        historyTimestampsMs: [100, 100],
      }).velocityPerDay
    ).toBeNull();
    expect(
      computeCampaignMilestoneInsights({
        ...baseInput(),
        historyRaisedTotals: [0, 100],
        historyTimestampsMs: [200, 100],
      }).velocityPerDay
    ).toBeNull();
  });

  it('returns null velocity when history decreases', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      historyRaisedTotals: [300, 100],
    });
    expect(r.velocityPerDay).toBeNull();
  });

  it('returns null velocity when delta raised is zero', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      historyRaisedTotals: [100, 100],
    });
    expect(r.velocityPerDay).toBeNull();
  });

  it('uses synthetic day spacing when timestamps omitted', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      historyRaisedTotals: [0, 100, 200],
    });
    expect(r.velocityPerDay).toBeCloseTo(100 / 2, 5);
  });

  it('does not add velocity insight when goal already reached', () => {
    const t0 = Date.UTC(2026, 0, 1);
    const t1 = Date.UTC(2026, 0, 11);
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      raisedAmount: 1000,
      goalAmount: 1000,
      historyRaisedTotals: [0, 1000],
      historyTimestampsMs: [t0, t1],
    });
    expect(r.isGoalReached).toBe(true);
    expect(r.insights.some((i) => i.id === 'velocity')).toBe(false);
  });

  it('handles missing or non-array history', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      historyRaisedTotals: null as unknown as number[],
    });
    expect(r.chartSeries).toEqual([]);
    expect(r.velocityPerDay).toBeNull();
  });

  it('maps chart series to percent of goal', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      goalAmount: 200,
      historyRaisedTotals: [0, 100, 200],
    });
    expect(r.chartSeries.map((p) => p.value)).toEqual([0, 50, 100]);
  });

  it('achieves all thresholds at full funding', () => {
    const r = computeCampaignMilestoneInsights({
      ...baseInput(),
      raisedAmount: 1000,
      goalAmount: 1000,
    });
    expect(r.achievedThresholds).toEqual([...CELEBRATION_THRESHOLDS]);
  });
});

describe('MilestoneInsightsPanel', () => {
  it('renders title, percent, and progress bar', () => {
    render(<MilestoneInsightsPanel input={baseInput()} />);
    expect(screen.getByTestId('insight-title')).toHaveTextContent('Lunar Garden');
    expect(screen.getByTestId('insight-percent')).toHaveTextContent('25.0% funded');
    const bar = screen.getByTestId('funding-progress-bar');
    expect(bar).toHaveAttribute('aria-valuenow', '25');
    expect(screen.getByTestId('funding-progress-fill')).toHaveStyle({
      width: '25%',
    });
  });

  it('shows goal reached styling and copy', () => {
    render(
      <MilestoneInsightsPanel
        input={{ ...baseInput(), raisedAmount: 1000, goalAmount: 1000 }}
      />
    );
    expect(screen.getByTestId('insight-percent')).toHaveTextContent('goal reached');
    expect(screen.getByTestId('funding-progress-fill')).toHaveClass(
      'milestone-insights__bar-fill--done'
    );
    expect(screen.queryByTestId('insight-next')).not.toBeInTheDocument();
  });

  it('renders sparkline when history exists', () => {
    render(<MilestoneInsightsPanel input={baseInput()} />);
    const svg = screen.getByTestId('funding-sparkline');
    expect(svg.querySelector('polyline')?.getAttribute('points')).toBeTruthy();
  });

  it('omits sparkline without history points', () => {
    render(
      <MilestoneInsightsPanel
        input={{ ...baseInput(), historyRaisedTotals: [] }}
      />
    );
    expect(screen.queryByTestId('funding-sparkline')).not.toBeInTheDocument();
  });

  it('renders metrics, threshold rail, and pace', () => {
    render(<MilestoneInsightsPanel input={baseInput()} />);
    expect(screen.getByTestId('metric-raised')).toHaveTextContent('250');
    expect(screen.getByTestId('metric-goal')).toHaveTextContent('1000');
    expect(screen.getByTestId('metric-backers')).toHaveTextContent('3');
    expect(screen.getByTestId('threshold-rail')).toBeInTheDocument();
    expect(screen.getByTestId('metric-pace')).toBeInTheDocument();
  });

  it('hides detailed visualization when showDetailedViz is false', () => {
    render(
      <MilestoneInsightsPanel input={baseInput()} showDetailedViz={false} />
    );
    expect(screen.queryByTestId('metric-raised')).not.toBeInTheDocument();
    expect(screen.queryByTestId('threshold-rail')).not.toBeInTheDocument();
    expect(screen.queryByTestId('funding-sparkline')).not.toBeInTheDocument();
  });

  it('lists insight rows with stable ids', () => {
    render(<MilestoneInsightsPanel input={baseInput()} />);
    expect(screen.getByTestId('insight-list')).toBeInTheDocument();
    expect(screen.getByTestId('insight-next-milestone')).toBeInTheDocument();
  });

  it('respects custom testId and className', () => {
    const { container } = render(
      <MilestoneInsightsPanel
        input={baseInput()}
        testId="custom-panel"
        className="wrap"
      />
    );
    expect(screen.getByTestId('custom-panel')).toBeInTheDocument();
    expect(container.firstChild).toHaveClass('milestone-insights', 'wrap');
  });
});
