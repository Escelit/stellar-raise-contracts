/**
 * celebration_portability.tsx
 *
 * Portable milestone celebration component for the Stellar Raise crowdfunding dApp.
 *
 * @description
 * Renders a configurable celebration overlay when a campaign milestone is reached.
 * Designed to be portable — drop it into any page or campaign view with minimal props.
 *
 * Security assumptions:
 * - `milestoneLabel` is treated as user-supplied text and rendered as plain text (no dangerouslySetInnerHTML).
 * - `onDismiss` callback is caller-controlled; no internal side-effects beyond state reset.
 * - No network calls are made inside this component.
 *
 * @example
 * <CelebrationPortability
 *   milestoneLabel="50% Funded"
 *   milestonePercent={50}
 *   campaignName="Ocean Cleanup Fund"
 *   onDismiss={() => setShowCelebration(false)}
 * />
 */

import React, { useEffect, useRef, useState } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Severity tier controls visual style of the celebration banner. */
export type CelebrationTier = 'bronze' | 'silver' | 'gold' | 'platinum';

export interface CelebrationPortabilityProps {
  /** Human-readable milestone label, e.g. "25% Funded" */
  milestoneLabel: string;
  /** Numeric milestone percentage (0–100) used to derive the default tier */
  milestonePercent: number;
  /** Campaign display name shown in the celebration message */
  campaignName: string;
  /** Called when the user dismisses the overlay */
  onDismiss: () => void;
  /** Override the auto-derived celebration tier */
  tier?: CelebrationTier;
  /** Auto-dismiss after this many milliseconds (0 = never) */
  autoDismissMs?: number;
  /** Extra CSS class names forwarded to the root element */
  className?: string;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Derives a CelebrationTier from a milestone percentage.
 * 0–24  → bronze
 * 25–49 → silver
 * 50–74 → gold
 * 75+   → platinum
 */
export function deriveTier(percent: number): CelebrationTier {
  if (percent >= 75) return 'platinum';
  if (percent >= 50) return 'gold';
  if (percent >= 25) return 'silver';
  return 'bronze';
}

/** Maps a tier to a human-friendly emoji badge. */
export function tierBadge(tier: CelebrationTier): string {
  const badges: Record<CelebrationTier, string> = {
    bronze: '🥉',
    silver: '🥈',
    gold: '🥇',
    platinum: '🏆',
  };
  return badges[tier];
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * CelebrationPortability
 *
 * A self-contained, accessible milestone celebration overlay.
 * Supports auto-dismiss, keyboard dismissal (Escape), and focus trapping.
 */
export const CelebrationPortability: React.FC<CelebrationPortabilityProps> = ({
  milestoneLabel,
  milestonePercent,
  campaignName,
  onDismiss,
  tier,
  autoDismissMs = 0,
  className = '',
}) => {
  const resolvedTier = tier ?? deriveTier(milestonePercent);
  const badge = tierBadge(resolvedTier);

  const [visible, setVisible] = useState(true);
  const dismissBtnRef = useRef<HTMLButtonElement>(null);

  // Focus the dismiss button on mount for keyboard accessibility
  useEffect(() => {
    dismissBtnRef.current?.focus();
  }, []);

  // Auto-dismiss timer
  useEffect(() => {
    if (autoDismissMs <= 0) return;
    const timer = setTimeout(() => handleDismiss(), autoDismissMs);
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoDismissMs]);

  // Keyboard: dismiss on Escape
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') handleDismiss();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
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
      className={`celebration-portability celebration-portability--${resolvedTier} ${className}`.trim()}
      data-testid="celebration-portability"
    >
      {/* Backdrop */}
      <div
        className="celebration-portability__backdrop"
        onClick={handleDismiss}
        aria-hidden="true"
        data-testid="celebration-backdrop"
      />

      {/* Card */}
      <div className="celebration-portability__card" data-testid="celebration-card">
        <span className="celebration-portability__badge" aria-hidden="true">
          {badge}
        </span>

        <h2 className="celebration-portability__title">
          Milestone Reached!
        </h2>

        <p className="celebration-portability__campaign" data-testid="celebration-campaign">
          {campaignName}
        </p>

        <p className="celebration-portability__label" data-testid="celebration-label">
          {milestoneLabel}
        </p>

        <button
          ref={dismissBtnRef}
          className="celebration-portability__dismiss"
          onClick={handleDismiss}
          data-testid="celebration-dismiss"
          aria-label="Dismiss milestone celebration"
        >
          Continue
        </button>
      </div>
    </div>
  );
};

export default CelebrationPortability;
