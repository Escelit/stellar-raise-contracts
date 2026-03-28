/**
 * celebration_compatibility.test.tsx
 *
 * Comprehensive tests for CelebrationCompatibility and its exported helpers.
 *
 * Coverage:
 * - deriveMilestoneTier() boundary values and edge cases
 * - tierEmoji() mapping
 * - tierAccentColor() mapping
 * - resolveTheme() for all three theme values (light / dark / auto)
 * - Rendering: required props, optional props, data-testid presence
 * - Tier class and theme class applied to root element
 * - Dismiss: button click, backdrop click, Escape key
 * - Overlay unmounts after dismiss
 * - Auto-dismiss timer (fires, does not fire at 0, cleans up on unmount)
 * - Accessibility: role, aria-modal, aria-label
 * - Edge cases: empty strings, 0%, 100%, negative percent, percent > 100
 * - SSR-safe resolveTheme fallback (window.matchMedia absent)
 */

import React from 'react';
import { render, screen, fireEvent, act } from '@testing-library/react';
import {
  CelebrationCompatibility,
  CelebrationCompatibilityProps,
  deriveMilestoneTier,
  tierEmoji,
  tierAccentColor,
  resolveTheme,
  MilestoneTier,
  CelebrationTheme,
} from './celebration_compatibility';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function buildProps(
  overrides: Partial<CelebrationCompatibilityProps> = {}
): CelebrationCompatibilityProps {
  return {
    milestoneLabel: '50% Funded',
    milestonePercent: 50,
    campaignName: 'Ocean Cleanup Fund',
    onDismiss: jest.fn(),
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// deriveMilestoneTier()
// ---------------------------------------------------------------------------

describe('deriveMilestoneTier()', () => {
  it('returns bronze for 0', () => expect(deriveMilestoneTier(0)).toBe('bronze'));
  it('returns bronze for 24', () => expect(deriveMilestoneTier(24)).toBe('bronze'));
  it('returns silver for 25', () => expect(deriveMilestoneTier(25)).toBe('silver'));
  it('returns silver for 49', () => expect(deriveMilestoneTier(49)).toBe('silver'));
  it('returns gold for 50', () => expect(deriveMilestoneTier(50)).toBe('gold'));
  it('returns gold for 74', () => expect(deriveMilestoneTier(74)).toBe('gold'));
  it('returns platinum for 75', () => expect(deriveMilestoneTier(75)).toBe('platinum'));
  it('returns platinum for 100', () => expect(deriveMilestoneTier(100)).toBe('platinum'));
  it('returns bronze for negative percent', () => expect(deriveMilestoneTier(-5)).toBe('bronze'));
  it('returns platinum for percent > 100', () => expect(deriveMilestoneTier(200)).toBe('platinum'));
});

// ---------------------------------------------------------------------------
// tierEmoji()
// ---------------------------------------------------------------------------

describe('tierEmoji()', () => {
  const cases: [MilestoneTier, string][] = [
    ['bronze', '🥉'],
    ['silver', '🥈'],
    ['gold', '🥇'],
    ['platinum', '🏆'],
  ];
  it.each(cases)('returns correct emoji for %s', (tier, expected) => {
    expect(tierEmoji(tier)).toBe(expected);
  });
});

// ---------------------------------------------------------------------------
// tierAccentColor()
// ---------------------------------------------------------------------------

describe('tierAccentColor()', () => {
  it('returns a non-empty string for every tier', () => {
    const tiers: MilestoneTier[] = ['bronze', 'silver', 'gold', 'platinum'];
    tiers.forEach(t => expect(tierAccentColor(t).length).toBeGreaterThan(0));
  });

  it('returns distinct colours for each tier', () => {
    const colors = (['bronze', 'silver', 'gold', 'platinum'] as MilestoneTier[]).map(
      tierAccentColor
    );
    const unique = new Set(colors);
    expect(unique.size).toBe(4);
  });
});

// ---------------------------------------------------------------------------
// resolveTheme()
// ---------------------------------------------------------------------------

describe('resolveTheme()', () => {
  it('returns "light" for theme="light"', () => {
    expect(resolveTheme('light')).toBe('light');
  });

  it('returns "dark" for theme="dark"', () => {
    expect(resolveTheme('dark')).toBe('dark');
  });

  it('returns "light" for theme="auto" when matchMedia is absent (SSR fallback)', () => {
    const original = window.matchMedia;
    // @ts-ignore
    delete window.matchMedia;
    expect(resolveTheme('auto')).toBe('light');
    window.matchMedia = original;
  });

  it('returns "dark" for theme="auto" when prefers-color-scheme is dark', () => {
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: (query: string) => ({
        matches: query === '(prefers-color-scheme: dark)',
        media: query,
        onchange: null,
        addListener: jest.fn(),
        removeListener: jest.fn(),
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
        dispatchEvent: jest.fn(),
      }),
    });
    expect(resolveTheme('auto')).toBe('dark');
  });

  it('returns "light" for theme="auto" when prefers-color-scheme is light', () => {
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: (query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: jest.fn(),
        removeListener: jest.fn(),
        addEventListener: jest.fn(),
        removeEventListener: jest.fn(),
        dispatchEvent: jest.fn(),
      }),
    });
    expect(resolveTheme('auto')).toBe('light');
  });
});

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

describe('CelebrationCompatibility rendering', () => {
  it('renders the root overlay', () => {
    render(<CelebrationCompatibility {...buildProps()} />);
    expect(screen.getByTestId('celebration-compatibility')).toBeInTheDocument();
  });

  it('renders the card', () => {
    render(<CelebrationCompatibility {...buildProps()} />);
    expect(screen.getByTestId('celebration-compat-card')).toBeInTheDocument();
  });

  it('renders the backdrop', () => {
    render(<CelebrationCompatibility {...buildProps()} />);
    expect(screen.getByTestId('celebration-compat-backdrop')).toBeInTheDocument();
  });

  it('renders the dismiss button', () => {
    render(<CelebrationCompatibility {...buildProps()} />);
    expect(screen.getByTestId('celebration-compat-dismiss')).toBeInTheDocument();
  });

  it('displays the campaign name', () => {
    render(<CelebrationCompatibility {...buildProps({ campaignName: 'Solar Power Initiative' })} />);
    expect(screen.getByTestId('celebration-compat-campaign')).toHaveTextContent(
      'Solar Power Initiative'
    );
  });

  it('displays the milestone label', () => {
    render(<CelebrationCompatibility {...buildProps({ milestoneLabel: '75% Funded' })} />);
    expect(screen.getByTestId('celebration-compat-label')).toHaveTextContent('75% Funded');
  });

  it('applies the correct tier class', () => {
    render(<CelebrationCompatibility {...buildProps({ milestonePercent: 80 })} />);
    expect(screen.getByTestId('celebration-compatibility')).toHaveClass(
      'celebration-compatibility--platinum'
    );
  });

  it('applies tier override class when tier prop is provided', () => {
    render(<CelebrationCompatibility {...buildProps({ milestonePercent: 80, tier: 'bronze' })} />);
    expect(screen.getByTestId('celebration-compatibility')).toHaveClass(
      'celebration-compatibility--bronze'
    );
  });

  it('applies light theme class by default', () => {
    render(<CelebrationCompatibility {...buildProps({ theme: 'light' })} />);
    expect(screen.getByTestId('celebration-compatibility')).toHaveClass(
      'celebration-compatibility--light'
    );
  });

  it('applies dark theme class when theme="dark"', () => {
    render(<CelebrationCompatibility {...buildProps({ theme: 'dark' })} />);
    expect(screen.getByTestId('celebration-compatibility')).toHaveClass(
      'celebration-compatibility--dark'
    );
  });

  it('forwards extra className to root element', () => {
    render(<CelebrationCompatibility {...buildProps({ className: 'my-extra-class' })} />);
    expect(screen.getByTestId('celebration-compatibility')).toHaveClass('my-extra-class');
  });

  it('has correct aria role and aria-modal', () => {
    render(<CelebrationCompatibility {...buildProps()} />);
    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
  });

  it('has aria-label derived from milestoneLabel', () => {
    render(<CelebrationCompatibility {...buildProps({ milestoneLabel: '25% Funded' })} />);
    expect(screen.getByRole('dialog')).toHaveAttribute(
      'aria-label',
      'Milestone celebration: 25% Funded'
    );
  });
});

// ---------------------------------------------------------------------------
// Dismissal
// ---------------------------------------------------------------------------

describe('CelebrationCompatibility dismissal', () => {
  it('calls onDismiss when dismiss button is clicked', () => {
    const onDismiss = jest.fn();
    render(<CelebrationCompatibility {...buildProps({ onDismiss })} />);
    fireEvent.click(screen.getByTestId('celebration-compat-dismiss'));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('unmounts overlay after dismiss button click', () => {
    render(<CelebrationCompatibility {...buildProps()} />);
    fireEvent.click(screen.getByTestId('celebration-compat-dismiss'));
    expect(screen.queryByTestId('celebration-compatibility')).not.toBeInTheDocument();
  });

  it('calls onDismiss when backdrop is clicked', () => {
    const onDismiss = jest.fn();
    render(<CelebrationCompatibility {...buildProps({ onDismiss })} />);
    fireEvent.click(screen.getByTestId('celebration-compat-backdrop'));
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('calls onDismiss on Escape key press', () => {
    const onDismiss = jest.fn();
    render(<CelebrationCompatibility {...buildProps({ onDismiss })} />);
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onDismiss).toHaveBeenCalledTimes(1);
  });

  it('does NOT call onDismiss on non-Escape key press', () => {
    const onDismiss = jest.fn();
    render(<CelebrationCompatibility {...buildProps({ onDismiss })} />);
    fireEvent.keyDown(window, { key: 'Enter' });
    expect(onDismiss).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// Auto-dismiss
// ---------------------------------------------------------------------------

describe('CelebrationCompatibility auto-dismiss', () => {
  beforeEach(() => jest.useFakeTimers());
  afterEach(() => jest.useRealTimers());

  it('auto-dismisses after autoDismissMs', () => {
    const onDismiss = jest.fn();
    render(<CelebrationCompatibility {...buildProps({ onDismiss, autoDismissMs: 3000 })} />);
    expect(screen.getByTestId('celebration-compatibility')).toBeInTheDocument();
    act(() => jest.advanceTimersByTime(3000));
    expect(onDismiss).toHaveBeenCalledTimes(1);
    expect(screen.queryByTestId('celebration-compatibility')).not.toBeInTheDocument();
  });

  it('does NOT auto-dismiss when autoDismissMs is 0', () => {
    const onDismiss = jest.fn();
    render(<CelebrationCompatibility {...buildProps({ onDismiss, autoDismissMs: 0 })} />);
    act(() => jest.advanceTimersByTime(10_000));
    expect(onDismiss).not.toHaveBeenCalled();
    expect(screen.getByTestId('celebration-compatibility')).toBeInTheDocument();
  });

  it('does NOT auto-dismiss when autoDismissMs is negative', () => {
    const onDismiss = jest.fn();
    render(<CelebrationCompatibility {...buildProps({ onDismiss, autoDismissMs: -1 })} />);
    act(() => jest.advanceTimersByTime(10_000));
    expect(onDismiss).not.toHaveBeenCalled();
  });

  it('clears the timer on unmount (no memory leak)', () => {
    const onDismiss = jest.fn();
    const { unmount } = render(
      <CelebrationCompatibility {...buildProps({ onDismiss, autoDismissMs: 5000 })} />
    );
    unmount();
    act(() => jest.advanceTimersByTime(5000));
    expect(onDismiss).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

describe('CelebrationCompatibility edge cases', () => {
  it('renders with empty campaignName without crashing', () => {
    render(<CelebrationCompatibility {...buildProps({ campaignName: '' })} />);
    expect(screen.getByTestId('celebration-compat-campaign')).toHaveTextContent('');
  });

  it('renders with empty milestoneLabel without crashing', () => {
    render(<CelebrationCompatibility {...buildProps({ milestoneLabel: '' })} />);
    expect(screen.getByTestId('celebration-compat-label')).toHaveTextContent('');
  });

  it('renders correctly at 0% milestone (bronze tier)', () => {
    render(<CelebrationCompatibility {...buildProps({ milestonePercent: 0 })} />);
    expect(screen.getByTestId('celebration-compatibility')).toHaveClass(
      'celebration-compatibility--bronze'
    );
  });

  it('renders correctly at 100% milestone (platinum tier)', () => {
    render(<CelebrationCompatibility {...buildProps({ milestonePercent: 100 })} />);
    expect(screen.getByTestId('celebration-compatibility')).toHaveClass(
      'celebration-compatibility--platinum'
    );
  });

  it('handles negative milestonePercent gracefully (bronze tier)', () => {
    render(<CelebrationCompatibility {...buildProps({ milestonePercent: -10 })} />);
    expect(screen.getByTestId('celebration-compatibility')).toHaveClass(
      'celebration-compatibility--bronze'
    );
  });

  it('handles milestonePercent > 100 gracefully (platinum tier)', () => {
    render(<CelebrationCompatibility {...buildProps({ milestonePercent: 150 })} />);
    expect(screen.getByTestId('celebration-compatibility')).toHaveClass(
      'celebration-compatibility--platinum'
    );
  });

  it('does not throw when Escape is pressed after dismiss', () => {
    render(<CelebrationCompatibility {...buildProps()} />);
    fireEvent.click(screen.getByTestId('celebration-compat-dismiss'));
    expect(() => fireEvent.keyDown(window, { key: 'Escape' })).not.toThrow();
  });
});
