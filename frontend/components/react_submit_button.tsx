/**
 * @notice Optimized React Submit Button for script execution and Stellar tx.
 * @dev Secure state machine prevents double-tx, sanitizes labels/script output, pending for scripts.
 * @custom:security No double-submit (inFlightRef), XSS-safe labels/output/txHash, isMounted guard.
 */

import React, {
  useReducer,
  useCallback,
  useRef,
  useEffect,
  useMemo,
  type MouseEvent,
} from "react";

/** Button states optimized for script/tx flow. */
export type SubmitButtonState =
  | "idle"
  | "pending"
  | "success"
  | "error"
  | "disabled";

/** Per-state label overrides (sanitized on use). */
export interface SubmitButtonLabels {
  idle?: string;
  pending?: string;
  success?: string;
  error?: string;
  disabled?: string;
}

/** Props for script/tx integration. */
export interface ReactSubmitButtonProps {
  /** Current state (required). */
  state: SubmitButtonState;
  /** Previous state for transition validation. */
  previousState?: SubmitButtonState;
  /** Enforce strict transitions (default true). */
  strictTransitions?: boolean;
  /** Custom labels per state. */
  labels?: SubmitButtonLabels;
  /** Script output / tx result (truncated, sanitized). */
  scriptOutput?: unknown;
  /** Truncated tx hash for display. */
  txHash?: string;
  /** Click handler. */
  onClick?: (e: MouseEvent<HTMLButtonElement>) => void | Promise<void>;
  /** CSS class. */
  className?: string;
  /** Element ID. */
  id?: string;
  /** Button type (default 'button'). */
  type?: "button" | "submit" | "reset";
  /** External disabled. */
  disabled?: boolean;
}

// ── Reducer ──
type LocalAction = { type: "START_PENDING" } | { type: "END_PENDING" };
interface LocalState {
  isPending: boolean;
}

export function submitButtonReducer(
  state: LocalState,
  action: LocalAction
): LocalState {
  switch (action.type) {
    case "START_PENDING":
      return { isPending: true };
    case "END_PENDING":
      return { isPending: false };
    default:
      return state;
  }
}

// ── Constants ──
export const MAX_LABEL_LENGTH = 80;
export const MAX_HASH_DISPLAY = 12;
const CONTROL_CHAR_RE = /[\u0000-\u001F\u007F]/g;

export const DEFAULT_LABELS: Required<SubmitButtonLabels> = {
  idle: "Execute Script",
  pending: "Running...",
  success: "Success",
  error: "Retry",
  disabled: "Disabled",
};

export const ALLOWED_TRANSITIONS: Record<
  SubmitButtonState,
  SubmitButtonState[]
> = {
  idle: ["pending", "disabled"],
  pending: ["success", "error", "disabled"],
  success: ["idle", "disabled"],
  error: ["idle", "pending", "disabled"],
  disabled: ["idle"],
};

const BASE_STYLE: React.CSSProperties = {
  minHeight: "44px",
  minWidth: "140px",
  borderRadius: "8px",
  border: "1px solid #4f46e5",
  padding: "0.5rem 1rem",
  color: "#ffffff",
  fontWeight: 600,
  cursor: "pointer",
  transition: "all 0.2s ease",
  backgroundColor: "#4f46e5",
  display: "inline-flex",
  alignItems: "center",
  gap: "0.5rem",
};

const STATE_STYLES: Record<SubmitButtonState, React.CSSProperties> = {
  idle: { backgroundColor: "#4f46e5" },
  pending: { backgroundColor: "#6366f1" },
  success: { backgroundColor: "#16a34a", borderColor: "#15803d" },
  error: { backgroundColor: "#dc2626", borderColor: "#b91c1c" },
  disabled: {
    backgroundColor: "#9ca3af",
    borderColor: "#9ca3af",
    cursor: "not-allowed",
    opacity: 0.7,
  },
};

// ── Pure Helpers (tested independently) ──

/** Sanitizes label/script output. */
export function normalizeText(
  candidate: unknown,
  fallback: string,
  maxLen = MAX_LABEL_LENGTH
): string {
  if (typeof candidate !== "string") return fallback;
  const cleaned = candidate
    .replace(CONTROL_CHAR_RE, " ")
    .replace(/\s+/g, " ")
    .trim();
  if (!cleaned) return fallback;
  return cleaned.length <= maxLen
    ? cleaned
    : `${cleaned.slice(0, maxLen - 3)}...`;
}

/** Resolves safe label. */
export function resolveLabel(
  state: SubmitButtonState,
  labels?: SubmitButtonLabels
): string {
  return normalizeText(labels?.[state], DEFAULT_LABELS[state]);
}

/** Validates state transition. */
export function isValidTransition(
  from: SubmitButtonState,
  to: SubmitButtonState
): boolean {
  return from === to || ALLOWED_TRANSITIONS[from].includes(to);
}

/** Strict state resolver. */
export function resolveSafeState(
  state: SubmitButtonState,
  prev?: SubmitButtonState,
  strict = true
): SubmitButtonState {
  if (!strict || !prev || isValidTransition(prev, state)) return state;
  return prev;
}

/** Blocks clicks? */
export function isInteractionBlocked(
  state: SubmitButtonState,
  disabled = false,
  localPending = false
): boolean {
  return (
    Boolean(disabled) ||
    ["pending", "success", "disabled"].includes(state) ||
    localPending
  );
}

/** ARIA busy? */
export function isBusy(
  state: SubmitButtonState,
  localPending = false
): boolean {
  return state === "pending" || localPending;
}

// ── Component ──
const ReactSubmitButton: React.FC<ReactSubmitButtonProps> = ({
  state,
  previousState,
  strictTransitions = true,
  labels,
  scriptOutput,
  txHash,
  onClick,
  className = "",
  id,
  type = "button",
  disabled,
}) => {
  const [{ isPending: localPending }, dispatch] = useReducer(
    submitButtonReducer,
    { isPending: false }
  );
  const inFlightRef = useRef(false);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const resolvedState = useMemo(
    () => resolveSafeState(state, previousState, strictTransitions),
    [state, previousState, strictTransitions]
  );

  const label = useMemo(
    () => resolveLabel(resolvedState, labels),
    [resolvedState, labels]
  );

  const subtext = useMemo(() => {
    let text = "";
    if (txHash) text = `Tx: …${txHash.slice(-MAX_HASH_DISPLAY)}`;
    else if (scriptOutput)
      text = normalizeText(scriptOutput, "Script output", 40);
    return text ? `(${text})` : "";
  }, [txHash, scriptOutput]);

  const blocked = useMemo(
    () => isInteractionBlocked(resolvedState, disabled, localPending),
    [resolvedState, disabled, localPending]
  );

  const ariaBusy = isBusy(resolvedState, localPending);

  const handleClick = useCallback(
    async (e: MouseEvent<HTMLButtonElement>) => {
      if (inFlightRef.current || blocked || !onClick) return;
      inFlightRef.current = true;
      dispatch({ type: "START_PENDING" });
      try {
        await Promise.resolve(onClick(e));
      } catch {
        // Caller handles errors.
      } finally {
        inFlightRef.current = false;
        if (mountedRef.current) dispatch({ type: "END_PENDING" });
      }
    },
    [blocked, onClick]
  );

  const style = useMemo(
    () => ({ ...BASE_STYLE, ...STATE_STYLES[resolvedState] }),
    [resolvedState]
  );

  return (
    <button
      id={id}
      type={type}
      className={`submit-btn ${className}`.trim()}
      disabled={blocked}
      aria-label={`${label}${subtext ? ` ${subtext}` : ""}`}
      aria-live="polite"
      aria-busy={ariaBusy}
      onClick={!blocked ? handleClick : undefined}
      style={style}
      data-state={resolvedState}
      title={subtext || label}
    >
      {resolvedState === "pending" && (
        <svg
          width="16"
          height="16"
          viewBox="0 0 16 16"
          aria-hidden="true"
          style={{ animation: "spin 1s linear infinite" }}
        >
          <circle
            cx="8"
            cy="8"
            r="7"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeDasharray="28 28"
            strokeLinecap="round"
          />
        </svg>
      )}
      <span>{label}</span>
      {subtext && (
        <small style={{ fontSize: "0.8em", opacity: 0.8 }}>{subtext}</small>
      )}
    </button>
  );
};

export default ReactSubmitButton;
