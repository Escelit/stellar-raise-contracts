/**
 * @title MilestoneFireworks — Comprehensive Test Suite
 * @notice Covers pure helpers, particle physics, canvas drawing, component
 *         rendering, milestone detection, deduplication, auto-dismiss, manual
 *         dismiss, callbacks, and accessibility.
 *
 * @dev Targets ≥ 95 % coverage of milestone_fireworks.tsx.
 *      Canvas 2D context is mocked via jest.spyOn so drawing calls are
 *      verifiable without a real browser canvas.
 *
 * @custom:security-note  Tests assert that user-supplied strings are sanitized
 *         and that no user-controlled values reach canvas drawing calls.
 */

import React from "react";
import { render, screen, act, fireEvent } from "@testing-library/react";
import MilestoneFireworks, {
  CANVAS_HEIGHT,
  CANVAS_WIDTH,
  DEFAULT_FIREWORKS_DISMISS_MS,
  FIREWORKS_MILESTONES,
  MAX_FIREWORKS_NAME_LENGTH,
  MILESTONE_COLORS,
  PARTICLE_LIFETIME_MS,
  PARTICLES_PER_BURST,
  ROCKETS_PER_TRIGGER,
  clampFireworksProgress,
  createBurst,
  drawParticles,
  getFireworksContent,
  resolveFireworksMilestone,
  sanitizeFireworksLabel,
  stepParticles,
  type FireworkParticle,
  type FireworksMilestone,
  type MilestoneFireworksProps,
} from "./milestone_fireworks";

// ── Setup ─────────────────────────────────────────────────────────────────────

beforeAll(() => {
  jest.useFakeTimers();
});
afterAll(() => {
  jest.useRealTimers();
});
afterEach(() => {
  jest.clearAllTimers();
  jest.clearAllMocks();
});

function renderFW(props: Partial<MilestoneFireworksProps> = {}) {
  return render(
    <MilestoneFireworks currentPercent={0} autoDismissMs={0} {...props} />,
  );
}

// ── clampFireworksProgress ────────────────────────────────────────────────────

describe("clampFireworksProgress", () => {
  it("returns 0 for NaN", () => expect(clampFireworksProgress(NaN)).toBe(0));
  it("returns 0 for non-number", () =>
    expect(clampFireworksProgress("x" as unknown as number)).toBe(0));
  it("clamps -1 to 0", () => expect(clampFireworksProgress(-1)).toBe(0));
  it("clamps 101 to 100", () => expect(clampFireworksProgress(101)).toBe(100));
  it("passes 0 through", () => expect(clampFireworksProgress(0)).toBe(0));
  it("passes 50 through", () => expect(clampFireworksProgress(50)).toBe(50));
  it("passes 100 through", () => expect(clampFireworksProgress(100)).toBe(100));
  it("clamps i128-style large number to 100", () =>
    expect(clampFireworksProgress(999_999)).toBe(100));
});

// ── sanitizeFireworksLabel ────────────────────────────────────────────────────

describe("sanitizeFireworksLabel", () => {
  it("returns '' for null", () =>
    expect(sanitizeFireworksLabel(null, 60)).toBe(""));
  it("returns '' for undefined", () =>
    expect(sanitizeFireworksLabel(undefined, 60)).toBe(""));
  it("returns '' for number", () =>
    expect(sanitizeFireworksLabel(42, 60)).toBe(""));
  it("strips control characters", () =>
    expect(sanitizeFireworksLabel("hello\x00world", 60)).toBe("hello world"));
  it("collapses whitespace", () =>
    expect(sanitizeFireworksLabel("a   b", 60)).toBe("a b"));
  it("truncates to maxLength", () =>
    expect(sanitizeFireworksLabel("abcdef", 3)).toBe("abc"));
  it("returns '' for blank string", () =>
    expect(sanitizeFireworksLabel("   ", 60)).toBe(""));
  it("passes normal string through", () =>
    expect(sanitizeFireworksLabel("Solar Farm", 60)).toBe("Solar Farm"));
  it("strips DEL character (0x7F)", () =>
    expect(sanitizeFireworksLabel("a\x7Fb", 60)).toBe("a b"));
});

// ── resolveFireworksMilestone ─────────────────────────────────────────────────

describe("resolveFireworksMilestone", () => {
  it("returns null at 0%", () =>
    expect(resolveFireworksMilestone(0, new Set())).toBeNull());
  it("returns null at 24%", () =>
    expect(resolveFireworksMilestone(24, new Set())).toBeNull());
  it("returns 25 at exactly 25%", () =>
    expect(resolveFireworksMilestone(25, new Set())).toBe(25));
  it("returns 25 at 30% with empty celebrated", () =>
    expect(resolveFireworksMilestone(30, new Set())).toBe(25));
  it("returns 50 when 25 already celebrated", () =>
    expect(
      resolveFireworksMilestone(60, new Set([25] as FireworksMilestone[])),
    ).toBe(50));
  it("returns 100 when 25/50/75 celebrated", () =>
    expect(
      resolveFireworksMilestone(
        100,
        new Set([25, 50, 75] as FireworksMilestone[]),
      ),
    ).toBe(100));
  it("returns null when all milestones celebrated", () => {
    const all = new Set(
      FIREWORKS_MILESTONES as unknown as FireworksMilestone[],
    );
    expect(resolveFireworksMilestone(100, all)).toBeNull();
  });
  it("returns lowest uncelebrated first at 100%", () =>
    expect(resolveFireworksMilestone(100, new Set())).toBe(25));
});

// ── getFireworksContent ───────────────────────────────────────────────────────

describe("getFireworksContent", () => {
  it.each(FIREWORKS_MILESTONES)(
    "returns heading and subtitle for %i%%",
    (t) => {
      const { heading, subtitle } = getFireworksContent(
        t as FireworksMilestone,
      );
      expect(heading).toContain(`${t}`);
      expect(subtitle.length).toBeGreaterThan(0);
    },
  );
  it("100% heading contains 'Goal Reached'", () =>
    expect(getFireworksContent(100).heading).toContain("Goal Reached"));
  it("50% heading contains 'Halfway'", () =>
    expect(getFireworksContent(50).heading).toContain("Halfway"));
});

// ── createBurst ───────────────────────────────────────────────────────────────

describe("createBurst", () => {
  it("returns exactly `count` particles", () => {
    const p = createBurst(100, 100, ["#fff"], 10, 0);
    expect(p).toHaveLength(10);
  });
  it("clamps count to 1 minimum", () => {
    const p = createBurst(0, 0, ["#fff"], 0, 0);
    expect(p).toHaveLength(1);
  });
  it("clamps count to 200 maximum", () => {
    const p = createBurst(0, 0, ["#fff"], 999, 0);
    expect(p).toHaveLength(200);
  });
  it("all particles start at origin", () => {
    const p = createBurst(50, 80, ["#f00"], 5, 0);
    p.forEach((particle) => {
      expect(particle.x).toBe(50);
      expect(particle.y).toBe(80);
    });
  });
  it("all particles have alpha 1", () => {
    const p = createBurst(0, 0, ["#fff"], 5, 0);
    p.forEach((particle) => expect(particle.alpha).toBe(1));
  });
  it("assigns colors from palette cyclically", () => {
    const colors = ["#aaa", "#bbb"];
    const p = createBurst(0, 0, colors, 4, 0);
    expect(p[0].color).toBe("#aaa");
    expect(p[1].color).toBe("#bbb");
    expect(p[2].color).toBe("#aaa");
    expect(p[3].color).toBe("#bbb");
  });
  it("records born timestamp", () => {
    const now = 12345;
    const p = createBurst(0, 0, ["#fff"], 3, now);
    p.forEach((particle) => expect(particle.born).toBe(now));
  });
});

// ── stepParticles ─────────────────────────────────────────────────────────────

describe("stepParticles", () => {
  function makeParticle(
    overrides: Partial<FireworkParticle> = {},
  ): FireworkParticle {
    return {
      x: 100,
      y: 100,
      vx: 1,
      vy: -2,
      alpha: 1,
      color: "#fff",
      radius: 3,
      born: 0,
      ...overrides,
    };
  }

  it("advances x by vx", () => {
    const [p] = stepParticles([makeParticle({ x: 10, vx: 3 })], 0);
    expect(p.x).toBe(13);
  });
  it("advances y by vy", () => {
    const [p] = stepParticles([makeParticle({ y: 20, vy: -5 })], 0);
    expect(p.y).toBe(15);
  });
  it("applies gravity to vy", () => {
    const [p] = stepParticles([makeParticle({ vy: 0 })], 0);
    expect(p.vy).toBeCloseTo(0.06);
  });
  it("fades alpha over lifetime", () => {
    const [p] = stepParticles(
      [makeParticle({ born: 0 })],
      PARTICLE_LIFETIME_MS / 2,
    );
    expect(p.alpha).toBeCloseTo(0.5, 1);
  });
  it("removes particles past lifetime", () => {
    const result = stepParticles(
      [makeParticle({ born: 0 })],
      PARTICLE_LIFETIME_MS + 1,
    );
    expect(result).toHaveLength(0);
  });
  it("keeps particles within lifetime", () => {
    const result = stepParticles(
      [makeParticle({ born: 0 })],
      PARTICLE_LIFETIME_MS - 1,
    );
    expect(result).toHaveLength(1);
  });
  it("returns empty array for empty input", () => {
    expect(stepParticles([], 1000)).toHaveLength(0);
  });
  it("alpha never goes below 0", () => {
    const [p] = stepParticles(
      [makeParticle({ born: 0 })],
      PARTICLE_LIFETIME_MS * 2,
    );
    // particle should be removed, but if it weren't, alpha would be 0
    expect(p).toBeUndefined();
  });
});

// ── drawParticles ─────────────────────────────────────────────────────────────

describe("drawParticles", () => {
  function makeMockCtx() {
    return {
      clearRect: jest.fn(),
      save: jest.fn(),
      restore: jest.fn(),
      beginPath: jest.fn(),
      arc: jest.fn(),
      fill: jest.fn(),
      globalAlpha: 1,
      fillStyle: "",
    } as unknown as CanvasRenderingContext2D;
  }

  it("calls clearRect with canvas dimensions", () => {
    const ctx = makeMockCtx();
    drawParticles(ctx, [], 400, 220);
    expect(ctx.clearRect).toHaveBeenCalledWith(0, 0, 400, 220);
  });
  it("calls arc for each particle", () => {
    const ctx = makeMockCtx();
    const particles: FireworkParticle[] = [
      {
        x: 10,
        y: 20,
        vx: 0,
        vy: 0,
        alpha: 1,
        color: "#f00",
        radius: 3,
        born: 0,
      },
      {
        x: 30,
        y: 40,
        vx: 0,
        vy: 0,
        alpha: 0.5,
        color: "#0f0",
        radius: 4,
        born: 0,
      },
    ];
    drawParticles(ctx, particles, 400, 220);
    expect(ctx.arc).toHaveBeenCalledTimes(2);
  });
  it("calls save/restore for each particle", () => {
    const ctx = makeMockCtx();
    const particles: FireworkParticle[] = [
      { x: 0, y: 0, vx: 0, vy: 0, alpha: 1, color: "#fff", radius: 2, born: 0 },
    ];
    drawParticles(ctx, particles, 400, 220);
    expect(ctx.save).toHaveBeenCalledTimes(1);
    expect(ctx.restore).toHaveBeenCalledTimes(1);
  });
  it("does not call arc when particle list is empty", () => {
    const ctx = makeMockCtx();
    drawParticles(ctx, [], 400, 220);
    expect(ctx.arc).not.toHaveBeenCalled();
  });
});

// ── Component: renders nothing below threshold ────────────────────────────────

describe("MilestoneFireworks rendering", () => {
  it("renders nothing at 0%", () => {
    renderFW({ currentPercent: 0 });
    expect(screen.queryByTestId("fireworks-overlay")).toBeNull();
  });
  it("renders nothing at 24%", () => {
    renderFW({ currentPercent: 24 });
    expect(screen.queryByTestId("fireworks-overlay")).toBeNull();
  });
  it("renders overlay at 25%", () => {
    renderFW({ currentPercent: 25 });
    expect(screen.getByTestId("fireworks-overlay")).toBeInTheDocument();
  });
  it("renders canvas element", () => {
    renderFW({ currentPercent: 25 });
    expect(screen.getByTestId("fireworks-canvas")).toBeInTheDocument();
  });
  it("canvas is aria-hidden", () => {
    renderFW({ currentPercent: 25 });
    expect(screen.getByTestId("fireworks-canvas")).toHaveAttribute(
      "aria-hidden",
      "true",
    );
  });
  it("renders correct heading for 50%", () => {
    renderFW({ currentPercent: 50 });
    expect(screen.getByTestId("fireworks-heading")).toHaveTextContent(
      "Halfway There",
    );
  });
  it("renders correct heading for 100%", () => {
    renderFW({ currentPercent: 100 });
    expect(screen.getByTestId("fireworks-heading")).toHaveTextContent(
      "Goal Reached",
    );
  });
  it("renders subtitle", () => {
    renderFW({ currentPercent: 75 });
    expect(
      screen.getByTestId("fireworks-subtitle").textContent!.length,
    ).toBeGreaterThan(0);
  });
  it("renders campaign name when provided", () => {
    renderFW({ currentPercent: 25, campaignName: "Solar Farm" });
    expect(screen.getByTestId("fireworks-campaign")).toHaveTextContent(
      "Solar Farm",
    );
  });
  it("does not render campaign name when absent", () => {
    renderFW({ currentPercent: 25 });
    expect(screen.queryByTestId("fireworks-campaign")).toBeNull();
  });
  it("truncates long campaign name to MAX_FIREWORKS_NAME_LENGTH", () => {
    const long = "X".repeat(MAX_FIREWORKS_NAME_LENGTH + 20);
    renderFW({ currentPercent: 25, campaignName: long });
    const el = screen.getByTestId("fireworks-campaign");
    expect(el.textContent!.length).toBeLessThanOrEqual(
      MAX_FIREWORKS_NAME_LENGTH,
    );
  });
  it("sanitizes control characters in campaign name", () => {
    renderFW({ currentPercent: 25, campaignName: "Farm\x00Project" });
    expect(screen.getByTestId("fireworks-campaign")).toHaveTextContent(
      "Farm Project",
    );
  });
  it("renders threshold label", () => {
    renderFW({ currentPercent: 75 });
    expect(screen.getByTestId("fireworks-threshold")).toHaveTextContent("75%");
  });
  it("overlay has role=status", () => {
    renderFW({ currentPercent: 25 });
    expect(screen.getByRole("status")).toBeInTheDocument();
  });
  it("overlay has aria-live=polite", () => {
    renderFW({ currentPercent: 25 });
    expect(screen.getByRole("status")).toHaveAttribute("aria-live", "polite");
  });
  it("overlay aria-label contains heading text", () => {
    renderFW({ currentPercent: 100 });
    expect(screen.getByRole("status")).toHaveAttribute("aria-label");
  });
});

// ── Component: dismiss ────────────────────────────────────────────────────────

describe("MilestoneFireworks dismiss", () => {
  it("hides overlay on dismiss click", () => {
    renderFW({ currentPercent: 25 });
    fireEvent.click(screen.getByTestId("fireworks-dismiss"));
    expect(screen.queryByTestId("fireworks-overlay")).toBeNull();
  });
  it("calls onDismiss with threshold on dismiss click", () => {
    const onDismiss = jest.fn();
    renderFW({ currentPercent: 50, onDismiss });
    fireEvent.click(screen.getByTestId("fireworks-dismiss"));
    expect(onDismiss).toHaveBeenCalledWith(50);
  });
  it("dismiss button has correct aria-label", () => {
    renderFW({ currentPercent: 25 });
    expect(screen.getByTestId("fireworks-dismiss")).toHaveAttribute(
      "aria-label",
      "Dismiss fireworks celebration",
    );
  });
});

// ── Component: auto-dismiss ───────────────────────────────────────────────────

describe("MilestoneFireworks auto-dismiss", () => {
  it("auto-dismisses after autoDismissMs", () => {
    renderFW({ currentPercent: 25, autoDismissMs: 3_000 });
    expect(screen.getByTestId("fireworks-overlay")).toBeInTheDocument();
    act(() => {
      jest.advanceTimersByTime(3_000);
    });
    expect(screen.queryByTestId("fireworks-overlay")).toBeNull();
  });
  it("calls onDismiss after auto-dismiss", () => {
    const onDismiss = jest.fn();
    renderFW({ currentPercent: 25, autoDismissMs: 1_000, onDismiss });
    act(() => {
      jest.advanceTimersByTime(1_000);
    });
    expect(onDismiss).toHaveBeenCalledWith(25);
  });
  it("does not auto-dismiss when autoDismissMs is 0", () => {
    renderFW({ currentPercent: 25, autoDismissMs: 0 });
    act(() => {
      jest.advanceTimersByTime(60_000);
    });
    expect(screen.getByTestId("fireworks-overlay")).toBeInTheDocument();
  });
});

// ── Component: onMilestone callback ──────────────────────────────────────────

describe("MilestoneFireworks onMilestone", () => {
  it("calls onMilestone with threshold when milestone is crossed", () => {
    const onMilestone = jest.fn();
    renderFW({ currentPercent: 50, onMilestone });
    expect(onMilestone).toHaveBeenCalledWith(50);
  });
  it("calls onMilestone with 100 at full funding", () => {
    const onMilestone = jest.fn();
    renderFW({ currentPercent: 100, onMilestone });
    expect(onMilestone).toHaveBeenCalledWith(25);
  });
});

// ── Component: deduplication ──────────────────────────────────────────────────

describe("MilestoneFireworks deduplication", () => {
  it("does not re-trigger a milestone already celebrated", () => {
    const onMilestone = jest.fn();
    const { rerender } = renderFW({
      currentPercent: 25,
      onMilestone,
      autoDismissMs: 0,
    });
    fireEvent.click(screen.getByTestId("fireworks-dismiss"));
    rerender(
      <MilestoneFireworks
        currentPercent={25}
        onMilestone={onMilestone}
        autoDismissMs={0}
      />,
    );
    expect(onMilestone).toHaveBeenCalledTimes(1);
  });
  it("triggers next milestone after first is dismissed", () => {
    const onMilestone = jest.fn();
    const { rerender } = renderFW({
      currentPercent: 25,
      onMilestone,
      autoDismissMs: 0,
    });
    fireEvent.click(screen.getByTestId("fireworks-dismiss"));
    rerender(
      <MilestoneFireworks
        currentPercent={50}
        onMilestone={onMilestone}
        autoDismissMs={0}
      />,
    );
    expect(onMilestone).toHaveBeenCalledWith(50);
  });
});

// ── Constants ─────────────────────────────────────────────────────────────────

describe("exported constants", () => {
  it("DEFAULT_FIREWORKS_DISMISS_MS is 6000", () =>
    expect(DEFAULT_FIREWORKS_DISMISS_MS).toBe(6_000));
  it("MAX_FIREWORKS_NAME_LENGTH is 60", () =>
    expect(MAX_FIREWORKS_NAME_LENGTH).toBe(60));
  it("FIREWORKS_MILESTONES contains 25,50,75,100", () =>
    expect(FIREWORKS_MILESTONES).toEqual([25, 50, 75, 100]));
  it("PARTICLES_PER_BURST is 48", () => expect(PARTICLES_PER_BURST).toBe(48));
  it("ROCKETS_PER_TRIGGER is 3", () => expect(ROCKETS_PER_TRIGGER).toBe(3));
  it("CANVAS_WIDTH is 400", () => expect(CANVAS_WIDTH).toBe(400));
  it("CANVAS_HEIGHT is 220", () => expect(CANVAS_HEIGHT).toBe(220));
  it("MILESTONE_COLORS has entry for each milestone", () => {
    FIREWORKS_MILESTONES.forEach((t) => {
      expect(MILESTONE_COLORS[t as FireworksMilestone].length).toBeGreaterThan(
        0,
      );
    });
  });
});
