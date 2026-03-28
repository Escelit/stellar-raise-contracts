# Frontend Global Error Boundary Documentation

## Overview

The `frontend_global_error.tsx` module provides a secure, comprehensive error boundary implementation for React applications. It implements the React Error Boundary pattern to catch rendering errors, provide fallback UIs, and report errors securely.

## Installation

This module is part of the Stellar Raise frontend utilities. It requires React 16+ and is compatible with TypeScript projects.

```tsx
import { GlobalErrorBoundary, withErrorBoundary, useErrorBoundary } from './frontend_global_error';
```

## Features

- **Secure Error Handling**: Error messages are sanitized before display to prevent information leakage
- **Configurable Recovery Options**: Users can retry, reload, or dismiss errors
- **Accessibility Support**: Proper ARIA attributes for screen readers
- **Severity Classification**: Automatic error severity detection (low, medium, high, critical)
- **Error Reporting**: Optional error reporting to external endpoints
- **Type-Safe**: Full TypeScript support with comprehensive type definitions

## Usage

### Basic Usage

```tsx
import { GlobalErrorBoundary } from './frontend_global_error';

function App() {
  return (
    <GlobalErrorBoundary>
      <YourApplication />
    </GlobalErrorBoundary>
  );
}
```

### With Custom Configuration

```tsx
<GlobalErrorBoundary
  config={{
    enableLogging: true,
    showErrorDetails: false,
    enableRecovery: true,
    maxRetries: 3,
    reportingEndpoint: 'https://example.com/reports'
  }}
>
  <YourApplication />
</GlobalErrorBoundary>
```

### With Custom Fallback

```tsx
<GlobalErrorBoundary
  fallback={<CustomErrorUI message="Something went wrong" />}
>
  <YourApplication />
</GlobalErrorBoundary>
```

### With Error Callback

```tsx
<GlobalErrorBoundary
  onError={(error, errorInfo) => {
    console.error('Error caught:', error.message);
    // Send to error tracking service
  }}
>
  <YourApplication />
</GlobalErrorBoundary>
```

### Using HOC (Higher-Order Component)

```tsx
import { withErrorBoundary } from './frontend_global_error';

const WrappedComponent = withErrorBoundary(MyComponent, {
  enableLogging: true,
  showErrorDetails: false,
});
```

### Using the Hook

```tsx
import { useErrorBoundary } from './frontend_global_error';

function MyComponent() {
  const { error, resetError, triggerError, hasError } = useErrorBoundary();
  
  return (
    <div>
      {hasError && <div>Error occurred: {error?.message}</div>}
      <button onClick={() => triggerError(new Error('Test'))}>
        Trigger Error
      </button>
      <button onClick={resetError}>
        Reset Error
      </button>
    </div>
  );
}
```

## API Reference

### Constants

#### `ERROR_SEVERITY_LEVELS`
Error severity levels: `['low', 'medium', 'high', 'critical']`

#### `RECOVERY_ACTIONS`
Available recovery actions: `['retry', 'reload', 'navigate', 'dismiss']`

#### `DEFAULT_ERROR_BOUNDARY_CONFIG`
Default configuration:
```typescript
{
  enableLogging: true,
  showErrorDetails: false,
  enableRecovery: true,
  maxRetries: 3,
}
```

### Types

#### `ErrorSeverityLevel`
Type for error severity levels: `'low' | 'medium' | 'high' | 'critical'`

#### `RecoveryAction`
Type for recovery actions: `'retry' | 'reload' | 'navigate' | 'dismiss'`

#### `ErrorBoundaryConfig`
Configuration interface:
```typescript
interface ErrorBoundaryConfig {
  enableLogging: boolean;
  showErrorDetails: boolean;
  enableRecovery: boolean;
  fallback?: ReactNode;
  maxRetries: number;
  reportingEndpoint?: string;
}
```

#### `ErrorInfoType`
Error information interface:
```typescript
interface ErrorInfoType {
  message: string;
  stack?: string;
  componentStack?: string;
  timestamp: Date;
  severity: ErrorSeverityLevel;
  isHandled: boolean;
}
```

### Functions

#### `determineErrorSeverity(error: Error): ErrorSeverityLevel`
Determines the severity level of an error based on its message content.

**Severity Classification:**
- **Critical**: Network, fetch, blockchain, wallet errors
- **High**: Permission, unauthorized, authentication errors
- **Medium**: Validation, type, render errors
- **Low**: All other errors

#### `validateErrorBoundaryConfig(config: Partial<ErrorBoundaryConfig>): boolean`
Validates the error boundary configuration.

**Validation Rules:**
- `maxRetries` must be between 0 and 10
- `reportingEndpoint` must be a valid URL (if provided)

#### `createErrorInfo(error: Error, errorInfo: ErrorInfo): ErrorInfoType`
Creates a sanitized error info object from an error and React error info.

### Components

#### `GlobalErrorBoundary`
React component that catches JavaScript errors in its child component tree.

**Props:**
- `children: ReactNode` - Child components
- `config?: Partial<ErrorBoundaryConfig>` - Configuration
- `fallback?: ReactNode` - Custom fallback component
- `onError?: (error: Error, errorInfo: ErrorInfoType) => void` - Error callback
- `onRecover?: () => void` - Recovery callback

**Security Features:**
- Error messages are sanitized before display
- Stack traces only shown in development mode
- No sensitive data in error logs

### Higher-Order Components

#### `withErrorBoundary<P>(WrappedComponent, config?)`
Wraps a component with an error boundary.

### Hooks

#### `useErrorBoundary()`
Returns:
```typescript
{
  error: Error | null;
  resetError: () => void;
  triggerError: (error: Error) => void;
  hasError: boolean;
}
```

## Security Considerations

1. **Information Leakage Prevention**: Error messages are sanitized before display to prevent exposing sensitive information like API keys, tokens, or internal system details.

2. **Stack Trace Handling**: Stack traces are only shown in development mode (`NODE_ENV === 'development'`).

3. **Secure Logging**: Error logs do not include sensitive user data or system internals.

4. **Endpoint Validation**: Error reporting endpoints are validated before use to prevent SSRF attacks.

5. **Error Severity Classification**: Errors are classified by severity to help prioritize handling and reporting.

## Testing

The module includes comprehensive tests covering:

- Error boundary rendering behavior
- Error catching and fallback display
- Recovery actions (retry, reload, dismiss)
- Error severity determination
- Configuration validation
- Edge cases (empty messages, long messages, special characters)
- Accessibility attributes
- Security considerations

Run tests with:
```bash
npm test
```

Run tests with coverage:
```bash
npm run test:coverage
```

## Error Severity Levels

| Level | Description | Error Patterns |
|-------|-------------|----------------|
| Critical | System-level failures | Network, fetch, blockchain, wallet errors |
| High | Security/auth failures | Permission, unauthorized, authentication |
| Medium | Application errors | Validation, type, render errors |
| Low | Minor issues | Unknown or generic errors |

## Recovery Options

When an error occurs, users can:
- **Retry**: Attempt to re-render the component (configurable max attempts)
- **Reload**: Reload the entire page
- **Dismiss**: Dismiss the error and try to continue — **dangerous**: does not resolve the underlying error; only offer this for known transient failures

## Accessibility

The error boundary includes:
- `role="alert"` for screen reader announcements
- `aria-live="assertive"` for immediate announcements
- Keyboard-accessible recovery buttons

## Performance Considerations

- Minimal re-renders during error state
- Configurable retry attempts to prevent infinite loops
- Async error reporting to avoid blocking UI

## Example: Complete Integration

```tsx
import React from 'react';
import { GlobalErrorBoundary, withErrorBoundary, useErrorBoundary } from './frontend_global_error';

// Using HOC for main application
const AppWithErrorBoundary = withErrorBoundary(App, {
  enableLogging: true,
  showErrorDetails: false,
  enableRecovery: true,
  maxRetries: 3,
  reportingEndpoint: 'https://api.example.com/errors',
});

// Using hook in components
function FeatureWithErrorBoundary() {
  const { error, resetError, triggerError, hasError } = useErrorBoundary();
  
  return (
    <GlobalErrorBoundary>
      <YourFeature onError={triggerError} />
      {hasError && (
        <button onClick={resetError}>Try Again</button>
      )}
    </GlobalErrorBoundary>
  );
}

// Using the wrapped app
export default function Root() {
  return (
    <GlobalErrorBoundary
      fallback={<MaintenanceMode />}
      onError={(error, info) => {
        // Send to monitoring service
        analytics.track('error', { message: error.message, severity: info.severity });
      }}
    >
      <AppWithErrorBoundary />
    </GlobalErrorBoundary>
  );
}
```

## Migration Guide

If upgrading from a previous version:

1. Update imports to use new module path
2. Review error severity classifications
3. Test new recovery behavior
4. Validate error reporting endpoints

## License

This module is part of the Stellar Raise project and follows the same license terms.
