/**
 * @title CelebrationCompatibility
 * @notice Cross-environment milestone celebration component for the Stellar Raise
 *         crowdfunding dApp.
 *
 * @dev Designed for maximum compatibility across:
 *   - React 17 and React 18 (no concurrent-only APIs)
 *   - SSR / Next.js (no direct `window` / `document` access at module scope)
 *   - Jest + jsdom test environments
 *   - Browsers without CSS custom-property support (inline-style fallbacks)
 *
 * @custom:security
 *   - `milestoneLabel` and `campaignName` are rendered as plain text nodes;
 *     no `dangerouslySetInnerHTML` is used, eliminating XSS risk.
 *   - `onDismiss` is a caller-supplied callback; this component has no
 *     side-effects beyond local state transitions.
 *   - No network requests are made inside this component.
 *   - `autoDismissMs` is validated at runtime; non-positive values disable
 *     the timer to prevent accidental immediate dismissal.
 *
 * @custom:accessibility
 *   - Root element carries `role="dialog"` and `aria-modal="true"`.
 *   - `aria-label` is derived from `milestoneLabel` for screen-reader context.
 *   - Dismiss button receives focus on mount (keyboard-first UX).
 *   - Escape key dismisses the overlay (WCAG 2.1 SC 2.1.2).
 *   - Backdrop click dismisses the overlay for pointer users.
 *
 * @example
 * ```tsx
 * <CelebrationCompatibility
 *   milestoneLabel="Goal Reached"
 *   milestonePercent={100}
 *   campaignName="Clean Water Initiative"
 *   theme="dark"
 *   onDismiss={() => setShow(false)}
 * />
 * ```
 */

import React, { useEffect, useRef, useState, CSSProperties } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/**
 * @notice Visual theme applied to the celebration overlay.
 * @dev `"auto"` reads `prefers-color-scheme` at render time (SSR-safe via
 *      a `typeof window` guard that falls back to `"light"`).
 */
export type CelebrationTheme = 'light' | 'dark' | 'auto';

/**
 * @notice Milestone tier controlling the accent colour and badge icon.
 * @dev Derived automatically from `milestonePercent` when `tier` is omitted.
 */
export type MilestoneTier = 'bronze' | 'silver' | 'gold' | 'platinum';

/**
 * @notice Props accepted by `CelebrationCompatibility`.
 */
export interface CelebrationCompatibilityProps {
  /** Human-readable milestone label, e.g. "50% Funded". Rendered as plain text. */
  milestoneLabel: string;
  /** Numeric milestone percentage (0–100) used to derive the default tier. */
  milestonePercent: number;
  /** Campaign display name shown in the celebration message. Rendered as plain text. */
  campaignName: string;
  /** Called when the user (or auto-dismiss timer) closes the overlay. */
  onDismiss: () => void;
  /** Visual theme. Defaults to `"auto"`. */
  theme?: CelebrationTheme;
  /** Override the auto-derived milestone tier. */
  tier?: MilestoneTier;
  /** Auto-dismiss after this many milliseconds. `0` or negative = never. */
  autoDismissMs?: number;
  /** Additional CSS class names forwarded to the root element. */
  className?: string;
}

// ---------------------------------------------------------------------------
// Pure helpers (exported for unit testing)
// ---------------------------------------------------------------------------

/**
 * @notice Derives a `MilestoneTier` from a milestone percentage.
 * @dev Boundary mapping:
 *   - [−∞, 25) → bronze
 *   - [25, 50)  → silver
 *   - [50, 75)  → gold
 *   - [75, +∞]  → platinum
 * @param percent - Milestone percentage value.
 * @return The corresponding `MilestoneTier`.
 */
export function deriveMilestoneTier(percent: number): MilestoneTier {
  if (percent >= 75) return 'platinum';
  if (percent >= 50) return 'gold';
  if (percent >= 25) return 'silver';
  return 'bronze';
}

/**
 * @notice Returns the emoji badge for a given tier.
 * @param tier - The milestone tier.
 * @return An emoji string representing the tier.
 */
export function tierEmoji(tier: MilestoneTier): string {
  const map: Record<MilestoneTier, string> = {
    bronze: '🥉',
    silver: '🥈',
    gold: '🥇',
    platinum: '🏆',
  };
  return map[tier];
}

/**
 * @notice Returns the accent colour hex for a given tier.
 * @dev Used as an inline-style fallback for environments without CSS custom
 *      property support (e.g. older Android WebViews).
 * @param tier - The milestone tier.
 * @return A CSS hex colour string.
 */
export function tierAccentColor(tier: MilestoneTier): string {
  const map: Record<MilestoneTier, string> = {
    bronze: '#CD7F32',
    silver: '#A8A9AD',
    gold: '#FFD700',
    platinum: '#E5E4E2',
  };
  return map[tier];
}

/**
 * @notice Resolves the effective theme, handling the `"auto"` sentinel.
 * @dev SSR-safe: falls back to `"light"` when `window` is not available.
 * @param theme - The requested theme value.
 * @return `"light"` or `"dark"`.
 */
export function resolveTheme(theme: CelebrationTheme): 'light' | 'dark' {
  if (theme === 'light' || theme === 'dark') return theme;
  if (typeof window !== 'undefined' && window.matchMedia) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return 'light';
}

// ---------------------------------------------------------------------------
// Style helpers
// ---------------------------------------------------------------------------

function cardStyles(resolvedTheme: 'light' | 'dark', accentColor: string): CSSProperties {
  const isDark = resolvedTheme === 'dark';
  return {
    position: 'relative',
    zIndex: 1001,
    backgroundColor: isDark ? '#1A2332' : '#FFFFFF',
    color: isDark ? '#F0F4F8' : '#0A1929',
    border: `2px solid ${accentColor}`,
    borderRadius: '12px',
    padding: '2rem',
    maxWidth: '420px',
    width: '90%',
    textAlign: 'center',
    boxShadow: `0 8px 32px rgba(0,0,0,${isDark ? '0.5' : '0.15'})`,
  };
}

function backdropStyles(): CSSProperties {
  return {
    position: 'fixed',
    inset: 0,
    backgroundColor: 'rgba(0,0,0,0.6)',
    zIndex: 1000,
  };
}

function overlayStyles(): CSSProperties {
  return {
    position: 'fixed',
    inset: 0,
    zIndex: 1000,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  };
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * @notice Cross-environment milestone celebration overlay.
 * @dev See module-level NatSpec for full compatibility, security, and
 *      accessibility notes.
 * @param props - See `CelebrationCompatibilityProps`.
 */
export const CelebrationCompatibility: React.FC<CelebrationCompatibilityProps> = ({
  milestoneLabel,
  milestonePercent,
  campaignName,
  onDismiss,
  theme = 'auto',
  tier,
  autoDismissMs = 0,
  className = '',
}) => {
  const resolvedTier = tier ?? deriveMilestoneTier(milestonePercent);
  const resolvedTheme = resolveTheme(theme);
  const accentColor = tierAccentColor(resolvedTier);
  const emoji = tierEmoji(resolvedTier);

  const [visible, setVisible] = useState(true);
  const dismissBtnRef = useRef<HTMLButtonElement>(null);

  // Focus dismiss button on mount for keyboard-first UX
  useEffect(() => {
    dismissBtnRef.current?.focus();
  }, []);

  // Auto-dismiss timer — only active when autoDismissMs > 0
  useEffect(() => {
    if (autoDismissMs <= 0) return;
    const id = setTimeout(handleDismiss, autoDismissMs);
    return () => clearTimeout(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoDismissMs]);

  // Escape key handler — compatible with React 17 synthetic events and jsdom
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') handleDismiss();
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function handleDismiss() {
    setVisible(false);
    onDismiss();
  }

  if (!visible) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label={`Milestone celebration: ${milestoneLabel}`}
      className={`celebration-compatibility celebration-compatibility--${resolvedTier} celebration-compatibility--${resolvedTheme} ${className}`.trim()}
      data-testid="celebration-compatibility"
      style={overlayStyles()}
    >
      {/* Backdrop — dismisses on click for pointer users */}
      <div
        aria-hidden="true"
        data-testid="celebration-compat-backdrop"
        onClick={handleDismiss}
        style={backdropStyles()}
      />

      {/* Card */}
      <div
        data-testid="celebration-compat-card"
        style={cardStyles(resolvedTheme, accentColor)}
      >
        {/* Tier badge */}
        <div
          aria-hidden="true"
          data-testid="celebration-compat-badge"
          style={{ fontSize: '3rem', marginBottom: '0.5rem' }}
        >
          {emoji}
        </div>

        {/* Title */}
        <h2
          data-testid="celebration-compat-title"
          style={{ margin: '0 0 0.5rem', fontSize: '1.5rem', fontWeight: 700 }}
        >
          Milestone Reached!
        </h2>

        {/* Campaign name */}
        <p
          data-testid="celebration-compat-campaign"
          style={{ margin: '0 0 0.25rem', fontWeight: 600, fontSize: '1rem' }}
        >
          {campaignName}
        </p>

        {/* Milestone label */}
        <p
          data-testid="celebration-compat-label"
          style={{
            margin: '0 0 1.5rem',
            fontSize: '0.875rem',
            color: accentColor,
            fontWeight: 600,
          }}
        >
          {milestoneLabel}
        </p>

        {/* Dismiss button */}
        <button
          ref={dismissBtnRef}
          data-testid="celebration-compat-dismiss"
          onClick={handleDismiss}
          aria-label="Dismiss milestone celebration"
          style={{
            backgroundColor: accentColor,
            color: resolvedTheme === 'dark' ? '#0A1929' : '#FFFFFF',
            border: 'none',
            borderRadius: '8px',
            padding: '0.75rem 2rem',
            fontSize: '1rem',
            fontWeight: 700,
            cursor: 'pointer',
            width: '100%',
          }}
        >
          Continue
        </button>
      </div>
    </div>
  );
};

export default CelebrationCompatibility;
