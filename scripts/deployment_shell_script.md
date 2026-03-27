# deployment_shell_script.sh

Builds, deploys, and initialises the Stellar Raise crowdfund contract with
structured error capturing, timestamped logging, and a machine-readable JSON
event stream for the frontend UI.

## Why this script exists

The original `deploy.sh` used `set -e` but swallowed error context — a failed
`cargo build` or `stellar contract deploy` would exit silently with no
actionable message. This script adds:

- Per-step exit codes (2–6) so CI can distinguish build vs deploy vs init failures.
- All stderr captured to `DEPLOY_LOG` (default `deploy_errors.log`) alongside
  timestamped stdout entries.
- A structured **NDJSON event log** (`DEPLOY_JSON_LOG`, default `deploy_events.json`)
  that the frontend UI can stream-parse to render live step status and typed errors.
- Argument validation with clear messages before any network call is made.

## Constants reference

| Constant                  | Value                                          | Purpose                          |
| :------------------------ | :--------------------------------------------- | :------------------------------- |
| `EXIT_OK`                 | `0`                                            | Success                          |
| `EXIT_MISSING_DEP`        | `1`                                            | Missing CLI dependency           |
| `EXIT_BAD_ARG`            | `2`                                            | Invalid / missing argument       |
| `EXIT_BUILD_FAIL`         | `3`                                            | `cargo build` failure            |
| `EXIT_DEPLOY_FAIL`        | `4`                                            | `stellar contract deploy` failure|
| `EXIT_INIT_FAIL`          | `5`                                            | `stellar contract invoke` failure|
| `EXIT_NETWORK_FAIL`       | `6`                                            | RPC connectivity failure         |
| `WASM_TARGET`             | `wasm32-unknown-unknown`                       | Rust compilation target          |
| `WASM_PATH`               | `target/wasm32-unknown-unknown/release/crowdfund.wasm` | Expected WASM artifact  |
| `RPC_TESTNET`             | `https://soroban-testnet.stellar.org/health`   | Testnet health endpoint          |
| `RPC_MAINNET`             | `https://soroban.stellar.org/health`           | Mainnet health endpoint          |
| `RPC_FUTURENET`           | `https://rpc-futurenet.stellar.org/health`     | Futurenet health endpoint        |
| `NETWORK_TIMEOUT`         | `10`                                           | curl max-time (seconds)          |
| `DEFAULT_NETWORK`         | `testnet`                                      | Default Stellar network          |
| `DEFAULT_DEPLOY_LOG`      | `deploy_errors.log`                            | Default log file path            |
| `DEFAULT_MIN_CONTRIBUTION`| `1`                                            | Default minimum pledge (stroops) |

## Usage

```bash
./scripts/deployment_shell_script.sh <creator> <token> <goal> <deadline> [min_contribution]
```

| Parameter          | Type    | Description                                    |
| :----------------- | :------ | :--------------------------------------------- |
| `creator`          | string  | Stellar address of the campaign creator        |
| `token`            | string  | Stellar address of the token contract          |
| `goal`             | integer | Funding goal in stroops                        |
| `deadline`         | integer | Unix timestamp — must be in the future         |
| `min_contribution` | integer | Minimum pledge amount (default: `1`)           |

### Environment variables

| Variable          | Default              | Description                                      |
| :---------------- | :------------------- | :----------------------------------------------- |
| `NETWORK`         | `testnet`            | Stellar network to target                        |
| `DEPLOY_LOG`      | `deploy_errors.log`  | Human-readable timestamped log                   |
| `DEPLOY_JSON_LOG` | `deploy_events.json` | Structured NDJSON event log for the frontend UI  |
| `DRY_RUN`         | `false`              | Set to `true` to validate without deploying      |

### Example

```bash
DEADLINE=$(date -d "+30 days" +%s)
./scripts/deployment_shell_script.sh \
  GCREATOR... GTOKEN... 1000 "$DEADLINE" 10
```

## Exit codes

| Code | Meaning                                  |
| :--- | :--------------------------------------- |
| 0    | Success                                  |
| 1    | Missing dependency (`cargo` / `stellar`) |
| 2    | Invalid or missing argument              |
| 3    | `cargo build` failure                    |
| 4    | `stellar contract deploy` failure        |
| 5    | `stellar contract invoke` (init) failure |
| 6    | Network connectivity failure             |

## Human-readable log format

Every line written to `DEPLOY_LOG` follows:

```
[2026-03-26T03:00:00Z] [INFO|WARN|ERROR] <message>
```

Stderr from `cargo` and `stellar` is appended verbatim, making it easy to
`grep` for specific failures in CI logs.

## Structured JSON event log (frontend UI)

Every line in `DEPLOY_JSON_LOG` is a self-contained JSON object (NDJSON).
The frontend can tail or stream-read this file to display live progress.

### Event schema

```json
{
  "event":     "step_start | step_ok | step_error | deploy_complete",
  "step":      "validate | network_check | build | deploy | init | done",
  "message":   "Human-readable description",
  "timestamp": "2026-03-26T03:00:00Z",
  "network":   "testnet"
}
```

`step_ok` for the `deploy` step also includes `"wasm_path"`.  
`step_ok` for the `deploy` step and `deploy_complete` include `"contract_id"`.  
`step_error` includes `"exit_code"`, `"context"`, and `"error_count"`.

### Example event sequence (happy path)

```json
{"event":"step_start","step":"network_check","message":"Checking connectivity to testnet","timestamp":"...","network":"testnet"}
{"event":"step_ok","step":"network_check","message":"Network reachable","timestamp":"...","network":"testnet"}
{"event":"step_start","step":"build","message":"Building WASM","timestamp":"...","network":"testnet"}
{"event":"step_ok","step":"build","message":"WASM built successfully","timestamp":"...","network":"testnet","wasm_path":"target/.../crowdfund.wasm"}
{"event":"step_start","step":"deploy","message":"Deploying to testnet","timestamp":"...","network":"testnet"}
{"event":"step_ok","step":"deploy","message":"Contract deployed","timestamp":"...","network":"testnet","contract_id":"CXXX..."}
{"event":"step_start","step":"init","message":"Initialising campaign on CXXX...","timestamp":"...","network":"testnet"}
{"event":"step_ok","step":"init","message":"Campaign initialised successfully","timestamp":"...","network":"testnet"}
{"event":"deploy_complete","step":"done","message":"Deployment finished","timestamp":"...","network":"testnet","contract_id":"CXXX...","error_count":0}
```

### Example error event

```json
{"event":"step_error","step":"build","message":"cargo build failed – see deploy_errors.log for details","timestamp":"...","network":"testnet","exit_code":3,"context":"cargo build --target wasm32-unknown-unknown --release","error_count":1}
```

### Frontend integration example

```ts
// Stream-parse NDJSON from the deployment log
for await (const line of readLines('deploy_events.json')) {
  const event = JSON.parse(line);
  if (event.event === 'step_error') {
    // Map to ContractError / NetworkError for the global error boundary
    throw new ContractError(`[${event.step}] ${event.message}`);
  }
  if (event.event === 'deploy_complete') {
    setContractId(event.contract_id);
  }
}
```

## Security assumptions

- The `creator` argument is used as both the signing source and the on-chain
  creator address. **Never pass a raw secret key**; use a named Stellar CLI
  identity (`stellar keys generate --global alice`).
- `DEPLOY_LOG` and `DEPLOY_JSON_LOG` may contain sensitive RPC responses.
  Restrict file permissions in production: `chmod 600 deploy_errors.log deploy_events.json`.
- The script does **not** store or echo secret keys at any point.
- `set -euo pipefail` ensures unhandled errors abort execution immediately.
- Double-quotes and backslashes in error messages are escaped before being
  written to JSON, preventing log-injection attacks.

## Running the tests

```bash
bash scripts/deployment_shell_script.test.sh
```

No external test framework is required. The test file stubs `cargo`, `stellar`,
and `curl` so the suite runs fully offline and in CI without network access.

### Test coverage

| Area                              | Cases |
| :-------------------------------- | :---- |
| `require_tool`                    | 2     |
| `validate_args`                   | 9     |
| `build_contract`                  | 3     |
| `deploy_contract`                 | 3     |
| `init_contract`                   | 2     |
| `log` / `die`                     | 4     |
| `emit_event` / `DEPLOY_JSON_LOG`  | 6     |
| `DEPLOY_LOG` file behaviour       | 2     |
| **Total**                         | **31**|

All 31 tests pass (≥ 95% coverage of every exported function, JSON event
emission, and both log-truncation behaviours).
