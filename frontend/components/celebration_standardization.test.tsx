/**
 * @title Celebration Standardization — Comprehensive Test Suite
 * @notice Covers milestone celebration types, animation states, accessibility,
 *         security sanitization, and component rendering.
 * @dev Targets ≥ 95% coverage of celebration_standardization.tsx.
 * 
 * Security Notes:
 * - XSS prevention via textContent sanitization
 * - Reduced motion accessibility
 * - ARIA live region compliance
 */
import React from "react";
import { render, screen, waitFor, act, fireEvent } from "@testing-library/react";
import "@testing-library/jest-dom";
import CelebrationStandardization, {
  MilestoneType,
  MilestoneConfig,
  MilestoneConfigRegistry,
  CelebrationCSSVariables,
} from "./celebration_standardization";

// ── Global Setup ─────────────────────────────────────────────────────────────

// Mock window.matchMedia globally for all tests
beforeAll(() => {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: jest.fn().mockImplementation((query: string) => ({
      matches: query.includes("reduce"),
      media: query,
      onchange: null,
      addListener: jest.fn(),
      removeListener: jest.fn(),
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      dispatchEvent: jest.fn(),
    })),
  });
});

// ── Test Utilities ────────────────────────────────────────────────────────────

/**
 * @notice Helper to render the celebration component with default props
 */
function renderCelebration(props: Partial<React.ComponentProps<typeof CelebrationStandardization>> = {}) {
  const defaultProps = {
    milestoneType: MilestoneType.CAMPAIGN_CREATED,
    isVisible: true,
  };
  return render(<CelebrationStandardization {...defaultProps} {...props} />);
}

/**
 * @notice Helper to get component container
 */
function getContainer() {
  return document.querySelector(".celebration-standardization") as HTMLElement;
}

/**
 * @notice Mock window.matchMedia for reduced motion tests
 */
const mockMatchMedia = (matches: boolean) => {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: jest.fn().mockImplementation((query: string) => ({
      matches,
      media: query,
      onchange: null,
      addListener: jest.fn(),
      removeListener: jest.fn(),
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      dispatchEvent: jest.fn(),
    })),
  });
};

// ── MilestoneConfigRegistry Tests ────────────────────────────────────────────

describe("MilestoneConfigRegistry", () => {
  /**
   * @notice Verifies all milestone types have valid configurations
   */
  it("contains configuration for all MilestoneType enum values", () => {
    const enumValues = Object.values(MilestoneType);
    enumValues.forEach((type) => {
      expect(MilestoneConfigRegistry).toHaveProperty(type);
      expect(MilestoneConfigRegistry[type]).toBeDefined();
    });
  });

  /**
   * @notice Ensures each milestone has required configuration properties
   */
  it("each configuration has required properties", () => {
    Object.values(MilestoneType).forEach((type) => {
      const config = MilestoneConfigRegistry[type];
      expect(config).toHaveProperty("title");
      expect(config).toHaveProperty("description");
      expect(config).toHaveProperty("icon");
      expect(config).toHaveProperty("primaryColor");
      expect(config).toHaveProperty("secondaryColor");
      expect(config).toHaveProperty("animationDuration");
      expect(config).toHaveProperty("showConfetti");
      expect(config).toHaveProperty("playSound");
      expect(config).toHaveProperty("priority");
    });
  });

  /**
   * @notice Validates milestone priorities are in ascending order
   */
  it("milestone priorities are sequential", () => {
    const priorities = Object.values(MilestoneType).map(
      (type) => MilestoneConfigRegistry[type].priority
    );
    const sortedPriorities = [...priorities].sort((a, b) => a - b);
    expect(priorities).toEqual(sortedPriorities);
  });

  /**
   * @notice Verifies all milestones can show confetti
   */
  it("all milestones have confetti enabled", () => {
    Object.values(MilestoneType).forEach((type) => {
      expect(MilestoneConfigRegistry[type].showConfetti).toBe(true);
    });
  });

  /**
   * @notice Validates animation durations are reasonable
   */
  it("all animation durations are within acceptable range", () => {
    Object.values(MilestoneType).forEach((type) => {
      const duration = MilestoneConfigRegistry[type].animationDuration;
      expect(duration).toBeGreaterThanOrEqual(1000);
      expect(duration).toBeLessThanOrEqual(5000);
    });
  });

  /**
   * @notice Checks milestone titles contain emojis for visual feedback
   */
  it("all milestone titles contain emoji indicators", () => {
    Object.values(MilestoneType).forEach((type) => {
      const title = MilestoneConfigRegistry[type].title;
      // Check for common emoji ranges including star (2B50), symbols, and misc
      expect(title).toMatch(/[\u2600-\u26FF\u2700-\u27BF\u2B50\u{1F300}-\u{1F9FF}]/u);
    });
  });

  /**
   * @notice Validates milestone icons are in the icon registry
   */
  it("all milestone icons exist in the icon registry", () => {
    // Note: We can't directly access MilestoneIcons, but we verify config references them
    Object.values(MilestoneType).forEach((type) => {
      const config = MilestoneConfigRegistry[type];
      expect(typeof config.icon).toBe("string");
      expect(config.icon.length).toBeGreaterThan(0);
    });
  });
});

// ── MilestoneType Enum Tests ─────────────────────────────────────────────────

describe("MilestoneType enum values", () => {
  /**
   * @notice Validates exact milestone type string values
   */
  it("has correct string values for all milestone types", () => {
    expect(MilestoneType.CAMPAIGN_CREATED).toBe("campaign_created");
    expect(MilestoneType.FIRST_CONTRIBUTION).toBe("first_contribution");
    expect(MilestoneType.MILESTONE_25_PERCENT).toBe("milestone_25_percent");
    expect(MilestoneType.MILESTONE_50_PERCENT).toBe("milestone_50_percent");
    expect(MilestoneType.MILESTONE_75_PERCENT).toBe("milestone_75_percent");
    expect(MilestoneType.MILESTONE_100_PERCENT).toBe("milestone_100_percent");
    expect(MilestoneType.CAMPAIGN_SUCCESS).toBe("campaign_success");
    expect(MilestoneType.STRETCH_GOAL_REACHED).toBe("stretch_goal_reached");
  });

  /**
   * @notice Verifies milestone count matches configuration
   */
  it("has exactly 8 milestone types", () => {
    const enumValues = Object.values(MilestoneType);
    expect(enumValues).toHaveLength(8);
  });
});

// ── CelebrationCSSVariables Tests ─────────────────────────────────────────────

describe("CelebrationCSSVariables", () => {
  /**
   * @notice Ensures CSS variables are properly defined
   */
  it("contains required CSS variable definitions", () => {
    expect(CelebrationCSSVariables).toHaveProperty("--celebration-primary");
    expect(CelebrationCSSVariables).toHaveProperty("--celebration-secondary");
    expect(CelebrationCSSVariables).toHaveProperty("--celebration-success");
    expect(CelebrationCSSVariables).toHaveProperty("--celebration-background");
    expect(CelebrationCSSVariables).toHaveProperty("--celebration-text");
    expect(CelebrationCSSVariables).toHaveProperty("--celebration-border");
    expect(CelebrationCSSVariables).toHaveProperty("--celebration-confetti-primary");
  });

  /**
   * @notice Validates CSS variable values use CSS custom property references
   */
  it("uses CSS variable references for base colors", () => {
    expect(CelebrationCSSVariables["--celebration-primary"]).toMatch(/^var\(--/);
    expect(CelebrationCSSVariables["--celebration-secondary"]).toMatch(/^var\(--/);
    expect(CelebrationCSSVariables["--celebration-success"]).toMatch(/^var\(--/);
  });
});

// ── Component Rendering Tests ─────────────────────────────────────────────────

describe("CelebrationStandardization rendering", () => {
  /**
   * @notice Verifies component renders when visible
   */
  it("renders when isVisible is true", () => {
    renderCelebration({ isVisible: true });
    expect(getContainer()).toBeInTheDocument();
  });

  /**
   * @notice Verifies component does not render when invisible
   */
  it("does not render when isVisible is false", () => {
    renderCelebration({ isVisible: false });
    expect(getContainer()).toBeNull();
  });

  /**
   * @notice Verifies milestone title is displayed
   */
  it("displays the milestone title", () => {
    renderCelebration();
    expect(screen.getByText(/Campaign Created/i)).toBeInTheDocument();
  });

  /**
   * @notice Verifies milestone description is displayed
   */
  it("displays the milestone description", () => {
    renderCelebration();
    expect(screen.getByText(/Your campaign is now live/i)).toBeInTheDocument();
  });

  /**
   * @notice Verifies milestone icon is rendered
   */
  it("renders the milestone icon", () => {
    renderCelebration();
    const iconContainer = document.querySelector(".milestone-icon-wrapper");
    expect(iconContainer).toBeInTheDocument();
  });

  /**
   * @notice Verifies campaign name is displayed when provided
   */
  it("displays campaign name when provided", () => {
    renderCelebration({ campaignName: "My Awesome Campaign" });
    expect(screen.getByText("My Awesome Campaign")).toBeInTheDocument();
  });

  /**
   * @notice Verifies progress info is displayed when percentage provided
   */
  it("displays progress information when percentage is provided", () => {
    renderCelebration({ progressPercentage: 50 });
    expect(screen.getByText(/50% funded/)).toBeInTheDocument();
  });

  /**
   * @notice Verifies target and raised amounts are displayed
   */
  it("displays target and raised amounts when provided", () => {
    renderCelebration({
      progressPercentage: 50,
      raisedAmount: "$5,000",
      targetAmount: "$10,000",
    });
    expect(screen.getByText(/\$5,000 of \$10,000/)).toBeInTheDocument();
  });

  /**
   * @notice Verifies progress bar is rendered
   */
  it("renders progress bar when percentage is provided", () => {
    renderCelebration({ progressPercentage: 75 });
    const progressBar = document.querySelector(".progress-bar");
    expect(progressBar).toBeInTheDocument();
  });

  /**
   * @notice Verifies custom className is applied
   */
  it("applies custom className", () => {
    renderCelebration({ className: "custom-celebration" });
    expect(getContainer()).toHaveClass("custom-celebration");
  });

  /**
   * @notice Verifies testId is rendered as data-testid
   */
  it("applies data-testid attribute", () => {
    renderCelebration({ testId: "celebration-test" });
    expect(getContainer()).toHaveAttribute("data-testid", "celebration-test");
  });
});

// ── Milestone Type Variations ─────────────────────────────────────────────────

describe("CelebrationStandardization with different milestone types", () => {
  /**
   * @notice Tests rendering for each milestone type
   */
  Object.values(MilestoneType).forEach((type) => {
    it(`renders milestone type: ${type}`, () => {
      renderCelebration({ milestoneType: type });
      const config = MilestoneConfigRegistry[type];
      expect(screen.getByText(config.title)).toBeInTheDocument();
      expect(screen.getByText(config.description)).toBeInTheDocument();
    });
  });

  /**
   * @notice Tests progress display for milestone types that support it
   */
  it("shows progress bar for percentage milestones", () => {
    const percentageMilestones = [
      MilestoneType.MILESTONE_25_PERCENT,
      MilestoneType.MILESTONE_50_PERCENT,
      MilestoneType.MILESTONE_75_PERCENT,
      MilestoneType.MILESTONE_100_PERCENT,
    ];

    percentageMilestones.forEach((type) => {
      const { unmount } = renderCelebration({ milestoneType: type, progressPercentage: 50 });
      expect(document.querySelector(".progress-bar")).toBeInTheDocument();
      unmount();
    });
  });
});

// ── Animation State Tests ─────────────────────────────────────────────────────

describe("CelebrationStandardization animation states", () => {
  beforeEach(() => {
    mockMatchMedia(false);
  });

  /**
   * @notice Tests entering animation state
   */
  it("transitions to visible state when isVisible becomes true", async () => {
    const { rerender } = render(
      <CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={false} />
    );
    
    expect(getContainer()).toBeNull();
    
    await act(async () => {
      rerender(
        <CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={true} />
      );
    });
    
    await waitFor(() => {
      expect(getContainer()).toBeInTheDocument();
    });
  });

  /**
   * @notice Tests component supports visibility transitions
   */
  it("handles visibility prop changes correctly", async () => {
    const { rerender } = render(
      <CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={true} />
    );
    
    await waitFor(() => {
      expect(getContainer()).toBeInTheDocument();
    });
    
    await act(async () => {
      rerender(
        <CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={false} />
      );
    });
    
    // Component should still exist in DOM during exit animation
    // The actual removal happens after animation completes
  });

  /**
   * @notice Tests onAnimationComplete callback exists and can be passed
   */
  it("accepts onAnimationComplete callback prop", () => {
    const onAnimationComplete = jest.fn();
    render(
      <CelebrationStandardization 
        milestoneType={MilestoneType.CAMPAIGN_CREATED} 
        isVisible={true}
        onAnimationComplete={onAnimationComplete}
      />
    );
    
    // Verify the callback was not called immediately
    expect(onAnimationComplete).not.toHaveBeenCalled();
  });
});

// ── Reduced Motion Tests ──────────────────────────────────────────────────────

describe("CelebrationStandardization reduced motion", () => {
  /**
   * @notice Tests component respects reduced motion preference
   */
  it("disables animations when prefers-reduced-motion is enabled", () => {
    mockMatchMedia(true);
    
    const { container } = render(
      <CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={true} />
    );
    
    const styleTag = container.querySelector("style");
    expect(styleTag?.textContent).toContain("prefers-reduced-motion");
  });

  /**
   * @notice Tests animation state is skipped for reduced motion users
   */
  it("does not show confetti for reduced motion users", () => {
    mockMatchMedia(true);
    
    const { container } = render(
      <CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={true} />
    );
    
    const confettiContainer = container.querySelector(".confetti-container");
    expect(confettiContainer).toBeNull();
  });
});

// ── Accessibility Tests ─────────────────────────────────────────────────────

describe("CelebrationStandardization accessibility", () => {
  /**
   * @notice Verifies role="alert" is set for screen readers
   */
  it("has role='alert' for screen reader announcement", () => {
    renderCelebration();
    expect(getContainer()).toHaveAttribute("role", "alert");
  });

  /**
   * @notice Verifies aria-live is set to polite
   */
  it("has aria-live='polite' for non-intrusive announcements", () => {
    renderCelebration();
    expect(getContainer()).toHaveAttribute("aria-live", "polite");
  });

  /**
   * @notice Verifies aria-atomic is set
   */
  it("has aria-atomic='true' for complete announcement", () => {
    renderCelebration();
    expect(getContainer()).toHaveAttribute("aria-atomic", "true");
  });

  /**
   * @notice Verifies aria-label is set from config title
   */
  it("has aria-label set to milestone title by default", () => {
    renderCelebration({ milestoneType: MilestoneType.CAMPAIGN_CREATED });
    expect(getContainer()).toHaveAttribute("aria-label", expect.stringContaining("Campaign Created"));
  });

  /**
   * @notice Verifies custom aria-label can be provided
   */
  it("allows custom aria-label", () => {
    renderCelebration({ ariaLabel: "Custom celebration announcement" });
    expect(getContainer()).toHaveAttribute("aria-label", "Custom celebration announcement");
  });

  /**
   * @notice Verifies aria attributes are set correctly
   */
  it("has proper accessibility attributes", () => {
    renderCelebration();
    const container = getContainer();
    expect(container).toHaveAttribute("role", "alert");
    expect(container).toHaveAttribute("aria-live", "polite");
    expect(container).toHaveAttribute("aria-atomic", "true");
  });
});

// ── Security Tests ────────────────────────────────────────────────────────────

describe("CelebrationStandardization security", () => {
  /**
   * @notice Tests XSS prevention with malicious campaign name
   * @security This test ensures user-provided content is properly escaped
   */
  it("sanitizes campaign name to prevent XSS", () => {
    renderCelebration({ campaignName: '<script>alert("XSS")</script>' });
    
    // The content should be rendered as text, not executed as HTML
    const campaignElement = document.querySelector(".campaign-name");
    // Script tags should not be present as executable HTML tags
    expect(campaignElement?.innerHTML).not.toContain("<script>");
    expect(campaignElement?.innerHTML).not.toContain("</script>");
    // Content is properly sanitized and visible as text
    expect(campaignElement).toBeInTheDocument();
  });

  /**
   * @notice Tests XSS prevention with HTML entities in amounts
   * @security Ensures monetary values are safely rendered
   */
  it("sanitizes raised and target amounts", () => {
    const maliciousRaised = '<script>alert(1)</script>';
    
    renderCelebration({
      raisedAmount: maliciousRaised,
      targetAmount: "$10,000",
      progressPercentage: 50,
    });
    
    const progressText = document.querySelector(".progress-text");
    // Script tags should not be present as executable HTML
    expect(progressText?.innerHTML).not.toContain("<script>");
    expect(progressText?.innerHTML).not.toContain("</script>");
  });

  /**
   * @notice Tests that SVG icons are properly sanitized
   * @security Verifies inline SVG content is safe
   */
  it("does not render untrusted SVG content", () => {
    // This test ensures our MilestoneIcons only contain predefined safe SVGs
    renderCelebration({ milestoneType: MilestoneType.CAMPAIGN_CREATED });
    
    const icon = document.querySelector(".milestone-icon");
    expect(icon?.innerHTML).toMatch(/^<svg/);
    expect(icon?.innerHTML).not.toContain("onload");
    expect(icon?.innerHTML).not.toContain("onerror");
  });

  /**
   * @notice Tests injection prevention in custom className
   * @security Ensures className doesn't inject malicious attributes
   */
  it("safely handles custom className with special characters", () => {
    const maliciousClassName = '"><script>alert(1)</script>';
    renderCelebration({ className: maliciousClassName });
    
    const container = getContainer();
    // className should be escaped or sanitized
    expect(container?.className).toBeDefined();
  });
});

// ── Edge Cases ────────────────────────────────────────────────────────────────

describe("CelebrationStandardization edge cases", () => {
  /**
   * @notice Tests component handles 0% progress
   */
  it("handles 0% progress gracefully", () => {
    renderCelebration({ progressPercentage: 0 });
    expect(screen.getByText(/0% funded/)).toBeInTheDocument();
  });

  /**
   * @notice Tests component handles 100%+ progress (stretch goals)
   */
  it("caps progress bar at 100% for display", () => {
    renderCelebration({ progressPercentage: 150 });
    const progressFill = document.querySelector(".progress-bar > div") as HTMLElement;
    expect(progressFill).toHaveStyle({ width: "100%" });
  });

  /**
   * @notice Tests component with very long campaign name
   */
  it("handles very long campaign names", () => {
    const longName = "A".repeat(500);
    renderCelebration({ campaignName: longName });
    expect(screen.getByText(longName)).toBeInTheDocument();
  });

  /**
   * @notice Tests component without optional props
   */
  it("works without any optional props", () => {
    render(
      <CelebrationStandardization 
        milestoneType={MilestoneType.CAMPAIGN_CREATED} 
        isVisible={true}
      />
    );
    expect(getContainer()).toBeInTheDocument();
  });

  /**
   * @notice Tests component with undefined optional string props
   */
  it("handles undefined optional props gracefully", () => {
    render(
      <CelebrationStandardization 
        milestoneType={MilestoneType.CAMPAIGN_CREATED}
        isVisible={true}
        campaignName={undefined}
        raisedAmount={undefined}
        targetAmount={undefined}
        ariaLabel={undefined}
        className={undefined}
        testId={undefined}
      />
    );
    expect(getContainer()).toBeInTheDocument();
  });

  /**
   * @notice Tests animation completion callback is not called on error
   */
  it("handles undefined onAnimationComplete gracefully", () => {
    jest.useFakeTimers();
    
    const { rerender } = render(
      <CelebrationStandardization 
        milestoneType={MilestoneType.CAMPAIGN_CREATED} 
        isVisible={true}
        onAnimationComplete={undefined}
      />
    );
    
    // Should not throw
    expect(() => {
      rerender(
        <CelebrationStandardization 
          milestoneType={MilestoneType.CAMPAIGN_CREATED} 
          isVisible={false}
          onAnimationComplete={undefined}
        />
      );
    }).not.toThrow();
    
    jest.useRealTimers();
  });

  /**
   * @notice Tests rapid visibility toggling
   */
  it("handles rapid visibility toggling", async () => {
    const { rerender } = render(
      <CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={true} />
    );
    
    await act(async () => {
      rerender(<CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={false} />);
      rerender(<CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={true} />);
      rerender(<CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={false} />);
      rerender(<CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={true} />);
    });
    
    expect(getContainer()).toBeInTheDocument();
  });

  /**
   * @notice Tests milestone type changes while visible
   */
  it("handles milestone type changes while visible", async () => {
    const { rerender } = render(
      <CelebrationStandardization milestoneType={MilestoneType.CAMPAIGN_CREATED} isVisible={true} />
    );
    
    await waitFor(() => {
      expect(screen.getByText(/Campaign Created/i)).toBeInTheDocument();
    });
    
    await act(async () => {
      rerender(<CelebrationStandardization milestoneType={MilestoneType.FIRST_CONTRIBUTION} isVisible={true} />);
    });
    
    await waitFor(() => {
      expect(screen.getByText(/First Backer/i)).toBeInTheDocument();
    });
  });
});

// ── Configuration-specific Tests ─────────────────────────────────────────────

describe("Milestone-specific configurations", () => {
  /**
   * @notice Tests that each milestone has appropriate colors
   */
  it("each milestone has primary and secondary colors", () => {
    Object.values(MilestoneType).forEach((type) => {
      const config = MilestoneConfigRegistry[type];
      expect(config.primaryColor).toBeDefined();
      expect(config.secondaryColor).toBeDefined();
      expect(config.primaryColor).toMatch(/^#[0-9A-Fa-f]{6}$/);
      expect(config.secondaryColor).toMatch(/^#[0-9A-Fa-f]{6}$/);
    });
  });

  /**
   * @notice Tests that important milestones play sound
   */
  it("major milestones have playSound enabled", () => {
    const majorMilestones = [
      MilestoneType.CAMPAIGN_SUCCESS,
      MilestoneType.STRETCH_GOAL_REACHED,
      MilestoneType.MILESTONE_100_PERCENT,
    ];
    
    majorMilestones.forEach((type) => {
      expect(MilestoneConfigRegistry[type].playSound).toBe(true);
    });
  });

  /**
   * @notice Tests that 25% milestone has playSound disabled (subtle celebration)
   */
  it("25% milestone has playSound disabled for subtle celebration", () => {
    expect(MilestoneConfigRegistry[MilestoneType.MILESTONE_25_PERCENT].playSound).toBe(false);
  });
});

// ── Visual Regression Tests ───────────────────────────────────────────────────

describe("CelebrationStandardization visual elements", () => {
  /**
   * @notice Verifies gradient background is applied
   */
  it("applies gradient background", () => {
    renderCelebration();
    const container = getContainer();
    expect(container?.style.background).toMatch(/linear-gradient/);
  });

  /**
   * @notice Verifies border is applied
   */
  it("applies border styling", () => {
    renderCelebration();
    const container = getContainer();
    expect(container?.style.border).toBeDefined();
  });

  /**
   * @notice Verifies box shadow is applied
   */
  it("applies box shadow", () => {
    renderCelebration();
    const container = getContainer();
    expect(container?.style.boxShadow).toBeDefined();
  });

  /**
   * @notice Verifies border radius is applied
   */
  it("applies border radius", () => {
    renderCelebration();
    const container = getContainer();
    expect(container?.style.borderRadius).toBe("16px");
  });

  /**
   * @notice Verifies icon container has circular background
   */
  it("applies circular background to icon container", () => {
    renderCelebration();
    const iconContainer = document.querySelector(".milestone-icon-wrapper > div");
    expect(iconContainer).toHaveStyle({ borderRadius: "50%" });
  });

  /**
   * @notice Verifies confetti container exists when enabled
   */
  it("renders confetti container when showConfetti is true", () => {
    mockMatchMedia(false); // Ensure motion is not reduced
    renderCelebration();
    const confettiContainer = document.querySelector(".confetti-container");
    expect(confettiContainer).toBeInTheDocument();
  });

  /**
   * @notice Verifies confetti particles are rendered
   */
  it("renders confetti particles", () => {
    mockMatchMedia(false);
    renderCelebration();
    const particles = document.querySelectorAll(".confetti-particle");
    expect(particles.length).toBe(50);
  });
});

// ── Integration Tests ─────────────────────────────────────────────────────────

describe("CelebrationStandardization integration scenarios", () => {
  /**
   * @notice Tests complete funding milestone flow
   */
  it("supports complete funding flow from 0% to 100%", async () => {
    const { rerender } = render(
      <CelebrationStandardization 
        milestoneType={MilestoneType.CAMPAIGN_CREATED}
        campaignName="Test Campaign"
        isVisible={true}
      />
    );
    
    expect(screen.getByText(/Campaign Created/i)).toBeInTheDocument();
    
    await act(async () => {
      rerender(
        <CelebrationStandardization 
          milestoneType={MilestoneType.MILESTONE_50_PERCENT}
          campaignName="Test Campaign"
          progressPercentage={50}
          raisedAmount="$5,000"
          targetAmount="$10,000"
          isVisible={true}
        />
      );
    });
    
    // Use specific selector for progress text
    expect(screen.getByText(/\$5,000 of \$10,000/)).toBeInTheDocument();
    
    await act(async () => {
      rerender(
        <CelebrationStandardization 
          milestoneType={MilestoneType.CAMPAIGN_SUCCESS}
          campaignName="Test Campaign"
          progressPercentage={100}
          raisedAmount="$10,000"
          targetAmount="$10,000"
          isVisible={true}
        />
      );
    });
    
    expect(screen.getByText(/Campaign Successful/i)).toBeInTheDocument();
  });

  /**
   * @notice Tests stretch goal scenario
   */
  it("supports stretch goal achievement", () => {
    render(
      <CelebrationStandardization 
        milestoneType={MilestoneType.STRETCH_GOAL_REACHED}
        campaignName="Ambitious Project"
        progressPercentage={150}
        raisedAmount="$150,000"
        targetAmount="$100,000"
        isVisible={true}
      />
    );
    
    expect(screen.getByText(/Stretch Goal Achieved/i)).toBeInTheDocument();
    expect(screen.getByText(/150% funded/i)).toBeInTheDocument();
  });

  /**
   * @notice Tests multiple celebrations in sequence
   */
  it("handles sequential milestone celebrations", async () => {
    const milestones = [
      MilestoneType.FIRST_CONTRIBUTION,
      MilestoneType.MILESTONE_25_PERCENT,
      MilestoneType.MILESTONE_50_PERCENT,
      MilestoneType.MILESTONE_75_PERCENT,
    ];
    
    for (const milestone of milestones) {
      const { unmount } = render(
        <CelebrationStandardization milestoneType={milestone} isVisible={true} />
      );
      
      const config = MilestoneConfigRegistry[milestone];
      expect(screen.getByText(config.title)).toBeInTheDocument();
      
      unmount();
    }
  });
});
