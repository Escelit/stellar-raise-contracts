/**
 * celebration_portability.test.tsx
 *
 * Comprehensive tests for the CelebrationPortability component and its helpers.
 *
 * Coverage areas:
 * - deriveTier() boundary values
 * - tierBadge() mapping
 * - Rendering with required props
 * - Auto-dismiss timer
 * - Keyboard (Escape) dismissal
 * - Backdrop click dismissal
 * - Dismiss button click
 * - Tier override prop
 * - Visibility toggle (unmounts after dismiss)
 * - Accessibility attributes
 * - Edge cases: 0%, 100%, negative percent, empty strings
 */

import React from 'react';
import { render, screen, fireEvent, act } from '@testing-library/react';
import {
  CelebrationPortability,
  CelebrationPortabilityProps,
  deriveTier,
  tierBadge,
  CelebrationTier,
} from './celebration_portability';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function buildProps(overrides: Partial<CelebrationPortabilityProps> = {}): CelebrationPortabilityProps {
  return {
    milestoneLabel: '50% Funded',
    milestonePercent: 50,
    campaignName: 'Ocean Cleanup Fund',
    onDismiss: jest.fn(),
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// deriveTier()
// ---------------------------------------------------------------------------

describe('deriveTier()', () => {
  it('returns bronze for 0%', () => {
    expect(deriveTier(0)).toBe('bronze');
  });

  it('returns bronze for 24%', () => {
    expect(deriveTier(24)).toBe('bronze');
  });

  it('returns silver for 25%', () => {
    expect(deriveTier(25)).toBe('silver');
  });

  it('returns silver for 49%', () => {
    expect(deriveTier(49)).toBe('silver');
  });

  it('returns gold for 50%', () => {
    expect(deriveTier(50)).toBe('gold');
  });

  it('returns gold for 74%', () => {
    expect(deriveTier(74)).toBe('gold');
  });

  it('returns platinum for 75%', () => {
    expect(deriveTier(75)).toBe('platinum');
  });

  it('returns platinum for 100%', () => {
    expect(deriveTier(100)).toBe('platinum');
  });

  it('returns bronze for negative percent (edge case)', () => {
    expect(deriveTier(-10)).toBe('bronze');
  });

  it('returns platinum for percent > 100 (edge case)', () => {
    expect(deriveTier(150)).toBe('platinum');
  });
});

// ---------------------------------------------------------------------------
// tierBadge()
// ---------------------------------------------------------------------------

describe('tierBadge()', () => {
  const cases: [CelebrationTier, string][] = [
    ['bronze', '🥉'],
    ['silver', '🥈'],
    ['gold', '🥇'],
    ['platinum', '🏆'],
  ];

  it.each(cases)('returns correct badge for %s', (tier, expected) => {
    expect(tierBadge(tier)).toBe(expected);
  });
});

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

describe('CelebrationPortability rendering', () => {
  it('renders the celebration card', () => {
    render(<CelebrationPortability {...buildProps()} />);
    expect(screen.getByTestId('celebration-portability')).toBeInTheDocument();
    expect(screen.getByTestId('celebration-card')).toBeInTheDocument();
  });

  it('displays the campaign name', () => {
    render(<CelebrationPortability {...buildProps({ campaignName: 'Solar Power Initiative' })} />);
    expect(screen.getByTestId('celebration-campaign')).toHaveTextContent('Solar Power Initiative');
  });

  it('displays the milestone label', () => {
    render(<CelebrationPortability {...buildProps({ milestoneLabel: '75% Funded' })} />);
    expect(screen.getByTestId('celebration-label')).toHaveTextContent('75% Funded');
  });

  it('renders the dismiss button', () => {
    render(<CelebrationPortability {...buildProps()} />);
    expect(screen.getByTestId('celebration-dismiss')).toBeInTheDocument();
  });

  it('renders the backdrop', () => {
    render(<CelebrationPortability {...buildProps()} />);
    expect(screen.getByTestId('celebration-backdrop')).toBeInTheDocument();
  });

  it('applies the correct tier class', () => {
    render(<CelebrationPortability {...buildProps({ milestonePercent: 80 })} />);
    expect(screen.getByTestId('celebration-portability')).toHaveClass(
      'celebration-portability--platinum'
    );
  });

  it('applies tier override class when tier prop is provided', () => {
    render(<CelebrationPortability {...buildProps({ milestonePercent: 80, tier: 'bronze' })} />);
    expect(screen.getByTestId('celebration-portability')).toHaveClass(
      'celebration-portability--bronze'
    );
  });

  it('forwards extra className', () => {
    render(<CelebrationPortability {...buildProps({ className: 'my-custom-class' })} />);
    expect(screen.getByTestId('celebration-portability')).toHaveClass('my-custom-class');
  });

  it('has correct aria role and label', () => {
    render(<CelebrationPortability {...buildProps({ milestoneLabel: '25% Funded' })} />);
    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(dialog).toHaveAttribute('aria-label', 'Milestone celebration: 25% Funded');
  });
});

// ---------------------------------------------------------------------------
// Dismissal
// ---------------------------------------------------------------------------

describe('CelebrationPortability dismissal', () => {
  it('calls onDismiss when dismiss button is clicked', () => {
    const onDismiss = jest.fn();
    render(<CelebrationPortability {...buildProps({ onDismiss })} />);
    fireEvent.click(screen.getByTestId('celebration-dismiss'));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('unmounts the overlay after dismiss button click', () => {
    render(<CelebrationPortability {...buildProps()} />);
    fireEvent.click(screen.getByTestId('celebration-dismiss'));
    expect(screen.queryByTestId('celebration-portability')).not.toBeInTheDocument();
  });

  it('calls onDismiss when backdrop is clicked', () => {
    const onDismiss = jest.fn();
    render(<CelebrationPortability {...buildProps({ onDismiss })} />);
    fireEvent.click(screen.getByTestId('celebration-backdrop'));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('calls onDismiss on Escape key press', () => {
    const onDismiss = jest.fn();
    render(<CelebrationPortability {...buildProps({ onDismiss })} />);
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('does NOT call onDismiss on non-Escape key press', () => {
    const onDismiss = jest.fn();
    render(<CelebrationPortability {...buildProps({ onDismiss })} />);
    fireEvent.keyDown(window, { key: 'Enter' });
    expect(onDismiss).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// Auto-dismiss
// ---------------------------------------------------------------------------

describe('CelebrationPortability auto-dismiss', () => {
  beforeEach(() => jest.useFakeTimers());
  afterEach(() => jest.useRealTimers());

  it('auto-dismisses after autoDismissMs', () => {
    const onDismiss = jest.fn();
    render(<CelebrationPortability {...buildProps({ onDismiss, autoDismissMs: 3000 })} />);
    expect(screen.getByTestId('celebration-portability')).toBeInTheDocument();
    act(() => jest.advanceTimersByTime(3000));
    expect(onDismiss).toHaveBeenCalledTimes(1);
    expect(screen.queryByTestId('celebration-portability')).not.toBeInTheDocument();
  });

  it('does NOT auto-dismiss when autoDismissMs is 0', () => {
    const onDismiss = jest.fn();
    render(<CelebrationPortability {...buildProps({ onDismiss, autoDismissMs: 0 })} />);
    act(() => jest.advanceTimersByTime(10000));
    expect(onDismiss).not.toHaveBeenCalled();
    expect(screen.getByTestId('celebration-portability')).toBeInTheDocument();
  });

  it('clears the timer on unmount (no memory leak)', () => {
    const onDismiss = jest.fn();
    const { unmount } = render(
      <CelebrationPortability {...buildProps({ onDismiss, autoDismissMs: 5000 })} />
    );
    unmount();
    act(() => jest.advanceTimersByTime(5000));
    // onDismiss should NOT be called after unmount
    expect(onDismiss).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

describe('CelebrationPortability edge cases', () => {
  it('renders with empty campaignName without crashing', () => {
    render(<CelebrationPortability {...buildProps({ campaignName: '' })} />);
    expect(screen.getByTestId('celebration-campaign')).toHaveTextContent('');
  });

  it('renders with empty milestoneLabel without crashing', () => {
    render(<CelebrationPortability {...buildProps({ milestoneLabel: '' })} />);
    expect(screen.getByTestId('celebration-label')).toHaveTextContent('');
  });

  it('renders correctly at 0% milestone', () => {
    render(<CelebrationPortability {...buildProps({ milestonePercent: 0, milestoneLabel: '0% Funded' })} />);
    expect(screen.getByTestId('celebration-portability')).toHaveClass('celebration-portability--bronze');
  });

  it('renders correctly at 100% milestone', () => {
    render(<CelebrationPortability {...buildProps({ milestonePercent: 100, milestoneLabel: '100% Funded' })} />);
    expect(screen.getByTestId('celebration-portability')).toHaveClass('celebration-portability--platinum');
  });

  it('does not render after onDismiss is triggered twice (idempotent)', () => {
    const onDismiss = jest.fn();
    render(<CelebrationPortability {...buildProps({ onDismiss })} />);
    fireEvent.click(screen.getByTestId('celebration-dismiss'));
    // Component is gone; second dismiss should not throw
    expect(() => fireEvent.keyDown(window, { key: 'Escape' })).not.toThrow();
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });
});
