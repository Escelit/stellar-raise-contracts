/**
 * @title React Submit Button Tests
 * @notice Validates state transitions, accessibility flags, and security-aware label handling.
 */
import {
  isSubmitButtonBusy,
  isSubmitButtonDisabled,
  resolveSubmitButtonLabel,
  type SubmitButtonLabels,
  type SubmitButtonState,
} from "./react_submit_button";

describe("resolveSubmitButtonLabel", () => {
  it("returns default labels for every known state", () => {
    const states: SubmitButtonState[] = ["idle", "submitting", "success", "error", "disabled"];
    const output = states.map((state) => resolveSubmitButtonLabel(state));

    expect(output).toEqual(["Submit", "Submitting...", "Submitted", "Try Again", "Submit Disabled"]);
  });

  it("uses custom labels when valid", () => {
    const labels: SubmitButtonLabels = {
      idle: "Send Now",
      submitting: "Please wait",
      success: "Done",
      error: "Retry",
      disabled: "Locked",
    };

    expect(resolveSubmitButtonLabel("idle", labels)).toBe("Send Now");
    expect(resolveSubmitButtonLabel("submitting", labels)).toBe("Please wait");
    expect(resolveSubmitButtonLabel("success", labels)).toBe("Done");
    expect(resolveSubmitButtonLabel("error", labels)).toBe("Retry");
    expect(resolveSubmitButtonLabel("disabled", labels)).toBe("Locked");
  });

  it("falls back to defaults for empty or whitespace labels", () => {
    const labels: SubmitButtonLabels = {
      idle: "",
      submitting: "   ",
    };

    expect(resolveSubmitButtonLabel("idle", labels)).toBe("Submit");
    expect(resolveSubmitButtonLabel("submitting", labels)).toBe("Submitting...");
  });

  it("trims custom labels and limits overly long labels", () => {
    const veryLongLabel = `${"A".repeat(90)} trailing text`;
    const labels: SubmitButtonLabels = {
      success: `   ${veryLongLabel}   `,
    };

    const resolved = resolveSubmitButtonLabel("success", labels);
    expect(resolved.length).toBe(80);
    expect(resolved.endsWith("...")).toBe(true);
  });

  it("keeps potentially hostile text as plain label content", () => {
    const hostile = "<img src=x onerror=alert(1) />";
    const labels: SubmitButtonLabels = { error: hostile };

    // Security note: React renders strings as text, not executable HTML.
    expect(resolveSubmitButtonLabel("error", labels)).toBe(hostile);
  });
});

describe("isSubmitButtonDisabled", () => {
  it("returns true for submitting and disabled states", () => {
    expect(isSubmitButtonDisabled("submitting")).toBe(true);
    expect(isSubmitButtonDisabled("disabled")).toBe(true);
  });

  it("returns false for active states when disabled flag is not set", () => {
    expect(isSubmitButtonDisabled("idle")).toBe(false);
    expect(isSubmitButtonDisabled("success")).toBe(false);
    expect(isSubmitButtonDisabled("error")).toBe(false);
  });

  it("respects explicit disabled override", () => {
    expect(isSubmitButtonDisabled("idle", true)).toBe(true);
    expect(isSubmitButtonDisabled("success", true)).toBe(true);
  });
});

describe("isSubmitButtonBusy", () => {
  it("is true only while submitting", () => {
    expect(isSubmitButtonBusy("submitting")).toBe(true);
    expect(isSubmitButtonBusy("idle")).toBe(false);
    expect(isSubmitButtonBusy("success")).toBe(false);
    expect(isSubmitButtonBusy("error")).toBe(false);
    expect(isSubmitButtonBusy("disabled")).toBe(false);
  });
});
