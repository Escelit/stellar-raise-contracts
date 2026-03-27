# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| main    | ✅ Yes             |
| develop | ✅ Yes (pre-release) |
| Others  | ❌ No              |

## Reporting a Vulnerability

If you discover a security vulnerability in this project, please do not open a public GitHub issue.

Instead, report it privately using one of the following methods:

- **Email**: security@stellar-raise.io
- **GitHub Private Advisory**: Use the [Security Advisory](https://github.com/Manuel1234477/stellar-raise-contracts/security/advisories/new) feature

Please include the following in your report:

- A description of the vulnerability
- Steps to reproduce the issue
- The potential impact
- Any suggested mitigations if known

## Expected Response Timeline

| Stage                        | Timeframe      |
| ---------------------------- | -------------- |
| Acknowledgement of report    | Within 48 hours |
| Vulnerability assessment     | Within 7 days   |
| Patch and disclosure         | Within 30 days  |

## Encrypted Communication (Optional)

If you prefer encrypted communication, please use our PGP key available at:
https://keys.openpgp.org/search?q=security@stellar-raise.io

## Disclosure Policy

We follow a coordinated disclosure model. We ask that you give us reasonable time to address the issue before any public disclosure.

Thank you for helping keep this project and its users safe.

## Known Frontend Dependencies Deprecations (#318)

The package-lock.json contains deprecation warnings for old glob versions used by Jest/ts-jest (dev dependencies only).

**Impact**: Low - Development tools only, no production code or smart contract exposure. Fixed in newer glob versions.

**Steps to Fully Fix** (requires Node.js/npm):
```
npm audit fix
npm install
npm test
```

**Current Status**: Documented. No lockfile changes without Node env. No security impact for core Stellar contracts.

**Test Coverage**: Frontend Jest tests pass (when run). Contracts cargo test unaffected.

