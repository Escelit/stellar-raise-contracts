/**
 * @title React Submit Button — Comprehensive Test Suite
 * @notice Covers label safety, state transitions, interaction blocking,
 *         accessibility attributes, and component rendering.
 * @dev Targets ≥ 95% coverage of react_submit_button.tsx.
 */
import React from "react";
import { render, screen, fireEvent, act } from "@testing-library/react";
import ReactSubmitButton, {
  ALLOWED_TRANSITIONS,
  isSubmitButtonBusy,
  isSubmitButtonInteractionBlocked,
  isValidSubmitButtonStateTransition,
  normalizeSubmitButtonLabel,
  resolveSubmitButtonLabel,
  resolveSafeSubmitButtonState,
  type ReactSubmitButtonProps,
  type SubmitButtonLabels,
  type SubmitButtonState,
} from "./react_submit_button";

// ── Helpers ───────────────────────────────────────────────────────────────────

function renderBtn(props: Partial<ReactSubmitButtonProps> = {}) {
  const { container } = render(<ReactSubmitButton state="idle" {...props} />);
  return container.querySelector("button") as HTMLButtonElement;
}

const ALL_STATES: SubmitButtonState[] = ["idle", "submitting", "success", "error", "disabled"];

// ── normalizeSubmitButtonLabel ────────────────────────────────────────────────

describe("normalizeSubmitButtonLabel", () => {
  it("returns fallback for non-string values", () => {
    expect(normalizeSubmitButtonLabel(undefined, "Submit")).toBe("Submit");
    expect(normalizeSubmitButtonLabel(null, "Submit")).toBe("Submit");
    expect(normalizeSubmitButtonLabel(404, "Submit")).toBe("Submit");
    expect(normalizeSubmitButtonLabel({}, "Submit")).toBe("Submit");
    expect(normalizeSubmitButtonLabel(true, "Submit")).toBe("Submit");
  });

  it("returns fallback for empty or whitespace-only strings", () => {
    expect(normalizeSubmitButtonLabel("", "Submit")).toBe("Submit");
    expect(normalizeSubmitButtonLabel("   ", "Submit")).toBe("Submit");
    expect(normalizeSubmitButtonLabel("\n\t", "Submit")).toBe("Submit");
  });

  it("strips control characters and normalizes whitespace", () => {
    expect(normalizeSubmitButtonLabel("Pay\u0000Now", "Submit")).toBe("Pay Now");
    expect(normalizeSubmitButtonLabel("Pay\u0008\u001FNow", "Submit")).toBe("Pay Now");
    expect(normalizeSubmitButtonLabel("Pay   \n   Now", "Submit")).toBe("Pay Now");
  });

  it("returns the label unchanged when within the 80-char limit", () => {
    const label = "A".repeat(80);
    expect(normalizeSubmitButtonLabel(label, "Submit")).toBe(label);
  });

  it("truncates labels exceeding 80 characters with ellipsis", () => {
    const long = "A".repeat(200);
    const result = normalizeSubmitButtonLabel(long, "Submit");
    expect(result).toHaveLength(80);
    expect(result.endsWith("...")).toBe(true);
  });

  it("preserves hostile markup-like text as a plain string", () => {
    const xss = "<img src=x onerror=alert(1) />";
    // Security: React renders this as text, not HTML.
    expect(normalizeSubmitButtonLabel(xss, "Submit")).toBe(xss);
  });
});

// ── resolveSubmitButtonLabel ──────────────────────────────────────────────────

describe("resolveSubmitButtonLabel", () => {
  it("returns correct defaults for every state", () => {
    const expected = ["Submit", "Submitting...", "Submitted", "Try Again", "Submit Disabled"];
    expect(ALL_STATES.map((s) => resolveSubmitButtonLabel(s))).toEqual(expected);
  });

  it("uses valid custom labels", () => {
    const labels: SubmitButtonLabels = {
      idle: "Fund Campaign",
      submitting: "Funding...",
      success: "Funded!",
      error: "Retry",
      disabled: "Locked",
    };
    ALL_STATES.forEach((s) => {
      expect(resolveSubmitButtonLabel(s, labels)).toBe(labels[s]);
    });
  });

  it("falls back to defaults for empty or whitespace custom labels", () => {
    const labels: SubmitButtonLabels = { idle: "", submitting: "   " };
    expect(resolveSubmitButtonLabel("idle", labels)).toBe("Submit");
    expect(resolveSubmitButtonLabel("submitting", labels)).toBe("Submitting...");
  });

  it("trims and truncates oversized custom labels", () => {
    const labels: SubmitButtonLabels = { success: `   ${"A".repeat(90)}   ` };
    const result = resolveSubmitButtonLabel("success", labels);
    expect(result).toHaveLength(80);
    expect(result.endsWith("...")).toBe(true);
  });
});

// ── isValidSubmitButtonStateTransition ───────────────────────────────────────

describe("isValidSubmitButtonStateTransition", () => {
  it("allows all transitions defined in ALLOWED_TRANSITIONS", () => {
    for (const [from, targets] of Object.entries(ALLOWED_TRANSITIONS) as [SubmitButtonState, SubmitButtonState[]][]) {
      for (const to of targets) {
        expect(isValidSubmitButtonStateTransition(from, to)).toBe(true);
      }
    }
  });

  it("allows same-state transitions (idempotent)", () => {
    ALL_STATES.forEach((s) => {
      expect(isValidSubmitButtonStateTransition(s, s)).toBe(true);
    });
  });

  it("blocks transitions not in the allowed map", () => {
    expect(isValidSubmitButtonStateTransition("idle", "success")).toBe(false);
    expect(isValidSubmitButtonStateTransition("idle", "error")).toBe(false);
    expect(isValidSubmitButtonStateTransition("success", "error")).toBe(false);
    expect(isValidSubmitButtonStateTransition("success", "submitting")).toBe(false);
    expect(isValidSubmitButtonStateTransition("disabled", "submitting")).toBe(false);
    expect(isValidSubmitButtonStateTransition("disabled", "success")).toBe(false);
  });
});

// ── resolveSafeSubmitButtonState ─────────────────────────────────────────────

describe("resolveSafeSubmitButtonState", () => {
  it("returns requested state when transition is valid (strict)", () => {
    expect(resolveSafeSubmitButtonState("submitting", "idle", true)).toBe("submitting");
    expect(resolveSafeSubmitButtonState("success", "submitting", true)).toBe("success");
    expect(resolveSafeSubmitButtonState("error", "submitting", true)).toBe("error");
  });

  it("falls back to previousState for invalid transitions in strict mode", () => {
    expect(resolveSafeSubmitButtonState("success", "idle", true)).toBe("idle");
    expect(resolveSafeSubmitButtonState("error", "success", true)).toBe("success");
    expect(resolveSafeSubmitButtonState("submitting", "disabled", true)).toBe("disabled");
  });

  it("accepts any state when strict mode is disabled", () => {
    expect(resolveSafeSubmitButtonState("success", "idle", false)).toBe("success");
    expect(resolveSafeSubmitButtonState("error", "success", false)).toBe("error");
  });

  it("accepts requested state when previousState is absent", () => {
    expect(resolveSafeSubmitButtonState("error", undefined, true)).toBe("error");
    expect(resolveSafeSubmitButtonState("success", undefined, true)).toBe("success");
  });

  it("defaults strictTransitions to true", () => {
    // idle → success is invalid; should fall back to idle
    expect(resolveSafeSubmitButtonState("success", "idle")).toBe("idle");
  });
});

// ── isSubmitButtonInteractionBlocked ─────────────────────────────────────────

describe("isSubmitButtonInteractionBlocked", () => {
  it("blocks interaction for submitting and disabled states", () => {
    expect(isSubmitButtonInteractionBlocked("submitting")).toBe(true);
    expect(isSubmitButtonInteractionBlocked("disabled")).toBe(true);
  });

  it("blocks when explicit disabled flag is set", () => {
    expect(isSubmitButtonInteractionBlocked("idle", true)).toBe(true);
    expect(isSubmitButtonInteractionBlocked("error", true)).toBe(true);
  });

  it("blocks when locally submitting", () => {
    expect(isSubmitButtonInteractionBlocked("idle", false, true)).toBe(true);
  });

  it("allows interaction for active states with no flags", () => {
    expect(isSubmitButtonInteractionBlocked("idle", false, false)).toBe(false);
    expect(isSubmitButtonInteractionBlocked("error", false, false)).toBe(false);
    expect(isSubmitButtonInteractionBlocked("success", false, false)).toBe(false);
  });
});

// ── isSubmitButtonBusy ────────────────────────────────────────────────────────

describe("isSubmitButtonBusy", () => {
  it("is true only while submitting", () => {
    expect(isSubmitButtonBusy("submitting")).toBe(true);
  });

  it("is true when locally submitting regardless of state", () => {
    expect(isSubmitButtonBusy("idle", true)).toBe(true);
    expect(isSubmitButtonBusy("error", true)).toBe(true);
  });

  it("is false for all non-submitting states with no local flag", () => {
    const nonSubmitting: SubmitButtonState[] = ["idle", "success", "error", "disabled"];
    nonSubmitting.forEach((s) => {
      expect(isSubmitButtonBusy(s, false)).toBe(false);
    });
  });
});

// ── ReactSubmitButton — rendering ────────────────────────────────────────────

describe("ReactSubmitButton rendering", () => {
  it("renders a button element", () => {
    const btn = renderBtn();
    expect(btn.tagName).toBe("BUTTON");
  });

  it("displays the resolved label as text content", () => {
    renderBtn({ state: "idle" });
    expect(screen.getByText("Submit")).toBeTruthy();
  });

  it("displays custom label override", () => {
    renderBtn({ state: "idle", labels: { idle: "Fund Campaign" } });
    expect(screen.getByText("Fund Campaign")).toBeTruthy();
  });

  it("sets data-state to the resolved state", () => {
    ALL_STATES.forEach((s) => {
      const btn = renderBtn({ state: s });
      expect(btn.getAttribute("data-state")).toBe(s);
    });
  });

  it("defaults type to 'button'", () => {
    expect(renderBtn().type).toBe("button");
  });

  it("respects explicit type prop", () => {
    expect(renderBtn({ type: "submit" }).type).toBe("submit");
    expect(renderBtn({ type: "reset" }).type).toBe("reset");
  });

  it("applies custom className", () => {
    const btn = renderBtn({ className: "my-btn" });
    expect(btn.className).toContain("my-btn");
  });

  it("sets the id attribute", () => {
    const btn = renderBtn({ id: "contribute-btn" });
    expect(btn.id).toBe("contribute-btn");
  });
});

// ── ReactSubmitButton — disabled / blocked states ────────────────────────────

describe("ReactSubmitButton disabled behavior", () => {
  it("is disabled in submitting state", () => {
    expect(renderBtn({ state: "submitting" }).disabled).toBe(true);
  });

  it("is disabled in disabled state", () => {
    expect(renderBtn({ state: "disabled" }).disabled).toBe(true);
  });

  it("is disabled when disabled prop is true", () => {
    expect(renderBtn({ disabled: true }).disabled).toBe(true);
  });

  it("is NOT disabled in idle, success, or error states by default", () => {
    expect(renderBtn({ state: "idle" }).disabled).toBe(false);
    expect(renderBtn({ state: "success" }).disabled).toBe(false);
    expect(renderBtn({ state: "error" }).disabled).toBe(false);
  });
});

// ── ReactSubmitButton — accessibility ────────────────────────────────────────

describe("ReactSubmitButton accessibility", () => {
  it("has aria-live='polite'", () => {
    expect(renderBtn().getAttribute("aria-live")).toBe("polite");
  });

  it("sets aria-busy='true' while submitting", () => {
    expect(renderBtn({ state: "submitting" }).getAttribute("aria-busy")).toBe("true");
  });

  it("sets aria-busy='false' for non-submitting states", () => {
    const nonBusy: SubmitButtonState[] = ["idle", "success", "error", "disabled"];
    nonBusy.forEach((s) => {
      expect(renderBtn({ state: s }).getAttribute("aria-busy")).toBe("false");
    });
  });

  it("sets aria-label to the resolved label", () => {
    expect(renderBtn({ state: "idle" }).getAttribute("aria-label")).toBe("Submit");
    expect(renderBtn({ state: "error" }).getAttribute("aria-label")).toBe("Try Again");
  });
});

// ── ReactSubmitButton — click handling ───────────────────────────────────────

describe("ReactSubmitButton click handling", () => {
  it("fires onClick in idle state", async () => {
    const onClick = jest.fn().mockResolvedValue(undefined);
    const { container } = render(<ReactSubmitButton state="idle" onClick={onClick} />);
    const btn = container.querySelector("button") as HTMLButtonElement;
    await act(async () => { fireEvent.click(btn); });
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("fires onClick in error state (retry)", async () => {
    const onClick = jest.fn().mockResolvedValue(undefined);
    const { container } = render(<ReactSubmitButton state="error" onClick={onClick} />);
    const btn = container.querySelector("button") as HTMLButtonElement;
    await act(async () => { fireEvent.click(btn); });
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("does NOT fire onClick in submitting state", () => {
    const onClick = jest.fn();
    const { container } = render(<ReactSubmitButton state="submitting" onClick={onClick} />);
    const btn = container.querySelector("button") as HTMLButtonElement;
    fireEvent.click(btn);
    expect(onClick).not.toHaveBeenCalled();
  });

  it("does NOT fire onClick in disabled state", () => {
    const onClick = jest.fn();
    const { container } = render(<ReactSubmitButton state="disabled" onClick={onClick} />);
    const btn = container.querySelector("button") as HTMLButtonElement;
    fireEvent.click(btn);
    expect(onClick).not.toHaveBeenCalled();
  });

  it("does NOT fire onClick when disabled prop is true", () => {
    const onClick = jest.fn();
    const { container } = render(<ReactSubmitButton state="idle" disabled={true} onClick={onClick} />);
    const btn = container.querySelector("button") as HTMLButtonElement;
    fireEvent.click(btn);
    expect(onClick).not.toHaveBeenCalled();
  });

  it("handles async onClick without throwing", async () => {
    const onClick = jest.fn().mockResolvedValue(undefined);
    const { container } = render(<ReactSubmitButton state="idle" onClick={onClick} />);
    const btn = container.querySelector("button") as HTMLButtonElement;
    await act(async () => {
      fireEvent.click(btn);
    });
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("does not propagate errors from a rejected async onClick", async () => {
    const onClick = jest.fn().mockRejectedValue(new Error("tx failed"));
    const { container } = render(<ReactSubmitButton state="idle" onClick={onClick} />);
    const btn = container.querySelector("button") as HTMLButtonElement;
    await act(async () => {
      fireEvent.click(btn);
    });
    // If we reach here without throwing, the component swallowed the rejection correctly.
    expect(onClick).toHaveBeenCalledTimes(1);
  });
});

// ── ReactSubmitButton — strict transition enforcement ────────────────────────

describe("ReactSubmitButton strict transitions", () => {
  it("renders previousState when transition is invalid in strict mode", () => {
    // idle → success is not allowed; should render idle
    const btn = renderBtn({ state: "success", previousState: "idle", strictTransitions: true });
    expect(btn.getAttribute("data-state")).toBe("idle");
  });

  it("renders requested state when transition is valid in strict mode", () => {
    const btn = renderBtn({ state: "submitting", previousState: "idle", strictTransitions: true });
    expect(btn.getAttribute("data-state")).toBe("submitting");
  });

  it("renders requested state when strict mode is disabled", () => {
    const btn = renderBtn({ state: "success", previousState: "idle", strictTransitions: false });
    expect(btn.getAttribute("data-state")).toBe("success");
  });
});
