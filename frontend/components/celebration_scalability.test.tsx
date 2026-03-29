/**
 * @title CelebrationScalability — Comprehensive Test Suite
 * @notice Covers pure helpers, component rendering, queue draining,
 *         deduplication, auto-dismiss, manual dismiss, and accessibility.
 * @dev Targets ≥ 95% coverage of celebration_scalability.tsx.
 */
import React from "react";
import { render, screen, act, fireEvent } from "@testing-library/react";
import CelebrationScalability, {
  DEFAULT_AUTO_DISMISS_MS,
  MAX_LABEL_LENGTH,
  MAX_MILESTONES,
  clampPercent,
  findCrossedMilestones,
  isValidMilestone,
  prepareMilestones,
  sanitizeMilestoneLabel,
  type CelebrationScalabilityProps,
  type ScalableMilestone,
} from "./celebration_scalability";

// ── Setup ─────────────────────────────────────────────────────────────────────

beforeAll(() => { jest.useFakeTimers(); });
afterAll(() => { jest.useRealTimers(); });
afterEach(() => {
  jest.clearAllTimers();
  jest.clearAllMocks();
});

const MS25: ScalableMilestone = { id: "ms-25", thresholdPercent: 25, label: "25% Funded" };
const MS50: ScalableMilestone = { id: "ms-50", thresholdPercent: 50, label: "50% Funded" };
const MS75: ScalableMilestone = { id: "ms-75", thresholdPercent: 75, label: "75% Funded" };
const MS100: ScalableMilestone = { id: "ms-100", thresholdPercent: 100, label: "Goal Reached" };

function renderComponent(props: Partial<CelebrationScalabilityProps> = {}) {
  return render(
    <CelebrationScalability
      currentPercent={0}
      milestones={[MS25, MS50, MS75, MS100]}
      autoDismissMs={0}
      {...props}
    />
  );
}

// ── clampPercent ──────────────────────────────────────────────────────────────

describe("clampPercent", () => {
  it("returns 0 for NaN", () => expect(clampPercent(NaN)).toBe(0));
  it("returns 0 for non-number", () => expect(clampPercent("x" as unknown as number)).toBe(0));
  it("clamps below 0 to 0", () => expect(clampPercent(-5)).toBe(0));
  it("clamps above 100 to 100", () => expect(clampPercent(200)).toBe(100));
  it("passes through 0", () => expect(clampPercent(0)).toBe(0));
  it("passes through 50", () => expect(clampPercent(50)).toBe(50));
  it("passes through 100", () => expect(clampPercent(100)).toBe(100));
});

// ── sanitizeMilestoneLabel ────────────────────────────────────────────────────

describe("sanitizeMilestoneLabel", () => {
  it("returns empty string for non-string", () => {
    expect(sanitizeMilestoneLabel(null)).toBe("");
    expect(sanitizeMilestoneLabel(undefined)).toBe("");
    expect(sanitizeMilestoneLabel(42)).toBe("");
  });
  it("strips control characters", () => {
    expect(sanitizeMilestoneLabel("hello\x00world")).toBe("hello world");
  });
  it("collapses whitespace", () => {
    expect(sanitizeMilestoneLabel("a   b")).toBe("a b");
  });
  it("truncates to MAX_LABEL_LENGTH", () => {
    const long = "A".repeat(MAX_LABEL_LENGTH + 10);
    expect(sanitizeMilestoneLabel(long).length).toBe(MAX_LABEL_LENGTH);
  });
  it("returns empty string for blank input", () => {
    expect(sanitizeMilestoneLabel("   ")).toBe("");
  });
  it("passes through normal string", () => {
    expect(sanitizeMilestoneLabel("50% Funded")).toBe("50% Funded");
  });
});

// ── isValidMilestone ──────────────────────────────────────────────────────────

describe("isValidMilestone", () => {
  it("returns true for valid milestone", () => expect(isValidMilestone(MS25)).toBe(true));
  it("returns false for null", () => expect(isValidMilestone(null)).toBe(false));
  it("returns false for missing id", () => {
    expect(isValidMilestone({ id: "", thresholdPercent: 25, label: "x" })).toBe(false);
  });
  it("returns false for negative threshold", () => {
    expect(isValidMilestone({ id: "x", thresholdPercent: -1, label: "x" })).toBe(false);
  });
  it("returns false for threshold > 100", () => {
    expect(isValidMilestone({ id: "x", thresholdPercent: 101, label: "x" })).toBe(false);
  });
  it("returns false for non-string label", () => {
    expect(isValidMilestone({ id: "x", thresholdPercent: 50, label: 42 })).toBe(false);
  });
  it("returns true for threshold 0", () => {
    expect(isValidMilestone({ id: "x", thresholdPercent: 0, label: "x" })).toBe(true);
  });
  it("returns true for threshold 100", () => {
    expect(isValidMilestone({ id: "x", thresholdPercent: 100, label: "x" })).toBe(true);
  });
});

// ── findCrossedMilestones ─────────────────────────────────────────────────────

describe("findCrossedMilestones", () => {
  const sorted = [MS25, MS50, MS75, MS100];

  it("returns empty when progress is 0", () => {
    expect(findCrossedMilestones(sorted, 0, new Set())).toHaveLength(0);
  });
  it("returns MS25 at exactly 25%", () => {
    expect(findCrossedMilestones(sorted, 25, new Set())).toEqual([MS25]);
  });
  it("returns MS25 and MS50 at 60%", () => {
    const result = findCrossedMilestones(sorted, 60, new Set());
    expect(result).toContain(MS25);
    expect(result).toContain(MS50);
  });
  it("excludes already-celebrated milestones", () => {
    const result = findCrossedMilestones(sorted, 60, new Set(["ms-25"]));
    expect(result).not.toContain(MS25);
    expect(result).toContain(MS50);
  });
  it("returns all 4 at 100% with empty celebrated set", () => {
    expect(findCrossedMilestones(sorted, 100, new Set())).toHaveLength(4);
  });
  it("returns empty when all celebrated", () => {
    const all = new Set(["ms-25", "ms-50", "ms-75", "ms-100"]);
    expect(findCrossedMilestones(sorted, 100, all)).toHaveLength(0);
  });
});

// ── prepareMilestones ─────────────────────────────────────────────────────────

describe("prepareMilestones", () => {
  it("sorts by thresholdPercent ascending", () => {
    const result = prepareMilestones([MS100, MS25, MS50]);
    expect(result.map((m) => m.thresholdPercent)).toEqual([25, 50, 100]);
  });
  it("deduplicates by id", () => {
    const result = prepareMilestones([MS25, MS25, MS50]);
    expect(result).toHaveLength(2);
  });
  it("filters invalid milestones", () => {
    const invalid = { id: "", thresholdPercent: 50, label: "x" } as ScalableMilestone;
    expect(prepareMilestones([invalid, MS50])).toEqual([MS50]);
  });
  it("caps at MAX_MILESTONES", () => {
    const many = Array.from({ length: MAX_MILESTONES + 10 }, (_, i) => ({
      id: `ms-${i}`,
      thresholdPercent: i % 101,
      label: `Milestone ${i}`,
    }));
    expect(prepareMilestones(many).length).toBeLessThanOrEqual(MAX_MILESTONES);
  });
  it("returns empty array for empty input", () => {
    expect(prepareMilestones([])).toEqual([]);
  });
});

// ── Component: renders nothing below first threshold ─────────────────────────

describe("CelebrationScalability rendering", () => {
  it("renders nothing when progress is 0", () => {
    renderComponent({ currentPercent: 0 });
    expect(screen.queryByTestId("scalable-celebration-overlay")).toBeNull();
  });

  it("renders overlay when 25% is crossed", () => {
    renderComponent({ currentPercent: 25 });
    expect(screen.getByTestId("scalable-celebration-overlay")).toBeInTheDocument();
  });

  it("renders the milestone label", () => {
    renderComponent({ currentPercent: 25 });
    expect(screen.getByTestId("scalable-celebration-label")).toHaveTextContent("25% Funded");
  });

  it("renders threshold text", () => {
    renderComponent({ currentPercent: 50 });
    expect(screen.getByTestId("scalable-celebration-threshold")).toHaveTextContent("50%");
  });

  it("sanitizes label before render", () => {
    const dirty: ScalableMilestone = { id: "ms-x", thresholdPercent: 30, label: "hello\x00world" };
    renderComponent({ currentPercent: 30, milestones: [dirty] });
    expect(screen.getByTestId("scalable-celebration-label")).toHaveTextContent("hello world");
  });
});

// ── Component: queue draining ─────────────────────────────────────────────────

describe("CelebrationScalability queue", () => {
  it("shows queue remaining count", () => {
    // 100% crosses all 4 milestones; first is shown, 3 remain
    renderComponent({ currentPercent: 100 });
    expect(screen.getByTestId("scalable-queue-remaining")).toHaveTextContent("3 more milestones pending");
  });

  it("shows singular 'milestone' for 1 remaining", () => {
    // 50% crosses MS25 and MS50; first shown, 1 remains
    renderComponent({ currentPercent: 50 });
    expect(screen.getByTestId("scalable-queue-remaining")).toHaveTextContent("1 more milestone pending");
  });

  it("drains queue after dismiss", () => {
    renderComponent({ currentPercent: 50 });
    // First: MS25
    expect(screen.getByTestId("scalable-celebration-label")).toHaveTextContent("25% Funded");
    fireEvent.click(screen.getByTestId("scalable-dismiss-button"));
    // Second: MS50
    expect(screen.getByTestId("scalable-celebration-label")).toHaveTextContent("50% Funded");
  });
});

// ── Component: dismiss ────────────────────────────────────────────────────────

describe("CelebrationScalability dismiss", () => {
  it("hides overlay when queue is empty after dismiss", () => {
    renderComponent({ currentPercent: 25 });
    fireEvent.click(screen.getByTestId("scalable-dismiss-button"));
    expect(screen.queryByTestId("scalable-celebration-overlay")).toBeNull();
  });

  it("calls onDismiss with milestone when dismissed", () => {
    const onDismiss = jest.fn();
    renderComponent({ currentPercent: 25, onDismiss });
    fireEvent.click(screen.getByTestId("scalable-dismiss-button"));
    expect(onDismiss).toHaveBeenCalledWith(MS25);
  });

  it("dismiss button has aria-label", () => {
    renderComponent({ currentPercent: 25 });
    expect(screen.getByTestId("scalable-dismiss-button")).toHaveAttribute("aria-label", "Dismiss celebration");
  });
});

// ── Component: auto-dismiss ───────────────────────────────────────────────────

describe("CelebrationScalability auto-dismiss", () => {
  it("auto-dismisses after autoDismissMs", () => {
    renderComponent({ currentPercent: 25, milestones: [MS25], autoDismissMs: 2000 });
    expect(screen.getByTestId("scalable-celebration-overlay")).toBeInTheDocument();
    act(() => { jest.advanceTimersByTime(2000); });
    expect(screen.queryByTestId("scalable-celebration-overlay")).toBeNull();
  });

  it("calls onDismiss after auto-dismiss", () => {
    const onDismiss = jest.fn();
    renderComponent({ currentPercent: 25, milestones: [MS25], autoDismissMs: 1000, onDismiss });
    act(() => { jest.advanceTimersByTime(1000); });
    expect(onDismiss).toHaveBeenCalledWith(MS25);
  });

  it("does not auto-dismiss when autoDismissMs is 0", () => {
    renderComponent({ currentPercent: 25, milestones: [MS25], autoDismissMs: 0 });
    act(() => { jest.advanceTimersByTime(10_000); });
    expect(screen.getByTestId("scalable-celebration-overlay")).toBeInTheDocument();
  });
});

// ── Component: onCelebrate callback ──────────────────────────────────────────

describe("CelebrationScalability onCelebrate", () => {
  it("calls onCelebrate with milestone when triggered", () => {
    const onCelebrate = jest.fn();
    renderComponent({ currentPercent: 25, milestones: [MS25], onCelebrate });
    expect(onCelebrate).toHaveBeenCalledWith(MS25);
  });
});

// ── Component: deduplication ──────────────────────────────────────────────────

describe("CelebrationScalability deduplication", () => {
  it("does not re-trigger a milestone already celebrated", () => {
    const onCelebrate = jest.fn();
    const { rerender } = renderComponent({ currentPercent: 25, milestones: [MS25], onCelebrate, autoDismissMs: 0 });
    fireEvent.click(screen.getByTestId("scalable-dismiss-button"));
    rerender(
      <CelebrationScalability currentPercent={25} milestones={[MS25]} onCelebrate={onCelebrate} autoDismissMs={0} />
    );
    expect(onCelebrate).toHaveBeenCalledTimes(1);
  });
});

// ── Accessibility ─────────────────────────────────────────────────────────────

describe("CelebrationScalability accessibility", () => {
  it("overlay has role=status", () => {
    renderComponent({ currentPercent: 25 });
    expect(screen.getByRole("status")).toBeInTheDocument();
  });
  it("overlay has aria-live=polite", () => {
    renderComponent({ currentPercent: 25 });
    expect(screen.getByRole("status")).toHaveAttribute("aria-live", "polite");
  });
});

// ── Constants ─────────────────────────────────────────────────────────────────

describe("exported constants", () => {
  it("DEFAULT_AUTO_DISMISS_MS is 5000", () => expect(DEFAULT_AUTO_DISMISS_MS).toBe(5_000));
  it("MAX_LABEL_LENGTH is 80", () => expect(MAX_LABEL_LENGTH).toBe(80));
  it("MAX_MILESTONES is 100", () => expect(MAX_MILESTONES).toBe(100));
});
