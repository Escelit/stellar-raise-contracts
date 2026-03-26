# Deployment Shell Script Error Capturing Bugfix Design

## Overview

The script `scripts/deployment_shell_script.sh` declares exit-code constants
(`EXIT_OK`, `EXIT_MISSING_DEP`, `EXIT_BAD_ARG`, `EXIT_BUILD_FAIL`,
`EXIT_DEPLOY_FAIL`, `EXIT_INIT_FAIL`, `EXIT_NETWORK_FAIL`), RPC URL constants
(`RPC_TESTNET`, `RPC_MAINNET`, `RPC_FUTURENET`), and a default argument
constant (`DEFAULT_MIN_CONTRIBUTION`) but never assigns values to any of them.
Under `set -u` these would cause an "unbound variable" error; without it they
silently expand to empty strings, producing three concrete failures:

1. `print_help` renders blank exit-code columns instead of the numeric values 0–6.
2. `check_network` calls `curl` with an empty URL, always failing with a
   misleading "network connectivity" error even when the network is fine.
3. Omitting the optional `min_contribution` argument sets it to
   `$DEFAULT_MIN_CONTRIBUTION` (empty), which immediately fails the
   `^[0-9]+$` regex in `validate_args`.

The fix is minimal: add the missing constant assignments in the
"Exit code constants" block at the top of the script.

## Glossary

- **Bug_Condition (C)**: Any code path that dereferences one of the seven
  unassigned constants (`EXIT_*`, `RPC_*`, `DEFAULT_MIN_CONTRIBUTION`).
- **Property (P)**: After the fix, every dereference of those constants
  produces the correct numeric or URL value, not an empty string.
- **Preservation**: All behaviour that does not depend on the missing constants
  (logging helpers, `die`/`warn`, explicit `min_contribution` argument,
  unknown-network skip path, build/deploy/init steps) must remain identical.
- **`print_help`**: The function in `scripts/deployment_shell_script.sh` that
  prints usage text including the exit-code table.
- **`check_network`**: The function that performs a `curl` connectivity check
  against the RPC endpoint for the selected network.
- **`validate_args`**: The function that validates all positional arguments,
  including `min_contribution`.
- **`DEFAULT_MIN_CONTRIBUTION`**: The fallback value used when the caller omits
  the fifth positional argument.

## Bug Details

### Fault Condition

The bug manifests whenever the script reaches any of the three affected code
paths: `print_help`, `check_network` (for known networks), or the
`min_contribution` default expansion in `main`. The constants are referenced
but were never given values in the variable-declaration block.

**Formal Specification:**

```
FUNCTION isBugCondition(input)
  INPUT: input – a script invocation (arguments + environment)
  OUTPUT: boolean

  RETURN (
    input invokes --help
    OR (input.NETWORK IN ['testnet', 'mainnet', 'futurenet']
        AND network connectivity check is reached)
    OR (input does not supply a 5th positional argument
        AND DEFAULT_MIN_CONTRIBUTION is empty)
  )
  AND (EXIT_OK = ""
       OR RPC_TESTNET = ""
       OR DEFAULT_MIN_CONTRIBUTION = "")
END FUNCTION
```

### Examples

- `./deployment_shell_script.sh --help` → exit-code table shows blank cells
  instead of `0  1  2  3  4  5  6`.
- `NETWORK=testnet ./deployment_shell_script.sh G... G... 1000 9999999999` →
  `curl ""` fails immediately; script exits 6 ("network connectivity failure")
  even when Stellar testnet is reachable.
- `./deployment_shell_script.sh G... G... 1000 9999999999` (no 5th arg) →
  `min_contribution` is `""`, fails `^[0-9]+$`, script exits 2
  ("min_contribution must be a positive integer").
- `./deployment_shell_script.sh G... G... 1000 9999999999 100` (explicit 5th
  arg) → unaffected; `DEFAULT_MIN_CONTRIBUTION` is never used.

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**

- Explicit `min_contribution` argument continues to be validated and used as-is.
- `die`, `warn`, `log`, `emit_event`, `run_captured` helpers are unaffected.
- Unknown-network path in `check_network` (the `warn` + `return 0` branch)
  continues to skip the connectivity check.
- Build, deploy, and init steps are unaffected.
- `--dry-run` mode continues to validate args and skip network/build/deploy/init.

**Scope:**
All invocations that do NOT reach `print_help`, the known-network branch of
`check_network`, or the `DEFAULT_MIN_CONTRIBUTION` expansion are completely
unaffected by this fix. This includes:

- Any invocation that supplies all five positional arguments explicitly.
- Invocations targeting an unknown/custom network name.
- All error paths triggered by missing `creator`, `token`, `goal`, or
  `deadline` arguments.

## Hypothesized Root Cause

1. **Incomplete variable-declaration block**: The "Exit code constants" section
   at the top of the script contains only a comment header and the runtime
   variables (`NETWORK`, `DEPLOY_LOG`, etc.). The seven `EXIT_*` constants,
   three `RPC_*` URL constants, and `DEFAULT_MIN_CONTRIBUTION` were referenced
   in function bodies but their assignment lines were never written.

2. **`set -u` not triggered during development**: The script uses
   `set -euo pipefail`. Under `set -u`, referencing an unset variable is a
   fatal error. If the script was tested only with all five positional arguments
   and without `--help`, the `EXIT_*` and `RPC_*` references in `print_help`
   and `check_network` would never be reached, masking the omission.

3. **Silent empty-string expansion in `print_help`**: Because `print_help` uses
   a heredoc, the empty expansions produce blank output rather than an error,
   making the bug easy to overlook in casual review.

4. **`DEFAULT_MIN_CONTRIBUTION` only matters at the call site**: The default is
   applied via `${positional[4]:-$DEFAULT_MIN_CONTRIBUTION}` in `main`, so the
   bug only surfaces when the caller omits the fifth argument.

## Correctness Properties

Property 1: Fault Condition - Constants Expand to Correct Values

_For any_ script invocation where the bug condition holds (isBugCondition
returns true), the fixed script SHALL expand each constant to its intended
value: `EXIT_OK=0`, `EXIT_MISSING_DEP=1`, `EXIT_BAD_ARG=2`,
`EXIT_BUILD_FAIL=3`, `EXIT_DEPLOY_FAIL=4`, `EXIT_INIT_FAIL=5`,
`EXIT_NETWORK_FAIL=6`, `RPC_TESTNET`, `RPC_MAINNET`, and `RPC_FUTURENET` to
their respective Stellar Horizon/RPC URLs, and `DEFAULT_MIN_CONTRIBUTION` to a
positive integer — so that `print_help` renders correct exit-code columns,
`check_network` curls a valid URL, and omitting the fifth argument does not
cause a spurious validation failure.

**Validates: Requirements 2.1, 2.2, 2.3**

Property 2: Preservation - Non-Constant-Dependent Behavior Unchanged

_For any_ script invocation where the bug condition does NOT hold
(isBugCondition returns false — i.e., the caller supplies all five positional
arguments explicitly and does not invoke `--help` and targets an unknown
network), the fixed script SHALL produce exactly the same exit code, log
output, and JSON events as the original script, preserving all existing
argument-validation, error-handling, and deployment behaviour.

**Validates: Requirements 3.1, 3.2, 3.3**

## Fix Implementation

### Changes Required

**File**: `scripts/deployment_shell_script.sh`

**Section**: `# ── Exit code constants ──` block (immediately after `set -euo pipefail`)

**Specific Changes**:

1. **Add exit-code constant assignments** directly below the comment header:

   ```bash
   EXIT_OK=0
   EXIT_MISSING_DEP=1
   EXIT_BAD_ARG=2
   EXIT_BUILD_FAIL=3
   EXIT_DEPLOY_FAIL=4
   EXIT_INIT_FAIL=5
   EXIT_NETWORK_FAIL=6
   ```

2. **Add RPC URL constant assignments** in the same block:

   ```bash
   RPC_TESTNET="https://soroban-testnet.stellar.org"
   RPC_MAINNET="https://mainnet.stellar.validationcloud.io/v1/xycpM7GIGz7BKZQ7IQKM"
   RPC_FUTURENET="https://rpc-futurenet.stellar.org"
   ```

   _(Use the canonical Stellar Horizon/RPC endpoints; adjust if the project
   uses a different provider.)_

3. **Add DEFAULT_MIN_CONTRIBUTION assignment**:

   ```bash
   DEFAULT_MIN_CONTRIBUTION=1
   ```

   _(Value of `1` stroop is the minimal sensible default; adjust to match
   project requirements.)_

4. **No other changes required** — all function bodies, argument parsing,
   logging, and error-handling logic remain untouched.

## Testing Strategy

### Validation Approach

Two-phase approach: first run exploratory tests against the **unfixed** script
to confirm the bug manifests as described, then run fix-checking and
preservation tests against the **fixed** script.

### Exploratory Fault Condition Checking

**Goal**: Surface counterexamples that demonstrate the bug on unfixed code and
confirm the root cause (missing assignments, not a logic error elsewhere).

**Test Plan**: Source or invoke the script in a controlled environment, stub
out `curl`/`cargo`/`stellar` to avoid real network/build calls, and assert
that the affected outputs are incorrect on the unfixed version.

**Test Cases**:

1. **Help output test**: Run `./deployment_shell_script.sh --help` and grep for
   the exit-code table; assert that numeric values `0`–`6` are present.
   (Will fail on unfixed code — columns are blank.)
2. **Network check URL test**: Stub `curl` to echo its first argument; run with
   `NETWORK=testnet` and a valid set of args; assert the URL passed to `curl`
   is non-empty and starts with `https://`.
   (Will fail on unfixed code — `curl` receives an empty string.)
3. **Default min_contribution test**: Run without the fifth positional argument
   using a future deadline; assert the script does not exit 2.
   (Will fail on unfixed code — empty `min_contribution` fails regex.)
4. **Edge case — explicit fifth arg**: Run with all five args; assert behaviour
   is identical on both unfixed and fixed code (should pass on both).

**Expected Counterexamples**:

- `print_help` output contains empty strings where `0`–`6` should appear.
- `curl` is invoked with `""` as the URL, returning a non-200 / error.
- `validate_args` exits 2 with "min_contribution must be a positive integer"
  even though the caller intended to use the default.

### Fix Checking

**Goal**: Verify that for all inputs where the bug condition holds, the fixed
script produces the expected correct behavior.

**Pseudocode:**

```
FOR ALL input WHERE isBugCondition(input) DO
  result := deployment_shell_script_fixed(input)
  ASSERT expectedBehavior(result)
END FOR
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold, the
fixed script produces the same result as the original script.

**Pseudocode:**

```
FOR ALL input WHERE NOT isBugCondition(input) DO
  ASSERT deployment_shell_script_original(input) = deployment_shell_script_fixed(input)
END FOR
```

**Testing Approach**: Property-based testing is recommended for preservation
checking because the input space (argument combinations, environment variables,
network names) is large and manual enumeration would miss edge cases.

**Test Cases**:

1. **Explicit min_contribution preservation**: Verify that supplying the fifth
   argument explicitly produces identical exit codes and log lines on both
   unfixed and fixed code.
2. **Unknown network preservation**: Verify that `NETWORK=localnet` still
   triggers the `warn` + `return 0` path unchanged.
3. **Missing required args preservation**: Verify that omitting `creator` or
   `token` still exits 2 with the correct message on both versions.

### Unit Tests

- Test `print_help` output contains all seven numeric exit codes (0–6).
- Test `check_network` with `NETWORK=testnet` passes a non-empty HTTPS URL to
  `curl` (stub `curl`).
- Test `check_network` with `NETWORK=mainnet` and `NETWORK=futurenet` similarly.
- Test that omitting the fifth positional argument uses `DEFAULT_MIN_CONTRIBUTION`
  and passes `validate_args` without error.
- Test edge case: `DEFAULT_MIN_CONTRIBUTION` must satisfy `^[0-9]+$`.

### Property-Based Tests

- Generate random valid argument sets (all five args supplied) and verify the
  fixed script's exit code and log structure match the original for all inputs
  that don't reach the buggy constants.
- Generate random network names not in `[testnet, mainnet, futurenet]` and
  verify the unknown-network warn path is preserved.
- Generate random `min_contribution` values (positive integers) and verify
  validation behaviour is unchanged after the fix.

### Integration Tests

- Full dry-run with `--dry-run` flag and all five args: verify exit 0 and
  correct NDJSON events emitted.
- Full dry-run omitting the fifth arg: verify exit 0 (default used) and
  `min_contribution` value in logs matches `DEFAULT_MIN_CONTRIBUTION`.
- `--help` invocation: verify all seven exit-code rows are present and
  correctly formatted in the output.
