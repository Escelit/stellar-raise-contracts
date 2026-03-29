use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

/// Input validation module for crowdfunding contract
/// Provides comprehensive validation for all contract inputs to ensure security and data integrity

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error_message: String,
}

#[contract]
pub struct InputValidator;

#[contractimpl]
impl InputValidator {
    /// Validates campaign creation parameters
    /// 
    /// # Arguments
    /// * `env` - The contract environment
    /// * `creator` - Address of the campaign creator
    /// * `goal` - Funding goal amount
    /// * `deadline` - Campaign deadline timestamp
    /// * `title` - Campaign title
    /// 
    /// # Returns
    /// ValidationResult indicating if inputs are valid
    pub fn validate_campaign_creation(
        env: Env,
        creator: Address,
        goal: i128,
        deadline: u64,
        title: String,
    ) -> ValidationResult {
        // Validate goal is positive and within reasonable bounds
        if goal <= 0 {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Goal must be positive"),
            };
        }

        if goal > 1_000_000_000_000_000 {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Goal exceeds maximum allowed"),
            };
        }

        // Validate deadline is in the future
        let current_time = env.ledger().timestamp();
        if deadline <= current_time {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Deadline must be in the future"),
            };
        }

        // Validate deadline is not too far in the future (max 1 year)
        let max_deadline = current_time + (365 * 24 * 60 * 60);
        if deadline > max_deadline {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Deadline exceeds maximum duration"),
            };
        }

        // Validate title length
        let title_len = title.len();
        if title_len == 0 {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Title cannot be empty"),
            };
        }

        if title_len > 200 {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Title exceeds maximum length"),
            };
        }

        ValidationResult {
            is_valid: true,
            error_message: String::from_str(&env, ""),
        }
    }

    /// Validates contribution parameters
    /// 
    /// # Arguments
    /// * `env` - The contract environment
    /// * `contributor` - Address of the contributor
    /// * `amount` - Contribution amount
    /// * `campaign_active` - Whether the campaign is still active
    /// 
    /// # Returns
    /// ValidationResult indicating if inputs are valid
    pub fn validate_contribution(
        env: Env,
        contributor: Address,
        amount: i128,
        campaign_active: bool,
    ) -> ValidationResult {
        // Validate amount is positive
        if amount <= 0 {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Contribution must be positive"),
            };
        }

        // Validate minimum contribution
        if amount < 1_000_000 {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Contribution below minimum"),
            };
        }

        // Validate campaign is active
        if !campaign_active {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Campaign is not active"),
            };
        }

        ValidationResult {
            is_valid: true,
            error_message: String::from_str(&env, ""),
        }
    }

    /// Validates withdrawal parameters
    /// 
    /// # Arguments
    /// * `env` - The contract environment
    /// * `requester` - Address requesting withdrawal
    /// * `is_creator` - Whether requester is the campaign creator
    /// * `goal_reached` - Whether funding goal was reached
    /// * `deadline_passed` - Whether deadline has passed
    /// 
    /// # Returns
    /// ValidationResult indicating if withdrawal is allowed
    pub fn validate_withdrawal(
        env: Env,
        requester: Address,
        is_creator: bool,
        goal_reached: bool,
        deadline_passed: bool,
    ) -> ValidationResult {
        // Creator can withdraw only if goal reached
        if is_creator && !goal_reached {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Goal not reached"),
            };
        }

        // Contributors can withdraw only if goal not reached and deadline passed
        if !is_creator && goal_reached {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Cannot refund successful campaign"),
            };
        }

        if !is_creator && !deadline_passed {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Deadline not yet passed"),
            };
        }

        ValidationResult {
            is_valid: true,
            error_message: String::from_str(&env, ""),
        }
    }

    /// Validates address is not zero address
    /// 
    /// # Arguments
    /// * `env` - The contract environment
    /// * `address` - Address to validate
    /// 
    /// # Returns
    /// ValidationResult indicating if address is valid
    pub fn validate_address(env: Env, address: Address) -> ValidationResult {
        // In Soroban, addresses are validated by the SDK
        // This is a placeholder for additional custom validation
        ValidationResult {
            is_valid: true,
            error_message: String::from_str(&env, ""),
        }
    }

    /// Validates batch operation parameters
    /// 
    /// # Arguments
    /// * `env` - The contract environment
    /// * `batch_size` - Number of operations in batch
    /// 
    /// # Returns
    /// ValidationResult indicating if batch size is valid
    pub fn validate_batch_size(env: Env, batch_size: u32) -> ValidationResult {
        if batch_size == 0 {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Batch cannot be empty"),
            };
        }

        if batch_size > 100 {
            return ValidationResult {
                is_valid: false,
                error_message: String::from_str(&env, "Batch size exceeds maximum"),
            };
        }

        ValidationResult {
            is_valid: true,
            error_message: String::from_str(&env, ""),
        }
    }
}
