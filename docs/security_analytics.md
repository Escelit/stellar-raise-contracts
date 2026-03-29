# security_analytics

On-chain threat intelligence and security analytics for the Stellar Raise crowdfund contract.

## Overview

`security_analytics.rs` provides a suite of **read-only** threat detectors that can be called by automated tooling (CI pipelines, monitoring bots, SIEM integrations) to surface anomalous patterns in the contract's on-chain state.

Every detector emits a structured Soroban event so off-chain indexers can build a tamper-evident threat log without re-reading raw storage.

## Module Location

```
contracts/crowdfund/src/security_analytics.rs
contracts/crowdfund/src/security_analytics.test.rs
```

## Security Assumptions

| # | Assumption |
|---|-----------|
| 1 | **Read-only** — No function in this module writes to ledger storage. |
| 2 | **No auth required** — Detectors are permissionless so automated tooling does not need a privileged key. |
| 3 | **Overflow-safe** — All arithmetic uses `checked_*` operations; results saturate at `i128::MAX` / `u32::MAX` rather than panicking. |
| 4 | **Bounded iteration** — Functions that iterate contributors are bounded by `MAX_CONTRIBUTORS` (defined in `contract_state_size`). |
| 5 | **Deterministic** — Same ledger state → same result; no side-effects beyond event emission. |
| 6 | **Event integrity** — Events are emitted *after* the result is computed so a panicking host never emits a misleading event. |

## Public API

### Types

#### `ThreatSignal`

```rust
pub enum ThreatSignal {
    Clean,
    Alert(&'static str),
}
```

- `is_clean() -> bool` — `true` when no threat was detected.
- `description() -> &'static str` — Returns the alert message, or `""` when clean.

#### `AnalyticsReport`

```rust
pub struct AnalyticsReport {
    pub clean_count: u32,
    pub alert_count: u32,
    pub threat_free: bool,
}
```

Aggregated result of `run_security_scan`. `clean_count + alert_count` equals the number of checks that ran (5 mandatory + 1 optional whale check when contributor data is sufficient).

### Detectors

#### `detect_whale_concentration(env) -> Option<ThreatSignal>`

Detects whether any single contributor holds more than `WHALE_THRESHOLD_BPS` (50 %) of `total_raised`.

Returns `None` when there are fewer than `MIN_CONTRIBUTORS_FOR_CONCENTRATION` (2) contributors — analysis is not meaningful on a single-contributor campaign.

**Threat signal:** `"Whale concentration: single contributor exceeds 50% of total raised"`

#### `detect_overfunding(env) -> ThreatSignal`

Detects whether `total_raised > goal × OVERFUND_FACTOR` (3×). Over-funding beyond 3× the goal may indicate wash-trading or a misconfigured goal.

**Threat signal:** `"Over-funding anomaly: total raised exceeds 3x the campaign goal"`

#### `detect_deadline_manipulation(env) -> ThreatSignal`

Detects whether the campaign status is `Active` but the deadline has already passed. This inconsistency means `finalize()` has not been called and contributions are blocked by the deadline guard while the campaign appears active.

**Threat signal:** `"Deadline manipulation: campaign is Active but deadline has already passed"`

#### `detect_zero_contribution_spam(env) -> ThreatSignal`

Detects contributor list entries with a zero or negative recorded contribution. These bloat the list and inflate gas costs for `cancel()` and `collect_pledges()`.

**Threat signal:** `"Zero-contribution spam: contributor list contains address with zero balance"`

#### `detect_paused_with_active_status(env) -> ThreatSignal`

Detects the inconsistent state where the contract is paused but the campaign status is still `Active`. This means the campaign is stuck — contributors cannot contribute and the deadline may expire without finalization.

**Threat signal:** `"Stuck campaign: contract is paused while campaign status is Active"`

#### `detect_total_raised_exceeds_sum_of_contributions(env) -> ThreatSignal`

Detects whether `TotalRaised` is greater than the sum of all individual `Contribution(addr)` values. A discrepancy indicates an arithmetic bug or storage corruption.

**Threat signal:** `"Accounting mismatch: TotalRaised exceeds sum of individual contributions"`

### Aggregate Scan

#### `run_security_scan(env) -> AnalyticsReport`

Runs all detectors in order of increasing storage cost and returns an aggregated `AnalyticsReport`. Emits one `security_analytics` event per detector and a final `analytics_summary` event.

```rust
let report = run_security_scan(&env);
if !report.threat_free {
    // handle alerts
}
```

### Targeted Helpers

#### `is_financially_sound(env) -> bool`

Quick check: no overfunding **and** no accounting mismatch.

#### `is_campaign_state_consistent(env) -> bool`

Quick check: deadline and pause-state are consistent with campaign status.

## Thresholds (configurable constants)

| Constant | Default | Description |
|----------|---------|-------------|
| `WHALE_THRESHOLD_BPS` | `5_000` (50 %) | Single-contributor concentration limit |
| `MIN_CONTRIBUTORS_FOR_CONCENTRATION` | `2` | Minimum contributors before whale check runs |
| `OVERFUND_FACTOR` | `3` | Multiplier above goal that triggers overfunding alert |
| `MIN_RAISED_FOR_VELOCITY` | `0` | Minimum `total_raised` before whale check runs |

## Events Emitted

| Topic | Data | Description |
|-------|------|-------------|
| `("security_analytics", "<detector_name>")` | `bool` (is_clean) | Per-detector result |
| `("analytics_summary",)` | `(u32, u32)` (clean, alert) | Aggregate counts |

## Usage Examples

### Run a full security scan (Rust)

```rust
use crate::security_analytics::run_security_scan;

let report = run_security_scan(&env);
assert!(report.threat_free, "security scan found {} alerts", report.alert_count);
```

### Run a targeted financial check (Rust)

```rust
use crate::security_analytics::is_financially_sound;

if !is_financially_sound(&env) {
    panic!("financial invariant violated");
}
```

### Invoke from Stellar CLI (simulation — read-only)

```bash
# Full scan — emits events, returns AnalyticsReport
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <YOUR_SECRET_KEY> \
  -- run_security_scan
```

## Test Coverage

Tests are in `security_analytics.test.rs` and cover:

- All `ThreatSignal` helper methods.
- Every detector: happy path + all failure branches + boundary values.
- `run_security_scan`: clean state, each individual alert, count consistency, optional whale check inclusion/exclusion.
- `is_financially_sound` and `is_campaign_state_consistent` helpers.
- Edge cases: empty state, zero goal, single contributor, paused/active inconsistency, accounting mismatch.

Run with:

```bash
cargo test --workspace -- security_analytics
```

Expected output: all tests pass with no warnings.

## Integration with CI

Add to `.github/workflows/rust_ci.yml`:

```yaml
- name: Security analytics tests
  run: cargo test --workspace -- security_analytics --nocapture
```

## Related Modules

- `security_compliance_automation` — Static compliance checks (admin initialized, fee limits, etc.)
- `access_control` — Role separation and pause/unpause logic.
- `contract_state_size` — Contributor and storage size limits referenced by detectors.
