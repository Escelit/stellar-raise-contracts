/**
 * @title ReactSubmitButton — Shared Types and State Configuration
 * @notice Pure TypeScript exports with no React dependency.
 *         Imported by both the component and the test suite.
 *
 * @security All colour values are hardcoded constants — no dynamic CSS injection
 *           from user input is possible.
 */

// Re-export the canonical state type so consumers can import from one place.
export type { SubmitButtonState, SubmitButtonLabels, ReactSubmitButtonProps } from "./react_submit_button";
export { ALLOWED_TRANSITIONS } from "./react_submit_button";

// ── State configuration ───────────────────────────────────────────────────────

import type { SubmitButtonState } from "./react_submit_button";

/**
 * @notice Visual configuration for each button state.
 * @dev Centralising colours here makes security review straightforward —
 *      no dynamic style injection from user input.
 */
export const STATE_CONFIG: Record<
  SubmitButtonState,
  { label: string; backgroundColor: string; cursor: string; ariaLabel: string }
> = {
  idle: {
    label: "Submit",
    backgroundColor: "#4f46e5",
    cursor: "pointer",
    ariaLabel: "Submit",
  },
  submitting: {
    label: "Submitting\u2026",
    backgroundColor: "#6366f1",
    cursor: "not-allowed",
    ariaLabel: "Submitting, please wait",
  },
  success: {
    label: "Submitted \u2713",
    backgroundColor: "#16a34a",
    cursor: "default",
    ariaLabel: "Action completed successfully",
  },
  error: {
    label: "Try Again",
    backgroundColor: "#dc2626",
    cursor: "pointer",
    ariaLabel: "Action failed, click to retry",
  },
  disabled: {
    label: "Submit Disabled",
    backgroundColor: "#9ca3af",
    cursor: "not-allowed",
    ariaLabel: "Button disabled",
  },
};
