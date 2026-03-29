# Bugfix Requirements Document

## Introduction

The deployment shell script (`scripts/deployment_shell_script.sh`) declares exit code constants and configuration variables (`EXIT_OK`, `EXIT_MISSING_DEP`, `EXIT_BAD_ARG`, `EXIT_BUILD_FAIL`, `EXIT_DEPLOY_FAIL`, `EXIT_INIT_FAIL`, `EXIT_NETWORK_FAIL`, `RPC_TESTNET`, `RPC_MAINNET`, `RPC_FUTURENET`, `DEFAULT_MIN_CONTRIBUTION`) but never assigns them values. At runtime these variables expand to empty strings, causing three concrete failures: blank exit-code columns in help output, curl invocations with empty URLs that always fail with a misleading network error, and an empty default for `min_contribution` that fails numeric validation.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN `print_help` is called THEN the system renders blank exit-code columns instead of the numeric values 0–6

1.2 WHEN `check_network` is called with any network argument THEN the system calls `curl` with an empty URL, always failing with a misleading "network connectivity" error regardless of actual connectivity

1.3 WHEN the `min_contribution` argument is omitted THEN the system sets it to an empty string, which fails the `^[0-9]+$` regex validation in `validate_args`

### Expected Behavior (Correct)

2.1 WHEN `print_help` is called THEN the system SHALL display the correct numeric exit code values (0–6) in the help output

2.2 WHEN `check_network` is called with a valid network argument THEN the system SHALL call `curl` with the correct RPC URL for that network and report connectivity failures accurately

2.3 WHEN the `min_contribution` argument is omitted THEN the system SHALL use the assigned numeric default value for `DEFAULT_MIN_CONTRIBUTION`, passing the `^[0-9]+$` regex validation

### Unchanged Behavior (Regression Prevention)

3.1 WHEN all required arguments are provided and valid THEN the system SHALL CONTINUE TO execute the deployment flow without errors

3.2 WHEN an unsupported network argument is provided THEN the system SHALL CONTINUE TO exit with the appropriate error code and message

3.3 WHEN a required dependency is missing THEN the system SHALL CONTINUE TO exit with the dependency-missing error code

3.4 WHEN `min_contribution` is explicitly provided as a valid positive integer THEN the system SHALL CONTINUE TO accept it and proceed with deployment
