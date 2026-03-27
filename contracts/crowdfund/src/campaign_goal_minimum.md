# Campaign Goal Minimum Threshold Enforcement

## Overview
This documentation outlines the enforcement of a minimum campaign goal threshold in the smart contract. The goal is to prevent campaigns from being created with unrealistic or invalid funding targets.

## Purpose
- **Prevent Spam**: Discourages the creation of low-value campaigns that could clutter the platform.
- **Improve User Trust**: Ensures that all campaigns have a minimum feasible target.
- **Security**: Rejects zero-value or trivially small goals that could be exploited.
- **UX**: Provides clear feedback when a goal is invalid.

## Contract Logic
- **Minimum Threshold**: `MIN_CAMPAIGN_GOAL = 100`.
- **Validation**: Goals below `100` are rejected with a `panic!`.
- **Zero-Value Guard**: Goals equal to `0` are rejected.
- **Authentication**: Only authenticated creators can initiate a campaign.
- **Event Emission**: Publishes a `("campaign", "created")` event upon success.

## Usage
To create a campaign, call the `create_campaign` function with:
- `creator`: The address of the person starting the campaign.
- `goal`: The target funding amount (must be >= 100).

```rust
pub fn create_campaign(env: Env, creator: Address, goal: u64) { ... }
```

## Security Considerations
- **Authentication**: `creator.require_auth()` is called to ensure the request is authorized.
- **Validation**: Strict checks for `goal < MIN_CAMPAIGN_GOAL` and `goal == 0`.
- **Safe Storage**: Goal values are validated before any state changes are committed.

## Testing Process
Tests are provided in `campaign_goal_minimum.test.rs` and cover:
- Valid goal (>= 100).
- Below minimum goal (< 100).
- Zero goal.
- Exactly at minimum goal.
- Large boundary values (`u64::MAX`).
