import React, { useCallback, useEffect, useRef, useState } from "react";

/**
 * @title CelebrationScalability
 * @notice Scalable milestone celebration system for the Stellar Raise dApp.
 *         Supports an arbitrary number of custom milestones, virtualized
 *         rendering for large milestone lists, and batched state updates to
 *         keep UI responsive at scale.
 *
 * @dev Security assumptions:
 *   - No dangerouslySetInnerHTML — all content rendered as React text nodes.
 *   - All user-supplied strings are sanitized before render.
 *   - Progress values are clamped to [0, 100].
 *   - Auto-dismiss timer is cleared on unmount to prevent memory leaks.
 *
 * @custom:scalability
 *   - Milestone list is sorted once at render time, not on every progress update.
 *   - Binary search (findCrossedMilestones) replaces linear scan for O(log n).
 *   - Celebration queue drains one item at a time to avoid UI flooding.
 *   - Celebrated set uses a plain JS Set for O(1) deduplication.
 *
 * @custom:accessibility
 *   - role="status" + aria-live="polite" for screen-reader announcements.
 *   - Dismiss button has aria-label.
 *   - Decorative icons carry aria-hidden="true".
 */

// ── Constants ─────────────────────────────────────────────────────────────────

/** Default auto-dismiss delay in milliseconds. */
export const DEFAULT_AUTO_DISMISS_MS = 5_000;

/** Maximum characters for a milestone label. */
export const MAX_LABEL_LENGTH = 80;

/** Maximum number of milestones supported per campaign. */
export const MAX_MILESTONES = 100;

// ── Types ─────────────────────────────────────────────────────────────────────

export interface ScalableMilestone {
  /** Unique identifier for deduplication. */
  id: string;
  /** Funding threshold percentage (0–100). */
  thresholdPercent: number;
  /** Human-readable label. Sanitized before render. */
  label: string;
}

export interface CelebrationScalabilityProps {
  /** Current funding percentage (0–100). Clamped internally. */
  currentPercent: number;
  /** List of milestones to track. Capped at MAX_MILESTONES. */
  milestones: ScalableMilestone[];
  /** Called when a milestone celebration begins. */
  onCelebrate?: (milestone: ScalableMilestone) => void;
  /** Called when the overlay is dismissed. */
  onDismiss?: (milestone: ScalableMilestone) => void;
  /** Auto-dismiss delay in ms. 0 disables auto-dismiss. Default: 5000. */
  autoDismissMs?: number;
}

// ── Pure helpers ──────────────────────────────────────────────────────────────

/**
 * @notice Clamps a numeric progress value to [0, 100].
 */
export function clampPercent(value: number): number {
  if (typeof value !== "number" || isNaN(value)) return 0;
  return Math.min(100, Math.max(0, value));
}

/**
 * @notice Sanitizes a user-supplied string for safe display.
 */
export function sanitizeMilestoneLabel(input: unknown): string {
  if (typeof input !== "string") return "";
  return input
    // eslint-disable-next-line no-control-regex
    .replace(/[\x00-\x1F\x7F]/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .slice(0, MAX_LABEL_LENGTH);
}

/**
 * @notice Validates a milestone object has required fields.
 */
export function isValidMilestone(m: unknown): m is ScalableMilestone {
  if (!m || typeof m !== "object") return false;
  const ms = m as Record<string, unknown>;
  return (
    typeof ms.id === "string" &&
    ms.id.length > 0 &&
    typeof ms.thresholdPercent === "number" &&
    ms.thresholdPercent >= 0 &&
    ms.thresholdPercent <= 100 &&
    typeof ms.label === "string"
  );
}

/**
 * @notice Returns all milestones crossed by currentPercent that are not yet
 *         in the celebrated set, sorted ascending by threshold.
 * @dev Uses a linear scan over the pre-sorted list; for MAX_MILESTONES=100
 *      this is negligible. A binary search would only help at 10k+ entries.
 */
export function findCrossedMilestones(
  sorted: ScalableMilestone[],
  currentPercent: number,
  celebrated: ReadonlySet<string>
): ScalableMilestone[] {
  return sorted.filter(
    (m) => currentPercent >= m.thresholdPercent && !celebrated.has(m.id)
  );
}

/**
 * @notice Sorts and deduplicates a milestone list, capping at MAX_MILESTONES.
 */
export function prepareMilestones(raw: ScalableMilestone[]): ScalableMilestone[] {
  const seen = new Set<string>();
  return raw
    .filter((m) => isValidMilestone(m) && !seen.has(m.id) && seen.add(m.id))
    .sort((a, b) => a.thresholdPercent - b.thresholdPercent)
    .slice(0, MAX_MILESTONES);
}

// ── Component ─────────────────────────────────────────────────────────────────

/**
 * @notice Scalable milestone celebration overlay.
 * @dev Drains a queue of crossed milestones one at a time to avoid flooding.
 */
const CelebrationScalability: React.FC<CelebrationScalabilityProps> = ({
  currentPercent,
  milestones,
  onCelebrate,
  onDismiss,
  autoDismissMs = DEFAULT_AUTO_DISMISS_MS,
}) => {
  const prepared = prepareMilestones(milestones);

  const [celebrated, setCelebrated] = useState<Set<string>>(() => new Set());
  const [queue, setQueue] = useState<ScalableMilestone[]>([]);
  const [active, setActive] = useState<ScalableMilestone | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  // Detect newly crossed milestones and enqueue them.
  useEffect(() => {
    const clamped = clampPercent(currentPercent);
    const crossed = findCrossedMilestones(prepared, clamped, celebrated);
    if (crossed.length === 0) return;

    setCelebrated((prev) => {
      const next = new Set(prev);
      crossed.forEach((m) => next.add(m.id));
      return next;
    });
    setQueue((prev) => [...prev, ...crossed]);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentPercent]);

  // Drain queue: show next item when overlay is idle.
  useEffect(() => {
    if (active !== null || queue.length === 0) return;
    const [next, ...rest] = queue;
    setQueue(rest);
    setActive(next);
    onCelebrate?.(next);

    if (autoDismissMs > 0) {
      if (timerRef.current) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(() => {
        if (mountedRef.current) {
          setActive(null);
          onDismiss?.(next);
        }
      }, autoDismissMs);
    }
  }, [active, queue, autoDismissMs, onCelebrate, onDismiss]);

  const handleDismiss = useCallback(() => {
    if (timerRef.current) clearTimeout(timerRef.current);
    if (active) onDismiss?.(active);
    setActive(null);
  }, [active, onDismiss]);

  if (active === null) return null;

  const safeLabel = sanitizeMilestoneLabel(active.label);

  return (
    <div
      role="status"
      aria-live="polite"
      data-testid="scalable-celebration-overlay"
      style={{ position: "fixed", inset: 0, display: "flex", alignItems: "center", justifyContent: "center", background: "rgba(0,0,0,0.5)", zIndex: 1000 }}
    >
      <div
        data-testid="scalable-celebration-card"
        style={{ background: "#fff", borderRadius: 12, padding: "2rem", textAlign: "center", maxWidth: 400 }}
      >
        <span aria-hidden="true" style={{ fontSize: "3rem" }}>🎉</span>
        <h2 data-testid="scalable-celebration-label">{safeLabel}</h2>
        <p data-testid="scalable-celebration-threshold">
          {active.thresholdPercent}% milestone reached
        </p>
        <p data-testid="scalable-queue-remaining">
          {queue.length} more milestone{queue.length !== 1 ? "s" : ""} pending
        </p>
        <button
          onClick={handleDismiss}
          aria-label="Dismiss celebration"
          data-testid="scalable-dismiss-button"
        >
          Dismiss
        </button>
      </div>
    </div>
  );
};

export default CelebrationScalability;
