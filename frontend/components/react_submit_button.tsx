import React from "react";

/**
 * @title React Submit Button Component States
 * @notice Centralized submit-button state model for consistent UX and safe defaults.
 * @dev Uses strict union types and deterministic state mapping to reduce misuse.
 */
export type SubmitButtonState = "idle" | "submitting" | "success" | "error" | "disabled";

/**
 * @title Submit Button Labels
 * @notice Optional custom labels for each user-visible state.
 */
export interface SubmitButtonLabels {
  idle?: string;
  submitting?: string;
  success?: string;
  error?: string;
  disabled?: string;
}

/**
 * @title Submit Button Props
 * @notice Defines behavior, labeling, and accessibility requirements.
 */
export interface ReactSubmitButtonProps {
  state: SubmitButtonState;
  labels?: SubmitButtonLabels;
  onClick?: () => void | Promise<void>;
  className?: string;
  id?: string;
  type?: "button" | "submit";
  disabled?: boolean;
}

const DEFAULT_LABELS: Required<SubmitButtonLabels> = {
  idle: "Submit",
  submitting: "Submitting...",
  success: "Submitted",
  error: "Try Again",
  disabled: "Submit Disabled",
};

/**
 * @title Safe Label Resolver
 * @notice Provides a bounded, non-empty label to avoid empty UI text states.
 * @dev React escapes text content by default; this function only normalizes.
 */
export function resolveSubmitButtonLabel(
  state: SubmitButtonState,
  labels?: SubmitButtonLabels,
): string {
  const candidate = labels?.[state];

  if (typeof candidate !== "string") {
    return DEFAULT_LABELS[state];
  }

  const normalized = candidate.trim();
  if (!normalized) {
    return DEFAULT_LABELS[state];
  }

  return normalized.length > 80 ? `${normalized.slice(0, 77)}...` : normalized;
}

/**
 * @title Disabled Guard
 * @notice Computes final disabled state from explicit disabled flag and workflow state.
 */
export function isSubmitButtonDisabled(state: SubmitButtonState, disabled?: boolean): boolean {
  return Boolean(disabled) || state === "disabled" || state === "submitting";
}

/**
 * @title Aria Busy Guard
 * @notice Announces loading semantics only during active submission.
 */
export function isSubmitButtonBusy(state: SubmitButtonState): boolean {
  return state === "submitting";
}

const BASE_STYLE: React.CSSProperties = {
  minHeight: "44px",
  minWidth: "120px",
  borderRadius: "8px",
  border: "1px solid #4f46e5",
  padding: "0.5rem 1rem",
  color: "#ffffff",
  fontWeight: 600,
  cursor: "pointer",
  transition: "opacity 0.2s ease",
  backgroundColor: "#4f46e5",
};

const STATE_STYLE_MAP: Record<SubmitButtonState, React.CSSProperties> = {
  idle: { backgroundColor: "#4f46e5" },
  submitting: { backgroundColor: "#6366f1" },
  success: { backgroundColor: "#16a34a", borderColor: "#15803d" },
  error: { backgroundColor: "#dc2626", borderColor: "#b91c1c" },
  disabled: { backgroundColor: "#9ca3af", borderColor: "#9ca3af", cursor: "not-allowed", opacity: 0.9 },
};

/**
 * @title React Submit Button
 * @notice Reusable submit button with typed state machine for scalable workflows.
 * @dev Avoids exposing raw HTML injection paths and enforces accessible semantics.
 */
const ReactSubmitButton = ({
  state,
  labels,
  onClick,
  className,
  id,
  type = "button",
  disabled,
}: ReactSubmitButtonProps) => {
  const label = resolveSubmitButtonLabel(state, labels);
  const computedDisabled = isSubmitButtonDisabled(state, disabled);
  const ariaBusy = isSubmitButtonBusy(state);

  return (
    <button
      id={id}
      type={type}
      className={className}
      disabled={computedDisabled}
      aria-busy={ariaBusy}
      aria-live="polite"
      aria-label={label}
      onClick={computedDisabled ? undefined : onClick}
      style={{
        ...BASE_STYLE,
        ...STATE_STYLE_MAP[state],
        ...(computedDisabled ? { cursor: "not-allowed", opacity: 0.7 } : {}),
      }}
    >
      {label}
    </button>
  );
};

export default ReactSubmitButton;
