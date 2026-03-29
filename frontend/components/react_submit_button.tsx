import React, { useState } from "react";

/**
 * @title React Submit Button States
 * @notice Typed submit button with a strict state machine, safe label handling,
 *         double-submit prevention, and ARIA accessibility semantics.
 * @dev Labels are rendered as React text nodes — no dangerouslySetInnerHTML path.
 *      Interaction is blocked while submitting to prevent duplicate blockchain transactions.
 */

// ── Types ─────────────────────────────────────────────────────────────────────

/**
 * @notice All supported visual/interaction states.
 * @dev Allowed transitions:
 *   idle        → submitting | disabled
 *   submitting  → success | error | disabled
 *   success     → idle | disabled
 *   error       → idle | submitting | disabled
 *   disabled    → idle
import React from "react";

/**
 * @title React Submit Button Component States
 * @notice Centralized submit-button state model for consistent UX and safe defaults.
 * @dev Uses strict union types and deterministic state mapping to reduce misuse.
import React, { useMemo, useState } from "react";
import React, { useState } from "react";

/**
 * @title React Submit Button States
 * @notice Typed submit button with a strict state machine, safe label handling,
 *         double-submit prevention, and ARIA accessibility semantics.
 * @dev Labels are rendered as React text nodes — no dangerouslySetInnerHTML path.
 *      Interaction is blocked while submitting to prevent duplicate blockchain transactions.
 */

// ── Types ─────────────────────────────────────────────────────────────────────

/**
import React, { useState } from "react";

/**
 * @title React Submit Button States
 * @notice Typed submit button with a strict state machine, safe label handling,
 *         double-submit prevention, and ARIA accessibility semantics.
 * @dev Labels are rendered as React text nodes — no dangerouslySetInnerHTML path.
 *      Interaction is blocked while submitting to prevent duplicate blockchain transactions.
 */

// ── Types ─────────────────────────────────────────────────────────────────────

/**
 * @notice All supported visual/interaction states.
 * @dev Allowed transitions:
 *   idle        → submitting | disabled
 *   submitting  → success | error | disabled
 *   success     → idle | disabled
 *   error       → idle | submitting | disabled
 *   disabled    → idle
 */
export type SubmitButtonState = "idle" | "submitting" | "success" | "error" | "disabled";

/**
 * @notice Optional per-state label overrides.
 * @dev Values are normalized: non-strings, empty strings, and control characters are rejected.
 * @title Submit Button Labels
 * @notice Optional custom labels for each user-visible state.
 * @notice Optional override labels for each supported state.
 * @notice Optional override labels for each supported state.
 * @notice Optional custom labels for each user-visible state.
 * @notice Optional per-state label overrides.
 * @dev Values are normalized: non-strings, empty strings, and control characters are rejected.
 */
export interface SubmitButtonLabels {
  idle?: string;
  submitting?: string;
  success?: string;
  error?: string;
  disabled?: string;
}

/**
 * @notice Props accepted by ReactSubmitButton.
 * @param state          Current button state (required).
 * @param previousState  Previous state used for strict transition validation.
 * @param strictTransitions  When true, invalid state jumps fall back to previousState.
 * @param labels         Optional label overrides per state.
 * @param onClick        Async-safe click handler; blocked while submitting/disabled.
 * @param className      Additional CSS class.
 * @param id             HTML id attribute.
 * @param type           HTML button type. Default: "button".
 * @param disabled       External disabled override.
 */
export interface ReactSubmitButtonProps {
  state: SubmitButtonState;
  previousState?: SubmitButtonState;
  strictTransitions?: boolean;
  labels?: SubmitButtonLabels;
  onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void | Promise<void>;
  className?: string;
  id?: string;
  type?: "button" | "submit" | "reset";
  disabled?: boolean;
}

// ── Constants ─────────────────────────────────────────────────────────────────

const MAX_LABEL_LENGTH = 80;
const CONTROL_CHAR_RE = /[\u0000-\u001F\u007F]/g;

 * @title Submit Button Props
 * @notice Defines behavior, labeling, and accessibility requirements.
 */
export interface ReactSubmitButtonProps {
  state: SubmitButtonState;
  labels?: SubmitButtonLabels;
  onClick?: () => void | Promise<void>;
 * @title React Submit Button Props
 * @notice Contract for state, label overrides, and interaction behavior.
 * @notice Props accepted by ReactSubmitButton.
 * @param state          Current button state (required).
 * @param previousState  Previous state used for strict transition validation.
 * @param strictTransitions  When true, invalid state jumps fall back to previousState.
 * @param labels         Optional label overrides per state.
 * @param onClick        Async-safe click handler; blocked while submitting/disabled.
 * @param className      Additional CSS class.
 * @param id             HTML id attribute.
 * @param type           HTML button type. Default: "button".
 * @param disabled       External disabled override.
 */
export interface ReactSubmitButtonProps {
  state: SubmitButtonState;
  previousState?: SubmitButtonState;
  strictTransitions?: boolean;
  labels?: SubmitButtonLabels;
  onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void | Promise<void>;
  className?: string;
  id?: string;
  type?: "button" | "submit" | "reset";
  disabled?: boolean;
}

// ── Constants ─────────────────────────────────────────────────────────────────

const MAX_LABEL_LENGTH = 80;
const CONTROL_CHAR_RE = /[\u0000-\u001F\u007F]/g;

const DEFAULT_LABELS: Required<SubmitButtonLabels> = {
  idle: "Submit",
  submitting: "Submitting...",
  success: "Submitted",
  error: "Try Again",
  disabled: "Submit Disabled",
};

/**
 * @notice Allowed state transitions.
 * @dev Centralised here so both the guard and tests share one source of truth.
 */
export const ALLOWED_TRANSITIONS: Record<SubmitButtonState, SubmitButtonState[]> = {
  idle: ["submitting", "disabled"],
  submitting: ["success", "error", "disabled"],
  success: ["idle", "disabled"],
  error: ["idle", "submitting", "disabled"],
  disabled: ["idle"],
};
 * @title Safe Label Resolver
 * @notice Provides a bounded, non-empty label to avoid empty UI text states.
 * @dev React escapes text content by default; this function only normalizes.
const MAX_LABEL_LENGTH = 80;
const CONTROL_CHARACTER_REGEX = /[\u0000-\u001F\u007F]/g;

const ALLOWED_STATE_TRANSITIONS: Record<SubmitButtonState, SubmitButtonState[]> = {
 * @notice Allowed state transitions.
 * @dev Centralised here so both the guard and tests share one source of truth.
 */
export const ALLOWED_TRANSITIONS: Record<SubmitButtonState, SubmitButtonState[]> = {
  idle: ["submitting", "disabled"],
  submitting: ["success", "error", "disabled"],
  success: ["idle", "disabled"],
  error: ["idle", "submitting", "disabled"],
  disabled: ["idle"],
};

/**
 * @title Label Normalizer
 * @notice Enforces string-only, bounded, and readable labels.
 * @dev Removes control characters and normalizes whitespace to reduce rendering abuse vectors.
 */
export function normalizeSubmitButtonLabel(candidate: unknown, fallback: string): string {
  if (typeof candidate !== "string") {
    return fallback;
  }

  const withoutControlCharacters = candidate.replace(CONTROL_CHARACTER_REGEX, " ");
  const normalized = withoutControlCharacters.replace(/\s+/g, " ").trim();

  if (!normalized) {
    return fallback;
  }

  if (normalized.length <= MAX_LABEL_LENGTH) {
    return normalized;
  }

  return `${normalized.slice(0, MAX_LABEL_LENGTH - 3)}...`;
}

/**
 * @title Label Resolver
 * @notice Returns a safe, non-empty label for the current state.
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
  return normalizeSubmitButtonLabel(labels?.[state], DEFAULT_LABELS[state]);
}

/**
 * @title State Transition Validator
 * @notice Validates deterministic transitions between known button states.
 */
export function isValidSubmitButtonStateTransition(
  previousState: SubmitButtonState,
  nextState: SubmitButtonState,
): boolean {
  if (previousState === nextState) {
    return true;
  }

  return ALLOWED_STATE_TRANSITIONS[previousState].includes(nextState);
}

/**
 * @title Safe State Resolver
 * @notice Resolves final state and blocks invalid transitions when strict mode is enabled.
 */
export function resolveSafeSubmitButtonState(
  state: SubmitButtonState,
  previousState?: SubmitButtonState,
  strictTransitions = true,
): SubmitButtonState {
  if (!strictTransitions || !previousState) {
    return state;
  }

  return isValidSubmitButtonStateTransition(previousState, state) ? state : previousState;
}

/**
 * @title Interaction Block Guard
 * @notice Disables interaction while submitting, disabled, or during local in-flight execution.
 */
export function isSubmitButtonInteractionBlocked(
  state: SubmitButtonState,
  disabled = false,
  isLocallySubmitting = false,
): boolean {
  return Boolean(disabled) || state === "disabled" || state === "submitting" || isLocallySubmitting;
}

/**
 * @title Busy State Guard
 * @notice Reflects busy accessibility semantics for submission and in-flight local execution.
 */
export function isSubmitButtonBusy(
  state: SubmitButtonState,
  isLocallySubmitting = false,
): boolean {
  return state === "submitting" || isLocallySubmitting;
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

/**
 * @notice Per-state style overrides.
 * @security All values are hardcoded constants — no dynamic CSS injection from user input.
 */
const STATE_STYLES: Record<SubmitButtonState, React.CSSProperties> = {
const STATE_STYLE_MAP: Record<SubmitButtonState, React.CSSProperties> = {
  idle: { backgroundColor: "#4f46e5" },
  submitting: { backgroundColor: "#6366f1" },
  success: { backgroundColor: "#16a34a", borderColor: "#15803d" },
  error: { backgroundColor: "#dc2626", borderColor: "#b91c1c" },
  disabled: { backgroundColor: "#9ca3af", borderColor: "#9ca3af", cursor: "not-allowed", opacity: 0.9 },
  disabled: {
    backgroundColor: "#9ca3af",
    borderColor: "#9ca3af",
    cursor: "not-allowed",
    opacity: 0.9,
  },
  disabled: { backgroundColor: "#9ca3af", borderColor: "#9ca3af", cursor: "not-allowed", opacity: 0.9 },
};

// ── Pure helpers (exported for unit testing) ──────────────────────────────────

/**
 * @title normalizeSubmitButtonLabel
 * @notice Sanitizes a candidate label: rejects non-strings, strips control characters,
 *         normalizes whitespace, and truncates to MAX_LABEL_LENGTH.
 * @param candidate  Untrusted input (may be any type).
 * @param fallback   Returned when candidate is unusable.
 * @security Prevents blank CTA states and layout-abuse via oversized labels.
 */
export function normalizeSubmitButtonLabel(candidate: unknown, fallback: string): string {
  if (typeof candidate !== "string") return fallback;
  const cleaned = candidate.replace(CONTROL_CHAR_RE, " ").replace(/\s+/g, " ").trim();
  if (!cleaned) return fallback;
  if (cleaned.length <= MAX_LABEL_LENGTH) return cleaned;
  return `${cleaned.slice(0, MAX_LABEL_LENGTH - 3)}...`;
}

/**
 * @title resolveSubmitButtonLabel
 * @notice Returns a safe, non-empty label for the given state.
 * @param state   Current button state.
 * @param labels  Optional overrides.
 */
export function resolveSubmitButtonLabel(
  state: SubmitButtonState,
  labels?: SubmitButtonLabels,
): string {
  return normalizeSubmitButtonLabel(labels?.[state], DEFAULT_LABELS[state]);
}

/**
 * @title isValidSubmitButtonStateTransition
 * @notice Returns true when moving from `from` to `to` is an allowed transition.
 * @dev Same-state updates are always allowed (idempotent).
 */
export function isValidSubmitButtonStateTransition(
  from: SubmitButtonState,
  to: SubmitButtonState,
): boolean {
  return from === to || ALLOWED_TRANSITIONS[from].includes(to);
}

/**
 * @title resolveSafeSubmitButtonState
 * @notice In strict mode, falls back to `previousState` when the transition is invalid.
 * @param state            Requested next state.
 * @param previousState    Last known valid state.
 * @param strictTransitions  Enables transition enforcement. Default: true.
 */
export function resolveSafeSubmitButtonState(
  state: SubmitButtonState,
  previousState?: SubmitButtonState,
  strictTransitions = true,
): SubmitButtonState {
  if (!strictTransitions || !previousState) return state;
  return isValidSubmitButtonStateTransition(previousState, state) ? state : previousState;
}

/**
 * @title isSubmitButtonInteractionBlocked
 * @notice Returns true when the button must not respond to clicks.
 * @param state              Resolved button state.
 * @param disabled           External disabled flag.
 * @param isLocallySubmitting  True while an async onClick is in-flight.
 * @security Prevents duplicate blockchain transactions on rapid clicks.
 */
export function isSubmitButtonInteractionBlocked(
  state: SubmitButtonState,
  disabled = false,
  isLocallySubmitting = false,
): boolean {
  return Boolean(disabled) || state === "disabled" || state === "submitting" || isLocallySubmitting;
}

/**
 * @title isSubmitButtonBusy
 * @notice Returns true when aria-busy should be set (active submission in progress).
 * @param state              Resolved button state.
 * @param isLocallySubmitting  True while an async onClick is in-flight.
 */
export function isSubmitButtonBusy(
  state: SubmitButtonState,
  isLocallySubmitting = false,
): boolean {
  return state === "submitting" || isLocallySubmitting;
}

// ── Component ─────────────────────────────────────────────────────────────────

/**
 * @title ReactSubmitButton
 * @notice Reusable submit button with strict state machine, safe labels,
 *         double-submit prevention, and ARIA accessibility.
 * @dev onClick is wrapped in a local in-flight guard so async handlers cannot
 *      trigger a second submission before the first resolves.
 */
const ReactSubmitButton = ({
  state,
  previousState,
  strictTransitions = true,
/**
 * @title React Submit Button
 * @notice Reusable submit button with typed state machine for scalable workflows.
 * @dev Avoids exposing raw HTML injection paths and enforces accessible semantics.
 */
const ReactSubmitButton = ({
  state,
 * @notice Reusable submit button with secure labels and transition-aware state handling.
 * @dev `onClick` is blocked while in-flight to reduce duplicate submissions.
 * @title normalizeSubmitButtonLabel
 * @notice Sanitizes a candidate label: rejects non-strings, strips control characters,
 *         normalizes whitespace, and truncates to MAX_LABEL_LENGTH.
 * @param candidate  Untrusted input (may be any type).
 * @param fallback   Returned when candidate is unusable.
 * @security Prevents blank CTA states and layout-abuse via oversized labels.
 */
export function normalizeSubmitButtonLabel(candidate: unknown, fallback: string): string {
  if (typeof candidate !== "string") return fallback;
  const cleaned = candidate.replace(CONTROL_CHAR_RE, " ").replace(/\s+/g, " ").trim();
  if (!cleaned) return fallback;
  if (cleaned.length <= MAX_LABEL_LENGTH) return cleaned;
  return `${cleaned.slice(0, MAX_LABEL_LENGTH - 3)}...`;
}

/**
 * @title resolveSubmitButtonLabel
 * @notice Returns a safe, non-empty label for the given state.
 * @param state   Current button state.
 * @param labels  Optional overrides.
 */
export function resolveSubmitButtonLabel(
  state: SubmitButtonState,
  labels?: SubmitButtonLabels,
): string {
  return normalizeSubmitButtonLabel(labels?.[state], DEFAULT_LABELS[state]);
}

/**
 * @title isValidSubmitButtonStateTransition
 * @notice Returns true when moving from `from` to `to` is an allowed transition.
 * @dev Same-state updates are always allowed (idempotent).
 */
export function isValidSubmitButtonStateTransition(
  from: SubmitButtonState,
  to: SubmitButtonState,
): boolean {
  return from === to || ALLOWED_TRANSITIONS[from].includes(to);
}

/**
 * @title resolveSafeSubmitButtonState
 * @notice In strict mode, falls back to `previousState` when the transition is invalid.
 * @param state            Requested next state.
 * @param previousState    Last known valid state.
 * @param strictTransitions  Enables transition enforcement. Default: true.
 */
export function resolveSafeSubmitButtonState(
  state: SubmitButtonState,
  previousState?: SubmitButtonState,
  strictTransitions = true,
): SubmitButtonState {
  if (!strictTransitions || !previousState) return state;
  return isValidSubmitButtonStateTransition(previousState, state) ? state : previousState;
}

/**
 * @title isSubmitButtonInteractionBlocked
 * @notice Returns true when the button must not respond to clicks.
 * @param state              Resolved button state.
 * @param disabled           External disabled flag.
 * @param isLocallySubmitting  True while an async onClick is in-flight.
 * @security Prevents duplicate blockchain transactions on rapid clicks.
 *           Success state is also blocked to prevent re-submission after confirmation.
 */
export function isSubmitButtonInteractionBlocked(
  state: SubmitButtonState,
  disabled = false,
  isLocallySubmitting = false,
): boolean {
  return Boolean(disabled) || state === "disabled" || state === "submitting" || state === "success" || isLocallySubmitting;
}

/**
 * @title isSubmitButtonBusy
 * @notice Returns true when aria-busy should be set (active submission in progress).
 * @param state              Resolved button state.
 * @param isLocallySubmitting  True while an async onClick is in-flight.
 */
export function isSubmitButtonBusy(
  state: SubmitButtonState,
  isLocallySubmitting = false,
): boolean {
  return state === "submitting" || isLocallySubmitting;
}

// ── Component ─────────────────────────────────────────────────────────────────

/**
 * @title ReactSubmitButton
 * @notice Reusable submit button with strict state machine, safe labels,
 *         double-submit prevention, and ARIA accessibility.
 * @dev onClick is wrapped in a local in-flight guard so async handlers cannot
 *      trigger a second submission before the first resolves.
 */
const ReactSubmitButton = ({
  state,
  previousState,
  strictTransitions = true,
  labels,
  onClick,
  className,
  id,
  type = "button",
  disabled,
}: ReactSubmitButtonProps) => {
  const [isLocallySubmitting, setIsLocallySubmitting] = useState(false);

  const resolvedState = resolveSafeSubmitButtonState(state, previousState, strictTransitions);
  const label = resolveSubmitButtonLabel(resolvedState, labels);
  const blocked = isSubmitButtonInteractionBlocked(resolvedState, disabled, isLocallySubmitting);
  const ariaBusy = isSubmitButtonBusy(resolvedState, isLocallySubmitting);

  const handleClick = async (event: React.MouseEvent<HTMLButtonElement>) => {
    if (blocked || !onClick) return;
    setIsLocallySubmitting(true);
    try {
      await Promise.resolve(onClick(event));
    } catch {
      // Errors are the caller's responsibility; we only reset local state.
    } finally {
      setIsLocallySubmitting(false);
    }
  };
  const label = resolveSubmitButtonLabel(state, labels);
  const computedDisabled = isSubmitButtonDisabled(state, disabled);
  const ariaBusy = isSubmitButtonBusy(state);
  const [isLocallySubmitting, setIsLocallySubmitting] = useState(false);

  const resolvedState = resolveSafeSubmitButtonState(state, previousState, strictTransitions);
  const label = resolveSubmitButtonLabel(resolvedState, labels);
  const blocked = isSubmitButtonInteractionBlocked(resolvedState, disabled, isLocallySubmitting);
  const ariaBusy = isSubmitButtonBusy(resolvedState, isLocallySubmitting);

  const handleClick = async (event: React.MouseEvent<HTMLButtonElement>) => {
    if (blocked || !onClick) return;
    setIsLocallySubmitting(true);
    try {
      await Promise.resolve(onClick(event));
    } catch {
      // Errors are the caller's responsibility; we only reset local state.
    } finally {
      setIsLocallySubmitting(false);
    }
  };

  return (
    <button
      id={id}
      type={type}
      className={className}
      disabled={blocked}
      aria-busy={ariaBusy}
      aria-live="polite"
      aria-label={label}
      data-state={resolvedState}
      onClick={blocked ? undefined : handleClick}
      style={{ ...BASE_STYLE, ...STATE_STYLES[resolvedState] }}
      disabled={computedDisabled}
      aria-busy={ariaBusy}
      aria-live="polite"
      aria-label={label}
      onClick={computedDisabled ? undefined : onClick}
      style={{
        ...BASE_STYLE,
        ...STATE_STYLE_MAP[state],
      onClick={computedDisabled ? undefined : handleClick}
      style={{
        ...BASE_STYLE,
        ...STATE_STYLE_MAP[resolvedState],
      onClick={computedDisabled ? undefined : onClick}
      style={{
        ...BASE_STYLE,
        ...STATE_STYLE_MAP[state],
        ...(computedDisabled ? { cursor: "not-allowed", opacity: 0.7 } : {}),
      }}
      data-state={resolvedState}
      onClick={blocked ? undefined : handleClick}
      style={{ ...BASE_STYLE, ...STATE_STYLES[resolvedState] }}
    >
      {label}
/**
 * @title React Submit Button Component
 * @notice Standardized submit button with consistent states for testing and developer experience.
 * @dev Implements idle, loading, disabled, and variant states. Prevents double-submit when loading.
 * @custom:security Prevents injection via children; uses type="submit" for form semantics.
 */

import React from "react";

import "./forms/Forms.css";

/** @dev Button variant matching Forms.css classes */
export type SubmitButtonVariant =
  | "primary"
  | "secondary"
  | "danger"
  | "outline";

/** @dev Standardized state for testing and DX */
export type SubmitButtonState = "idle" | "loading" | "disabled";

export interface ReactSubmitButtonProps
  extends Omit<
    React.ButtonHTMLAttributes<HTMLButtonElement>,
    "type" | "disabled" | "children"
  > {
  /** @dev Button label. Use string only; avoids injection. */
  children: React.ReactNode;
  /** @dev When true, shows spinner and prevents click. Prevents double-submit. */
  isLoading?: boolean;
  /** @dev Explicit disabled state (e.g. form validation). */
  disabled?: boolean;
  /** @dev Visual variant. Default: primary. */
  variant?: SubmitButtonVariant;
  /** @dev Full-width layout. Default: false. */
  fullWidth?: boolean;
  /** @dev Accessible label when loading. Default: "Loading..." */
  loadingLabel?: string;
  /** @dev Form id to associate with (optional). */
  form?: string;
}

const VARIANT_CLASS: Record<SubmitButtonVariant, string> = {
  primary: "btn btn--primary",
  secondary: "btn btn--secondary",
  danger: "btn btn--danger",
  outline: "btn btn--outline",
};

/**
 * @title SubmitButton
 * @notice Standardized submit button component with consistent states.
 * @dev Renders a <button type="submit"> with loading spinner, disabled handling, and variant styles.
 *      When isLoading, button is disabled and shows loadingLabel. Combines disabled + isLoading for explicit control.
 */
const ReactSubmitButton = ({
  children,
  isLoading = false,
  disabled = false,
  variant = "primary",
  fullWidth = false,
  loadingLabel = "Loading...",
  form,
  className = "",
  onClick,
  "aria-busy": ariaBusy,
  ...rest
}: ReactSubmitButtonProps) => {
  const isDisabled = disabled || isLoading;
  const baseClass = VARIANT_CLASS[variant];
  const fullClass = fullWidth ? "btn--full" : "";
  const combinedClassName = [baseClass, fullClass, className].filter(Boolean).join(" ");

  const handleClick = isDisabled
    ? undefined
    : (e: React.MouseEvent<HTMLButtonElement>) => onClick?.(e);

  return (
    <button
      type="submit"
      form={form}
      className={combinedClassName}
      disabled={isDisabled}
      aria-busy={ariaBusy ?? isLoading}
      aria-disabled={isDisabled}
      onClick={handleClick}
      {...rest}
    >
      {isLoading ? (
        <>
          <span
            className="btn__spinner"
            role="status"
            aria-hidden="true"
          />
          <span>{loadingLabel}</span>
        </>
      ) : (
        children
      )}
/**
 * @title SubmitButton
 * @notice A robust, accessible React submit button component with full state management.
 * @dev Handles idle, loading, success, error, and disabled states with clear visual
 *      feedback. Designed for crowdfunding transaction flows where state clarity is
 *      critical to user trust and security.
 *
 * @security
 * - The `onClick` handler is only invoked when the button is in the `idle` state,
 *   preventing duplicate submissions (double-spend protection).
 * - The button is rendered as `type="submit"` by default and `type="button"` when
 *   used outside a form, preventing accidental form submissions.
 * - No user-supplied strings are injected as HTML; all dynamic content is text-only.
 * - `aria-disabled` is set alongside the native `disabled` attribute so assistive
 *   technologies correctly announce the button state.
 */

import React, { useCallback, useEffect, useRef, useState } from "react";
import {
  ButtonState,
  SubmitButtonProps,
  STATE_CONFIG,
} from "./react_submit_button_types";

export type { ButtonState, SubmitButtonProps };
export { STATE_CONFIG };

/**
 * @title SubmitButton
 * @notice Accessible submit button with idle / loading / success / error / disabled states.
 *
 * @param props - See {@link SubmitButtonProps}
 *
 * @example
 * ```tsx
 * <SubmitButton
 *   label="Fund Campaign"
 *   onClick={async () => { await submitTransaction(); }}
 * />
 * ```
 *
 * @security
 * - Clicks are ignored in loading/success/disabled states (prevents double-submit).
 * - `resetDelay` defaults to 2500 ms; callers may increase it but not set it below 0.
 * - The component cleans up its reset timer on unmount to prevent state updates on
 *   unmounted components (memory-leak / stale-closure protection).
 */
const SubmitButton: React.FC<SubmitButtonProps> = ({
  label,
  onClick,
  disabled = false,
  resetDelay = 2500,
  type = "submit",
  style,
  "data-testid": testId,
}) => {
  const [state, setState] = useState<ButtonState>(disabled ? "disabled" : "idle");
  const resetTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Sync external `disabled` prop → internal state when not mid-flight.
  useEffect(() => {
    if (state === "loading" || state === "success" || state === "error") return;
    setState(disabled ? "disabled" : "idle");
  }, [disabled, state]);

  // Cleanup timer on unmount.
  useEffect(() => {
    return () => {
      if (resetTimerRef.current !== null) {
        clearTimeout(resetTimerRef.current);
      }
    };
  }, []);

  /**
   * @notice Handles button click.
   * @dev Guards against clicks in non-idle states to prevent duplicate submissions.
   */
  const handleClick = useCallback(async () => {
    if (state !== "idle" && state !== "error") return;

    setState("loading");

    try {
      await onClick();
      setState("success");
      resetTimerRef.current = setTimeout(() => {
        setState(disabled ? "disabled" : "idle");
      }, Math.max(0, resetDelay));
    } catch {
      setState("error");
      resetTimerRef.current = setTimeout(() => {
        setState(disabled ? "disabled" : "idle");
      }, Math.max(0, resetDelay));
    }
  }, [state, onClick, disabled, resetDelay]);

  const isInteractive = state === "idle" || state === "error";
  const isNativeDisabled =
    state === "loading" || state === "disabled" || state === "success";

  const config = STATE_CONFIG[state];
  const displayLabel =
    state === "idle" || state === "disabled" ? label : config.label;
  const ariaLabel =
    state === "idle" || state === "disabled" ? label : config.ariaLabel;

  return (
    <button
      type={type}
      onClick={isInteractive ? handleClick : undefined}
      disabled={isNativeDisabled}
      aria-disabled={!isInteractive}
      aria-label={ariaLabel}
      aria-busy={state === "loading"}
      data-state={state}
      data-testid={testId}
      style={{
        ...baseStyle,
        backgroundColor: config.backgroundColor,
        cursor: config.cursor,
        opacity: state === "disabled" ? 0.6 : 1,
        ...(style as React.CSSProperties),
      }}
    >
      {state === "loading" && <span style={spinnerStyle} aria-hidden="true" />}
      <span>{displayLabel}</span>
    </button>
  );
};

export default ReactSubmitButton;
// ── Styles ────────────────────────────────────────────────────────────────────

const baseStyle: React.CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  gap: "0.5rem",
  padding: "0.625rem 1.5rem",
  borderRadius: "0.5rem",
  border: "none",
  color: "#ffffff",
  fontSize: "0.9375rem",
  fontWeight: 600,
  letterSpacing: "0.01em",
  transition: "background-color 0.2s ease, opacity 0.2s ease",
  userSelect: "none",
  minWidth: "9rem",
};

const spinnerStyle: React.CSSProperties = {
  display: "inline-block",
  width: "0.875rem",
  height: "0.875rem",
  border: "2px solid rgba(255,255,255,0.4)",
  borderTopColor: "#ffffff",
  borderRadius: "50%",
  animation: "spin 0.7s linear infinite",
};

export default SubmitButton;
