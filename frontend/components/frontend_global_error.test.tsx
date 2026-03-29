import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import {
  FrontendGlobalErrorBoundary,
  ContractError,
  NetworkError,
  TransactionError,
  ErrorReport,
  MAX_RETRIES,
} from './frontend_global_error';

const originalConsoleError = console.error;
beforeAll(() => { console.error = jest.fn(); });
afterAll(() => { console.error = originalConsoleError; });
beforeEach(() => { jest.clearAllMocks(); });

const Throw = ({ error }: { error: Error }) => { throw error; };

// ---------------------------------------------------------------------------
// Custom error classes
// ---------------------------------------------------------------------------

describe('Custom error classes', () => {
  it('ContractError has correct name and extends Error', () => {
    const e = new ContractError('bad contract');
    expect(e.name).toBe('ContractError');
    expect(e.message).toBe('bad contract');
    expect(e).toBeInstanceOf(Error);
  });
  it('NetworkError has correct name', () => {
    const e = new NetworkError('timeout');
    expect(e.name).toBe('NetworkError');
    expect(e).toBeInstanceOf(Error);
  });
  it('TransactionError has correct name', () => {
    const e = new TransactionError('rejected');
    expect(e.name).toBe('TransactionError');
    expect(e).toBeInstanceOf(Error);
  });
});

// ---------------------------------------------------------------------------
// Normal rendering (no error)
// ---------------------------------------------------------------------------

describe('Normal rendering (no error)', () => {
  it('renders children when no error is thrown', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <div data-testid="child">Safe Content</div>
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByTestId('child')).toBeTruthy();
    expect(screen.getByText('Safe Content')).toBeTruthy();
  });
  it('renders null when children is omitted', () => {
    const { container } = render(<FrontendGlobalErrorBoundary />);
    expect(container.firstChild).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Generic error fallback
// ---------------------------------------------------------------------------

describe('Generic error fallback', () => {
  it('renders the default fallback UI on error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('Simulated crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('alert')).toBeTruthy();
    expect(screen.getByText('Documentation Loading Error')).toBeTruthy();
  });
  it('shows the "Try Again" button', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('button', { name: 'Try Again' })).toBeTruthy();
  });
  it('shows the "Go Home" button', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('button', { name: 'Go Home' })).toBeTruthy();
  });
  it('calls console.error with the caught error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('Simulated documentation crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(console.error).toHaveBeenCalledWith(
      'Documentation Error Boundary caught an error:',
      expect.any(Error),
      expect.objectContaining({ componentStack: expect.any(String) }),
    );
  });
  it('has role="alert" and aria-live="assertive"', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('alert').getAttribute('aria-live')).toBe('assertive');
  });
});

// ---------------------------------------------------------------------------
// Smart contract error fallback
// ---------------------------------------------------------------------------

describe('Smart contract error fallback', () => {
  const contractErrors: Array<[string, Error]> = [
    ['ContractError instance', new ContractError('contract call failed')],
    ['NetworkError instance', new NetworkError('horizon timeout')],
    ['TransactionError instance', new TransactionError('tx rejected')],
    ['stellar keyword', new Error('stellar network error')],
    ['soroban keyword', new Error('soroban invocation failed')],
    ['transaction keyword', new Error('transaction simulation error')],
    ['blockchain keyword', new Error('blockchain ledger closed')],
    ['wallet keyword', new Error('wallet connection lost')],
    ['xdr keyword', new Error('xdr decode error')],
    ['horizon keyword', new Error('horizon api error')],
  ];

  contractErrors.forEach(([label, err]) => {
    it('shows Smart Contract Error for ' + label, () => {
      render(
        <FrontendGlobalErrorBoundary>
          <Throw error={err} />
        </FrontendGlobalErrorBoundary>,
      );
      expect(screen.getByText('Smart Contract Error')).toBeTruthy();
    });
  });

  it('shows blockchain-specific guidance text', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new ContractError('insufficient funds')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText(/Check your wallet balance/i)).toBeTruthy();
  });

  it('does NOT show Documentation Loading Error for contract errors', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new ContractError('bad call')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.queryByText('Documentation Loading Error')).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Custom fallback prop
// ---------------------------------------------------------------------------

describe('Custom fallback prop', () => {
  it('renders the custom fallback when provided', () => {
    render(
      <FrontendGlobalErrorBoundary fallback={<div data-testid="cf">Custom Error View</div>}>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByTestId('cf')).toBeTruthy();
    expect(screen.getByText('Custom Error View')).toBeTruthy();
  });
  it('does NOT render the default fallback when custom fallback is provided', () => {
    render(
      <FrontendGlobalErrorBoundary fallback={<div>Custom</div>}>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.queryByText('Documentation Loading Error')).toBeNull();
    expect(screen.queryByText('Smart Contract Error')).toBeNull();
  });
  it('custom fallback overrides smart contract fallback too', () => {
    render(
      <FrontendGlobalErrorBoundary fallback={<div data-testid="cf2">My Fallback</div>}>
        <Throw error={new ContractError('bad')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByTestId('cf2')).toBeTruthy();
    expect(screen.queryByText('Smart Contract Error')).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Recovery via Try Again
// ---------------------------------------------------------------------------

describe('Recovery via Try Again', () => {
  it('re-renders children after clicking Try Again when error is resolved', () => {
    let shouldThrow = true;
    const RecoverableComponent = () => {
      if (shouldThrow) throw new Error('Temporary error');
      return <div>Recovered Content</div>;
    };
    render(
      <FrontendGlobalErrorBoundary>
        <RecoverableComponent />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Documentation Loading Error')).toBeTruthy();
    shouldThrow = false;
    fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    expect(screen.getByText('Recovered Content')).toBeTruthy();
    expect(screen.queryByText('Documentation Loading Error')).toBeNull();
  });
  it('shows the fallback again if the child still throws after retry', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent error')} />
      </FrontendGlobalErrorBoundary>,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    expect(screen.getByText('Documentation Loading Error')).toBeTruthy();
  });
  it('recovery works for contract errors too', () => {
    let shouldThrow = true;
    const RecoverableContract = () => {
      if (shouldThrow) throw new ContractError('contract failed');
      return <div>Contract OK</div>;
    };
    render(
      <FrontendGlobalErrorBoundary>
        <RecoverableContract />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Smart Contract Error')).toBeTruthy();
    shouldThrow = false;
    fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    expect(screen.getByText('Contract OK')).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// Retry cap (gas efficiency)
// ---------------------------------------------------------------------------

describe('Retry cap — gas efficiency', () => {
  it('MAX_RETRIES is exported and is a positive integer', () => {
    expect(typeof MAX_RETRIES).toBe('number');
    expect(MAX_RETRIES).toBeGreaterThan(0);
  });

  it('hides Try Again button after MAX_RETRIES exhausted', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent')} />
      </FrontendGlobalErrorBoundary>,
    );
    // Exhaust all retries
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.queryByRole('button', { name: 'Try Again' })).toBeNull();
  });

  it('shows max-retry message after retries exhausted', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent')} />
      </FrontendGlobalErrorBoundary>,
    );
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.getByText(/Maximum retry attempts reached/i)).toBeTruthy();
  });

  it('Go Home button remains visible after retries exhausted', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent')} />
      </FrontendGlobalErrorBoundary>,
    );
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.getByRole('button', { name: 'Go Home' })).toBeTruthy();
  });

  it('retry cap applies to contract errors too', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new ContractError('persistent contract error')} />
      </FrontendGlobalErrorBoundary>,
    );
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.queryByRole('button', { name: 'Try Again' })).toBeNull();
    expect(screen.getByText(/Maximum retry attempts reached/i)).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// Error classification caching (gas efficiency)
// ---------------------------------------------------------------------------

describe('Error classification caching', () => {
  it('classifies the same error instance consistently across multiple renders', () => {
    const err = new ContractError('cached');
    const onError = jest.fn();
    const { unmount } = render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={err} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(true);
    unmount();
    // Re-render with same error instance — classification must be consistent
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={err} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[1][0].isSmartContractError).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Non-Error thrown values (reliability)
// ---------------------------------------------------------------------------

describe('Non-Error thrown values', () => {
  it('handles a thrown string without crashing', () => {
    const ThrowString = () => { throw 'string error'; };
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <ThrowString />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
    expect(screen.getByRole('alert')).toBeTruthy();
  });

  it('handles a thrown null without crashing', () => {
    const ThrowNull = () => { throw null; };
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <ThrowNull />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
    expect(screen.getByRole('alert')).toBeTruthy();
  });

  it('handles a thrown number without crashing', () => {
    const ThrowNumber = () => { throw 42; };
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <ThrowNumber />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
    expect(screen.getByRole('alert')).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// onError callback — called exactly once per error (gas efficiency)
// ---------------------------------------------------------------------------

describe('onError callback', () => {
  it('calls onError with a structured report when an error is caught', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new Error('callback test')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError).toHaveBeenCalledTimes(1);
    const report: ErrorReport = onError.mock.calls[0][0];
    expect(report.message).toBe('callback test');
    expect(report.timestamp).toBeTruthy();
    expect(typeof report.isSmartContractError).toBe('boolean');
    expect(report.errorName).toBe('Error');
  });
  it('sets isSmartContractError=true for ContractError', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new ContractError('bad')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(true);
  });
  it('sets isSmartContractError=false for generic errors', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new Error('generic')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(false);
  });
  it('does not throw if onError is not provided', () => {
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <Throw error={new Error('no callback')} />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
  });
  it('onError is called exactly once per error event, not on every render', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new Error('once')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError).toHaveBeenCalledTimes(1);
  });
});

// ---------------------------------------------------------------------------
// Accessibility
// ---------------------------------------------------------------------------

describe('Accessibility', () => {
  it('fallback container has role alert', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('a11y test')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('alert')).toBeTruthy();
  });
  it('Try Again button has aria-label', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('a11y')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('button', { name: 'Try Again' }).getAttribute('aria-label')).toBe('Try Again');
  });
  it('Go Home button has aria-label', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('a11y')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('button', { name: 'Go Home' }).getAttribute('aria-label')).toBe('Go Home');
  });
  it('icon span is aria-hidden', () => {
    const { container } = render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('icon test')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(container.querySelector('[aria-hidden="true"]')).toBeTruthy();
  });
  it('max-retry status message has role="status"', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent')} />
      </FrontendGlobalErrorBoundary>,
    );
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.getByRole('status')).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// Error classification edge cases
// ---------------------------------------------------------------------------

describe('Error classification edge cases', () => {
  it('classifies NetworkError as smart contract error', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new NetworkError('timeout')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(true);
  });
  it('classifies TransactionError as smart contract error', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new TransactionError('rejected')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(true);
  });
  it('classifies plain Error with invoke keyword as contract error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('invoke failed')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Smart Contract Error')).toBeTruthy();
  });
  it('does not classify a plain TypeError as a contract error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new TypeError('cannot read property')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Documentation Loading Error')).toBeTruthy();
    expect(screen.queryByText('Smart Contract Error')).toBeNull();
  });
  it('handles errors with empty messages gracefully', () => {
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <Throw error={new Error('')} />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
    expect(screen.getByRole('alert')).toBeTruthy();
  });
  it('classifies ledger keyword as contract error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('ledger sequence mismatch')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Smart Contract Error')).toBeTruthy();
  });
import { FrontendGlobalErrorBoundary } from './frontend_global_error';
} from './frontend_global_error';

// Suppress console.error in tests to cleanly handle expected error boundaries output
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import GlobalErrorBoundary, {
  ContractError,
  NetworkError,
  TransactionError,
} from './frontend_global_error';

/**
 * Test suite for GlobalErrorBoundary component.
 *
 * Tests cover:
 * - Rendering children when no error occurs
 * - Catching and displaying React errors
 * - Handling smart contract specific errors
 * - Retry functionality
 * - Error reporting
 * - Custom fallback UI
 */

// Mock console.error to avoid test output pollution
const originalConsoleError = console.error;
beforeAll(() => { console.error = jest.fn(); });
afterAll(() => { console.error = originalConsoleError; });
beforeEach(() => { jest.clearAllMocks(); });

const Throw = ({ error }: { error: Error }) => { throw error; };

const ThrowError = ({ message = "Test error" }) => {
  throw new Error(message);
};

describe('FrontendGlobalErrorBoundary', () => {
// Mock window.location for navigation tests
let mockHref = 'http://localhost/';
const mockLocation = {
  get href() {
    return mockHref;
  },
  set href(value: string) {
    mockHref = value;
  },
  assign: jest.fn(),
  reload: jest.fn(),
};

delete (global as any).window;
(global as any).window = {
  location: mockLocation,
  navigator: {
    userAgent: 'test-user-agent',
  },
};

/**
 * Test component that throws an error for testing error boundary.
 */
const ErrorThrowingComponent: React.FC<{ shouldThrow?: boolean; errorType?: string }> = ({
  shouldThrow = true,
  errorType = 'generic',
}) => {
  if (shouldThrow) {
    switch (errorType) {
      case 'contract':
        throw new ContractError('Smart contract execution failed');
      case 'network':
        throw new NetworkError('Network connection lost');
      case 'transaction':
        throw new TransactionError('Transaction reverted');
      default:
        throw new Error('Something went wrong');
    }
  }
  return <div>No error</div>;
};

/**
 * Test component for custom fallback testing.
 */
const CustomFallback: React.FC = () => <div>Custom Error UI</div>;

describe('GlobalErrorBoundary', () => {
  beforeEach(() => {
    jest.clearAllMocks();
// ---------------------------------------------------------------------------
// Custom error classes
// ---------------------------------------------------------------------------

describe('Custom error classes', () => {
  it('ContractError has correct name and extends Error', () => {
    const e = new ContractError('bad contract');
    expect(e.name).toBe('ContractError');
    expect(e.message).toBe('bad contract');
    expect(e).toBeInstanceOf(Error);
  });
  it('NetworkError has correct name', () => {
    const e = new NetworkError('timeout');
    expect(e.name).toBe('NetworkError');
    expect(e).toBeInstanceOf(Error);
  });
  it('TransactionError has correct name', () => {
    const e = new TransactionError('rejected');
    expect(e.name).toBe('TransactionError');
    expect(e).toBeInstanceOf(Error);
  });
});

// ---------------------------------------------------------------------------
// Normal rendering (no error)
// ---------------------------------------------------------------------------

describe('Normal rendering (no error)', () => {
  it('renders children when no error is thrown', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <div data-testid="child">Safe Content</div>
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByTestId('child')).toBeTruthy();
    expect(screen.getByText('Safe Content')).toBeTruthy();
  });
  it('renders null when children is omitted', () => {
    const { container } = render(<FrontendGlobalErrorBoundary />);
    expect(container.firstChild).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Generic error fallback
// ---------------------------------------------------------------------------

describe('Generic error fallback', () => {
  it('renders the default fallback UI on error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('Simulated crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('alert')).toBeTruthy();
    expect(screen.getByText('Documentation Loading Error')).toBeTruthy();
  });
  it('shows the "Try Again" button', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('button', { name: 'Try Again' })).toBeTruthy();
  });
  it('shows the "Go Home" button', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('button', { name: 'Go Home' })).toBeTruthy();
  });
  it('calls console.error with the caught error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('Simulated documentation crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(console.error).toHaveBeenCalledWith(
      'Documentation Error Boundary caught an error:',
      expect.any(Error),
      expect.objectContaining({ componentStack: expect.any(String) }),
    );
  });
  it('has role="alert" and aria-live="assertive"', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('alert').getAttribute('aria-live')).toBe('assertive');
  });
});

// ---------------------------------------------------------------------------
// Smart contract error fallback
// ---------------------------------------------------------------------------

describe('Smart contract error fallback', () => {
  const contractErrors: Array<[string, Error]> = [
    ['ContractError instance', new ContractError('contract call failed')],
    ['NetworkError instance', new NetworkError('horizon timeout')],
    ['TransactionError instance', new TransactionError('tx rejected')],
    ['stellar keyword', new Error('stellar network error')],
    ['soroban keyword', new Error('soroban invocation failed')],
    ['transaction keyword', new Error('transaction simulation error')],
    ['blockchain keyword', new Error('blockchain ledger closed')],
    ['wallet keyword', new Error('wallet connection lost')],
    ['xdr keyword', new Error('xdr decode error')],
    ['horizon keyword', new Error('horizon api error')],
  ];

  contractErrors.forEach(([label, err]) => {
    it('shows Smart Contract Error for ' + label, () => {
      render(
        <FrontendGlobalErrorBoundary>
          <Throw error={err} />
        </FrontendGlobalErrorBoundary>,
      );
      expect(screen.getByText('Smart Contract Error')).toBeTruthy();
    });
  });

  it('shows blockchain-specific guidance text', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new ContractError('insufficient funds')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText(/Check your wallet balance/i)).toBeTruthy();
  });

  it('does NOT show Documentation Loading Error for contract errors', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new ContractError('bad call')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.queryByText('Documentation Loading Error')).toBeNull();
  });
});
  describe('Normal operation', () => {
    test('renders children when no error occurs', () => {
      render(
        <GlobalErrorBoundary>
          <div>Test content</div>
        </GlobalErrorBoundary>
      );

      expect(screen.getByText('Test content')).toBeInTheDocument();
    });

    test('renders multiple children correctly', () => {
      render(
        <GlobalErrorBoundary>
          <div>First child</div>
          <div>Second child</div>
        </GlobalErrorBoundary>
      );

      expect(screen.getByText('First child')).toBeInTheDocument();
      expect(screen.getByText('Second child')).toBeInTheDocument();
    });
  });

  describe('Error handling', () => {
    test('catches and displays generic React errors', () => {
      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent errorType="generic" />
        </GlobalErrorBoundary>
      );

      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
      expect(screen.getByText(/unexpected error occurred/)).toBeInTheDocument();
      expect(screen.getByText('⚠️')).toBeInTheDocument();
    });

    test('identifies and handles smart contract errors', () => {
      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent errorType="contract" />
        </GlobalErrorBoundary>
      );

      expect(screen.getByText('Smart Contract Error')).toBeInTheDocument();
      expect(screen.getByText(/blockchain transaction/)).toBeInTheDocument();
      expect(screen.getByText('🔗')).toBeInTheDocument();
    });

    test('handles network errors appropriately', () => {
      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent errorType="network" />
        </GlobalErrorBoundary>
      );

      expect(screen.getByText('Smart Contract Error')).toBeInTheDocument();
      expect(screen.getByText(/blockchain transaction/)).toBeInTheDocument();
    });

    test('handles transaction errors appropriately', () => {
      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent errorType="transaction" />
        </GlobalErrorBoundary>
      );

      expect(screen.getByText('Smart Contract Error')).toBeInTheDocument();
      expect(screen.getByText(/blockchain transaction/)).toBeInTheDocument();
    });

    test('logs errors to console for debugging', () => {
      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent />
        </GlobalErrorBoundary>
      );

      expect(console.error).toHaveBeenCalledWith(
        'Global Error Boundary caught an error:',
        expect.any(Error),
        expect.any(Object)
      );
    });

    test('creates error report with proper structure', () => {
      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent errorType="contract" />
        </GlobalErrorBoundary>
      );

      expect(console.error).toHaveBeenCalledWith(
        'Error Report:',
        expect.objectContaining({
          message: expect.any(String),
          stack: expect.any(String),
          componentStack: expect.any(String),
          timestamp: expect.any(String),
          userAgent: expect.any(String),
          url: expect.any(String),
          isSmartContractError: true,
        })
      );
    });
  });

  describe('Recovery functionality', () => {
    test('retry button resets error state', async () => {
      const { rerender } = render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent shouldThrow={true} />
        </GlobalErrorBoundary>
      );

      expect(screen.getByText('Something went wrong')).toBeInTheDocument();

      // Click retry button
      fireEvent.click(screen.getByText('Try Again'));

      // The component should re-render and show the error again since we're still throwing
      // But the state should be reset, so if we change the component to not throw, it should work
      expect(screen.getByText('Something went wrong')).toBeInTheDocument();
    });

    test.skip('go home button navigates to home page', () => {
      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent />
        </GlobalErrorBoundary>
      );

      fireEvent.click(screen.getByText('Go Home'));

      expect(mockLocation.href).toBe('/');
    });
  });

  describe('Custom fallback', () => {
    test('uses custom fallback when provided', () => {
      render(
        <GlobalErrorBoundary fallback={<CustomFallback />}>
          <ErrorThrowingComponent />
        </GlobalErrorBoundary>
      );

      expect(screen.getByText('Custom Error UI')).toBeInTheDocument();
      expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument();
    });
  });

  describe('Development mode features', () => {
    const originalNodeEnv = process.env.NODE_ENV;

    beforeEach(() => {
      process.env.NODE_ENV = 'development';
    });

    afterEach(() => {
      process.env.NODE_ENV = originalNodeEnv;
    });

    test('shows error details in development mode', () => {
      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent />
        </GlobalErrorBoundary>
      );

      expect(screen.getByText('Error Details (Development)')).toBeInTheDocument();
      expect(screen.getByText(/Error: Something went wrong/)).toBeInTheDocument();
    });

    test('hides error details in production mode', () => {
      process.env.NODE_ENV = 'production';

      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent />
        </GlobalErrorBoundary>
      );

      expect(screen.queryByText('Error Details (Development)')).not.toBeInTheDocument();
    });
  });

  describe('Error classification', () => {
    test('correctly identifies smart contract error patterns', () => {
      const contractError = new Error('Contract execution failed');
      const networkError = new Error('Network timeout occurred');
      const transactionError = new Error('Transaction reverted');
      const genericError = new Error('Generic error');

      // Test the static method directly
      expect(GlobalErrorBoundary.isSmartContractError(contractError)).toBe(true);
      expect(GlobalErrorBoundary.isSmartContractError(networkError)).toBe(true);
      expect(GlobalErrorBoundary.isSmartContractError(transactionError)).toBe(true);
      expect(GlobalErrorBoundary.isSmartContractError(genericError)).toBe(false);
    });

    test('recognizes custom error classes', () => {
      const contractError = new ContractError('Test');
      const networkError = new NetworkError('Test');
      const transactionError = new TransactionError('Test');

      expect(contractError instanceof ContractError).toBe(true);
      expect(networkError instanceof NetworkError).toBe(true);
      expect(transactionError instanceof TransactionError).toBe(true);
    });
  });

  describe('Accessibility', () => {
    test('error UI is keyboard accessible', () => {
      render(
        <GlobalErrorBoundary>
          <ErrorThrowingComponent />
        </GlobalErrorBoundary>
      );

      const retryButton = screen.getByText('Try Again');
      const homeButton = screen.getByText('Go Home');

      // Check that buttons are focusable
      retryButton.focus();
      expect(document.activeElement).toBe(retryButton);

      homeButton.focus();
      expect(document.activeElement).toBe(homeButton);
    });

// ---------------------------------------------------------------------------
// Custom fallback prop
// ---------------------------------------------------------------------------

describe('Custom fallback prop', () => {
  it('renders the custom fallback when provided', () => {
    render(
      <FrontendGlobalErrorBoundary fallback={<div data-testid="cf">Custom Error View</div>}>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByTestId('cf')).toBeTruthy();
    expect(screen.getByText('Custom Error View')).toBeTruthy();
  });
  it('does NOT render the default fallback when custom fallback is provided', () => {
    render(
      <FrontendGlobalErrorBoundary fallback={<div>Custom</div>}>
        <Throw error={new Error('crash')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.queryByText('Documentation Loading Error')).toBeNull();
    expect(screen.queryByText('Smart Contract Error')).toBeNull();
  });
  it('custom fallback overrides smart contract fallback too', () => {
    render(
      <FrontendGlobalErrorBoundary fallback={<div data-testid="cf2">My Fallback</div>}>
        <Throw error={new ContractError('bad')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByTestId('cf2')).toBeTruthy();
    expect(screen.queryByText('Smart Contract Error')).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// Recovery via Try Again
// ---------------------------------------------------------------------------

describe('Recovery via Try Again', () => {
  it('re-renders children after clicking Try Again when error is resolved', () => {
    let shouldThrow = true;
    const RecoverableComponent = () => {
      if (shouldThrow) throw new Error('Temporary error');
      return <div>Recovered Content</div>;
    };
    render(
      <FrontendGlobalErrorBoundary>
        <RecoverableComponent />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Documentation Loading Error')).toBeTruthy();
    shouldThrow = false;
    fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    expect(screen.getByText('Recovered Content')).toBeTruthy();
    expect(screen.queryByText('Documentation Loading Error')).toBeNull();
  });
  it('shows the fallback again if the child still throws after retry', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent error')} />
      </FrontendGlobalErrorBoundary>,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    expect(screen.getByText('Documentation Loading Error')).toBeTruthy();
  });
  it('recovery works for contract errors too', () => {
    let shouldThrow = true;
    const RecoverableContract = () => {
      if (shouldThrow) throw new ContractError('contract failed');
      return <div>Contract OK</div>;
    };
    render(
      <FrontendGlobalErrorBoundary>
        <RecoverableContract />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Smart Contract Error')).toBeTruthy();
    shouldThrow = false;
    fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    expect(screen.getByText('Contract OK')).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// Retry cap (gas efficiency)
// ---------------------------------------------------------------------------

describe('Retry cap — gas efficiency', () => {
  it('MAX_RETRIES is exported and is a positive integer', () => {
    expect(typeof MAX_RETRIES).toBe('number');
    expect(MAX_RETRIES).toBeGreaterThan(0);
  });

  it('hides Try Again button after MAX_RETRIES exhausted', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent')} />
      </FrontendGlobalErrorBoundary>,
    );
    // Exhaust all retries
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.queryByRole('button', { name: 'Try Again' })).toBeNull();
  });

  it('shows max-retry message after retries exhausted', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent')} />
      </FrontendGlobalErrorBoundary>,
    );
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.getByText(/Maximum retry attempts reached/i)).toBeTruthy();
  });

  it('Go Home button remains visible after retries exhausted', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent')} />
      </FrontendGlobalErrorBoundary>,
    );
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.getByRole('button', { name: 'Go Home' })).toBeTruthy();
  });

  it('retry cap applies to contract errors too', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new ContractError('persistent contract error')} />
      </FrontendGlobalErrorBoundary>,
    );
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.queryByRole('button', { name: 'Try Again' })).toBeNull();
    expect(screen.getByText(/Maximum retry attempts reached/i)).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// Error classification caching (gas efficiency)
// ---------------------------------------------------------------------------

describe('Error classification caching', () => {
  it('classifies the same error instance consistently across multiple renders', () => {
    const err = new ContractError('cached');
    const onError = jest.fn();
    const { unmount } = render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={err} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(true);
    unmount();
    // Re-render with same error instance — classification must be consistent
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={err} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[1][0].isSmartContractError).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Non-Error thrown values (reliability)
// ---------------------------------------------------------------------------

describe('Non-Error thrown values', () => {
  it('handles a thrown string without crashing', () => {
    const ThrowString = () => { throw 'string error'; };
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <ThrowString />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
    expect(screen.getByRole('alert')).toBeTruthy();
  });

  it('handles a thrown null without crashing', () => {
    const ThrowNull = () => { throw null; };
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <ThrowNull />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
    expect(screen.getByRole('alert')).toBeTruthy();
  });

  it('handles a thrown number without crashing', () => {
    const ThrowNumber = () => { throw 42; };
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <ThrowNumber />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
    expect(screen.getByRole('alert')).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// onError callback — called exactly once per error (gas efficiency)
// ---------------------------------------------------------------------------

describe('onError callback', () => {
  it('calls onError with a structured report when an error is caught', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new Error('callback test')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError).toHaveBeenCalledTimes(1);
    const report: ErrorReport = onError.mock.calls[0][0];
    expect(report.message).toBe('callback test');
    expect(report.timestamp).toBeTruthy();
    expect(typeof report.isSmartContractError).toBe('boolean');
    expect(report.errorName).toBe('Error');
  });
  it('sets isSmartContractError=true for ContractError', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new ContractError('bad')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(true);
  });
  it('sets isSmartContractError=false for generic errors', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new Error('generic')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(false);
  });
  it('does not throw if onError is not provided', () => {
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <Throw error={new Error('no callback')} />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
  });
  it('onError is called exactly once per error event, not on every render', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new Error('once')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError).toHaveBeenCalledTimes(1);
  });
});

// ---------------------------------------------------------------------------
// Accessibility
// ---------------------------------------------------------------------------

describe('Accessibility', () => {
  it('fallback container has role alert', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('a11y test')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('alert')).toBeTruthy();
  });
  it('Try Again button has aria-label', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('a11y')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('button', { name: 'Try Again' }).getAttribute('aria-label')).toBe('Try Again');
  });
  it('Go Home button has aria-label', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('a11y')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByRole('button', { name: 'Go Home' }).getAttribute('aria-label')).toBe('Go Home');
  });
  it('icon span is aria-hidden', () => {
    const { container } = render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('icon test')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(container.querySelector('[aria-hidden="true"]')).toBeTruthy();
  });
  it('max-retry status message has role="status"', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('persistent')} />
      </FrontendGlobalErrorBoundary>,
    );
    for (let i = 0; i < MAX_RETRIES; i++) {
      fireEvent.click(screen.getByRole('button', { name: 'Try Again' }));
    }
    expect(screen.getByRole('status')).toBeTruthy();
  });
});

// ---------------------------------------------------------------------------
// Error classification edge cases
// ---------------------------------------------------------------------------

describe('Error classification edge cases', () => {
  it('classifies NetworkError as smart contract error', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new NetworkError('timeout')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(true);
  });
  it('classifies TransactionError as smart contract error', () => {
    const onError = jest.fn();
    render(
      <FrontendGlobalErrorBoundary onError={onError}>
        <Throw error={new TransactionError('rejected')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(onError.mock.calls[0][0].isSmartContractError).toBe(true);
  });
  it('classifies plain Error with invoke keyword as contract error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('invoke failed')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Smart Contract Error')).toBeTruthy();
  });
  it('does not classify a plain TypeError as a contract error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new TypeError('cannot read property')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Documentation Loading Error')).toBeTruthy();
    expect(screen.queryByText('Smart Contract Error')).toBeNull();
  });
  it('handles errors with empty messages gracefully', () => {
    expect(() =>
      render(
        <FrontendGlobalErrorBoundary>
          <Throw error={new Error('')} />
        </FrontendGlobalErrorBoundary>,
      ),
    ).not.toThrow();
    expect(screen.getByRole('alert')).toBeTruthy();
  });
  it('classifies ledger keyword as contract error', () => {
    render(
      <FrontendGlobalErrorBoundary>
        <Throw error={new Error('ledger sequence mismatch')} />
      </FrontendGlobalErrorBoundary>,
    );
    expect(screen.getByText('Smart Contract Error')).toBeTruthy();
  });
});
