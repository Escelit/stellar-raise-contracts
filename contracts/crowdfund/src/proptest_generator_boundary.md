# Proptest Generator Boundary Contract

The `ProptestGeneratorBoundary` contract serves as the central authority for all boundary conditions and validation constants used in property-based testing and input validation for the Stellar Raise platform.

## Rationale

Property-based tests (proptests) can generate a wide range of inputs. Without strictly defined boundaries, tests may:
- Face division-by-zero errors in progress calculations.
- Generate unrealistic campaign durations that don't reflect frontend UI behaviors.
- Experience integer overflows if boundary values are not capped.

By exposing these constants via a contract, off-chain scripts can dynamically retrieve the current platform limits, improving the consistency between tests and deployment environments.

## Features

- **Queryable Limits**: Get current `min`/`max` values for goals, deadlines, and contribution floors.
- **Input Validation**: Call `is_valid_deadline_offset` or `is_valid_goal` to check parameters against platform standards.
- **Safety Computations**: Provides `compute_progress_bps` to ensure consistent progress reporting (capped at 100 %).
- **Test Management**: Helper functions like `clamp_proptest_cases` keep CI runtimes within manageable limits.

## Contract Functions

### `deadline_offset_min()` / `deadline_offset_max()`
Returns the accepted range for campaign duration (in seconds).

### `goal_min()` / `goal_max()`
Returns the accepted range for funding targets.

### `compute_progress_bps(raised, goal)`
Calculates basis points (0 to 10,000) for campaign progress. Handles edge cases like zero goal or over-funding automatically.

### `is_valid_goal(goal)`
Returns `true` if the goal is within safe operating limits.

## Integration

Used by the Crowdfund test suite and external deployment scripts to validate inputs before submitting transactions.
