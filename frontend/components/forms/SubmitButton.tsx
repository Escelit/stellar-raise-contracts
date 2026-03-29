/**
 * SubmitButton
 *
 * A submit button for crowdfunding actions (contribute, withdraw, refund).
 *
 * States
 * ------
 * @state idle      - Default, ready to submit.
 * @state loading   - Transaction in-flight; button is disabled and shows a spinner.
 * @state success   - Transaction confirmed; button shows a success label.
 * @state error     - Transaction failed; button shows an error label.
 * @state disabled  - Externally disabled (deadline passed, goal met, etc.).
 *
 * Security assumptions
 * --------------------
 * - The parent is responsible for validating inputs before calling `onClick`.
 * - The component never submits data itself; it only surfaces state to the user.
 * - `loading` state prevents double-submission by disabling the button.
 */

import React from "react";

export type SubmitButtonState = "idle" | "loading" | "success" | "error";

export interface SubmitButtonProps {
  /** Current button state */
  state?: SubmitButtonState;
  /** Label shown in idle state */
  label: string;
  /** Label shown while loading */
  loadingLabel?: string;
  /** Label shown on success */
  successLabel?: string;
  /** Label shown on error */
  errorLabel?: string;
  /** Externally disable the button regardless of state */
  disabled?: boolean;
  /** Click handler – only fired when state is idle */
  onClick?: () => void;
  /** Render as full-width */
  fullWidth?: boolean;
  /** HTML button type (default: "submit") */
  type?: "submit" | "button" | "reset";
  /** Additional CSS class */
  className?: string;
}

const Spinner = () => (
  <svg
    aria-hidden="true"
    width="16"
    height="16"
    viewBox="0 0 16 16"
    fill="none"
    style={{ animation: "spin 0.8s linear infinite" }}
  >
    <circle
      cx="8"
      cy="8"
      r="6"
      stroke="currentColor"
      strokeWidth="2"
      strokeDasharray="28"
      strokeDashoffset="10"
      strokeLinecap="round"
    />
    <style>{`@keyframes spin { to { transform: rotate(360deg); } }`}</style>
  </svg>
);

const STATE_LABELS: Record<SubmitButtonState, string> = {
  idle: "",
  loading: "Processing…",
  success: "Success!",
  error: "Failed – try again",
};

const STATE_CLASSES: Record<SubmitButtonState, string> = {
  idle: "btn--primary",
  loading: "btn--primary",
  success: "btn--primary submit-btn--success",
  error: "btn--danger",
};

/**
 * SubmitButton component.
 *
 * @example
 * <SubmitButton
 *   label="Contribute"
 *   state={txState}
 *   onClick={handleContribute}
 * />
 */
const SubmitButton: React.FC<SubmitButtonProps> = ({
  state = "idle",
  label,
  loadingLabel,
  successLabel,
  errorLabel,
  disabled = false,
  onClick,
  fullWidth = false,
  type = "submit",
  className = "",
}) => {
  const isDisabled = disabled || state === "loading" || state === "success";

  const resolvedLabel = (() => {
    switch (state) {
      case "loading":
        return loadingLabel ?? STATE_LABELS.loading;
      case "success":
        return successLabel ?? STATE_LABELS.success;
      case "error":
        return errorLabel ?? STATE_LABELS.error;
      default:
        return label;
    }
  })();

  const stateClass = STATE_CLASSES[state];
  const widthClass = fullWidth ? " btn--full" : "";
  const combinedClass = `btn ${stateClass}${widthClass}${className ? ` ${className}` : ""}`;

  return (
    <button
      type={type}
      className={combinedClass}
      disabled={isDisabled}
      aria-disabled={isDisabled}
      aria-busy={state === "loading"}
      aria-live="polite"
      onClick={state === "idle" || state === "error" ? onClick : undefined}
      data-state={state}
    >
      {state === "loading" && <Spinner />}
      <span>{resolvedLabel}</span>
    </button>
  );
};

export default SubmitButton;
