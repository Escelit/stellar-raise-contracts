import React, { useEffect, useMemo, useRef, useState } from "react";

/** All supported milestone statuses for maintainability display. */
export type MilestoneStatus = "pending" | "reached" | "celebrated" | "failed";

/** A single campaign milestone used by the maintainability panel. */
export interface Milestone {
  id: string;
  label: string;
  targetPercent: number;
  status: MilestoneStatus;
  reachedAt?: number;
}

/** Props for the `CelebrationMaintainability` component. */
export interface CelebrationMaintainabilityProps {
  milestones: Milestone[];
  currentPercent: number;
  campaignName?: string;
  autoDismissMs?: number;
  onReview?: () => void;
  className?: string;
  id?: string;
}

/** Default auto-dismiss delay for the maintainability banner. */
export const DEFAULT_MAINTAINABILITY_DISMISS_MS = 5_000;

/** Threshold that indicates a campaign is on a healthy maintainability path. */
export const MAINTAINABILITY_GOAL_THRESHOLD = 75;

/** Safely clamps a percentage value to the [0, 100] range. */
export function clampPercent(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.max(0, Math.min(100, value));
}

/** Truncates milestone labels to a safe display length. */
export function formatMilestoneLabel(label: string, maxLen = 80): string {
  if (typeof label !== "string" || !label.trim()) {
    return "Untitled milestone";
  }
  const cleaned = label.trim();
  return cleaned.length <= maxLen ? cleaned : `${cleaned.slice(0, maxLen - 1)}…`;
}

/** Finds the next pending milestone sorted by target percentage. */
export function getNextPendingMilestone(milestones: Milestone[]): Milestone | null {
  const pending = milestones.filter((milestone) => milestone.status === "pending");
  if (pending.length === 0) return null;
  return pending.reduce((next, current) =>
    current.targetPercent < next.targetPercent ? current : next,
  );
}

/** Builds the maintainability summary text for the current campaign state. */
export function buildMaintainabilitySummary(
  currentPercent: number,
  nextMilestone: Milestone | null,
): string {
  const progress = clampPercent(currentPercent);
  if (!nextMilestone) {
    return "All scheduled milestones are complete. Maintainability is stable.";
  }

  if (progress >= nextMilestone.targetPercent) {
    return `Milestone “${formatMilestoneLabel(nextMilestone.label)}” is ready for celebration.`;
  }

  if (progress >= MAINTAINABILITY_GOAL_THRESHOLD) {
    return `Campaign is tracking well. Next milestone: ${formatMilestoneLabel(nextMilestone.label)}.`;
  }

  return `Maintainability review recommended before the next milestone (${formatMilestoneLabel(
    nextMilestone.label,
  )}).`;
}

function MilestoneBadge({ status }: { status: MilestoneStatus }) {
  const label = status === "celebrated" ? "Celebrated" : status === "reached" ? "Reached" : status === "pending" ? "Pending" : "Failed";
  const colour = status === "failed" ? "#d64541" : status === "celebrated" ? "#28a745" : status === "pending" ? "#ffb100" : "#3b82f6";

  return (
    <span
      style={{
        backgroundColor: colour,
        borderRadius: 999,
        color: "#fff",
        display: "inline-flex",
        fontSize: 12,
        fontWeight: 600,
        letterSpacing: "0.03em",
        padding: "0.25rem 0.65rem",
      }}
    >
      {label}
    </span>
  );
}

function MilestoneRow({ milestone }: { milestone: Milestone }) {
  return (
    <li
      style={{
        display: "flex",
        justifyContent: "space-between",
        gap: "1rem",
        marginBottom: "0.75rem",
      }}
    >
      <div>
        <div style={{ fontWeight: 600, marginBottom: "0.25rem" }}>
          {formatMilestoneLabel(milestone.label)}
        </div>
        <div style={{ color: "#6b7280", fontSize: "0.875rem" }}>
          Target: {clampPercent(milestone.targetPercent)}%
        </div>
      </div>
      <MilestoneBadge status={milestone.status} />
    </li>
  );
}

/** Renders a maintainability-focused milestone celebration banner. */
export default function CelebrationMaintainability(
  props: CelebrationMaintainabilityProps,
) {
  const {
    milestones,
    currentPercent,
    campaignName,
    autoDismissMs = DEFAULT_MAINTAINABILITY_DISMISS_MS,
    onReview,
    className,
    id,
  } = props;

  const [visible, setVisible] = useState(true);
  const nextMilestone = useMemo(() => getNextPendingMilestone(milestones), [milestones]);
  const summary = useMemo(
    () => buildMaintainabilitySummary(currentPercent, nextMilestone),
    [currentPercent, nextMilestone],
  );

  const timeoutRef = useRef<number | null>(null);

  useEffect(() => {
    if (autoDismissMs <= 0 || !visible) return undefined;
    timeoutRef.current = window.setTimeout(() => setVisible(false), autoDismissMs);
    return () => {
      if (timeoutRef.current !== null) {
        window.clearTimeout(timeoutRef.current);
      }
    };
  }, [autoDismissMs, visible]);

  if (!visible) return null;

  return (
    <section
      id={id}
      className={className}
      role="status"
      aria-live="polite"
      aria-label="Campaign maintainability celebration"
      style={{
        border: "1px solid #d1d5db",
        borderRadius: 16,
        padding: "1.25rem",
        backgroundColor: "#ffffff",
      }}
    >
      <div style={{ display: "flex", justifyContent: "space-between", gap: "1rem" }}>
        <div>
          <p
            style={{
              color: "#111827",
              fontSize: "1rem",
              fontWeight: 700,
              margin: 0,
            }}
          >
            {campaignName ? `${campaignName} milestone maintainability` : "Milestone maintainability"}
          </p>
          <p style={{ color: "#4b5563", marginTop: "0.5rem", marginBottom: "1rem" }}>
            {summary}
          </p>
        </div>
        <button
          type="button"
          onClick={onReview}
          style={{
            borderRadius: 999,
            backgroundColor: "#2563eb",
            border: "none",
            color: "#fff",
            cursor: "pointer",
            fontWeight: 700,
            padding: "0.75rem 1rem",
          }}
        >
          Review
        </button>
      </div>
      <div style={{ marginTop: "1rem" }}>
        <h2 style={{ fontSize: "0.95rem", marginBottom: "0.75rem", color: "#111827" }}>
          Upcoming maintainability milestones
        </h2>
        {milestones.length > 0 ? (
          <ul style={{ listStyle: "none", margin: 0, padding: 0 }}>
            {milestones.map((milestone) => (
              <MilestoneRow key={milestone.id} milestone={milestone} />
            ))}
          </ul>
        ) : (
          <p style={{ color: "#6b7280", margin: 0 }}>No milestones available.</p>
        )}
      </div>
    </section>
  );
}
