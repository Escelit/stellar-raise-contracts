# frontend_global_error — Global Error Boundary Component

Technical reference for the React global error boundary component built for the Stellar Raise frontend application.

---

## Overview

`FrontendGlobalErrorBoundary` catches synchronous render-phase errors anywhere in the wrapped component tree, classifies them (generic vs. smart-contract), logs a structured report, and renders an appropriate fallback UI with a capped "Try Again" recovery path.

```
Error thrown → getDerivedStateFromError → componentDidCatch → fallback UI → recovery
The `GlobalErrorBoundary` component provides comprehensive error handling for React applications, with special focus on smart contract and blockchain-related errors. It prevents application crashes by catching JavaScript errors anywhere in the component tree and displaying user-friendly fallback UI.

```
Error thrown → getDerivedStateFromError → componentDidCatch → fallback UI → recovery
```

---

## Gas-Efficiency Improvements (v2)

This version introduces several changes that reduce redundant computation and prevent resource-wasting retry loops:

| Improvement | Detail |
| :--- | :--- |
| **Classification cache** | `isSmartContractError` results are stored in a `WeakMap` keyed on the Error instance. Repeated renders with the same error object skip the string-scan entirely. |
| **`onError` called once** | The reporting callback is invoked in `componentDidCatch` (once per caught error), not in `render` (which may run many times). Prevents duplicate events to paid observability services. |
| **Retry cap (`MAX_RETRIES`)** | Retries are capped at `MAX_RETRIES = 3`. After exhaustion the "Try Again" button is hidden and a status message is shown, preventing infinite re-render loops on unrecoverable errors. |
| **Non-Error normalisation** | Thrown strings, numbers, and `null` are normalised to `Error` in `getDerivedStateFromError` so downstream code never needs to guard against non-Error values. |

---

## Component API

### `FrontendGlobalErrorBoundary`

```tsx
interface FrontendGlobalErrorBoundaryProps {
  children?: ReactNode;
  fallback?: ReactNode;
  onError?: (report: ErrorReport) => void;
}
```

| Prop | Description |
| :--- | :--- |
| `children` | Component tree to protect. |
| `fallback` | Optional custom fallback UI — replaces the built-in fallback entirely. |
| `onError` | Callback invoked once per caught error with a sanitised `ErrorReport`. |

### `ErrorReport`

```tsx
interface ErrorReport {
  message: string;
  stack: string | undefined;          // undefined in production
  componentStack: string | null | undefined; // undefined in production
  timestamp: string;                  // ISO 8601
  isSmartContractError: boolean;
  errorName: string;
}
```

### `MAX_RETRIES`

Exported constant (`number`). Controls how many times the user can click "Try Again" before the button is hidden. Default: `3`.

---

## Custom Error Classes

```tsx
class ContractError extends Error  // smart contract / Soroban invocation failures
class NetworkError  extends Error  // Horizon API / RPC connectivity issues
class TransactionError extends Error  // tx submission, signing, confirmation failures
```

Throwing one of these classes guarantees the boundary shows the "Smart Contract Error" fallback regardless of the message content.

---

## Error Classification

The boundary classifies an error as a smart-contract error when:

1. It is an instance of `ContractError`, `NetworkError`, or `TransactionError`, **or**
2. Its `name` or `message` contains one of: `contract`, `stellar`, `soroban`, `transaction`, `blockchain`, `ledger`, `horizon`, `xdr`, `invoke`, `wallet`.

The result is cached per Error instance (WeakMap) so repeated renders do not re-scan the string.

---

## Fallback UI Variants

### Smart Contract Error

- 🔗 icon
- Title: "Smart Contract Error"
- Guidance: wallet balance / connection check
- Buttons: "Try Again" (hidden after `MAX_RETRIES`), "Go Home"

### Generic Error

- ⚠️ icon
- Title: "Documentation Loading Error"
- Buttons: "Try Again" (hidden after `MAX_RETRIES`), "Go Home"

Both variants show a `role="status"` message once retries are exhausted.

In development (`NODE_ENV !== 'production'`) a collapsible `<details>` block shows the raw error message.

---

## Security Considerations

- Stack traces and component stacks are **omitted in production** to prevent information disclosure.
- Fallback UI uses only static strings — no raw error data is injected into `innerHTML`, preventing XSS from crafted error messages.
- The `onError` callback receives a sanitised report; callers must not log `error.stack` in production.
- `ContractError` / `TransactionError` messages must never contain XDR blobs, signing keys, or raw contract state.

---

## Lifecycle

```
Error thrown in child
  └─ getDerivedStateFromError(error)   ← pure, sync; normalises non-Error values
       └─ componentDidCatch(error, info) ← side-effects: console.error + onError (once)
            └─ render()                  ← shows fallback; retry button visible if retryCount < MAX_RETRIES
## Component API

### `FrontendGlobalErrorBoundary`

```tsx
interface FrontendGlobalErrorBoundaryProps {
  children?: ReactNode;
  fallback?: ReactNode;
  onError?: (report: ErrorReport) => void;
}
```

| Prop | Description |
| :--- | :--- |
| `children` | Component tree to protect. |
| `fallback` | Optional custom fallback UI — replaces the built-in fallback entirely. |
| `onError` | Callback invoked once per caught error with a sanitised `ErrorReport`. |

### `ErrorReport`

```tsx
interface ErrorReport {
  message: string;
  stack: string | undefined;          // undefined in production
  componentStack: string | null | undefined; // undefined in production
  timestamp: string;                  // ISO 8601
  isSmartContractError: boolean;
  errorName: string;
}
```

### `MAX_RETRIES`

Exported constant (`number`). Controls how many times the user can click "Try Again" before the button is hidden. Default: `3`.

---

## Custom Error Classes

```tsx
class ContractError extends Error  // smart contract / Soroban invocation failures
class NetworkError  extends Error  // Horizon API / RPC connectivity issues
class TransactionError extends Error  // tx submission, signing, confirmation failures
```

---

## Limitations

- Does **not** catch errors in async event handlers, `setTimeout`, Promises, or SSR.
- Does **not** catch errors thrown inside the boundary's own `render` method.
- Nested boundaries can be used for more granular isolation.
## Error Handling Flow

1. **Error Occurrence**: JavaScript error thrown in component tree
2. **State Update**: `getDerivedStateFromError` updates component state
3. **Error Logging**: `componentDidCatch` logs error details
4. **Fallback Rendering**: Error UI displayed instead of crashed component
5. **Recovery Options**: User can retry or navigate away
Throwing one of these classes guarantees the boundary shows the "Smart Contract Error" fallback regardless of the message content.

---

## Error Classification

The boundary classifies an error as a smart-contract error when:

1. It is an instance of `ContractError`, `NetworkError`, or `TransactionError`, **or**
2. Its `name` or `message` contains one of: `contract`, `stellar`, `soroban`, `transaction`, `blockchain`, `ledger`, `horizon`, `xdr`, `invoke`, `wallet`.

The result is cached per Error instance (WeakMap) so repeated renders do not re-scan the string.

---

## Fallback UI Variants

### Smart Contract Error

- 🔗 icon
- Title: "Smart Contract Error"
- Guidance: wallet balance / connection check
- Buttons: "Try Again" (hidden after `MAX_RETRIES`), "Go Home"

### Generic Error

- ⚠️ icon
- Title: "Documentation Loading Error"
- Buttons: "Try Again" (hidden after `MAX_RETRIES`), "Go Home"

Both variants show a `role="status"` message once retries are exhausted.

```tsx
private reportError(error: Error, errorInfo: ErrorInfo) {
  const errorReport = {
    message: error.message,
    stack: error.stack,
    componentStack: errorInfo.componentStack,
    timestamp: new Date().toISOString(),
    userAgent: navigator.userAgent,
    url: window.location.href,
    isSmartContractError: this.state.isSmartContractError,
  };

  // Send to error reporting service (Sentry, LogRocket, etc.)
}
```

### Integration Points

Ready for integration with:
- **Sentry**: `Sentry.captureException(error, { contexts: { react: errorInfo } })`
- **LogRocket**: `LogRocket.captureException(error, { extra: errorInfo })`
- **Custom Analytics**: Send to internal error tracking systems

---

## Usage Examples

### Basic

```tsx
import FrontendGlobalErrorBoundary from '../components/frontend_global_error';

function App() {
  return (
    <FrontendGlobalErrorBoundary>
      <MainApplication />
    </FrontendGlobalErrorBoundary>
### Basic Usage

```tsx
import GlobalErrorBoundary from '../components/frontend_global_error';

function App() {
  return (
    <GlobalErrorBoundary>
      <MainApplication />
    </GlobalErrorBoundary>
  );
}
```

### With error reporting

```tsx
import FrontendGlobalErrorBoundary, { ErrorReport } from '../components/frontend_global_error';

function App() {
  const handleError = (report: ErrorReport) => {
    // report.stack is undefined in production — safe to forward
    Sentry.captureMessage(report.message, { extra: report });
  };
  return (
    <FrontendGlobalErrorBoundary onError={handleError}>
      <MainApplication />
    </FrontendGlobalErrorBoundary>
### With Custom Fallback

```tsx
import GlobalErrorBoundary from '../components/frontend_global_error';

const CustomErrorUI = () => (
  <div>
    <h1>Oops! Something broke</h1>
    <button onClick={() => window.location.reload()}>
      Reload Page
    </button>
  </div>
);

function App() {
  return (
    <GlobalErrorBoundary fallback={<CustomErrorUI />}>
      <MainApplication />
    </GlobalErrorBoundary>
  );
}
```

### Throwing typed errors in contract code

```tsx
import { ContractError } from '../components/frontend_global_error';

async function contribute(amount: number) {
  try {
    await contract.invoke('contribute', { amount });
  } catch (err) {
    throw new ContractError('Contribution failed — check wallet balance');
  }
}
```

### Next.js `_app.tsx`

```tsx
import FrontendGlobalErrorBoundary from '../components/frontend_global_error';

function MyApp({ Component, pageProps }) {
  return (
    <FrontendGlobalErrorBoundary>
      <Component {...pageProps} />
    </FrontendGlobalErrorBoundary>
  );
}
export default MyApp;
```

---

## Test Coverage

| Category | Tests |
| :--- | :--- |
| Custom error classes | 3 |
| Normal rendering | 2 |
| Generic error fallback | 5 |
| Smart contract fallback | 12 |
| Custom fallback prop | 3 |
| Recovery via Try Again | 3 |
| Retry cap (gas efficiency) | 5 |
| Classification caching | 1 |
| Non-Error thrown values | 3 |
| onError callback | 5 |
| Accessibility | 5 |
| Error classification edge cases | 6 |

Target: ≥ 95 % statement coverage, 100 % function coverage.
### Error Throwing in Components

```tsx
import { ContractError, NetworkError } from '../components/frontend_global_error';

// In a smart contract interaction component
try {
  await contract.call();
} catch (error) {
  if (error.message.includes('insufficient funds')) {
    throw new ContractError('Insufficient funds for transaction');
  }
  throw error;
}
```

---

## Testing Coverage

### Test Categories

- ✅ **Normal Operation**: Renders children when no errors
- ✅ **Error Catching**: Handles React errors gracefully
- ✅ **Smart Contract Errors**: Special handling for blockchain errors
- ✅ **Recovery**: Retry functionality works correctly
- ✅ **Custom Fallbacks**: Respects custom error UI
- ✅ **Development Mode**: Shows error details in dev
- ✅ **Error Classification**: Correctly identifies error types
- ✅ **Accessibility**: Error UI is keyboard accessible

### Test Coverage Metrics

- **Statements**: 95%+
- **Branches**: 90%+
- **Functions**: 100%
- **Lines**: 95%+
In development (`NODE_ENV !== 'production'`) a collapsible `<details>` block shows the raw error message.

---

## Security Considerations

- Stack traces and component stacks are **omitted in production** to prevent information disclosure.
- Fallback UI uses only static strings — no raw error data is injected into `innerHTML`, preventing XSS from crafted error messages.
- The `onError` callback receives a sanitised report; callers must not log `error.stack` in production.
- `ContractError` / `TransactionError` messages must never contain XDR blobs, signing keys, or raw contract state.

---

## Lifecycle

```
Error thrown in child
  └─ getDerivedStateFromError(error)   ← pure, sync; normalises non-Error values
       └─ componentDidCatch(error, info) ← side-effects: console.error + onError (once)
            └─ render()                  ← shows fallback; retry button visible if retryCount < MAX_RETRIES
```

---

## Limitations

- Does **not** catch errors in async event handlers, `setTimeout`, Promises, or SSR.
- Does **not** catch errors thrown inside the boundary's own `render` method.
- Nested boundaries can be used for more granular isolation.

---

## Usage Examples

### Basic

```tsx
import FrontendGlobalErrorBoundary from '../components/frontend_global_error';

function App() {
  return (
    <FrontendGlobalErrorBoundary>
      <MainApplication />
    </FrontendGlobalErrorBoundary>
  );
}
```

### With error reporting

```tsx
import FrontendGlobalErrorBoundary, { ErrorReport } from '../components/frontend_global_error';

function App() {
  const handleError = (report: ErrorReport) => {
    // report.stack is undefined in production — safe to forward
    Sentry.captureMessage(report.message, { extra: report });
  };
  return (
    <FrontendGlobalErrorBoundary onError={handleError}>
      <MainApplication />
    </FrontendGlobalErrorBoundary>
  );
}
```

### Throwing typed errors in contract code

```tsx
import { ContractError } from '../components/frontend_global_error';

async function contribute(amount: number) {
  try {
    await contract.invoke('contribute', { amount });
  } catch (err) {
    throw new ContractError('Contribution failed — check wallet balance');
  }
}
```

### Next.js `_app.tsx`

```tsx
import FrontendGlobalErrorBoundary from '../components/frontend_global_error';

function MyApp({ Component, pageProps }) {
  return (
    <FrontendGlobalErrorBoundary>
      <Component {...pageProps} />
    </FrontendGlobalErrorBoundary>
  );
}
export default MyApp;
```

---

## Test Coverage

| Category | Tests |
| :--- | :--- |
| Custom error classes | 3 |
| Normal rendering | 2 |
| Generic error fallback | 5 |
| Smart contract fallback | 12 |
| Custom fallback prop | 3 |
| Recovery via Try Again | 3 |
| Retry cap (gas efficiency) | 5 |
| Classification caching | 1 |
| Non-Error thrown values | 3 |
| onError callback | 5 |
| Accessibility | 5 |
| Error classification edge cases | 6 |

- **Error Analytics**: Integration with error tracking dashboards
- **User Feedback**: Allow users to report additional error context
- **Error Recovery Strategies**: Automatic retry with exponential backoff
- **Offline Support**: Special handling for network connectivity issues

### Extensibility

- **Plugin System**: Allow custom error handlers and classifiers
- **Error Context**: Additional metadata collection for better debugging
- **Recovery Actions**: Configurable recovery strategies per error type
- **Recovery Actions**: Configurable recovery strategies per error type
Target: ≥ 95 % statement coverage, 100 % function coverage.
