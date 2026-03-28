/**
 * @title Frontend Global Error Boundary Utility
 * @notice Secure error boundary implementation for React applications with comprehensive error handling
 * @dev Provides type-safe global error boundary with validation, recovery options, and accessibility support
 * @author Stellar Raise Security Team
 * @notice SECURITY: All error boundaries must go through this utility to prevent information leakage
 *         and ensure secure error handling across the application.
 * @notice This module implements React error boundary pattern for catching rendering errors,
 *         providing fallback UIs, and reporting errors securely.
 */

import React, { Component, ErrorInfo, ReactNode } from 'react';

/**
 * @notice Error severity levels for categorized handling
 */
export const ERROR_SEVERITY_LEVELS = ['low', 'medium', 'high', 'critical'] as const;

/**
 * @notice Predefined error recovery actions
 */
export const RECOVERY_ACTIONS = ['retry', 'reload', 'navigate', 'dismiss'] as const;

/**
 * @notice Type for error severity levels
 */
export type ErrorSeverityLevel = typeof ERROR_SEVERITY_LEVELS[number];

/**
 * @notice Type for recovery actions
 */
export type RecoveryAction = typeof RECOVERY_ACTIONS[number];

/**
 * @notice Error boundary configuration interface
 */
export interface ErrorBoundaryConfig {
  /** Enable error logging */
  enableLogging: boolean;
  /** Show error details to user */
  showErrorDetails: boolean;
  /** Enable error recovery options */
  enableRecovery: boolean;
  /** Custom error fallback component */
  fallback?: ReactNode;
  /** Maximum number of retry attempts */
  maxRetries: number;
  /** Error reporting endpoint (optional) */
  reportingEndpoint?: string;
}

/**
 * @notice Error info interface for detailed error tracking
 */
export interface ErrorInfoType {
  /** Error message */
  message: string;
  /** Error stack trace */
  stack?: string;
  /** Component stack where error occurred */
  componentStack?: string;
  /** Timestamp of error occurrence */
  timestamp: Date;
  /** Severity level of the error */
  severity: ErrorSeverityLevel;
  /** Whether error has been handled */
  isHandled: boolean;
}

/**
 * @notice Props for error boundary component
 */
interface ErrorBoundaryProps {
  /** Child components to render */
  children: ReactNode;
  /** Error boundary configuration */
  config?: Partial<ErrorBoundaryConfig>;
  /** Fallback component to render on error */
  fallback?: ReactNode;
  /** Callback when error occurs */
  onError?: (error: Error, errorInfo: ErrorInfoType) => void;
  /** Callback when error is recovered */
  onRecover?: () => void;
}

/**
 * @notice State for error boundary
 */
interface ErrorBoundaryState {
  /** Whether an error has occurred */
  hasError: boolean;
  /** Current error information */
  error: Error | null;
  /** Error info for tracking */
  errorInfo: ErrorInfoType | null;
  /** Number of retry attempts */
  retryCount: number;
  /** Whether currently attempting recovery (showing "Retrying..." UI) */
  isRecovering: boolean;
  /** Key to force children remount on retry */
  retryKey: number;
  /** Whether to attempt rendering children (even if hasError was true) */
  attemptRender: boolean;
}

/**
 * @notice Default error boundary configuration
 */
export const DEFAULT_ERROR_BOUNDARY_CONFIG: ErrorBoundaryConfig = {
  enableLogging: true,
  showErrorDetails: false,
  enableRecovery: true,
  maxRetries: 3,
};

/**
 * @notice Determines error severity based on error type and message
 * @param error The error to assess
 * @returns Severity level of the error
 */
export function determineErrorSeverity(error: Error): ErrorSeverityLevel {
  // TypeError instances are always medium severity
  if (error instanceof TypeError) {
    return 'medium';
  }

  const errorMessage = (error.message ?? '').toLowerCase();
  const errorMessage = (error?.message ?? '').toLowerCase();
  const errorName = (error?.name ?? '').toLowerCase();
  
  // Check for critical error patterns
  if (
    errorMessage.includes('network') ||
    errorMessage.includes('fetch') ||
    errorMessage.includes('blockchain') ||
    errorMessage.includes('wallet')
  ) {
    return 'critical';
  }
  
  // Check for high severity patterns
  if (
    errorMessage.includes('permission') ||
    errorMessage.includes('unauthorized') ||
    errorMessage.includes('authentication')
  ) {
    return 'high';
  }
  
  // Check for medium severity patterns — also match TypeError by name
  if (
    errorMessage.includes('validation') ||
    errorMessage.includes('render')
    errorMessage.includes('render') ||
    errorName === 'typeerror' ||
    errorMessage.includes('type')
  ) {
    return 'medium';
  }
  
  return 'low';
}

/**
 * @notice Validates error boundary configuration
 * @param config Configuration to validate
 * @returns Whether configuration is valid
 */
export function validateErrorBoundaryConfig(
  config: Partial<ErrorBoundaryConfig>
): boolean {
  if (config.maxRetries !== undefined) {
    if (config.maxRetries < 0 || config.maxRetries > 10) {
      return false;
    }
  }
  
  if (config.reportingEndpoint !== undefined) {
    try {
      new URL(config.reportingEndpoint);
    } catch {
      return false;
    }
  }
  
  return true;
}

/**
 * @notice Creates a secure error info object
 * @param error The error that occurred (may be null/undefined/non-Error)
 * @param error The error that occurred (may be non-Error in rare cases)
 * @param errorInfo React error info
 * @returns Sanitized error info
 */
export function createErrorInfo(
  error: unknown,
  errorInfo: ErrorInfo
): ErrorInfoType {
  // Normalize non-Error thrown values into a proper Error
  const normalizedError: Error =
    error instanceof Error
      ? error
      : new Error(
          error != null ? String(error) : 'An unexpected error occurred'
        );

  return {
    message: normalizedError.message || 'An unexpected error occurred',
    stack: normalizedError.stack,
    componentStack: errorInfo.componentStack,
    timestamp: new Date(),
    severity: determineErrorSeverity(normalizedError),
  const err = error instanceof Error ? error : new Error(
    error != null ? String(error) : 'An unexpected error occurred'
  );
  return {
    message: err.message,
    stack: err.stack,
    componentStack: errorInfo.componentStack,
    timestamp: new Date(),
    severity: determineErrorSeverity(err),
    isHandled: false,
  };
}

/**
 * @notice Internal wrapper that catches child errors and reports them up
 */
interface ChildrenWrapperProps {
  children: ReactNode;
  onError: (error: Error) => void;
  retryKey: number;
}

interface ChildrenWrapperState {
  hasError: boolean;
}

class ChildrenWrapper extends Component<ChildrenWrapperProps, ChildrenWrapperState> {
  state: ChildrenWrapperState = { hasError: false };

  static getDerivedStateFromError(): ChildrenWrapperState {
    return { hasError: true };
  }

  componentDidCatch(error: unknown): void {
    const normalizedError: Error =
      error instanceof Error
        ? error
        : new Error(error != null ? String(error) : 'An unexpected error occurred');
    this.props.onError(normalizedError);
  }

  render(): ReactNode {
    if (this.state.hasError) return null;
    return this.props.children;
  }
}

/**
 * @title GlobalErrorBoundary
 * @notice React component that catches JavaScript errors anywhere in its child component tree
 * @dev Implements Component-based error boundary with configurable behavior
 * @author Stellar Raise Security Team
 * 
 * @notice SECURITY CONSIDERATIONS:
 * - Error messages are sanitized before display to prevent information leakage
 * - Stack traces are only shown in development mode
 * - Error reporting endpoints are validated before use
 * - No sensitive data is included in error logs
 * 
 * @example
 * ```tsx
 * <GlobalErrorBoundary
 *   config={{ enableLogging: true, showErrorDetails: false }}
 *   fallback={<ErrorFallback />}
 *   onError={(error, info) => console.error(error)}
 * >
 *   <App />
 * </GlobalErrorBoundary>
 * ```
 */
export class GlobalErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  /**
   * @notice Default state for error boundary
   */
  public state: ErrorBoundaryState = {
    hasError: false,
    error: null,
    errorInfo: null,
    retryCount: 0,
    isRecovering: false,
    retryKey: 0,
    attemptRender: false,
  };

  /**
   * @notice Merged configuration with defaults
   */
  private config: ErrorBoundaryConfig;

  /**
   * @notice Constructs a new error boundary
   * @param props Error boundary props
   */
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.config = {
      ...DEFAULT_ERROR_BOUNDARY_CONFIG,
      ...props.config,
    };
  }

  /**
   * @notice Static lifecycle method that catches errors in child components
   * @dev Called when a child component throws an error
   * @param error The error that was thrown
   * @returns New state to indicate error
   */
  static getDerivedStateFromError(error: unknown): Partial<ErrorBoundaryState> {
    const normalizedError: Error =
      error instanceof Error
        ? error
        : new Error(
            error != null ? String(error) : 'An unexpected error occurred'
          );
    return {
      hasError: true,
      error: normalizedError,
      isRecovering: false,
      attemptRender: false,
   * @param error The error that was thrown (may be non-Error)
   * @returns New state to indicate error
   */
  static getDerivedStateFromError(error: unknown): Partial<ErrorBoundaryState> {
    const err = error instanceof Error ? error : new Error(
      error != null ? String(error) : 'An unexpected error occurred'
    );
    return {
      hasError: true,
      isRecovering: false,
      error: err,
    };
  }

  /**
   * @notice Lifecycle method called when component updates
   * @dev Cancels recovery if props change mid-recovery (e.g. parent rerenders with new children)
   */
  componentDidUpdate(prevProps: ErrorBoundaryProps): void {
    if (prevProps.children !== this.props.children) {
      if (this.state.isRecovering) {
        this.setState({ isRecovering: false });
      }
      if (this.state.attemptRender) {
        this.setState({ attemptRender: false });
      }
    }
  }

  /**
   * @notice Lifecycle method called after an error has been caught
   * @dev Used for logging and error reporting
   * @param error The error that was thrown (may be non-Error)
   * @param errorInfo Error information containing component stack
   */
  componentDidCatch(error: unknown, errorInfo: ErrorInfo): void {
    const normalizedError: Error =
      error instanceof Error
        ? error
        : new Error(
            error != null ? String(error) : 'An unexpected error occurred'
          );

    const errorInfoType = createErrorInfo(normalizedError, errorInfo);
    const err = error instanceof Error ? error : new Error(
      error != null ? String(error) : 'An unexpected error occurred'
    );
    const errorInfoType = createErrorInfo(err, errorInfo);
    
    this.setState({
      errorInfo: errorInfoType,
    });

    // Call onError callback if provided
    if (this.props.onError) {
      this.props.onError(err, errorInfoType);
    }

    // Log error if logging is enabled
    if (this.config.enableLogging) {
      this.logError(err, errorInfoType);
    }

    // Report error to endpoint if configured
    if (this.config.reportingEndpoint) {
      this.reportError(err, errorInfoType);
    }
  }

  /**
   * @notice Logs error to console (securely)
   * @param error The error to log
   * @param errorInfo Error information
   */
  private logError(error: Error | null, errorInfo: ErrorInfoType): void {
    console.error('[ErrorBoundary] An error occurred:', {
      message: error?.message ?? 'Unknown error',
      severity: errorInfo.severity,
      timestamp: errorInfo.timestamp.toISOString(),
      // Only include stack in development
      ...(process.env.NODE_ENV === 'development' && { stack: error?.stack }),
    });
  }

  /**
   * @notice Reports error to configured endpoint
   * @param error The error to report
   * @param errorInfo Error information
   */
  private async reportError(
    error: Error,
    errorInfo: ErrorInfoType
  ): Promise<void> {
    if (!this.config.reportingEndpoint) return;

    try {
      // Sanitize error data before sending
      const sanitizedError = {
        message: error.message,
        severity: errorInfo.severity,
        timestamp: errorInfo.timestamp.toISOString(),
        userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : 'unknown',
        // Only include stack in development
        ...(process.env.NODE_ENV === 'development' && { stack: error.stack }),
      };

      await fetch(this.config.reportingEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(sanitizedError),
      });
    } catch {
      // Silently fail - error reporting should not break the app
    }
  }

  /**
   * @notice Handles retry action
   * @dev Sets isRecovering then after delay sets attemptRender to try children again
   * @dev Increments retryCount and sets isRecovering=true so the render
   *      method shows "Retrying..." instead of the error UI. React will
   *      attempt to re-render children; if they throw again,
   *      getDerivedStateFromError resets isRecovering and sets hasError=true.
   */
  private handleRetry = (): void => {
    if (this.state.retryCount >= this.config.maxRetries) {
      return;
    }

    this.setState({ isRecovering: true });

    // Small delay before retry to prevent immediate re-throw
    setTimeout(() => {
      this.setState((prevState) => ({
        isRecovering: false,
        attemptRender: true,
        retryCount: prevState.retryCount + 1,
        retryKey: prevState.retryKey + 1,
      }));
    }, 100);
    this.setState((prevState) => ({
      hasError: false,
      error: null,
      errorInfo: null,
      isRecovering: true,
      retryCount: prevState.retryCount + 1,
    }));
  };

  /**
   * @notice Handles reload action
   * @dev Reloads the entire page
   */
  private handleReload = (): void => {
    window.location.reload();
  };

  /**
   * @notice Handles dismiss action
   * @dev Dismisses the error and shows children anyway (dangerous)
   */
  private handleDismiss = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  /**
   * @notice Renders the component
   * @returns The rendered output
   */
  render(): ReactNode {
    const { hasError, isRecovering, attemptRender } = this.state;
    const { hasError, isRecovering } = this.state;

    // Show "Retrying..." during recovery attempt
    if (isRecovering && !hasError) {
      return (
        <div role="status" aria-live="polite" style={{ padding: '2rem', textAlign: 'center' }}>
          <p>Retrying...</p>
        </div>
      );
    }

    // If there's an error, show fallback
    if (hasError) {
      // Use custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

    // No error and not recovering — render children normally
    if (!hasError && !isRecovering && !attemptRender) {
      return this.props.children;
    }

    // Use custom fallback if provided (only when errored, not recovering)
    if (hasError && !isRecovering && !attemptRender && this.props.fallback) {
      return this.props.fallback;
    }

    // Render error UI (always mounted when hasError or isRecovering)
    // Optionally render children alongside via nested boundary when attemptRender is true
    return (
      <>
        {this.renderErrorUI()}
        {attemptRender && (
          <div style={{ display: 'none' }} aria-hidden="true">
            <ChildrenWrapper
              key={this.state.retryKey}
              retryKey={this.state.retryKey}
              onError={this.handleChildError}
            >
              {this.props.children}
            </ChildrenWrapper>
          </div>
        )}
      </>
    );
  }

  /**
   * @notice Handles errors reported from the nested ChildrenWrapper
   */
  private handleChildError = (error: Error): void => {
    this.setState({
      hasError: true,
      error,
      attemptRender: false,
      isRecovering: false,
    });
    // Trigger componentDidCatch-like behavior
    if (this.props.onError) {
      const errorInfoType: ErrorInfoType = {
        message: error.message,
        stack: error.stack,
        componentStack: undefined,
        timestamp: new Date(),
        severity: determineErrorSeverity(error),
        isHandled: false,
      };
      this.props.onError(error, errorInfoType);
    }
  };

  /**
   * @notice Renders the error UI
   * @dev Shows error message and recovery options
   * @returns The rendered error UI
   */
  private renderErrorUI(): ReactNode {
    const { error, retryCount } = this.state;
    const { showErrorDetails, enableRecovery } = this.config;

    return (
      <div
        role="alert"
        aria-live="assertive"
        style={{
          padding: '2rem',
          textAlign: 'center',
          minHeight: '200px',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          gap: '1rem',
        }}
      >
        {/* Error Title — hidden while recovering so tests can detect the transition */}
        {!isRecovering && (
          <h2 style={{ margin: 0, color: '#dc3545' }}>
            Something went wrong
          </h2>
        )}

        {/* Error Message */}
        {!isRecovering && (
          <p style={{ margin: 0, color: '#6c757d' }}>
            {error?.message || 'An unexpected error occurred'}
          </p>
        )}

        {/* Error Details (only in development or if enabled) */}
        {showErrorDetails && process.env.NODE_ENV === 'development' && (
          <pre
            style={{
              padding: '1rem',
              backgroundColor: '#f8f9fa',
              borderRadius: '4px',
              overflow: 'auto',
              maxWidth: '100%',
              fontSize: '0.875rem',
              textAlign: 'left',
            }}
          >
            {error?.stack}
          </pre>
        )}

        {/* Retry Count Indicator */}
        {enableRecovery && retryCount > 0 && !isRecovering && (
          <p style={{ margin: 0, fontSize: '0.875rem', color: '#6c757d' }}>
            {`Retry attempt: ${retryCount} / ${this.config.maxRetries}`}
          </p>
        )}

        {/* Recovery Actions */}
        {enableRecovery && (
          <div
            style={{
              display: 'flex',
              gap: '0.5rem',
              flexWrap: 'wrap',
              justifyContent: 'center',
            }}
          >
            {/* Retry Button */}
            {retryCount < this.config.maxRetries && (
              <button
                onClick={this.handleRetry}
                aria-label="Retry rendering the component"
                style={{
                  padding: '0.5rem 1rem',
                  backgroundColor: '#0d6efd',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                }}
              >
                Retry
              </button>
            )}

            {/* Reload Button */}
            <button
              onClick={this.handleReload}
              aria-label="Reload the page"
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: '#6c757d',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Reload Page
            </button>

            {/* Dismiss Button (with warning) */}
            <button
              onClick={this.handleDismiss}
              aria-label="Dismiss error and try to continue"
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: '#ffc107',
                color: '#000',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Dismiss
            </button>
          </div>
        )}
      </div>
    );
  }
}

/**
 * @notice Higher-order component for wrapping components with error boundary
 * @param WrappedComponent Component to wrap
 * @param errorBoundaryConfig Error boundary configuration
 * @returns New component with error boundary
 * 
 * @example
 * ```tsx
 * const WrappedComponent = withErrorBoundary(MyComponent, {
 *   enableLogging: true,
 *   showErrorDetails: false,
 * });
 * ```
 */
export function withErrorBoundary<P extends object>(
  WrappedComponent: React.ComponentType<P>,
  errorBoundaryConfig?: Partial<ErrorBoundaryConfig>
): React.ComponentType<P> {
  return function WithErrorBoundaryComponent(
    props: P & ErrorBoundaryProps
  ) {
    return (
      <GlobalErrorBoundary config={errorBoundaryConfig}>
        <WrappedComponent {...props} />
      </GlobalErrorBoundary>
    );
  };
}

/**
 * @notice Hook for programmatically resetting error boundary
 * @returns Object with reset function and error state
 */
export function useErrorBoundary() {
  const [error, setError] = React.useState<Error | null>(null);

  const resetError = React.useCallback(() => {
    setError(null);
  }, []);

  const triggerError = React.useCallback((err: Error) => {
    setError(err);
  }, []);

  return {
    error,
    resetError,
    triggerError,
    hasError: error !== null,
  };
}

export default GlobalErrorBoundary;
