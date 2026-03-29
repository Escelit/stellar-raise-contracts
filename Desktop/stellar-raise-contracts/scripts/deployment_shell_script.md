# deployment_shell_script.sh

Builds, deploys, and initialises the Stellar Raise crowdfund contract on a target Stellar network with structured error capturing and CI/CD-friendly logging.

## Overview

The script performs three sequential steps:

1. **Build** — compiles the contract to WASM via `cargo build --target wasm32-unknown-unknown --release`
2. **Deploy** — uploads the WASM to the target network via `stellar contract deploy`
3. **Init** — calls `initialize` on the deployed contract via `stellar contract invoke`

Every step emits structured NDJSON events to `DEPLOY_JSON_LOG` (default: `deploy_events.json`) and human-readable timestamped lines to `DEPLOY_LOG` (default: `deploy_errors.log`). Both logs are truncated at the start of each run.

## Usage

```bash
bash scripts/deployment_shell_script.sh [OPTIONS] <creator> <token> <goal> <deadline> [min_contribution]
```

### Positional Arguments

| Argument           | Description                                                |
| ------------------ | ---------------------------------------------------------- |
| `creator`          | Stellar address of the campaign creator (signing identity) |
| `token`            | Stellar address of the token contract                      |
| `goal`             | Funding goal in stroops (positive integer)                 |
| `deadline`         | Unix timestamp for campaign end (must be in the future)    |
| `min_contribution` | Minimum pledge amount in stroops (default: `1`)            |

### Options

| Flag        | Description                                                 |
| ----------- | ----------------------------------------------------------- |
| `--help`    | Print usage and exit 0                                      |
| `--dry-run` | Validate arguments and dependencies, skip build/deploy/init |

### Environment Variables

| Variable          | Default              | Description                                                |
| ----------------- | -------------------- | ---------------------------------------------------------- |
| `NETWORK`         | `testnet`            | Target Stellar network (`testnet`, `mainnet`, `futurenet`) |
| `DEPLOY_LOG`      | `deploy_errors.log`  | Path for human-readable log output                         |
| `DEPLOY_JSON_LOG` | `deploy_events.json` | Path for structured NDJSON event log                       |
| `DRY_RUN`         | `false`              | Set to `true` to enable dry-run mode                       |

## Exit Codes

| Code | Constant            | Meaning                                               |
| ---- | ------------------- | ----------------------------------------------------- |
| 0    | `EXIT_OK`           | Success                                               |
| 1    | `EXIT_MISSING_DEP`  | Required tool (`cargo`, `stellar`) not found          |
| 2    | `EXIT_BAD_ARG`      | Invalid or missing argument                           |
| 3    | `EXIT_BUILD_FAIL`   | `cargo build` failed or WASM artifact missing         |
| 4    | `EXIT_DEPLOY_FAIL`  | `stellar contract deploy` failed or returned empty ID |
| 5    | `EXIT_INIT_FAIL`    | `stellar contract invoke -- initialize` failed        |
| 6    | `EXIT_NETWORK_FAIL` | RPC connectivity check failed                         |

## Structured Event Log (NDJSON)

Each line in `DEPLOY_JSON_LOG` is a self-contained JSON object:

```json
{"event":"step_ok","step":"build","message":"WASM built successfully","timestamp":"2025-01-01T00:00:00Z","network":"testnet","wasm_path":"target/..."}
{"event":"deploy_complete","step":"done","message":"Deployment finished","timestamp":"...","network":"testnet","contract_id":"CXXX","error_count":0}
```

Event types: `step_start`, `step_ok`, `step_error`, `deploy_complete`

Steps: `validate`, `network_check`, `build`, `deploy`, `init`, `done`

## Bugfix: Unassigned Constants (PR #417)

Prior to this fix, the script declared exit-code constants (`EXIT_OK`–`EXIT_NETWORK_FAIL`), RPC URL constants (`RPC_TESTNET`, `RPC_MAINNET`, `RPC_FUTURENET`), and `DEFAULT_MIN_CONTRIBUTION` but never assigned values to them. This caused three concrete failures:

- `--help` rendered blank exit-code columns instead of `0`–`6`
- `check_network` called `curl` with an empty URL, always failing with a misleading network error
- Omitting the fifth positional argument set `min_contribution` to `""`, failing the `^[0-9]+$` regex in `validate_args`

The fix adds the missing assignments in the `# ── Exit code constants ──` block at the top of the script. No function logic was changed.

## Security Notes

- Never pass a raw secret key as the `creator` argument or `--source` flag. Use a named Stellar CLI identity.
- RPC URLs are hardcoded constants; review `RPC_MAINNET` before production use and replace with your own provider endpoint if needed.
- The script runs under `set -euo pipefail` — any unhandled error causes immediate exit with a structured log entry.

## Running Tests

```bash
bash scripts/deployment_shell_script.test.sh
```

All tests are self-contained (no external framework). Exit 0 means all pass.

## Dependencies

- `bash` >= 4.0
- `cargo` + `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
- `stellar` CLI >= 0.0.18
- `curl` (for network connectivity pre-check)
