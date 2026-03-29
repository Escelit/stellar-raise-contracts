import React, { useCallback, useEffect, useRef, useState } from "react";

/**
 * @title MilestoneFireworks
 * @notice Renders an animated fireworks celebration overlay when a crowdfunding
 *         campaign crosses a funding milestone. Fires canvas-based rocket bursts
 *         at 25 %, 50 %, 75 %, and 100 % funding, with a dismissible banner and
 *         an accessible live region for screen readers.
 *
 * @dev Security assumptions:
 *   - No dangerouslySetInnerHTML — all content rendered as React text nodes.
 *   - All user-supplied strings (campaignName) are sanitized before render.
 *   - Progress values are clamped to [0, 100] to prevent layout abuse.
 *   - Canvas drawing uses only hardcoded color palettes — no user-controlled CSS.
 *   - All timers and animation frames are cancelled on unmount to prevent leaks.
 *   - onMilestone / onDismiss callbacks are guarded against post-unmount calls.
 *
 * @custom:accessibility
 *   - role="status" + aria-live="polite" for screen-reader announcements.
 *   - Dismiss button has aria-label for assistive technology.
 *   - Canvas is aria-hidden="true" — purely decorative.
 *   - Overlay is focusable and traps focus while visible.
 */

// ── Constants ─────────────────────────────────────────────────────────────────

/** Milestone thresholds as funding percentages. */
export const FIREWORKS_MILESTONES = [25, 50, 75, 100] as const;
export type FireworksMilestone = (typeof FIREWORKS_MILESTONES)[number];

/** Auto-dismiss delay in milliseconds. 0 disables auto-dismiss. */
export const DEFAULT_FIREWORKS_DISMISS_MS = 6_000;

/** Maximum characters for campaign name display. */
export const MAX_FIREWORKS_NAME_LENGTH = 60;

/** Number of particles per firework burst. */
export const PARTICLES_PER_BURST = 48;

/** Number of simultaneous rocket launches per trigger. */
export const ROCKETS_PER_TRIGGER = 3;

/** Firework particle lifetime in milliseconds. */
export const PARTICLE_LIFETIME_MS = 1_200;

/** Canvas width in logical pixels. */
export const CANVAS_WIDTH = 400;

/** Canvas height in logical pixels. */
export const CANVAS_HEIGHT = 220;

// ── Colour palette (hardcoded — no user-controlled values) ────────────────────

/** @notice Firework burst colours per milestone. */
export const MILESTONE_COLORS: Record<FireworksMilestone, string[]> = {
  25: ["#34d399", "#6ee7b7", "#a7f3d0", "#d1fae5"],
  50: ["#60a5fa", "#93c5fd", "#bfdbfe", "#dbeafe"],
  75: ["#f59e0b", "#fbbf24", "#fcd34d", "#fde68a"],
  100: ["#f43f5e", "#fb7185", "#fda4af", "#4f46e5", "#818cf8", "#fbbf24"],
};

// ── Pure helpers ──────────────────────────────────────────────────────────────

/**
 * @notice Clamps a numeric progress value to [0, 100].
 * @param value Raw progress percentage.
 * @returns Clamped value, or 0 for non-numeric input.
 */
export function clampFireworksProgress(value: number): number {
  if (typeof value !== "number" || isNaN(value)) return 0;
  return Math.min(100, Math.max(0, value));
}

/**
 * @notice Sanitizes a user-supplied string for safe display.
 *   - Rejects non-strings.
 *   - Strips control characters (U+0000–U+001F, U+007F).
 *   - Collapses whitespace.
 *   - Truncates to maxLength.
 * @param input     Raw string.
 * @param maxLength Maximum allowed length.
 * @returns Sanitized string, or "" on invalid input.
 */
export function sanitizeFireworksLabel(
  input: unknown,
  maxLength: number,
): string {
  if (typeof input !== "string") return "";
  const cleaned = input
    .replace(/[\u0000-\u001F\u007F]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  return cleaned.slice(0, maxLength);
}

/**
 * @notice Returns the first uncelebrated milestone crossed by currentPercent.
 * @param currentPercent  Clamped progress percentage.
 * @param celebrated      Set of already-celebrated thresholds.
 * @returns The next threshold to celebrate, or null if none.
 */
export function resolveFireworksMilestone(
  currentPercent: number,
  celebrated: ReadonlySet<FireworksMilestone>,
): FireworksMilestone | null {
  for (const t of FIREWORKS_MILESTONES) {
    if (currentPercent >= t && !celebrated.has(t)) return t;
  }
  return null;
}

/**
 * @notice Returns the heading and subtitle for a given milestone threshold.
 * @param threshold Milestone threshold.
 * @returns Object with heading and subtitle strings.
 */
export function getFireworksContent(threshold: FireworksMilestone): {
  heading: string;
  subtitle: string;
} {
  const map: Record<FireworksMilestone, { heading: string; subtitle: string }> =
    {
      25: {
        heading: "25% Funded! 🌱",
        subtitle: "Great start — keep the momentum going!",
      },
      50: {
        heading: "Halfway There! 🚀",
        subtitle: "50% funded — you're on track!",
      },
      75: {
        heading: "75% Funded! ⚡",
        subtitle: "Almost there — one final push!",
      },
      100: {
        heading: "Goal Reached! 🎆",
        subtitle: "Fully funded — congratulations!",
      },
    };
  return map[threshold];
}

// ── Particle types ────────────────────────────────────────────────────────────

/** @dev Internal particle state for canvas animation. */
export interface FireworkParticle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  alpha: number;
  color: string;
  radius: number;
  born: number;
}

/**
 * @notice Generates a burst of particles from a given origin point.
 * @dev Uses trigonometric spread — no user-controlled values in the output.
 * @param x       Burst origin X (canvas pixels).
 * @param y       Burst origin Y (canvas pixels).
 * @param colors  Colour palette for this burst.
 * @param count   Number of particles to generate.
 * @param now     Current timestamp (ms) for particle birth tracking.
 * @returns Array of `FireworkParticle` objects.
 */
export function createBurst(
  x: number,
  y: number,
  colors: string[],
  count: number,
  now: number,
): FireworkParticle[] {
  const particles: FireworkParticle[] = [];
  const safeCount = Math.max(1, Math.min(200, count));
  for (let i = 0; i < safeCount; i++) {
    const angle = (i / safeCount) * Math.PI * 2;
    const speed = 1.5 + (i % 4) * 0.8;
    particles.push({
      x,
      y,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed - 1.5,
      alpha: 1,
      color: colors[i % colors.length],
      radius: 3 + (i % 3),
      born: now,
    });
  }
  return particles;
}

/**
 * @notice Advances all particles by one animation frame.
 * @dev Applies gravity (vy += 0.06) and fades alpha linearly over lifetime.
 *      Particles past their lifetime are removed.
 * @param particles  Current particle array.
 * @param now        Current timestamp (ms).
 * @returns New particle array with dead particles removed.
 */
export function stepParticles(
  particles: FireworkParticle[],
  now: number,
): FireworkParticle[] {
  return particles
    .map((p) => ({
      ...p,
      x: p.x + p.vx,
      y: p.y + p.vy,
      vy: p.vy + 0.06,
      alpha: Math.max(0, 1 - (now - p.born) / PARTICLE_LIFETIME_MS),
    }))
    .filter((p) => p.alpha > 0);
}

/**
 * @notice Draws all particles onto a 2D canvas context.
 * @dev Clears the canvas before drawing. Safe: only uses hardcoded geometry.
 * @param ctx        Canvas 2D rendering context.
 * @param particles  Particles to draw.
 * @param width      Canvas logical width.
 * @param height     Canvas logical height.
 */
export function drawParticles(
  ctx: CanvasRenderingContext2D,
  particles: FireworkParticle[],
  width: number,
  height: number,
): void {
  ctx.clearRect(0, 0, width, height);
  for (const p of particles) {
    ctx.save();
    ctx.globalAlpha = p.alpha;
    ctx.fillStyle = p.color;
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
    ctx.fill();
    ctx.restore();
  }
}

// ── Props ─────────────────────────────────────────────────────────────────────

export interface MilestoneFireworksProps {
  /** Current funding percentage (0–100). Clamped internally. */
  currentPercent: number;
  /** Optional campaign name shown in the overlay. */
  campaignName?: string;
  /** Called when a new milestone firework is triggered. */
  onMilestone?: (threshold: FireworksMilestone) => void;
  /** Called when the overlay is dismissed. */
  onDismiss?: (threshold: FireworksMilestone) => void;
  /** Auto-dismiss delay in ms. 0 disables auto-dismiss. Default: 6000. */
  autoDismissMs?: number;
}

// ── Component ─────────────────────────────────────────────────────────────────

/**
 * @notice Milestone fireworks celebration overlay.
 * @dev Renders nothing when no uncelebrated milestone has been crossed.
 *      Canvas animation runs only while the overlay is visible.
 */
const MilestoneFireworks: React.FC<MilestoneFireworksProps> = ({
  currentPercent,
  campaignName,
  onMilestone,
  onDismiss,
  autoDismissMs = DEFAULT_FIREWORKS_DISMISS_MS,
}) => {
  const [celebrated, setCelebrated] = useState<Set<FireworksMilestone>>(
    () => new Set(),
  );
  const [active, setActive] = useState<FireworksMilestone | null>(null);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const particlesRef = useRef<FireworkParticle[]>([]);
  const rafRef = useRef<number | null>(null);
  const dismissTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mountedRef = useRef(true);

  // Track mount state for post-unmount guard.
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      if (rafRef.current !== null) cancelAnimationFrame(rafRef.current);
      if (dismissTimerRef.current) clearTimeout(dismissTimerRef.current);
    };
  }, []);

  // Detect newly-crossed milestones.
  useEffect(() => {
    const clamped = clampFireworksProgress(currentPercent);
    const next = resolveFireworksMilestone(clamped, celebrated);
    if (next === null) return;

    setCelebrated((prev) => new Set([...prev, next]));
    setActive(next);
    onMilestone?.(next);

    if (autoDismissMs > 0) {
      if (dismissTimerRef.current) clearTimeout(dismissTimerRef.current);
      dismissTimerRef.current = setTimeout(() => {
        if (mountedRef.current) {
          setActive(null);
          onDismiss?.(next);
        }
      }, autoDismissMs);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentPercent]);

  // Launch fireworks animation when a milestone becomes active.
  useEffect(() => {
    if (active === null) {
      particlesRef.current = [];
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
        rafRef.current = null;
      }
      return;
    }

    const colors = MILESTONE_COLORS[active];
    const now = performance.now();

    // Seed initial bursts at staggered positions.
    const origins = [
      { x: CANVAS_WIDTH * 0.25, y: CANVAS_HEIGHT * 0.35 },
      { x: CANVAS_WIDTH * 0.5, y: CANVAS_HEIGHT * 0.2 },
      { x: CANVAS_WIDTH * 0.75, y: CANVAS_HEIGHT * 0.35 },
    ].slice(0, ROCKETS_PER_TRIGGER);

    particlesRef.current = origins.flatMap((o) =>
      createBurst(o.x, o.y, colors, PARTICLES_PER_BURST, now),
    );

    const animate = (ts: number) => {
      if (!mountedRef.current) return;
      const canvas = canvasRef.current;
      const ctx = canvas?.getContext("2d");
      if (ctx) {
        particlesRef.current = stepParticles(particlesRef.current, ts);
        drawParticles(ctx, particlesRef.current, CANVAS_WIDTH, CANVAS_HEIGHT);
      }
      if (particlesRef.current.length > 0) {
        rafRef.current = requestAnimationFrame(animate);
      } else {
        rafRef.current = null;
      }
    };

    rafRef.current = requestAnimationFrame(animate);

    return () => {
      if (rafRef.current !== null) {
        cancelAnimationFrame(rafRef.current);
        rafRef.current = null;
      }
    };
  }, [active]);

  const handleDismiss = useCallback(() => {
    if (dismissTimerRef.current) clearTimeout(dismissTimerRef.current);
    const current = active;
    setActive(null);
    if (current !== null) onDismiss?.(current);
  }, [active, onDismiss]);

  if (active === null) return null;

  const { heading, subtitle } = getFireworksContent(active);
  const safeName = sanitizeFireworksLabel(
    campaignName,
    MAX_FIREWORKS_NAME_LENGTH,
  );

  return (
    <div
      role="status"
      aria-live="polite"
      aria-label={heading}
      data-testid="fireworks-overlay"
      style={overlayStyle}
    >
      {/* Canvas — decorative, aria-hidden */}
      <canvas
        ref={canvasRef}
        width={CANVAS_WIDTH}
        height={CANVAS_HEIGHT}
        aria-hidden="true"
        data-testid="fireworks-canvas"
        style={canvasStyle}
      />

      {/* Banner */}
      <div style={bannerStyle}>
        <h2 data-testid="fireworks-heading" style={headingStyle}>
          {heading}
        </h2>
        <p data-testid="fireworks-subtitle" style={subtitleStyle}>
          {subtitle}
        </p>
        {safeName && (
          <p data-testid="fireworks-campaign" style={campaignStyle}>
            {safeName}
          </p>
        )}
        <p data-testid="fireworks-threshold" style={thresholdStyle}>
          {active}% milestone reached
        </p>
        <button
          type="button"
          onClick={handleDismiss}
          aria-label="Dismiss fireworks celebration"
          data-testid="fireworks-dismiss"
          style={dismissStyle}
        >
          Dismiss
        </button>
      </div>
    </div>
  );
};

// ── Styles (hardcoded — no user-controlled values) ────────────────────────────

const overlayStyle: React.CSSProperties = {
  position: "fixed",
  inset: 0,
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  background: "rgba(0,0,0,0.65)",
  zIndex: 1200,
};

const canvasStyle: React.CSSProperties = {
  display: "block",
  pointerEvents: "none",
  borderRadius: "0.75rem 0.75rem 0 0",
};

const bannerStyle: React.CSSProperties = {
  background: "#fff",
  borderRadius: "0 0 1rem 1rem",
  padding: "1.5rem 2rem",
  textAlign: "center",
  maxWidth: `${CANVAS_WIDTH}px`,
  width: "100%",
  boxShadow: "0 8px 32px rgba(0,0,0,0.25)",
};

const headingStyle: React.CSSProperties = {
  margin: "0 0 0.5rem",
  fontSize: "1.5rem",
  fontWeight: 700,
  color: "#1e293b",
};

const subtitleStyle: React.CSSProperties = {
  margin: "0 0 0.25rem",
  fontSize: "0.9rem",
  color: "#374151",
};

const campaignStyle: React.CSSProperties = {
  margin: "0.25rem 0",
  fontSize: "0.85rem",
  color: "#6b7280",
  fontStyle: "italic",
};

const thresholdStyle: React.CSSProperties = {
  margin: "0.5rem 0 1rem",
  fontSize: "0.8rem",
  color: "#9ca3af",
};

const dismissStyle: React.CSSProperties = {
  padding: "0.5rem 1.75rem",
  borderRadius: "0.5rem",
  border: "none",
  background: "#4f46e5",
  color: "#fff",
  fontWeight: 600,
  fontSize: "0.875rem",
  cursor: "pointer",
};

export default MilestoneFireworks;
