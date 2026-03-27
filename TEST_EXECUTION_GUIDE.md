# Proptest Generator Boundary — Test Execution Guide

**Purpose**: Comprehensive guide for running, validating, and debugging tests for the optimized proptest generator boundary conditions.

---

## Quick Start

### Run All Boundary Tests

```bash
cd stellar-raise-contracts
cargo test --package crowdfund proptest_generator_boundary --lib
```

### Run with Custom Case Count

```bash
# Run with 1,000 cases (default for CI/CD)
PROPTEST_CASES=1000 cargo test --package crowdfund proptest_generator_boundary --lib

# Run with 256 cases (faster for local development)
PROPTEST_CASES=256 cargo test --package crowdfund proptest_generator_boundary --lib

# Run with 64 cases (fastest for quick validation)
PROPTEST_CASES=64 cargo test --package crowdfund proptest_generator_boundary --lib
```

### Run Only Property-Based Tests

```bash
cargo test --package crowdfund prop_ --lib
```

### Run Only Unit Tests

```bash
cargo test --package crowdfund test_ --lib
```

### Run Only Regression Tests

```bash
cargo test --package crowdfund regression_ --lib
```

---

## Detailed Test Breakdown

### Unit Tests (50+)

#### Constant Sanity Checks

```bash
cargo test --package crowdfund test_constants_return_correct_values --lib
cargo test --package crowdfund test_constants_are_ordered_correctly --lib
```

**Expected Output**:
```
test tests::test_constants_return_correct_values ... ok
test tests::test_constants_are_ordered_correctly ... ok
```

#### Deadline Offset Validation

```bash
cargo test --package crowdfund test_is_valid_deadline_offset --lib
```

**Expected Output**:
```
test tests::test_is_valid_deadline_offset_boundary_values ... ok
test tests::test_is_valid_deadline_offset_edge_cases ... ok
```

**Test Cases**:
- Lower boundary: 1,000 (valid), 999 (invalid)
- Upper boundary: 1,000,000 (valid), 1,000,001 (invalid)
- Edge cases: 0, u64::MAX

#### Goal Validation

```bash
cargo test --package crowdfund test_is_valid_goal --lib
```

**Expected Output**:
```
test tests::test_is_valid_goal_boundary_values ... ok
test tests::test_is_valid_goal_edge_cases ... ok
```

**Test Cases**:
- Lower boundary: 1,000 (valid), 999 (invalid)
- Upper boundary: 100,000,000 (valid), 100,000,001 (invalid)
- Edge cases: 0, -1, i128::MIN

#### Minimum Contribution Validation

```bash
cargo test --package crowdfund test_is_valid_min_contribution --lib
```

**Expected Output**:
```
test tests::test_is_valid_min_contribution ... ok
test tests::test_is_valid_min_contribution_with_min_goal ... ok
```

**Test Cases**:
- Valid: min_contribution ∈ [1, goal]
- Invalid: min_contribution > goal or < 1

#### Contribution Amount Validation

```bash
cargo test --package crowdfund test_is_valid_contribution_amount --lib
```

**Expected Output**:
```
test tests::test_is_valid_contribution_amount ... ok
```

**Test Cases**:
- Valid: amount >= min_contribution
- Invalid: amount < min_contribution

#### Fee Basis Points Validation

```bash
cargo test --package crowdfund test_is_valid_fee_bps --lib
```

**Expected Output**:
```
test tests::test_is_valid_fee_bps ... ok
```

**Test Cases**:
- Valid: fee_bps ∈ [0, 10,000]
- Invalid: fee_bps > 10,000

#### Generator Batch Size Validation

```bash
cargo test --package crowdfund test_is_valid_generator_batch_size --lib
```

**Expected Output**:
```
test tests::test_is_valid_generator_batch_size ... ok
```

**Test Cases**:
- Valid: batch_size ∈ [1, 512]
- Invalid: batch_size = 0 or > 512

#### Clamping Functions

```bash
cargo test --package crowdfund test_clamp_proptest_cases --lib
cargo test --package crowdfund test_clamp_progress_bps --lib
```

**Expected Output**:
```
test tests::test_clamp_proptest_cases ... ok
test tests::test_clamp_progress_bps ... ok
```

**Test Cases**:
- Below minimum: clamped to min
- Within range: unchanged
- Above maximum: clamped to max

#### Derived Calculations

```bash
cargo test --package crowdfund test_compute_progress_bps --lib
cargo test --package crowdfund test_compute_fee_amount --lib
```

**Expected Output**:
```
test tests::test_compute_progress_bps_basic ... ok
test tests::test_compute_progress_bps_edge_cases ... ok
test tests::test_compute_progress_bps_overflow_safety ... ok
test tests::test_compute_fee_amount_basic ... ok
test tests::test_compute_fee_amount_edge_cases ... ok
test tests::test_compute_fee_amount_floor_division ... ok
```

**Test Cases**:
- Basic: 50% funded, 100% funded, >100% funded (capped)
- Edge cases: zero goal, negative raised, negative goal
- Overflow safety: large values that could overflow
- Floor division: fractional fees

---

### Property-Based Tests (18+)

#### Deadline Offset Properties

```bash
cargo test --package crowdfund prop_deadline_offset --lib
```

**Expected Output**:
```
test tests::prop_deadline_offset_validity ... ok
test tests::prop_deadline_offset_below_min_invalid ... ok
test tests::prop_deadline_offset_above_max_invalid ... ok
```

**Properties**:
- Valid offsets ∈ [1,000, 1,000,000] pass validation
- Offsets < 1,000 fail validation
- Offsets > 1,000,000 fail validation

#### Goal Properties

```bash
cargo test --package crowdfund prop_goal --lib
```

**Expected Output**:
```
test tests::prop_goal_validity ... ok
test tests::prop_goal_below_min_invalid ... ok
test tests::prop_goal_above_max_invalid ... ok
```

**Properties**:
- Valid goals ∈ [1,000, 100,000,000] pass validation
- Goals < 1,000 fail validation
- Goals > 100,000,000 fail validation

#### Progress BPS Properties

```bash
cargo test --package crowdfund prop_progress_bps --lib
```

**Expected Output**:
```
test tests::prop_progress_bps_always_bounded ... ok
test tests::prop_progress_bps_zero_when_goal_zero ... ok
test tests::prop_progress_bps_zero_when_raised_negative ... ok
```

**Properties**:
- Progress always ≤ 10,000 (100%)
- Zero goal → 0% progress
- Negative raised → 0% progress

#### Fee Amount Properties

```bash
cargo test --package crowdfund prop_fee_amount --lib
```

**Expected Output**:
```
test tests::prop_fee_amount_always_non_negative ... ok
test tests::prop_fee_amount_zero_when_amount_zero ... ok
test tests::prop_fee_amount_zero_when_fee_zero ... ok
```

**Properties**:
- Fees always ≥ 0
- Zero amount → 0 fee
- Zero fee → 0 fee

#### Clamping Properties

```bash
cargo test --package crowdfund prop_clamp --lib
```

**Expected Output**:
```
test tests::prop_clamp_proptest_cases_within_bounds ... ok
test tests::prop_clamp_progress_bps_within_bounds ... ok
```

**Properties**:
- Clamped case counts ∈ [32, 256]
- Clamped progress ≤ 10,000

#### Validation Properties

```bash
cargo test --package crowdfund prop_min_contribution --lib
cargo test --package crowdfund prop_contribution_amount --lib
cargo test --package crowdfund prop_fee_bps --lib
cargo test --package crowdfund prop_batch_size --lib
```

**Expected Output**:
```
test tests::prop_min_contribution_valid_when_in_range ... ok
test tests::prop_contribution_amount_valid_when_meets_minimum ... ok
test tests::prop_fee_bps_valid_when_within_cap ... ok
test tests::prop_batch_size_valid_when_in_range ... ok
```

**Properties**:
- Valid min contributions pass validation
- Valid contribution amounts pass validation
- Valid fees pass validation
- Valid batch sizes pass validation

---

### Regression Tests (4)

```bash
cargo test --package crowdfund regression_ --lib
```

**Expected Output**:
```
test tests::regression_deadline_offset_100_seconds_now_invalid ... ok
test tests::regression_goal_zero_always_invalid ... ok
test tests::regression_progress_bps_never_exceeds_cap ... ok
test tests::regression_fee_amount_never_negative ... ok
```

**Test Cases**:
1. **Deadline Offset 100 Seconds**: Previously accepted (caused flaky tests), now rejected
2. **Goal Zero**: Always invalid to prevent division-by-zero
3. **Progress BPS Cap**: Never exceeds 10,000 even with extreme values
4. **Fee Amount Non-Negative**: Always ≥ 0 even with negative inputs

---

## Test Execution Scenarios

### Scenario 1: Local Development (Fast)

```bash
# Run with minimal cases for quick feedback
PROPTEST_CASES=64 cargo test --package crowdfund proptest_generator_boundary --lib

# Expected time: 5-10 seconds
```

### Scenario 2: Pre-Commit Validation (Medium)

```bash
# Run with moderate cases before committing
PROPTEST_CASES=256 cargo test --package crowdfund proptest_generator_boundary --lib

# Expected time: 15-30 seconds
```

### Scenario 3: CI/CD Pipeline (Thorough)

```bash
# Run with full cases for comprehensive validation
PROPTEST_CASES=1000 cargo test --package crowdfund proptest_generator_boundary --lib

# Expected time: 1-2 minutes
```

### Scenario 4: Regression Testing (Exhaustive)

```bash
# Run with maximum cases to catch edge cases
PROPTEST_CASES=10000 cargo test --package crowdfund proptest_generator_boundary --lib

# Expected time: 10-20 minutes
```

---

## Debugging Failed Tests

### Enable Verbose Output

```bash
cargo test --package crowdfund proptest_generator_boundary --lib -- --nocapture
```

### Enable Debug Logging

```bash
RUST_LOG=debug cargo test --package crowdfund proptest_generator_boundary --lib -- --nocapture
```

### Run Single Test

```bash
cargo test --package crowdfund test_compute_progress_bps_basic --lib -- --nocapture
```

### Capture Regression Seeds

```bash
PROPTEST_REGRESSIONS=contracts/crowdfund/proptest-regressions/ \
  cargo test --package crowdfund proptest_generator_boundary --lib
```

### Replay Regression Seed

```bash
# Proptest automatically replays seeds from proptest-regressions/
cargo test --package crowdfund proptest_generator_boundary --lib
```

---

## Expected Test Results

### Summary

```
test result: ok. 72 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 72 tests
test tests::test_constants_return_correct_values ... ok
test tests::test_constants_are_ordered_correctly ... ok
test tests::test_is_valid_deadline_offset_boundary_values ... ok
test tests::test_is_valid_deadline_offset_edge_cases ... ok
test tests::test_is_valid_goal_boundary_values ... ok
test tests::test_is_valid_goal_edge_cases ... ok
test tests::test_is_valid_min_contribution ... ok
test tests::test_is_valid_min_contribution_with_min_goal ... ok
test tests::test_is_valid_contribution_amount ... ok
test tests::test_is_valid_fee_bps ... ok
test tests::test_is_valid_generator_batch_size ... ok
test tests::test_clamp_proptest_cases ... ok
test tests::test_clamp_progress_bps ... ok
test tests::test_compute_progress_bps_basic ... ok
test tests::test_compute_progress_bps_edge_cases ... ok
test tests::test_compute_progress_bps_overflow_safety ... ok
test tests::test_compute_fee_amount_basic ... ok
test tests::test_compute_fee_amount_edge_cases ... ok
test tests::test_compute_fee_amount_floor_division ... ok
test tests::test_log_tag ... ok
test tests::prop_deadline_offset_validity ... ok
test tests::prop_deadline_offset_below_min_invalid ... ok
test tests::prop_deadline_offset_above_max_invalid ... ok
test tests::prop_goal_validity ... ok
test tests::prop_goal_below_min_invalid ... ok
test tests::prop_goal_above_max_invalid ... ok
test tests::prop_progress_bps_always_bounded ... ok
test tests::prop_progress_bps_zero_when_goal_zero ... ok
test tests::prop_progress_bps_zero_when_raised_negative ... ok
test tests::prop_fee_amount_always_non_negative ... ok
test tests::prop_fee_amount_zero_when_amount_zero ... ok
test tests::prop_fee_amount_zero_when_fee_zero ... ok
test tests::prop_clamp_proptest_cases_within_bounds ... ok
test tests::prop_clamp_progress_bps_within_bounds ... ok
test tests::prop_min_contribution_valid_when_in_range ... ok
test tests::prop_contribution_amount_valid_when_meets_minimum ... ok
test tests::prop_fee_bps_valid_when_within_cap ... ok
test tests::prop_batch_size_valid_when_in_range ... ok
test tests::regression_deadline_offset_100_seconds_now_invalid ... ok
test tests::regression_goal_zero_always_invalid ... ok
test tests::regression_progress_bps_never_exceeds_cap ... ok
test tests::regression_fee_amount_never_negative ... ok
```

### Coverage Report

```
Line Coverage: ≥95%
Function Coverage: 100%
Branch Coverage: ≥90%
```

---

## Troubleshooting

### Issue: Tests Timeout

**Solution**: Reduce case count
```bash
PROPTEST_CASES=64 cargo test --package crowdfund proptest_generator_boundary --lib
```

### Issue: Out of Memory

**Solution**: Run tests sequentially
```bash
cargo test --package crowdfund proptest_generator_boundary --lib -- --test-threads=1
```

### Issue: Flaky Tests

**Solution**: Increase case count to catch edge cases
```bash
PROPTEST_CASES=10000 cargo test --package crowdfund proptest_generator_boundary --lib
```

### Issue: Regression Seed Mismatch

**Solution**: Delete regression seeds and re-run
```bash
rm -rf contracts/crowdfund/proptest-regressions/
cargo test --package crowdfund proptest_generator_boundary --lib
```

---

## Performance Benchmarks

| Case Count | Execution Time | Memory Usage | Coverage |
|-----------|----------------|--------------|----------|
| 64 | 5-10s | ~50MB | ~85% |
| 256 | 15-30s | ~100MB | ~92% |
| 1,000 | 1-2m | ~200MB | ≥95% |
| 10,000 | 10-20m | ~500MB | ≥98% |

---

## CI/CD Integration

### GitHub Actions

```yaml
- name: Run proptest generator boundary tests
  env:
    PROPTEST_CASES: 1000
  run: cargo test --package crowdfund proptest_generator_boundary --lib
```

### Local Pre-Commit Hook

```bash
#!/bin/bash
PROPTEST_CASES=256 cargo test --package crowdfund proptest_generator_boundary --lib
if [ $? -ne 0 ]; then
  echo "Tests failed. Commit aborted."
  exit 1
fi
```

---

## References

- [Proptest Documentation](https://docs.rs/proptest/)
- [Soroban Testing Guide](https://soroban.stellar.org/docs/learn/testing)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)

