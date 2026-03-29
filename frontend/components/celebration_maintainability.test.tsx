/**
 * @file celebration_maintainability.test.tsx
 * @title Test Suite — CelebrationMaintainability
 *
 * @notice Tests the maintainability celebration component and helper functions.
 *
 * @dev Coverage targets:
 *   - clampPercent boundary handling
 *   - getNextPendingMilestone selection logic
 *   - buildMaintainabilitySummary variants
 *   - CelebrationMaintainability rendering and callback behavior
 */

import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import CelebrationMaintainability, {
  clampPercent,
  formatMilestoneLabel,
  getNextPendingMilestone,
  buildMaintainabilitySummary,
  type Milestone,
} from "./celebration_maintainability";

const makeMilestone = (overrides: Partial<Milestone> = {}): Milestone => ({
  id: "milestone-1",
  label: "Launch celebration",
  targetPercent: 50,
  status: "pending",
  ...overrides,
});

describe("clampPercent", () => {
  it("clamps negative values to zero", () => {
    expect(clampPercent(-12)).toBe(0);
  });

  it("clamps values above 100 to one hundred", () => {
    expect(clampPercent(200)).toBe(100);
  });

  it("passes through finite values in range", () => {
    expect(clampPercent(75)).toBe(75);
  });

  it("returns zero for non-finite inputs", () => {
    expect(clampPercent(NaN)).toBe(0);
    expect(clampPercent(Infinity)).toBe(0);
  });
});

describe("formatMilestoneLabel", () => {
  it("returns a fallback label for invalid input", () => {
    expect(formatMilestoneLabel((null as unknown) as string)).toBe(
      "Untitled milestone",
    );
  });

  it("truncates long labels", () => {
    const label = "a".repeat(100);
    expect(formatMilestoneLabel(label, 20)).toHaveLength(20);
  });
});

describe("getNextPendingMilestone", () => {
  it("returns the closest pending milestone by target percent", () => {
    const milestones = [
      makeMilestone({ id: "2", targetPercent: 75 }),
      makeMilestone({ id: "1", targetPercent: 25 }),
      makeMilestone({ id: "3", targetPercent: 100, status: "reached" }),
    ];
    const next = getNextPendingMilestone(milestones);
    expect(next).not.toBeNull();
    expect(next?.id).toBe("1");
  });

  it("returns null when there are no pending milestones", () => {
    const milestones = [makeMilestone({ status: "celebrated" })];
    expect(getNextPendingMilestone(milestones)).toBeNull();
  });
});

describe("buildMaintainabilitySummary", () => {
  it("returns stable text when no milestone is pending", () => {
    const summary = buildMaintainabilitySummary(100, null);
    expect(summary).toContain("All scheduled milestones are complete");
  });

  it("returns ready text when progress meets the next milestone", () => {
    const milestone = makeMilestone({ targetPercent: 50, label: "50% goal" });
    expect(buildMaintainabilitySummary(50, milestone)).toContain("ready for celebration");
  });

  it("recommends review when progress is below the maintainability threshold", () => {
    const milestone = makeMilestone({ targetPercent: 75, label: "75% goal" });
    expect(buildMaintainabilitySummary(30, milestone)).toContain("Maintainability review recommended");
  });
});

describe("CelebrationMaintainability component", () => {
  it("renders campaign name and upcoming milestones", () => {
    render(
      <CelebrationMaintainability
        milestones={[makeMilestone()]}
        currentPercent={40}
        campaignName="Test campaign"
      />,
    );

    expect(screen.getByText(/Test campaign milestone maintainability/i)).toBeInTheDocument();
    expect(screen.getByText(/Upcoming maintainability milestones/i)).toBeInTheDocument();
    expect(screen.getByText(/Launch celebration/i)).toBeInTheDocument();
  });

  it("invokes the review callback when the button is clicked", () => {
    const onReview = jest.fn();

    render(
      <CelebrationMaintainability
        milestones={[makeMilestone()]}
        currentPercent={60}
        onReview={onReview}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /review/i }));
    expect(onReview).toHaveBeenCalledTimes(1);
  });

  it("renders a no-milestones message when the list is empty", () => {
    render(
      <CelebrationMaintainability milestones={[]} currentPercent={20} />,
    );
    expect(screen.getByText(/No milestones available/i)).toBeInTheDocument();
  });
});
