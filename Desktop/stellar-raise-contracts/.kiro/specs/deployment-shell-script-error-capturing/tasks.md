# Implementation Plan

- [x] 1. Write bug condition exploration test
  - **Property 1: Fault Condition** - Constants Expand to Correct Values
  - **CRITICAL**: This test MUST FAIL on unfixed code - failure confirms the bug exists
  - **DO NOT attempt to fix the test or the code when it fails**
  - **NOTE**: This test encodes the expected behavior - it will validate the fix when it passes after implementation
  - **GOAL**: Surface counterexamples that demonstrate the bug exists
  - **Scoped PBT Approach**: Scope the property to the three concrete failing cases (--help, known-network check, omitted 5th arg) to ensure reproducibility
  - In `scripts/deployment_shell_script.test.sh`, add tests that:
    - Run `./deployment_shell_script.sh --help` and assert the exit-code table contains `0`–`6` (not blank cells)
    - Stub `curl` to echo its first argument; run with `NETWORK=testnet` and valid args; assert the URL passed to `curl` is non-empty and starts with `https://`
    - Run without the fifth positional argument; assert the script does NOT exit 2 ("min_contribution must be a positive integer")
  - The test assertions match the Expected Behavior Properties from design (requirements 2.1, 2.2, 2.3)
  - Run tests on UNFIXED code
  - **EXPECTED OUTCOME**: Tests FAIL (this is correct - it proves the bug exists)
  - Document counterexamples found:
    - `print_help` output contains empty strings where `0`–`6` should appear
    - `curl` is invoked with `""` as the URL
    - `validate_args` exits 2 with "min_contribution must be a positive integer" even when caller intended to use the default
  - Mark task complete when tests are written, run, and failures are documented
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. Write preservation property tests (BEFORE implementing fix)
  - **Property 2: Preservation** - Non-Constant-Dependent Behavior Unchanged
  - **IMPORTANT**: Follow observation-first methodology
  - Observe behavior on UNFIXED code for non-buggy inputs (isBugCondition returns false):
    - Observe: supplying all five positional arguments explicitly → script proceeds past `validate_args` without error
    - Observe: `NETWORK=localnet` (unknown network) → triggers `warn` + `return 0` path, skips connectivity check
    - Observe: omitting `creator` or `token` → exits 2 with the correct missing-argument message
    - Observe: missing required dependency → exits with dependency-missing error code
  - In `scripts/deployment_shell_script.test.sh`, add property-based tests that:
    - For all invocations supplying all five positional arguments explicitly: assert exit code and log structure match between unfixed and fixed code
    - For random network names not in `[testnet, mainnet, futurenet]`: assert the unknown-network `warn` + `return 0` path is preserved
    - For random valid positive-integer `min_contribution` values: assert validation behaviour is unchanged
  - Run tests on UNFIXED code
  - **EXPECTED OUTCOME**: Tests PASS (this confirms baseline behavior to preserve)
  - Mark task complete when tests are written, run, and passing on unfixed code
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 3. Fix unassigned constants in deployment_shell_script.sh
  - [x] 3.1 Implement the fix
    - In `scripts/deployment_shell_script.sh`, locate the `# ── Exit code constants ──` block immediately after `set -euo pipefail`
    - Add exit-code constant assignments:
      ```bash
      EXIT_OK=0
      EXIT_MISSING_DEP=1
      EXIT_BAD_ARG=2
      EXIT_BUILD_FAIL=3
      EXIT_DEPLOY_FAIL=4
      EXIT_INIT_FAIL=5
      EXIT_NETWORK_FAIL=6
      ```
    - Add RPC URL constant assignments:
      ```bash
      RPC_TESTNET="https://soroban-testnet.stellar.org"
      RPC_MAINNET="https://mainnet.stellar.validationcloud.io/v1/xycpM7GIGz7BKZQ7IQKM"
      RPC_FUTURENET="https://rpc-futurenet.stellar.org"
      ```
    - Add default contribution assignment:
      ```bash
      DEFAULT_MIN_CONTRIBUTION=1
      ```
    - No other changes — all function bodies, argument parsing, logging, and error-handling logic remain untouched
    - _Bug_Condition: isBugCondition(input) where input invokes --help OR input.NETWORK IN ['testnet','mainnet','futurenet'] OR input omits 5th positional arg — AND any of EXIT_OK, RPC_TESTNET, DEFAULT_MIN_CONTRIBUTION is ""_
    - _Expected_Behavior: print_help renders numeric exit codes 0–6; check_network curls a valid HTTPS URL; omitting 5th arg uses DEFAULT_MIN_CONTRIBUTION=1 and passes ^[0-9]+ validation_
    - _Preservation: All invocations supplying all five args explicitly, targeting unknown networks, or triggering missing-arg/missing-dep error paths produce identical exit codes, log output, and JSON events_
    - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 3.4_

  - [x] 3.2 Verify bug condition exploration test now passes
    - **Property 1: Expected Behavior** - Constants Expand to Correct Values
    - **IMPORTANT**: Re-run the SAME tests from task 1 - do NOT write new tests
    - The tests from task 1 encode the expected behavior (requirements 2.1, 2.2, 2.3)
    - When these tests pass, it confirms the expected behavior is satisfied
    - Run bug condition exploration tests from step 1
    - **EXPECTED OUTCOME**: Tests PASS (confirms bug is fixed)
    - _Requirements: 2.1, 2.2, 2.3_

  - [x] 3.3 Verify preservation tests still pass
    - **Property 2: Preservation** - Non-Constant-Dependent Behavior Unchanged
    - **IMPORTANT**: Re-run the SAME tests from task 2 - do NOT write new tests
    - Run preservation property tests from step 2
    - **EXPECTED OUTCOME**: Tests PASS (confirms no regressions)
    - Confirm all tests still pass after fix (no regressions introduced)

- [x] 4. Checkpoint - Ensure all tests pass
  - Run the full test suite in `scripts/deployment_shell_script.test.sh`
  - Ensure all tests pass; ask the user if any questions arise
