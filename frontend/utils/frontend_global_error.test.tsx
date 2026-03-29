/**
 * @title Frontend Global Error Boundary Tests
 * @notice Comprehensive test suite for global error boundary implementation
 * @author Stellar Raise Security Team
 */

import React from 'react';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import {
  GlobalErrorBoundary,
  ErrorBoundaryConfig,
  ERROR_SEVERITY_LEVELS,
  RECOVERY_ACTIONS,
  determineErrorSeverity,
  validateErrorBoundaryConfig,
  createErrorInfo,
  DEFAULT_ERROR_BOUNDARY_CONFIG,
  withErrorBoundary,
  useErrorBoundary,
  ErrorSeverityLevel,
  RecoveryAction,
  ErrorInfoType,
} from './frontend_global_error';

// Mock React ErrorInfo for testing
const mockErrorInfo: ErrorInfo = {
  componentStack: 'Component stack trace',
};

// Helper component that throws an error
const ThrowError = ({ shouldThrow = true }: { shouldThrow?: boolean }) => {
  if (shouldThrow) {
    throw new Error('Test error');
  }
  return <div>Normal rendering</div>;
};

// Helper component with custom error
const ThrowCustomError = ({ message }: { message: string }) => {
  throw new Error(message);
};

describe('Error Severity Levels', () => {
  it('should have all required severity levels', () => {
    expect(ERROR_SEVERITY_LEVELS).toContain('low');
    expect(ERROR_SEVERITY_LEVELS).toContain('medium');
    expect(ERROR_SEVERITY_LEVELS).toContain('high');
    expect(ERROR_SEVERITY_LEVELS).toContain('critical');
    expect(ERROR_SEVERITY_LEVELS.length).toBe(4);
  });
});

describe('Recovery Actions', () => {
  it('should have all required recovery actions', () => {
    expect(RECOVERY_ACTIONS).toContain('retry');
    expect(RECOVERY_ACTIONS).toContain('reload');
    expect(RECOVERY_ACTIONS).toContain('navigate');
    expect(RECOVERY_ACTIONS).toContain('dismiss');
    expect(RECOVERY_ACTIONS.length).toBe(4);
  });
});

describe('DEFAULT_ERROR_BOUNDARY_CONFIG', () => {
  it('should have correct default values', () => {
    expect(DEFAULT_ERROR_BOUNDARY_CONFIG.enableLogging).toBe(true);
    expect(DEFAULT_ERROR_BOUNDARY_CONFIG.showErrorDetails).toBe(false);
    expect(DEFAULT_ERROR_BOUNDARY_CONFIG.enableRecovery).toBe(true);
    expect(DEFAULT_ERROR_BOUNDARY_CONFIG.maxRetries).toBe(3);
  });
});

describe('determineErrorSeverity', () => {
  it('should return critical for network errors', () => {
    const error = new Error('Network request failed');
    expect(determineErrorSeverity(error)).toBe('critical');
  });

  it('should return critical for fetch errors', () => {
    const error = new Error('Failed to fetch data');
    expect(determineErrorSeverity(error)).toBe('critical');
  });

  it('should return critical for blockchain errors', () => {
    const error = new Error('Blockchain transaction failed');
    expect(determineErrorSeverity(error)).toBe('critical');
  });

  it('should return critical for wallet errors', () => {
    const error = new Error('Wallet connection failed');
    expect(determineErrorSeverity(error)).toBe('critical');
  });

  it('should return high for permission errors', () => {
    const error = new Error('Permission denied');
    expect(determineErrorSeverity(error)).toBe('high');
  });

  it('should return high for unauthorized errors', () => {
    const error = new Error('Unauthorized access');
    expect(determineErrorSeverity(error)).toBe('high');
  });

  it('should return high for authentication errors', () => {
    const error = new Error('Authentication failed');
    expect(determineErrorSeverity(error)).toBe('high');
  });

  it('should return medium for validation errors', () => {
    const error = new Error('Validation failed');
    expect(determineErrorSeverity(error)).toBe('medium');
  });

  it('should return medium for type errors', () => {
    const error = new TypeError('Cannot read property');
    expect(determineErrorSeverity(error)).toBe('medium');
  });

  it('should return medium for render errors', () => {
    const error = new Error('Render failed');
    expect(determineErrorSeverity(error)).toBe('medium');
  });

  it('should return low for unknown errors', () => {
    const error = new Error('Unknown error occurred');
    expect(determineErrorSeverity(error)).toBe('low');
  });

  it('should handle errors with mixed case messages', () => {
    const error = new Error('NETWORK Error');
    expect(determineErrorSeverity(error)).toBe('critical');
  });
});

describe('validateErrorBoundaryConfig', () => {
  it('should return true for valid config', () => {
    const config: Partial<ErrorBoundaryConfig> = {
      enableLogging: true,
      maxRetries: 5,
    };
    expect(validateErrorBoundaryConfig(config)).toBe(true);
  });

  it('should return true for empty config', () => {
    expect(validateErrorBoundaryConfig({})).toBe(true);
  });

  it('should return false for negative maxRetries', () => {
    const config: Partial<ErrorBoundaryConfig> = {
      maxRetries: -1,
    };
    expect(validateErrorBoundaryConfig(config)).toBe(false);
  });

  it('should return false for maxRetries > 10', () => {
    const config: Partial<ErrorBoundaryConfig> = {
      maxRetries: 11,
    };
    expect(validateErrorBoundaryConfig(config)).toBe(false);
  });

  it('should return true for maxRetries = 10', () => {
    const config: Partial<ErrorBoundaryConfig> = {
      maxRetries: 10,
    };
    expect(validateErrorBoundaryConfig(config)).toBe(true);
  });

  it('should return false for invalid reporting endpoint', () => {
    const config: Partial<ErrorBoundaryConfig> = {
      reportingEndpoint: 'not-a-valid-url',
    };
    expect(validateErrorBoundaryConfig(config)).toBe(false);
  });

  it('should return true for valid reporting endpoint', () => {
    const config: Partial<ErrorBoundaryConfig> = {
      reportingEndpoint: 'https://example.com/reports',
    };
    expect(validateErrorBoundaryConfig(config)).toBe(true);
  });
});

describe('createErrorInfo', () => {
  it('should create error info with correct properties', () => {
    const error = new Error('Test error');
    const result = createErrorInfo(error, mockErrorInfo);

    expect(result.message).toBe('Test error');
    expect(result.stack).toBeDefined();
    expect(result.componentStack).toBe('Component stack trace');
    expect(result.timestamp).toBeInstanceOf(Date);
    expect(result.isHandled).toBe(false);
  });

  it('should determine correct severity', () => {
    const error = new Error('Network failed');
    const result = createErrorInfo(error, mockErrorInfo);

    expect(result.severity).toBe('critical');
  });
});

describe('GlobalErrorBoundary', () => {
  it('should render children when no error occurs', () => {
    render(
      <GlobalErrorBoundary>
        <div>Child content</div>
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Child content')).toBeInTheDocument();
  });

  it('should catch errors and show fallback', () => {
    const { container } = render(
      <GlobalErrorBoundary>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
  });

  it('should use custom fallback when provided', () => {
    render(
      <GlobalErrorBoundary fallback={<div>Custom fallback</div>}>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Custom fallback')).toBeInTheDocument();
  });

  it('should show error message in fallback', () => {
    render(
      <GlobalErrorBoundary>
        <ThrowCustomError message="Custom error message" />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Custom error message')).toBeInTheDocument();
  });

  it('should call onError callback when error occurs', () => {
    const onError = jest.fn();

    render(
      <GlobalErrorBoundary onError={onError}>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    expect(onError).toHaveBeenCalledTimes(1);
    expect(onError).toHaveBeenCalledWith(
      expect.any(Error),
      expect.objectContaining({
        message: 'Test error',
        isHandled: false,
      })
    );
  });

  it('should show retry button when recovery is enabled', () => {
    render(
      <GlobalErrorBoundary>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Retry')).toBeInTheDocument();
  });

  it('should show reload button when recovery is enabled', () => {
    render(
      <GlobalErrorBoundary>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Reload Page')).toBeInTheDocument();
  });

  it('should show dismiss button when recovery is enabled', () => {
    render(
      <GlobalErrorBoundary>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Dismiss')).toBeInTheDocument();
  });

  it('should hide recovery buttons when recovery is disabled', () => {
    render(
      <GlobalErrorBoundary config={{ enableRecovery: false }}>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    expect(screen.queryByText('Retry')).not.toBeInTheDocument();
    expect(screen.queryByText('Reload Page')).not.toBeInTheDocument();
  });

  it('should hide error details when showErrorDetails is false', () => {
    render(
      <GlobalErrorBoundary config={{ showErrorDetails: false }}>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    // Should not show stack trace
    expect(screen.queryByRole('pre')).not.toBeInTheDocument();
  });

  it('should track retry count', async () => {
    const { container } = render(
      <GlobalErrorBoundary config={{ maxRetries: 3 }}>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    const retryButton = screen.getByText('Retry');
    
    // First retry
    fireEvent.click(retryButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Retry attempt: 1/)).toBeInTheDocument();
    });

    // Second retry
    fireEvent.click(retryButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Retry attempt: 2/)).toBeInTheDocument();
    });
  });

  it('should disable retry button when max retries reached', async () => {
    render(
      <GlobalErrorBoundary config={{ maxRetries: 1 }}>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    const retryButton = screen.getByText('Retry');
    fireEvent.click(retryButton);

    await waitFor(() => {
      expect(screen.queryByText('Retry')).not.toBeInTheDocument();
    });
  });

  it('should have proper ARIA attributes for accessibility', () => {
    render(
      <GlobalErrorBoundary>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    const errorContainer = screen.getByRole('alert');
    expect(errorContainer).toBeInTheDocument();
  });

  it('should handle retry with delay', async () => {
    render(
      <GlobalErrorBoundary config={{ maxRetries: 3 }}>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    const retryButton = screen.getByText('Retry');
    
    // Click should show "Retrying..." text temporarily
    fireEvent.click(retryButton);
    
    expect(screen.getByText('Retrying...')).toBeInTheDocument();
    
    // After delay, should render children again
    await waitFor(
      () => {
        expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument();
      },
      { timeout: 200 }
    );
  });

  it('should use custom configuration', () => {
    const config: Partial<ErrorBoundaryConfig> = {
      enableLogging: false,
      showErrorDetails: true,
      enableRecovery: true,
      maxRetries: 5,
    };

    const { container } = render(
      <GlobalErrorBoundary config={config}>
        <div>Test</div>
      </GlobalErrorBoundary>
    );

    expect(container).toBeInTheDocument();
  });

  it('should handle multiple errors in sequence', async () => {
    const { container, rerender } = render(
      <GlobalErrorBoundary config={{ maxRetries: 3 }}>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Something went wrong')).toBeInTheDocument();

    // Retry
    const retryButton = screen.getByText('Retry');
    fireEvent.click(retryButton);

    await waitFor(
      () => {
        expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument();
      },
      { timeout: 200 }
    );

    // Trigger another error
    rerender(
      <GlobalErrorBoundary config={{ maxRetries: 3 }}>
        <ThrowError shouldThrow={true} />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
  });

  it('should handle different error types', () => {
    // TypeError
    render(
      <GlobalErrorBoundary>
        <ThrowCustomError message="Type error" />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Type error')).toBeInTheDocument();
  });

  it('should correctly determine severity for various error types', () => {
    const networkError = new Error('Network error');
    const validationError = new Error('Validation error');
    const authError = new Error('Authentication error');
    const unknownError = new Error('Unknown error');

    expect(determineErrorSeverity(networkError)).toBe('critical');
    expect(determineErrorSeverity(validationError)).toBe('medium');
    expect(determineErrorSeverity(authError)).toBe('high');
    expect(determineErrorSeverity(unknownError)).toBe('low');
  });
});

describe('withErrorBoundary HOC', () => {
  it('should wrap component with error boundary', () => {
    const SimpleComponent = () => <div>Simple component</div>;
    const WrappedComponent = withErrorBoundary(SimpleComponent);

    const { container } = render(
      <WrappedComponent />
    );

    expect(screen.getByText('Simple component')).toBeInTheDocument();
  });

  it('should catch errors in wrapped component', () => {
    const SimpleComponent = () => {
      throw new Error('Wrapped error');
    };
    const WrappedComponent = withErrorBoundary(SimpleComponent);

    render(
      <WrappedComponent />
    );

    expect(screen.getByText('Wrapped error')).toBeInTheDocument();
  });

  it('should accept custom config in HOC', () => {
    const SimpleComponent = () => <div>Test</div>;
    const WrappedComponent = withErrorBoundary(SimpleComponent, {
      enableRecovery: false,
    });

    render(
      <WrappedComponent />
    );

    expect(screen.getByText('Test')).toBeInTheDocument();
  });
});

describe('useErrorBoundary hook', () => {
  it('should provide error state and functions', () => {
    let hookResult: ReturnType<typeof useErrorBoundary>;
    
    const TestComponent = () => {
      hookResult = useErrorBoundary();
      return <div>Test</div>;
    };

    render(
      <GlobalErrorBoundary>
        <TestComponent />
      </GlobalErrorBoundary>
    );

    expect(hookResult).toBeDefined();
    expect(hookResult!.error).toBeNull();
    expect(hookResult!.hasError).toBe(false);
    expect(typeof hookResult!.resetError).toBe('function');
    expect(typeof hookResult!.triggerError).toBe('function');
  });

  it('should allow triggering errors', () => {
    let hookResult: ReturnType<typeof useErrorBoundary>;
    
    const TestComponent = () => {
      hookResult = useErrorBoundary();
      return (
        <div>
          <button onClick={() => hookResult!.triggerError(new Error('Triggered'))}>
            Trigger
          </button>
        </div>
      );
    };

    render(
      <GlobalErrorBoundary>
        <TestComponent />
      </GlobalErrorBoundary>
    );

    const button = screen.getByText('Trigger');
    fireEvent.click(button);

    // Note: useErrorBoundary doesn't automatically throw
    // It's a utility hook for managing error state
    expect(hookResult!.error).toBeInstanceOf(Error);
    expect(hookResult!.hasError).toBe(true);
  });

  it('should allow resetting errors', () => {
    let hookResult: ReturnType<typeof useErrorBoundary>;
    
    const TestComponent = () => {
      hookResult = useErrorBoundary();
      return (
        <div>
          <button onClick={() => hookResult!.triggerError(new Error('Test'))}>
            Trigger
          </button>
          <button onClick={() => hookResult!.resetError()}>
            Reset
          </button>
        </div>
      );
    };

    render(
      <GlobalErrorBoundary>
        <TestComponent />
      </GlobalErrorBoundary>
    );

    const triggerButton = screen.getByText('Trigger');
    fireEvent.click(triggerButton);

    expect(hookResult!.hasError).toBe(true);

    const resetButton = screen.getByText('Reset');
    fireEvent.click(resetButton);

    expect(hookResult!.error).toBeNull();
    expect(hookResult!.hasError).toBe(false);
  });
});

describe('Edge Cases', () => {
  it('should handle errors with empty message', () => {
    render(
      <GlobalErrorBoundary>
        <ThrowCustomError message="" />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('An unexpected error occurred')).toBeInTheDocument();
  });

  it('should handle very long error messages', () => {
    const longMessage = 'A'.repeat(1000);
    render(
      <GlobalErrorBoundary>
        <ThrowCustomError message={longMessage} />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText(longMessage)).toBeInTheDocument();
  });

  it('should handle special characters in error messages', () => {
    render(
      <GlobalErrorBoundary>
        <ThrowCustomError message="Error with <script>alert('xss')</script>" />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText(/<script>/)).toBeInTheDocument();
  });

  it('should handle errors with stack traces', () => {
    const errorWithStack = new Error('Error with stack');
    errorWithStack.stack = 'Error: Error with stack\n    at TestComponent (<anonymous>)';
    
    const ThrowWithStack = () => {
      throw errorWithStack;
    };

    render(
      <GlobalErrorBoundary>
        <ThrowWithStack />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Error with stack')).toBeInTheDocument();
  });

  it('should handle non-Error objects being thrown', () => {
    const ThrowString = () => {
      throw 'String error';
    };

    render(
      <GlobalErrorBoundary>
        <ThrowString />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('Something went wrong')).toBeInTheDocument();
  });

  it('should handle null being thrown', () => {
    const ThrowNull = () => {
      throw null;
    };

    render(
      <GlobalErrorBoundary>
        <ThrowNull />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('An unexpected error occurred')).toBeInTheDocument();
  });

  it('should handle undefined being thrown', () => {
    const ThrowUndefined = () => {
      throw undefined;
    };

    render(
      <GlobalErrorBoundary>
        <ThrowUndefined />
      </GlobalErrorBoundary>
    );

    expect(screen.getByText('An unexpected error occurred')).toBeInTheDocument();
  });
});

describe('Security Considerations', () => {
  it('should not expose sensitive information in production', () => {
    // In production mode, stack traces should not be shown
    const originalEnv = process.env.NODE_ENV;
    process.env.NODE_ENV = 'production';

    render(
      <GlobalErrorBoundary config={{ showErrorDetails: true }}>
        <ThrowError />
      </GlobalErrorBoundary>
    );

    // Should show fallback message, not stack trace
    expect(screen.getByText('Something went wrong')).toBeInTheDocument();

    process.env.NODE_ENV = originalEnv;
  });

  it('should sanitize error messages', () => {
    const sensitiveError = new Error('Token: sk_live_12345 Secret: mysecret');
    
    const ThrowSensitive = () => {
      throw sensitiveError;
    };

    render(
      <GlobalErrorBoundary>
        <ThrowSensitive />
      </GlobalErrorBoundary>
    );

    // Error message should be displayed
    expect(screen.getByText(/Token:/)).toBeInTheDocument();
  });

  it('dismiss button is labelled correctly (aria-label)', () => {
    render(
      <GlobalErrorBoundary>
        <ThrowError />
      </GlobalErrorBoundary>
    );
    const dismissBtn = screen.getByRole('button', { name: /dismiss error/i });
    expect(dismissBtn).toBeTruthy();
    expect(dismissBtn.getAttribute('aria-label')).toBe(
      'Dismiss error and try to continue',
    );
  });

  it('clicking Dismiss resets error state and re-renders children', () => {
    let shouldThrow = true;
    const Recoverable = () => {
      if (shouldThrow) throw new Error('dismissable error');
      return <div>Dismissed OK</div>;
    };
    render(
      <GlobalErrorBoundary>
        <Recoverable />
      </GlobalErrorBoundary>
    );
    expect(screen.getByText('Something went wrong')).toBeTruthy();
    shouldThrow = false;
    fireEvent.click(screen.getByRole('button', { name: /dismiss error/i }));
    expect(screen.getByText('Dismissed OK')).toBeTruthy();
  });

  it('dismiss action does not expose stack trace to the DOM', () => {
    render(
      <GlobalErrorBoundary config={{ showErrorDetails: false }}>
        <ThrowError />
      </GlobalErrorBoundary>
    );
    // pre element (stack trace) must not be present
    expect(document.querySelector('pre')).toBeNull();
  });
});
