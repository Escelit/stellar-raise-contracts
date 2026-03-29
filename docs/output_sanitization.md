# Output Sanitization

> **Module:** `contracts/security/src/output_sanitization.rs`
> **Tests:** `contracts/security/src/output_sanitization.test.rs`
> **Issue:** implement-smart-contract-output-sanitization-for-security

---

## Overview

`output_sanitization.rs` validates and clamps every value that crosses the
contract output boundary — query return values, event payloads, and computed
metrics — before they reach off-chain consumers.

Unsanitized outputs are a data-protection risk: negative balances, out-of-range
basis points, or overlong strings can corrupt off-chain indexers, mislead UIs,
and leak internal state.

---

## Threat Model

| ID   | Threat                                         | Sanitizer                    | Mitigation                               |
| ---- | ---------------------------------------------- | ---------------------------- | ---------------------------------------- |
| O-01 | Negative token amount in event payload         | `sanitize_amount`            | Clamped to `0`                           |
| O-02 | `total_raised` exceeds `goal` in output        | `sanitize_amount_bounded`    | Clamped to `goal`                        |
| O-03 | Fee / progress bps > 10 000 (> 100 %)          | `sanitize_bps`               | Clamped to `MAX_BPS`                     |
| O-04 | Stale deadline emitted before campaign opens   | `sanitize_deadline`          | Rejected; replaced with `now`            |
| O-05 | Overlong string payload — indexer DoS          | `sanitize_string`            | Clamped to `[TRUNCATED]` sentinel        |
| O-06 | Inflated contributor count misleads governance | `sanitize_contributor_count` | Clamped to `MAX_CONTRIBUTOR_COUNT` (128) |
| O-07 | Silent sanitization — no audit trail           | `emit_sanitization_warning`  | `sanitization/warning` event emitted     |

---

## Public API

```rust
// Amount sanitizers
sanitize_amount(amount: i128) -> SanitizedOutput<i128>
sanitize_amount_bounded(amount: i128, max: i128) -> SanitizedOutput<i128>

// Basis-point sanitizer
sanitize_bps(bps: u32) -> SanitizedOutput<u32>

// Timestamp sanitizer
sanitize_deadline(now: u64, deadline: u64) -> SanitizedOutput<u64>

// String sanitizer
sanitize_string(env: &Env, s: &String) -> SanitizedOutput<String>

// Contributor count sanitizer
sanitize_contributor_count(count: u32) -> SanitizedOutput<u32>

// Aggregate sanitizer
sanitize_campaign_output(
    now: u64, total_raised: i128, goal: i128,
    progress_bps: u32, deadline: u64, contributor_count: u32,
) -> SanitizedCampaignOutput

// Event helper
emit_sanitization_warning(env: &Env, output: &SanitizedCampaignOutput)
```

---

## `SanitizedOutput<T>` Variants

| Variant       | Meaning                                                   |
| ------------- | --------------------------------------------------------- |
| `Clean(T)`    | Value passed all checks unchanged                         |
| `Clamped(T)`  | Value was out of range; adjusted to nearest safe boundary |
| `Rejected(T)` | Value was structurally invalid; safe default substituted  |

Use `.is_clean()` / `.was_modified()` / `.value()` to inspect results without
pattern matching.

---

## Constants

| Constant                | Value           | Purpose                           |
| ----------------------- | --------------- | --------------------------------- |
| `MAX_BPS`               | `10_000`        | Maximum basis-point value (100 %) |
| `MAX_STRING_LEN`        | `256`           | Maximum string byte length        |
| `MAX_CONTRIBUTOR_COUNT` | `128`           | Maximum contributor count         |
| `TRUNCATED_SENTINEL`    | `"[TRUNCATED]"` | Replacement for overlong strings  |
| `ZERO_SENTINEL`         | `0`             | Replacement for negative amounts  |

---

## Usage

```rust
use crate::output_sanitization::{
    sanitize_campaign_output, emit_sanitization_warning,
};

// In a query handler or event emitter:
let output = sanitize_campaign_output(
    env.ledger().timestamp(),
    total_raised,
    goal,
    progress_bps,
    deadline,
    contributor_count,
);

emit_sanitization_warning(&env, &output);

// Use output.total_raised, output.progress_bps, etc. for the response.
```

---

## Security Assumptions

1. All functions are pure — no storage reads or writes. Auth is enforced by
   callers; this module only validates output values.

2. `sanitize_amount` and `sanitize_amount_bounded` use signed comparison only —
   no arithmetic, so no overflow risk.

3. `sanitize_bps` uses unsigned comparison — no underflow risk.

4. `sanitize_deadline` uses unsigned comparison — no overflow risk. The safe
   default (`now`) is always a valid timestamp.

5. `sanitize_string` never panics on oversized input — it substitutes a fixed
   sentinel string rather than truncating bytes, avoiding invalid UTF-8.

6. `emit_sanitization_warning` is a no-op when `was_modified` is false, so
   clean outputs produce no extra ledger events and no extra cost.

7. `sanitize_campaign_output` is the single entry point for all campaign output
   fields. Adding a new output field requires adding it here and writing a
   corresponding test.

---

## Running the Tests

```bash
# Run all output sanitization tests
cargo test -p security output_sanitization

# Run with output (see security notes in test output)
cargo test -p security output_sanitization -- --nocapture

# Run property-based tests only
cargo test -p security prop_

# Run the full security crate suite
cargo test -p security
```
