import React, { Component, ErrorInfo, ReactNode } from 'react';

// ---------------------------------------------------------------------------
// Custom Error Classes
// ---------------------------------------------------------------------------

/**
 * @title ContractError
 * @dev Represents errors originating from smart contract execution on Stellar/Soroban.
 * Thrown when a contract invocation fails, returns an unexpected result, or
 * the transaction is rejected by the network.
 *
 * @custom:security Never include raw contract state or private keys in the message.
 */
export class ContractError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ContractError';
  }
}

/**
 * @title NetworkError
 * @dev Represents errors caused by network connectivity issues when communicating
 * with the Stellar Horizon API or RPC endpoints.
 */
export class NetworkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'NetworkError';
  }
}

/**
 * @title TransactionError
 * @dev Represents errors that occur during blockchain transaction submission,
 * signing, or confirmation phases.
 *
 * @custom:security Do not embed transaction XDR or signing keys in the message.
 */
export class TransactionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'TransactionError';
  }
}

// ---------------------------------------------------------------------------
// Error classification helpers
// ---------------------------------------------------------------------------

/** Keywords that indicate a smart-contract / blockchain related error. */
const CONTRACT_KEYWORDS = [
  'contract',
  'stellar',
  'soroban',
  'transaction',
  'blockchain',
  'ledger',
  'horizon',
  'xdr',
  'invoke',
  'wallet',
] as const;

/**
 * @dev Determines whether an error is related to smart contract execution.
 * Result is computed once per error instance and cached on the object to
 * avoid redundant string scans across multiple render cycles (gas efficiency).
 *
 * @param error The error to classify.
 * @return `true` if the error is contract/blockchain related.
 *
 * @custom:security This is a best-effort heuristic. Unknown error types default
 * to the generic handler, which is the safer path.
 */
const _classificationCache = new WeakMap<Error, boolean>();

function isSmartContractError(error: Error): boolean {
  if (_classificationCache.has(error)) {
    return _classificationCache.get(error)!;
  }
  let result: boolean;
  if (
    error instanceof ContractError ||
    error instanceof NetworkError ||
    error instanceof TransactionError
  ) {
    result = true;
  } else {
    const haystack = `${error.name} ${error.message}`.toLowerCase();
    result = CONTRACT_KEYWORDS.some((kw) => haystack.includes(kw));
  }
  _classificationCache.set(error, result);
  return result;
}

// ---------------------------------------------------------------------------
// Structured error report
// ---------------------------------------------------------------------------

export interface ErrorReport {
  message: string;
  stack: string | undefined;
  componentStack: string | null | undefined;
  timestamp: string;
  isSmartContractError: boolean;
  errorName: string;
}

/**
 * @dev Builds a structured, sanitised error report suitable for forwarding to
 * an external observability service (Sentry, Datadog, etc.).
 *
 * @custom:security Stack traces are included only in development mode so that
 * sensitive implementation details are not exposed in production logs.
 */
function buildErrorReport(
  error: Error,
  errorInfo: ErrorInfo,
  isContract: boolean,
): ErrorReport {
  const isDev = process.env.NODE_ENV !== 'production';
  return {
    message: error.message,
    stack: isDev ? error.stack : undefined,
    componentStack: isDev ? errorInfo.componentStack : undefined,
    timestamp: new Date().toISOString(),
    isSmartContractError: isContract,
    errorName: error.name,
  };
}

// ---------------------------------------------------------------------------
// Component types
// ---------------------------------------------------------------------------

/** Maximum number of retry attempts before the retry button is hidden. */
export const MAX_RETRIES = 3;

export interface FrontendGlobalErrorBoundaryProps {
  /**
   * @dev The child component tree to protect with this error boundary.
   */
  children?: ReactNode;

  /**
   * @dev Optional custom fallback UI. When provided it replaces the built-in
   * fallback entirely, giving callers full control over the error presentation.
   */
  fallback?: ReactNode;

  /**
   * @dev Optional callback invoked with a structured error report whenever an
   * error is caught. Use this to forward errors to Sentry, LogRocket, etc.
   *
   * @param report Sanitised error report (stack omitted in production).
   */
  onError?: (report: ErrorReport) => void;
}

interface BoundaryState {
  hasError: boolean;
  error: Error | null;
  isSmartContractError: boolean;
  /** Number of retry attempts made so far. */
  retryCount: number;
}

// ---------------------------------------------------------------------------
// FrontendGlobalErrorBoundary
// ---------------------------------------------------------------------------

/**
 * @title FrontendGlobalErrorBoundary
 * @dev React class-based error boundary for the Stellar Raise frontend.
 *
 * Catches synchronous render-phase errors anywhere in the wrapped component
 * tree, classifies them (generic vs. smart-contract), logs a structured report,
 * and renders an appropriate fallback UI with a "Try Again" recovery path.
 *
 * Gas-efficiency improvements over the previous version:
 *   - Error classification result is cached via WeakMap so repeated renders
 *     do not re-scan the error message string.
 *   - `onError` is called exactly once per error event (not on every render).
 *   - Retry attempts are capped at MAX_RETRIES to prevent infinite re-render
 *     loops that would waste resources on unrecoverable errors.
 *   - Non-Error thrown values are normalised in getDerivedStateFromError so
 *     componentDidCatch always receives a proper Error object.
 *
 * Lifecycle:
 *   Error thrown → getDerivedStateFromError (state update) →
 *   componentDidCatch (logging + reporting) → fallback render
 *
 * @custom:security
 *   - Stack traces are suppressed in production to prevent information disclosure.
 *   - The fallback UI uses only static strings; no raw error data is injected
 *     into innerHTML, preventing XSS from crafted error messages.
 *   - The `onError` callback receives a sanitised report; callers must not log
 *     raw `error.stack` in production.
 *
 * @custom:limitations
 *   - Does NOT catch errors in async event handlers, setTimeout, or SSR.
 *   - Does NOT catch errors thrown inside the boundary's own render method.
 *   - Nested boundaries can be used for more granular isolation.
 */
export class FrontendGlobalErrorBoundary extends Component<
  FrontendGlobalErrorBoundaryProps,
  BoundaryState
> {
  constructor(props: FrontendGlobalErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      isSmartContractError: false,
      retryCount: 0,
    };
    this.handleRetry = this.handleRetry.bind(this);
  }

  // -------------------------------------------------------------------------
  // Static lifecycle
  // -------------------------------------------------------------------------

  /**
   * @dev Updates component state so the next render shows the fallback UI.
   * Called synchronously during the render phase — must be a pure function.
   * Non-Error thrown values are normalised to Error here so downstream code
   * can always rely on a proper Error instance.
   *
   * @param error The value that was thrown (may not be an Error instance).
   * @return Partial state update.
   */
  static getDerivedStateFromError(error: unknown): Partial<BoundaryState> {
    const err =
      error instanceof Error
        ? error
        : new Error(error != null ? String(error) : 'An unexpected error occurred');
    return {
      hasError: true,
      error: err,
      isSmartContractError: isSmartContractError(err),
    };
  }

  // -------------------------------------------------------------------------
  // Instance lifecycle
  // -------------------------------------------------------------------------

  /**
   * @dev Called after an error has been thrown by a descendant component.
   * Responsible for side-effects: logging and external error reporting.
   * Invokes `onError` exactly once per caught error to avoid duplicate
   * reports (gas/cost efficiency for paid observability services).
   *
   * @param error The error that was thrown.
   * @param errorInfo React-provided component stack information.
   */
  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    // Use the normalised Error from state (set by getDerivedStateFromError)
    // rather than the raw thrown value, which may be a non-Error primitive.
    const normalisedError = this.state.error ?? error;
    const isContract = isSmartContractError(normalisedError);
    const report = buildErrorReport(normalisedError, errorInfo, isContract);

    console.error(
      'Documentation Error Boundary caught an error:',
      error,
      errorInfo,
    );

    if (typeof this.props.onError === 'function') {
      this.props.onError(report);
    }
  }

  // -------------------------------------------------------------------------
  // Recovery
  // -------------------------------------------------------------------------

  /**
   * @dev Resets error state so the child tree is re-rendered.
   * Capped at MAX_RETRIES to prevent infinite retry loops on unrecoverable
   * errors — each retry attempt consumes resources (network calls, re-renders).
   */
  handleRetry(): void {
    if (this.state.retryCount >= MAX_RETRIES) return;
    this.setState((prev) => ({
      hasError: false,
      error: null,
      isSmartContractError: false,
      retryCount: prev.retryCount + 1,
    }));
  }

  // -------------------------------------------------------------------------
  // Render
  // -------------------------------------------------------------------------

  render(): ReactNode {
    const { hasError, error, isSmartContractError: isContract, retryCount } =
      this.state;
    const { fallback, children } = this.props;
    const isDev = process.env.NODE_ENV !== 'production';
    const canRetry = retryCount < MAX_RETRIES;

    if (!hasError) {
      return children ?? null;
    }

    if (fallback) {
      return fallback;
    }

    if (isContract) {
      return (
        <div
          role="alert"
          aria-live="assertive"
          className="error-boundary error-boundary--contract"
          style={styles.container}
        >
          <span aria-hidden="true" style={styles.icon}>🔗</span>
          <h2 style={styles.title}>Smart Contract Error</h2>
          <p style={styles.message}>
            A blockchain interaction failed. This may be due to insufficient
            funds, a rejected transaction, or a temporary network issue.
          </p>
          <p style={styles.hint}>
            Check your wallet balance, ensure your wallet is connected, then try
            again.
          </p>
          {isDev && error && (
            <details style={styles.details}>
              <summary>Error Details (dev only)</summary>
              <pre style={styles.pre}>{error.message}</pre>
            </details>
          )}
          <div style={styles.actions}>
            {canRetry && (
              <button
                onClick={this.handleRetry}
                style={styles.primaryButton}
                aria-label="Try Again"
              >
                Try Again
              </button>
            )}
            <button
              onClick={() => { window.location.href = '/'; }}
              style={styles.secondaryButton}
              aria-label="Go Home"
            >
              Go Home
            </button>
          </div>
          {!canRetry && (
            <p style={styles.hint} role="status">
              Maximum retry attempts reached. Please reload the page.
            </p>
          )}
/**
 * @title ContractError
 * @dev Represents errors originating from smart contract execution on Stellar/Soroban.
 * Thrown when a contract invocation fails, returns an unexpected result, or
 * the transaction is rejected by the network.
 *
 * @custom:security Never include raw contract state or private keys in the message.
 */
export class ContractError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ContractError';
  }
}

/**
 * @title NetworkError
 * @dev Represents errors caused by network connectivity issues when communicating
 * with the Stellar Horizon API or RPC endpoints.
 */
export class NetworkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'NetworkError';
  }
}

/**
 * @title TransactionError
 * @dev Represents errors that occur during blockchain transaction submission,
 * signing, or confirmation phases.
 *
 * @custom:security Do not embed transaction XDR or signing keys in the message.
 */
export class TransactionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'TransactionError';
  }
}

// ---------------------------------------------------------------------------
// Error classification helpers
// ---------------------------------------------------------------------------

/** Keywords that indicate a smart-contract / blockchain related error. */
const CONTRACT_KEYWORDS = [
  'contract',
  'stellar',
  'soroban',
  'transaction',
  'blockchain',
  'ledger',
  'horizon',
  'xdr',
  'invoke',
  'wallet',
];

/**
 * @dev Determines whether an error is related to smart contract execution.
 * Result is computed once per error instance and cached on the object to
 * avoid redundant string scans across multiple render cycles (gas efficiency).
 *
 * @param error The error to classify.
 * @return `true` if the error is contract/blockchain related.
 *
 * @custom:security This is a best-effort heuristic. Unknown error types default
 * to the generic handler, which is the safer path.
 */
const _classificationCache = new WeakMap<Error, boolean>();

function isSmartContractError(error: Error): boolean {
  if (_classificationCache.has(error)) {
    return _classificationCache.get(error)!;
  }
  let result: boolean;
  if (
    error instanceof ContractError ||
    error instanceof NetworkError ||
    error instanceof TransactionError
  ) {
    result = true;
  } else {
    const haystack = `${error.name} ${error.message}`.toLowerCase();
    result = CONTRACT_KEYWORDS.some((kw) => haystack.includes(kw));
  }
  _classificationCache.set(error, result);
  return result;
}

// ---------------------------------------------------------------------------
// Structured error report
// ---------------------------------------------------------------------------

export interface ErrorReport {
  message: string;
  stack: string | undefined;
  componentStack: string | null | undefined;
  timestamp: string;
  isSmartContractError: boolean;
  errorName: string;
}

/**
 * @dev Builds a structured, sanitised error report suitable for forwarding to
 * an external observability service (Sentry, Datadog, etc.).
 *
 * @custom:security Stack traces are included only in development mode so that
 * sensitive implementation details are not exposed in production logs.
 */
function buildErrorReport(
  error: Error,
  errorInfo: ErrorInfo,
  isContract: boolean,
): ErrorReport {
  const isDev = process.env.NODE_ENV !== 'production';
  return {
    message: error.message,
    stack: isDev ? error.stack : undefined,
    componentStack: isDev ? errorInfo.componentStack : undefined,
    timestamp: new Date().toISOString(),
    isSmartContractError: isContract,
    errorName: error.name,
  };
}

// ---------------------------------------------------------------------------
// Component types
// ---------------------------------------------------------------------------

/** Maximum number of retry attempts before the retry button is hidden. */
export const MAX_RETRIES = 3;

export interface FrontendGlobalErrorBoundaryProps {
  /**
   * @dev The child component tree to protect with this error boundary.
   */
  children?: ReactNode;

  /**
   * @dev Optional custom fallback UI. When provided it replaces the built-in
   * fallback entirely, giving callers full control over the error presentation.
   */
  fallback?: ReactNode;

  /**
   * @dev Optional callback invoked with a structured error report whenever an
   * error is caught. Use this to forward errors to Sentry, LogRocket, etc.
   *
   * @param report Sanitised error report (stack omitted in production).
   */
  onError?: (report: ErrorReport) => void;
}

interface BoundaryState {
  hasError: boolean;
  error: Error | null;
  isSmartContractError: boolean;
  /** Number of retry attempts made so far. */
  retryCount: number;
}

// ---------------------------------------------------------------------------
// FrontendGlobalErrorBoundary
// ---------------------------------------------------------------------------

/**
 * @title FrontendGlobalErrorBoundary
 * @dev React class-based error boundary for the Stellar Raise frontend.
 *
 * Catches synchronous render-phase errors anywhere in the wrapped component
 * tree, classifies them (generic vs. smart-contract), logs a structured report,
 * and renders an appropriate fallback UI with a "Try Again" recovery path.
 *
 * Gas-efficiency improvements over the previous version:
 *   - Error classification result is cached via WeakMap so repeated renders
 *     do not re-scan the error message string.
 *   - `onError` is called exactly once per error event (not on every render).
 *   - Retry attempts are capped at MAX_RETRIES to prevent infinite re-render
 *     loops that would waste resources on unrecoverable errors.
 *   - Non-Error thrown values are normalised in getDerivedStateFromError so
 *     componentDidCatch always receives a proper Error object.
 *
 * Lifecycle:
 *   Error thrown → getDerivedStateFromError (state update) →
 *   componentDidCatch (logging + reporting) → fallback render
 *
 * @custom:security
 *   - Stack traces are suppressed in production to prevent information disclosure.
 *   - The fallback UI uses only static strings; no raw error data is injected
 *     into innerHTML, preventing XSS from crafted error messages.
 *   - The `onError` callback receives a sanitised report; callers must not log
 *     raw `error.stack` in production.
 *
 * @custom:limitations
 *   - Does NOT catch errors in async event handlers, setTimeout, or SSR.
 *   - Does NOT catch errors thrown inside the boundary's own render method.
 *   - Nested boundaries can be used for more granular isolation.
 */
export class FrontendGlobalErrorBoundary extends Component<
  FrontendGlobalErrorBoundaryProps,
  BoundaryState
> {
  constructor(props: FrontendGlobalErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      isSmartContractError: false,
      retryCount: 0,
    };
    this.handleRetry = this.handleRetry.bind(this);
  }

  // -------------------------------------------------------------------------
  // Static lifecycle
  // -------------------------------------------------------------------------

  /**
   * @dev Updates component state so the next render shows the fallback UI.
   * Called synchronously during the render phase — must be a pure function.
   * Non-Error thrown values are normalised to Error here so downstream code
   * can always rely on a proper Error instance.
   *
   * @param error The value that was thrown (may not be an Error instance).
   * @return Partial state update.
   */
  static getDerivedStateFromError(error: unknown): Partial<BoundaryState> {
    const err =
      error instanceof Error
        ? error
        : new Error(error != null ? String(error) : 'An unexpected error occurred');
    return {
      hasError: true,
      error: err,
      isSmartContractError: isSmartContractError(err),
    };
  }

  // -------------------------------------------------------------------------
  // Instance lifecycle
  // -------------------------------------------------------------------------

  /**
   * @dev Called after an error has been thrown by a descendant component.
   * Responsible for side-effects: logging and external error reporting.
   * Invokes `onError` exactly once per caught error to avoid duplicate
   * reports (gas/cost efficiency for paid observability services).
   *
   * @param error The error that was thrown.
   * @param errorInfo React-provided component stack information.
   */
  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    // Use the normalised Error from state (set by getDerivedStateFromError)
    // rather than the raw thrown value, which may be a non-Error primitive.
    const normalisedError = this.state.error ?? error;
    const isContract = isSmartContractError(normalisedError);
    const report = buildErrorReport(normalisedError, errorInfo, isContract);

    console.error(
      'Documentation Error Boundary caught an error:',
      error,
      errorInfo,
    );

    if (typeof this.props.onError === 'function') {
      this.props.onError(report);
    }
  }

  render(): ReactNode {
    if (this.state.hasError) {
 * Props for the GlobalErrorBoundary component.
 */
interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

/**
 * State for the GlobalErrorBoundary component.
 */
interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
  isSmartContractError: boolean;
}

/**
 * Global Error Boundary for handling React errors and smart contract errors.
 *
 * This component catches JavaScript errors anywhere in the component tree,
 * logs them, and displays a fallback UI instead of crashing the entire app.
 *
 * For smart contract errors, it provides specific handling and user-friendly
 * messages to improve UX when blockchain operations fail.
 *
 * @example
 * ```tsx
 * <GlobalErrorBoundary>
 *   <App />
 * </GlobalErrorBoundary>
 * ```
 */
class GlobalErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false, isSmartContractError: false };
  }

  /**
   * Static method to update state when an error occurs.
   * This is called during the render phase, so it should be pure.
   *
   * @param error - The error that was thrown
   * @returns Updated state with error information
   */
  static getDerivedStateFromError(error: Error): State {
    // Check if this is a smart contract related error
    const isSmartContractError = GlobalErrorBoundary.isSmartContractError(error);

    return {
      hasError: true,
      error,
      isSmartContractError,
    };
  }

  /**
   * Lifecycle method called when an error is caught.
   * Used for side effects like logging errors.
   *
   * @param error - The error that was thrown
   * @param errorInfo - Additional error information from React
   */
  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error for debugging (in production, send to error reporting service)
    console.error('Global Error Boundary caught an error:', error, errorInfo);

    // Update state with error info for potential recovery
    this.setState({
      error,
      errorInfo,
    });

    // In a real app, you might want to send this to an error reporting service
    // like Sentry, LogRocket, or similar
    this.reportError(error, errorInfo);
  }

  /**
   * Determines if an error is related to smart contract operations.
   * Public for testing purposes.
   *
   * @param error - The error to check
   * @returns True if the error is smart contract related
   */
  public static isSmartContractError(error: Error): boolean {
    const errorMessage = error.message.toLowerCase();
    const errorName = error.name.toLowerCase();

    // Common smart contract error patterns
    const smartContractPatterns = [
      'contract',
      'stellar',
      'soroban',
      'transaction',
      'network',
      'blockchain',
      'timeout',
      'insufficient',
      'unauthorized',
      'invalid',
      'overflow',
      'underflow',
    ];

    return (
      smartContractPatterns.some(pattern =>
        errorMessage.includes(pattern) || errorName.includes(pattern)
      ) ||
      // Check for specific error types
      error instanceof ContractError ||
      error instanceof NetworkError ||
      error instanceof TransactionError
    );
  }

  /**
   * Reports the error to external services.
   * In production, this would send to error reporting services.
   *
   * @param error - The error that occurred
   * @param errorInfo - Additional React error info
   */
  private reportError(error: Error, errorInfo: ErrorInfo) {
    // Example: Send to error reporting service
    // In a real implementation, you might use:
    // - Sentry: Sentry.captureException(error, { contexts: { react: errorInfo } })
    // - LogRocket: LogRocket.captureException(error, { extra: errorInfo })
    // - Custom analytics service

    const errorReport = {
      message: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
      timestamp: new Date().toISOString(),
      userAgent: typeof window !== 'undefined' ? window.navigator.userAgent : 'server',
      url: typeof window !== 'undefined' ? window.location.href : 'server',
      isSmartContractError: this.state.isSmartContractError,
    };

    // For now, just log to console. In production, send to service.
    console.error('Error Report:', errorReport);
  }

  /**
   * Attempts to recover from the error by resetting the error state.
   */
  private handleRetry = () => {
    this.setState({
      hasError: false,
      error: undefined,
      errorInfo: undefined,
      isSmartContractError: false,
    });
  };

  /**
   * Renders the error boundary.
   * If there's an error, shows the fallback UI; otherwise renders children.
   */
  render() {
    if (this.state.hasError) {
      // If a custom fallback is provided, use it
      if (this.props.fallback) {
        return this.props.fallback;
      }
  // -------------------------------------------------------------------------
  // Recovery
  // -------------------------------------------------------------------------

  /**
   * @dev Resets error state so the child tree is re-rendered.
   * Capped at MAX_RETRIES to prevent infinite retry loops on unrecoverable
   * errors — each retry attempt consumes resources (network calls, re-renders).
   */
  handleRetry(): void {
    if (this.state.retryCount >= MAX_RETRIES) return;
    this.setState((prev) => ({
      hasError: false,
      error: null,
      isSmartContractError: false,
      retryCount: prev.retryCount + 1,
    }));
  }

  // -------------------------------------------------------------------------
  // Render
  // -------------------------------------------------------------------------

  render(): ReactNode {
    const { hasError, error, isSmartContractError: isContract, retryCount } =
      this.state;
    const { fallback, children } = this.props;
    const isDev = process.env.NODE_ENV !== 'production';
    const canRetry = retryCount < MAX_RETRIES;

    if (!hasError) {
      return children ?? null;
    }

    if (fallback) {
      return fallback;
    }

    if (isContract) {
      return (
        <div
          role="alert"
          aria-live="assertive"
          className="error-boundary error-boundary--contract"
          style={styles.container}
        >
          <span aria-hidden="true" style={styles.icon}>🔗</span>
          <h2 style={styles.title}>Smart Contract Error</h2>
          <p style={styles.message}>
            A blockchain interaction failed. This may be due to insufficient
            funds, a rejected transaction, or a temporary network issue.
          </p>
          <p style={styles.hint}>
            Check your wallet balance, ensure your wallet is connected, then try
            again.
          </p>
          {isDev && error && (
            <details style={styles.details}>
              <summary>Error Details (dev only)</summary>
              <pre style={styles.pre}>{error.message}</pre>
            </details>
          )}
          <div style={styles.actions}>
            {canRetry && (
              <button
                onClick={this.handleRetry}
                style={styles.primaryButton}
                aria-label="Try Again"
              >
                Try Again
              </button>
            )}
            <button
              onClick={() => { window.location.href = '/'; }}
              style={styles.secondaryButton}
              aria-label="Go Home"
            >
              Go Home
            </button>
          </div>
          {!canRetry && (
            <p style={styles.hint} role="status">
              Maximum retry attempts reached. Please reload the page.
            </p>
          )}
        </div>
      );
      // Default error UI
      return <ErrorFallback
        error={this.state.error}
        isSmartContractError={this.state.isSmartContractError}
        onRetry={this.handleRetry}
      />;
    }

    return (
      <div
        role="alert"
        aria-live="assertive"
        className="error-boundary error-boundary--generic"
        style={styles.container}
      >
        <span aria-hidden="true" style={styles.icon}>⚠️</span>
        <h2 style={styles.title}>Documentation Loading Error</h2>
        <p style={styles.message}>
          We&apos;re sorry, but the documentation content failed to load due to
          an unexpected error.
        </p>
        {isDev && error && (
          <details style={styles.details}>
            <summary>Error Details (dev only)</summary>
            <pre style={styles.pre}>{error.message}</pre>
          </details>
        )}
        <div style={styles.actions}>
          {canRetry && (
            <button
              onClick={this.handleRetry}
              style={styles.primaryButton}
              aria-label="Try Again"
            >
              Try Again
            </button>
          )}
          <button
            onClick={this.handleRetry}
            style={styles.primaryButton}
            aria-label="Try Again"
          >
            Try Again
          </button>
          <button
            onClick={() => { window.location.href = '/'; }}
            style={styles.secondaryButton}
            aria-label="Go Home"
          >
            Go Home
          </button>
        </div>
        {!canRetry && (
          <p style={styles.hint} role="status">
            Maximum retry attempts reached. Please reload the page.
          </p>
        )}
      </div>
    );
  }
}

// ---------------------------------------------------------------------------
// Inline styles (no CSS dependency — boundary must render even if CSS fails)
// ---------------------------------------------------------------------------

const styles = {
  container: {
    padding: '24px',
    border: '1px solid #ff4d4f',
    borderRadius: '6px',
    backgroundColor: '#fff2f0',
    color: '#cf1322',
    maxWidth: '600px',
    margin: '40px auto',
    fontFamily: 'sans-serif',
  } as React.CSSProperties,
  icon: {
    fontSize: '2rem',
    display: 'block',
    marginBottom: '8px',
  } as React.CSSProperties,
  title: {
    margin: '0 0 8px',
    fontSize: '1.25rem',
    fontWeight: 600,
  } as React.CSSProperties,
  message: {
    margin: '0 0 8px',
    fontSize: '0.95rem',
    color: '#595959',
  } as React.CSSProperties,
  hint: {
    margin: '0 0 12px',
    fontSize: '0.875rem',
    color: '#8c8c8c',
  } as React.CSSProperties,
  details: {
    marginTop: '12px',
    marginBottom: '12px',
    fontSize: '0.8rem',
    color: '#595959',
  } as React.CSSProperties,
  pre: {
    whiteSpace: 'pre-wrap' as const,
    wordBreak: 'break-word' as const,
    background: '#f5f5f5',
    padding: '8px',
    borderRadius: '4px',
    fontSize: '0.75rem',
  } as React.CSSProperties,
  actions: {
    display: 'flex',
    gap: '12px',
    marginTop: '16px',
    flexWrap: 'wrap' as const,
  } as React.CSSProperties,
  primaryButton: {
    padding: '8px 18px',
    cursor: 'pointer',
    backgroundColor: '#cf1322',
    color: '#fff',
    border: 'none',
    borderRadius: '4px',
    fontSize: '0.9rem',
  } as React.CSSProperties,
  secondaryButton: {
    padding: '8px 18px',
    cursor: 'pointer',
    backgroundColor: '#fff',
    color: '#374151',
    border: '1px solid #d1d5db',
    borderRadius: '4px',
    fontSize: '0.9rem',
  } as React.CSSProperties,
};

export default FrontendGlobalErrorBoundary;
    return this.props.children;
      </div>
    );
  }
}

/**
 * Props for the ErrorFallback component.
 */
interface ErrorFallbackProps {
  error?: Error;
  isSmartContractError: boolean;
  onRetry: () => void;
}

/**
 * Default error fallback UI component.
 * Displays user-friendly error messages and recovery options.
 */
const ErrorFallback: React.FC<ErrorFallbackProps> = ({
  error,
  isSmartContractError,
  onRetry,
}) => {
  const getErrorMessage = () => {
    if (isSmartContractError) {
      return {
        title: 'Smart Contract Error',
        message: 'There was an issue with the blockchain transaction. This might be due to network congestion, insufficient funds, or a temporary service issue.',
        icon: '🔗',
      };
    }

    return {
      title: 'Something went wrong',
      message: 'An unexpected error occurred. Our team has been notified and is working to fix this issue.',
      icon: '⚠️',
    };
  };

  const { title, message, icon } = getErrorMessage();

  return (
    <div style={styles.container}>
      <div style={styles.content}>
        <div style={styles.icon}>{icon}</div>
        <h1 style={styles.title}>{title}</h1>
        <p style={styles.message}>{message}</p>

        {process.env.NODE_ENV === 'development' && error && (
          <details style={styles.errorDetails}>
            <summary style={styles.errorSummary}>Error Details (Development)</summary>
            <pre style={styles.errorText}>
              {error.name}: {error.message}
              {error.stack && `\n\n${error.stack}`}
            </pre>
          </details>
        )}

        <div style={styles.actions}>
          <button style={styles.primaryButton} onClick={onRetry}>
            Try Again
          </button>
          <button
            style={styles.secondaryButton}
            onClick={() => window.location.href = '/'}
          >
            Go Home
          </button>
        </div>
      </div>
    </div>
  );
};

/**
 * Custom error classes for better error categorization.
 */
export class ContractError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'ContractError';
  }
}

export class NetworkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'NetworkError';
  }
}

export class TransactionError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'TransactionError';
  }
}

/**
 * Styles for the error boundary components.
 */
const styles = {
  container: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    minHeight: '100vh',
    backgroundColor: '#f9fafb',
    fontFamily: 'system-ui, -apple-system, sans-serif',
    padding: '1rem',
  },
  content: {
    textAlign: 'center' as const,
    maxWidth: '500px',
    padding: '2rem',
    backgroundColor: 'white',
    borderRadius: '8px',
    boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
  },
  icon: {
    fontSize: '3rem',
    marginBottom: '1rem',
  },
  title: {
    fontSize: '1.5rem',
    fontWeight: 'bold',
    color: '#1f2937',
    margin: '0 0 1rem 0',
  },
  message: {
    color: '#6b7280',
    margin: '0 0 1.5rem 0',
    lineHeight: '1.5',
  },
  errorDetails: {
    textAlign: 'left' as const,
    margin: '1rem 0',
    border: '1px solid #e5e7eb',
    borderRadius: '4px',
  },
  errorSummary: {
    padding: '0.5rem',
    cursor: 'pointer',
    backgroundColor: '#f3f4f6',
    borderRadius: '4px 4px 0 0',
  },
  errorText: {
    padding: '0.5rem',
    backgroundColor: '#f9fafb',
    borderRadius: '0 0 4px 4px',
    fontSize: '0.875rem',
    whiteSpace: 'pre-wrap' as const,
    wordBreak: 'break-word' as const,
    color: '#dc2626',
  },
  actions: {
    display: 'flex',
    gap: '0.5rem',
    justifyContent: 'center',
    flexWrap: 'wrap' as const,
  },
  primaryButton: {
    padding: '0.5rem 1rem',
    backgroundColor: '#3b82f6',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '0.875rem',
    fontWeight: '500',
  },
  secondaryButton: {
    padding: '0.5rem 1rem',
    backgroundColor: 'white',
    color: '#6b7280',
    border: '1px solid #d1d5db',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '0.875rem',
    fontWeight: '500',
  },
};

export default GlobalErrorBoundary;
// ---------------------------------------------------------------------------
// Inline styles (no CSS dependency — boundary must render even if CSS fails)
// ---------------------------------------------------------------------------

const styles = {
  container: {
    padding: '24px',
    border: '1px solid #ff4d4f',
    borderRadius: '6px',
    backgroundColor: '#fff2f0',
    color: '#cf1322',
    maxWidth: '600px',
    margin: '40px auto',
    fontFamily: 'sans-serif',
  } as React.CSSProperties,
  icon: {
    fontSize: '2rem',
    display: 'block',
    marginBottom: '8px',
  } as React.CSSProperties,
  title: {
    margin: '0 0 8px',
    fontSize: '1.25rem',
    fontWeight: 600,
  } as React.CSSProperties,
  message: {
    margin: '0 0 8px',
    fontSize: '0.95rem',
    color: '#595959',
  } as React.CSSProperties,
  hint: {
    margin: '0 0 12px',
    fontSize: '0.875rem',
    color: '#8c8c8c',
  } as React.CSSProperties,
  details: {
    marginTop: '12px',
    marginBottom: '12px',
    fontSize: '0.8rem',
    color: '#595959',
  } as React.CSSProperties,
  pre: {
    whiteSpace: 'pre-wrap' as const,
    wordBreak: 'break-word' as const,
    background: '#f5f5f5',
    padding: '8px',
    borderRadius: '4px',
    fontSize: '0.75rem',
  } as React.CSSProperties,
  actions: {
    display: 'flex',
    gap: '12px',
    marginTop: '16px',
    flexWrap: 'wrap' as const,
  } as React.CSSProperties,
  primaryButton: {
    padding: '8px 18px',
    cursor: 'pointer',
    backgroundColor: '#cf1322',
    color: '#fff',
    border: 'none',
    borderRadius: '4px',
    fontSize: '0.9rem',
  } as React.CSSProperties,
  secondaryButton: {
    padding: '8px 18px',
    cursor: 'pointer',
    backgroundColor: '#fff',
    color: '#374151',
    border: '1px solid #d1d5db',
    borderRadius: '4px',
    fontSize: '0.9rem',
  } as React.CSSProperties,
};

export default FrontendGlobalErrorBoundary;
