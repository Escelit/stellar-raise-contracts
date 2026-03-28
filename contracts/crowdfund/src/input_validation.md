# Input Validation Module

## Overview

The input validation module provides comprehensive validation for all crowdfunding contract inputs to ensure security and data integrity. This module implements defensive programming practices to prevent invalid data from entering the contract state.

## Features

### 1. Campaign Creation Validation

Validates all parameters when creating a new crowdfunding campaign:

- **Goal Amount**: Must be positive and within reasonable bounds (max 1,000,000,000,000,000)
- **Deadline**: Must be in the future but not more than 1 year ahead
- **Title**: Must be non-empty and not exceed 200 characters
- **Creator Address**: Validated by Soroban SDK

### 2. Contribution Validation

Ensures contributions meet security and business requirements:

- **Amount**: Must be positive and meet minimum threshold (1,000,000 stroops)
- **Campaign Status**: Campaign must be active
- **Contributor Address**: Validated by Soroban SDK

### 3. Withdrawal Validation

Implements business logic for fund withdrawals:

- **Creator Withdrawals**: Only allowed when goal is reached
- **Contributor Refunds**: Only allowed when goal not reached and deadline passed
- **Authorization**: Validates requester identity

### 4. Batch Operation Validation

Validates batch processing parameters:

- **Batch Size**: Must be between 1 and 100 operations
- **Prevents DoS**: Limits batch size to prevent resource exhaustion

## Security Considerations

### Input Sanitization

All inputs are validated before processing to prevent:

- Integer overflow/underflow attacks
- Invalid state transitions
- Resource exhaustion attacks
- Unauthorized access

### Bounds Checking

Strict bounds checking on all numeric inputs:

- Goal amounts have upper and lower limits
- Deadlines are constrained to reasonable timeframes
- String lengths are limited to prevent storage bloat
- Batch sizes are capped to prevent DoS

### Business Logic Enforcement

Validation enforces critical business rules:

- Campaigns cannot have past deadlines
- Contributions require active campaigns
- Withdrawals follow success/failure logic
- Refunds only available when appropriate

## Usage Examples

### Validating Campaign Creation

```rust
use soroban_sdk::{Address, Env, String};

let env = Env::default();
let creator = Address::generate(&env);
let goal = 1_000_000_000;
let deadline = env.ledger().timestamp() + 86400; // 1 day
let title = String::from_str(&env, "My Campaign");

let result = InputValidator::validate_campaign_creation(
    env.clone(),
    creator,
    goal,
    deadline,
    title,
);

if !result.is_valid {
    panic!("Validation failed: {}", result.error_message);
}
```

### Validating Contributions

```rust
let contributor = Address::generate(&env);
let amount = 10_000_000;
let campaign_active = true;

let result = InputValidator::validate_contribution(
    env.clone(),
    contributor,
    amount,
    campaign_active,
);

if !result.is_valid {
    panic!("Invalid contribution: {}", result.error_message);
}
```

### Validating Withdrawals

```rust
let requester = Address::generate(&env);
let is_creator = true;
let goal_reached = true;
let deadline_passed = true;

let result = InputValidator::validate_withdrawal(
    env.clone(),
    requester,
    is_creator,
    goal_reached,
    deadline_passed,
);

if !result.is_valid {
    panic!("Withdrawal not allowed: {}", result.error_message);
}
```

## Testing

The module includes comprehensive tests covering:

- ✅ Valid inputs (happy path)
- ✅ Boundary conditions
- ✅ Invalid inputs (negative, zero, excessive values)
- ✅ Edge cases (empty strings, maximum values)
- ✅ Business logic violations

### Running Tests

```bash
cargo test input_validation
```

### Test Coverage

- Campaign creation: 8 test cases
- Contribution validation: 5 test cases
- Withdrawal validation: 6 test cases
- Address validation: 1 test case
- Batch size validation: 4 test cases

Total: 24 comprehensive test cases

## Integration

To integrate this module into the main contract:

1. Import the module in `lib.rs`:
```rust
mod input_validation;
use input_validation::InputValidator;
```

2. Call validation functions before state changes:
```rust
pub fn create_campaign(env: Env, creator: Address, goal: i128, deadline: u64, title: String) {
    let validation = InputValidator::validate_campaign_creation(
        env.clone(),
        creator.clone(),
        goal,
        deadline,
        title.clone(),
    );
    
    if !validation.is_valid {
        panic!("{}", validation.error_message);
    }
    
    // Proceed with campaign creation
}
```

## Performance Considerations

- All validations are O(1) complexity
- String length checks are efficient
- No external calls or storage reads
- Minimal gas overhead

## Future Enhancements

Potential improvements for future versions:

1. Custom validation rules per campaign type
2. Configurable limits via contract parameters
3. Advanced string validation (profanity filtering, etc.)
4. Rate limiting validation
5. Multi-signature validation support

## Security Audit Notes

This module has been designed with security best practices:

- ✅ No external dependencies
- ✅ No storage access (pure validation)
- ✅ Deterministic behavior
- ✅ Comprehensive error messages
- ✅ No panic conditions (returns ValidationResult)
- ✅ Bounds checking on all inputs
- ✅ Business logic enforcement

## License

This module is part of the stellar-raise-contracts project and follows the same license.
