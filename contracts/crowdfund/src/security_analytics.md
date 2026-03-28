# security_analytics

## Overview

`security_analytics` provides advanced on-chain security analytics and threat intelligence capabilities for the crowdfund contract ecosystem. This module enables real-time threat detection, anomaly analysis, access pattern monitoring, and security metrics aggregation.

Designed for integration with SIEM systems, automated threat response, and security audit trails, the module provides comprehensive visibility into contract security posture without requiring privileged access.

---

## Security Assumptions

1. **Read-only by default** — Analytics query functions do not mutate storage.
2. **Permissionless** — All analytics queries are accessible without `require_auth()` for integration with monitoring systems.
3. **Overflow-safe** — All arithmetic uses `checked_*` operations to prevent integer overflow.
4. **Bounded iteration** — All loops iterate over fixed or bounded sets (MAX_THREAT_LOG_SIZE, MAX_ACCESS_PATTERN_ENTRIES).
5. **Event-driven** — State changes emit structured events for audit trails and off-chain indexing.
6. **Non-blocking** — Analytics functions never revert transactions; results are advisory.
7. **Isolation** — Threat detection operates independently from contract business logic.

---

## Constants

| Constant                     | Value | Description                                          |
|------------------------------|-------|------------------------------------------------------|
| `MAX_THREAT_LOG_SIZE`        | 1000  | Maximum entries in threat log before rotation.      |
| `MAX_ACCESS_PATTERN_ENTRIES` | 500   | Maximum access pattern entries tracked per entity.   |

---

## Types

### ThreatSeverity

```rust
pub enum ThreatSeverity {
    Low,
    Medium,
    High,
    Critical,
}
```

Severity levels for detected threats:

| Variant   | Priority | `is_critical()` | Description                              |
|-----------|----------|-----------------|------------------------------------------|
| `Low`     | 1        | false           | Informational / monitoring only.        |
| `Medium`  | 2        | false           | Requires attention but not urgent.      |
| `High`    | 3        | true            | Significant concern, investigate soon.   |
| `Critical`| 4        | true            | Immediate action required.               |

### ThreatType

```rust
pub enum ThreatType {
    AbnormalContributionPattern,
    RepeatedFailure,
    SuspiciousAddressActivity,
    RapidStateChanges,
    AbnormalWithdrawalPattern,
    AuthAnomaly,
    StorageAnomaly,
    RateLimitExceeded,
    AccessPatternAnomaly,
}
```

Types of security threats detected by the analytics system.

### MetricType

```rust
pub enum MetricType {
    TotalContributions,
    TotalWithdrawals,
    TotalRefunds,
    FailedTransactions,
    UniqueContributors,
    AverageContributionSize,
    LargeTransactions,
    RevertedTransactions,
    AuthFailures,
    RateLimitTriggers,
}
```

Security metrics tracked for threat detection and analysis.

### ThreatRecord

```rust
pub struct ThreatRecord {
    pub timestamp: u64,
    pub threat_type: ThreatType,
    pub severity: ThreatSeverity,
    pub address: Option<Address>,
    pub details: Symbol,
    pub acknowledged: bool,
}
```

A single threat event record stored in the threat log.

### SecuritySummary

```rust
pub struct SecuritySummary {
    pub threat_count: u32,
    pub critical_threats: u32,
    pub total_transactions: u32,
    pub failure_rate_bps: u32,
    pub unique_addresses: u32,
    pub avg_transaction_size: i128,
    pub security_score: u8,
}
```

Aggregated security analytics for monitoring dashboards.

### AccessPatternEntry

```rust
pub struct AccessPatternEntry {
    pub timestamp: u64,
    pub action: Symbol,
    pub success: bool,
    pub value: i128,
}
```

An individual access record for behavioral analysis.

### AnomalyReport

```rust
pub struct AnomalyReport {
    pub anomaly_detected: bool,
    pub anomaly_type: Symbol,
    pub confidence: u8,
    pub description: Symbol,
}
```

Result of anomaly detection analysis.

---

## Threat Detection Functions

All detection functions are read-only and require no authorization.

### `detect_contribution_anomaly(env, contributor, amount) -> AnomalyReport`

Analyzes contribution patterns to detect anomalies:

| Condition                      | Result                                    |
|--------------------------------|------------------------------------------|
| Amount > 10,000 units          | `LARGE_CONTRIB` anomaly, 75% confidence  |
| >5 contributions/hour from addr| `RAPID_CONTRIB` anomaly, 85% confidence |
| Normal pattern                 | No anomaly detected                      |

### `detect_withdrawal_anomaly(env, withdrawer, amount) -> AnomalyReport`

Analyzes withdrawal patterns:

| Condition                       | Result                                     |
|---------------------------------|-------------------------------------------|
| Amount > 5,000 units            | `LARGE_WITHDRAW` anomaly, 80% confidence |
| >3 withdrawals/30min from addr | `RAPID_WITHDRAW` anomaly, 70% confidence |
| Normal pattern                  | No anomaly detected                       |

### `detect_auth_anomaly(env, address) -> AnomalyReport`

Detects authorization-related anomalies:

| Condition                       | Result                                     |
|---------------------------------|-------------------------------------------|
| >10 auth failures from addr     | `AUTH_FAILURES` anomaly, 90% confidence   |
| >5 admin actions/5min          | `ADMIN_BURST` anomaly, 75% confidence    |
| Normal pattern                  | No anomaly detected                       |

### `detect_storage_anomaly(env) -> AnomalyReport`

Detects storage-related anomalies:

| Condition                              | Result                                 |
|----------------------------------------|----------------------------------------|
| Active status past deadline           | `EXPIRED_ACTIVE` anomaly, 95% confidence |
| Cancelled campaign with funds         | `CANCELLED_FUNDS` anomaly, 85% confidence |
| Normal state                           | No anomaly detected                    |

---

## Access Pattern Functions

### `record_access_pattern(env, address, action, success, value)`

Records an access pattern entry for behavioral analysis. Emits `access_recorded` event.

### `get_access_pattern(env, address) -> Vec<AccessPatternEntry>`

Retrieves the access pattern for an address (up to MAX_ACCESS_PATTERN_ENTRIES entries).

### `analyze_access_pattern(env, address) -> (u32, u32, u64)`

Computes statistics for an address:
- Returns: `(total_actions, success_rate_bps, last_activity_timestamp)`
- `success_rate_bps` is the percentage in basis points (10000 = 100%)

---

## Threat Logging Functions

### `log_threat(env, threat_type, severity, address, details) -> u32`

Records a detected threat. Emits `threat_logged` event. Returns the log index.

### `get_threat_log(env, limit) -> Vec<ThreatRecord>`

Retrieves threat log entries (most recent first), up to the specified limit.

### `acknowledge_threat(env, index) -> bool`

Marks a threat as acknowledged. Emits `threat_ack` event.

---

## Security Metrics Functions

### `increment_metric(env, address, metric, delta)`

Increments a security metric counter. Use `None` for address for global metrics.

### `get_metric(env, address, metric) -> u32`

Retrieves the current value of a security metric.

### `get_security_summary(env) -> SecuritySummary`

Generates comprehensive security analytics including:

- Threat counts and critical threat counts
- Transaction statistics
- Failure rate (basis points)
- Unique address count
- Average transaction size
- **Security score** (0-100, higher is better)

### Security Score Calculation

```
Base: 100
Deductions:
- High failure rate (>1%): -30 max
- Auth failures (>5): -20 max
- Critical threats: -20 max
- Low transaction history: -10 max

Range: 0-100
```

---

## Rate Limiting Functions

### `check_rate_limit(env, address, action, window_secs, max_count) -> (bool, u32, u32)`

Checks if an address has exceeded rate limits:

| Return Value       | Description                                |
|-------------------|-------------------------------------------|
| `within_limit`    | `true` if under limit, `false` if exceeded|
| `current_count`   | Current action count in window            |
| `limit`           | The configured maximum                    |

### `record_rate_limit_violation(env, address, action)`

Records a rate limit violation as a threat and increments the violation metric.

---

## Threat Intelligence Functions

### `analyze_threat_trends(env, hours) -> (u32, ThreatSeverity)`

Analyzes threat patterns over a time window:
- Returns threat count and highest severity level
- Useful for detecting attack campaigns

### `get_blocked_addresses(env, severity) -> Vec<Address>`

Returns addresses flagged with threats at or above the specified severity. Useful for:
- External monitoring integration
- Blocklist generation
- Incident response

---

## Utility Functions

### `format_threat_severity(severity) -> &'static str`

Returns human-readable severity: `"LOW"`, `"MEDIUM"`, `"HIGH"`, `"CRITICAL"`.

### `format_threat_type(threat_type) -> &'static str`

Returns human-readable threat type string for logging.

### `get_risk_level(summary) -> &'static str`

Computes overall risk level from security summary:

| Security Score | Threats      | Risk Level |
|---------------|--------------|------------|
| ≥80          | 0 critical   | `LOW`      |
| ≥60          | <3 critical  | `MEDIUM`   |
| ≥40          | <10 critical | `HIGH`     |
| <40          | ≥10 critical | `CRITICAL` |

---

## Event Schema

| Event Name          | Topics                          | Data                                    |
|--------------------|--------------------------------|----------------------------------------|
| `access_recorded`  | `(access_recorded,)`           | `(address, timestamp, success)`        |
| `threat_logged`    | `(threat_logged, threat)`      | `(threat_type, severity_priority)`     |
| `threat_ack`       | `(threat_ack,)`                | `(index)`                              |

---

## Integration Examples

### Security Dashboard Integration

```rust
// Query security summary for dashboard
let summary = get_security_summary(&env);
if summary.security_score < 50 {
    // Alert security team
}
```

### SIEM Integration

```rust
// Export threat log to SIEM
let threats = get_threat_log(&env, 1000);
for threat in threats.iter() {
    // Send to SIEM via external call
}
```

### Automated Response

```rust
// Check for blocked addresses
let blocked = get_blocked_addresses(&env, ThreatSeverity::High);
if blocked.len() > 0 {
    // Trigger automated response
}
```

---

## Security Considerations

1. **Event Ordering** — Events are emitted after state is computed to ensure consistency.
2. **Storage Bounds** — Log rotation prevents unbounded storage growth.
3. **No Reverts** — Analytics functions return advisory data and never revert.
4. **Permissionless Queries** — Enables integration without privileged access.
5. **Deterministic Results** — Same state always produces same analytics.

---

## Test Coverage

Comprehensive test coverage includes:

- ThreatSeverity enum methods (priority, is_critical)
- All ThreatType variants
- Anomaly detection functions (contribution, withdrawal, auth, storage)
- Access pattern recording and retrieval
- Threat logging and rotation
- Security metrics increment and retrieval
- Security summary generation
- Rate limit checking
- Threat trend analysis
- Blocked addresses retrieval
- Utility functions
- Edge cases (overflow, empty states, boundary conditions)

---

## Related Modules

- [`security_compliance_automation`](security_compliance_automation.md) — Compliance checking for contract invariants
- [`role_based_access`](role_based_access.md) — Authorization and access control
- [`access_control`](access_control.md) — Role-based access control implementation
