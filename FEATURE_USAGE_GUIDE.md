# Individual Contribution Limit - Usage Guide

## Overview
The individual contribution limit feature prevents any single address from contributing more than a specified amount to a crowdfunding campaign. This helps ensure wider distribution of supporters and prevents "whale" dominance.

## When to Use

### Use Cases:
- **Community-focused campaigns** - Ensure broad participation
- **Fair distribution** - Prevent single large donors from dominating
- **Regulatory compliance** - Meet contribution limit requirements
- **Risk management** - Limit exposure to single contributors

### When NOT to Use:
- Campaigns seeking large institutional investors
- Projects that benefit from whale contributions
- When maximum flexibility is desired

## API Reference

### Initialize with Limit

```rust
pub fn initialize(
    env: Env,
    creator: Address,
    token: Address,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    max_individual_contribution: Option<i128>,  // NEW PARAMETER
    platform_config: Option<PlatformConfig>,
) -> Result<(), ContractError>
```

**Parameters:**
- `max_individual_contribution`: Optional maximum amount any single address can contribute
  - `None` - No limit (default behavior)
  - `Some(amount)` - Set specific limit

**Validation Rules:**
1. If set, must be positive (> 0)
2. If set, must be >= `min_contribution`
3. Enforced cumulatively across all contributions from same address

### View Helper

```rust
pub fn max_individual_contribution(env: Env) -> Option<i128>
```

Returns the configured limit or `None` if no limit is set.

### Error Handling

```rust
ContractError::IndividualLimitExceeded = 6
```

Returned when a contribution would cause the contributor's cumulative total to exceed the limit.

## Examples

### Example 1: Campaign with 500K Limit

```rust
// Initialize campaign with 500,000 token limit per contributor
client.initialize(
    &creator,
    &token_address,
    &1_000_000,           // goal
    &deadline,
    &1_000,               // min_contribution
    &Some(500_000),       // max_individual_contribution
    &None,                // platform_config
);

// Alice contributes 300K - succeeds
client.contribute(&alice, &300_000);

// Alice tries to contribute another 250K - fails (would total 550K)
let result = client.try_contribute(&alice, &250_000);
assert_eq!(result.unwrap_err().unwrap(), ContractError::IndividualLimitExceeded);

// Alice can contribute up to 200K more (500K - 300K = 200K)
client.contribute(&alice, &200_000);  // Now at exactly 500K
```

### Example 2: Campaign without Limit

```rust
// Initialize without limit - traditional behavior
client.initialize(
    &creator,
    &token_address,
    &10_000_000,
    &deadline,
    &1_000,
    &None,  // No max limit
    &None,
);

// Any contributor can contribute any amount (above minimum)
client.contribute(&whale, &5_000_000);  // Succeeds
client.contribute(&whale, &5_000_000);  // Also succeeds
```

### Example 3: Checking Current Limit

```rust
// Check if campaign has a limit
match client.max_individual_contribution() {
    Some(limit) => println!("Max per contributor: {}", limit),
    None => println!("No contribution limit"),
}

// Check contributor's remaining capacity
let current = client.contribution(&contributor);
if let Some(limit) = client.max_individual_contribution() {
    let remaining = limit - current;
    println!("Can contribute {} more", remaining);
}
```

## Best Practices

### 1. Set Appropriate Limits
```rust
// Good: Limit allows meaningful participation but prevents dominance
let goal = 1_000_000;
let max_individual = 100_000;  // 10% of goal

// Avoid: Limit too close to goal (defeats purpose)
let max_individual = 900_000;  // 90% of goal - not effective
```

### 2. Balance with Minimum
```rust
// Good: Wide range allows various contribution sizes
min_contribution: 1_000
max_individual_contribution: 500_000  // 500x range

// Avoid: Narrow range limits flexibility
min_contribution: 400_000
max_individual_contribution: 500_000  // Only 1.25x range
```

### 3. Consider Goal Size
```rust
// For small goals (< 100K): Consider no limit or high limit
goal: 50_000
max_individual: None  // or Some(50_000)

// For medium goals (100K - 1M): Set limit at 10-20% of goal
goal: 500_000
max_individual: Some(100_000)  // 20% of goal

// For large goals (> 1M): Set limit at 5-10% of goal
goal: 10_000_000
max_individual: Some(500_000)  // 5% of goal
```

## Testing Recommendations

### Unit Tests
```rust
#[test]
fn test_my_campaign_limits() {
    // Test exact limit boundary
    // Test exceeding limit
    // Test cumulative tracking
    // Test no limit behavior
}
```

### Integration Tests
- Test with real token transfers
- Test multiple contributors at limit
- Test refund scenarios with limits
- Test campaign success with distributed contributions

## Migration Guide

### Updating Existing Campaigns
Existing campaigns initialized without this parameter will continue to work:

```rust
// Old signature (still works)
client.initialize(
    &creator,
    &token_address,
    &goal,
    &deadline,
    &min_contribution,
    &None,  // platform_config
);

// New signature (add None for no limit)
client.initialize(
    &creator,
    &token_address,
    &goal,
    &deadline,
    &min_contribution,
    &None,  // max_individual_contribution - NEW
    &None,  // platform_config
);
```

## FAQ

**Q: Can the limit be changed after initialization?**
A: No, the limit is set at initialization and cannot be modified.

**Q: Does the limit apply to the creator?**
A: Yes, if the creator contributes to their own campaign, the limit applies.

**Q: What happens if someone tries to exceed the limit?**
A: The transaction fails with `ContractError::IndividualLimitExceeded` and no tokens are transferred.

**Q: Is the limit enforced per transaction or cumulatively?**
A: Cumulatively - all contributions from the same address are summed.

**Q: Can different contributors have different limits?**
A: No, the limit applies equally to all contributors.

**Q: Does the limit affect refunds?**
A: No, refunds return the full contributed amount regardless of limits.

## Related Features

- **Minimum Contribution** (#8) - Sets lower bound per transaction
- **Input Validation** (#2) - Validates all initialization parameters
- **Structured Errors** (#1) - Provides clear error messages
- **Overflow Protection** (#29) - Prevents arithmetic overflow in cumulative calculations
