#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_validate_campaign_creation_success() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let goal = 1_000_000_000;
    let deadline = env.ledger().timestamp() + 86400; // 1 day from now
    let title = String::from_str(&env, "Test Campaign");

    let result = InputValidator::validate_campaign_creation(
        env.clone(),
        creator,
        goal,
        deadline,
        title,
    );

    assert!(result.is_valid);
}

#[test]
fn test_validate_campaign_creation_negative_goal() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let goal = -100;
    let deadline = env.ledger().timestamp() + 86400;
    let title = String::from_str(&env, "Test Campaign");

    let result = InputValidator::validate_campaign_creation(
        env.clone(),
        creator,
        goal,
        deadline,
        title,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Goal must be positive"));
}

#[test]
fn test_validate_campaign_creation_zero_goal() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let goal = 0;
    let deadline = env.ledger().timestamp() + 86400;
    let title = String::from_str(&env, "Test Campaign");

    let result = InputValidator::validate_campaign_creation(
        env.clone(),
        creator,
        goal,
        deadline,
        title,
    );

    assert!(!result.is_valid);
}

#[test]
fn test_validate_campaign_creation_excessive_goal() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let goal = 2_000_000_000_000_000;
    let deadline = env.ledger().timestamp() + 86400;
    let title = String::from_str(&env, "Test Campaign");

    let result = InputValidator::validate_campaign_creation(
        env.clone(),
        creator,
        goal,
        deadline,
        title,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Goal exceeds maximum allowed"));
}

#[test]
fn test_validate_campaign_creation_past_deadline() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let goal = 1_000_000_000;
    let deadline = env.ledger().timestamp() - 1; // Past deadline
    let title = String::from_str(&env, "Test Campaign");

    let result = InputValidator::validate_campaign_creation(
        env.clone(),
        creator,
        goal,
        deadline,
        title,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Deadline must be in the future"));
}

#[test]
fn test_validate_campaign_creation_far_future_deadline() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let goal = 1_000_000_000;
    let deadline = env.ledger().timestamp() + (400 * 24 * 60 * 60); // Over 1 year
    let title = String::from_str(&env, "Test Campaign");

    let result = InputValidator::validate_campaign_creation(
        env.clone(),
        creator,
        goal,
        deadline,
        title,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Deadline exceeds maximum duration"));
}

#[test]
fn test_validate_campaign_creation_empty_title() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let goal = 1_000_000_000;
    let deadline = env.ledger().timestamp() + 86400;
    let title = String::from_str(&env, "");

    let result = InputValidator::validate_campaign_creation(
        env.clone(),
        creator,
        goal,
        deadline,
        title,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Title cannot be empty"));
}

#[test]
fn test_validate_campaign_creation_long_title() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let goal = 1_000_000_000;
    let deadline = env.ledger().timestamp() + 86400;
    let long_title = "a".repeat(201);
    let title = String::from_str(&env, &long_title);

    let result = InputValidator::validate_campaign_creation(
        env.clone(),
        creator,
        goal,
        deadline,
        title,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Title exceeds maximum length"));
}

#[test]
fn test_validate_contribution_success() {
    let env = Env::default();
    let contributor = Address::generate(&env);
    let amount = 10_000_000;
    let campaign_active = true;

    let result = InputValidator::validate_contribution(
        env.clone(),
        contributor,
        amount,
        campaign_active,
    );

    assert!(result.is_valid);
}

#[test]
fn test_validate_contribution_negative_amount() {
    let env = Env::default();
    let contributor = Address::generate(&env);
    let amount = -100;
    let campaign_active = true;

    let result = InputValidator::validate_contribution(
        env.clone(),
        contributor,
        amount,
        campaign_active,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Contribution must be positive"));
}

#[test]
fn test_validate_contribution_zero_amount() {
    let env = Env::default();
    let contributor = Address::generate(&env);
    let amount = 0;
    let campaign_active = true;

    let result = InputValidator::validate_contribution(
        env.clone(),
        contributor,
        amount,
        campaign_active,
    );

    assert!(!result.is_valid);
}

#[test]
fn test_validate_contribution_below_minimum() {
    let env = Env::default();
    let contributor = Address::generate(&env);
    let amount = 500_000; // Below minimum
    let campaign_active = true;

    let result = InputValidator::validate_contribution(
        env.clone(),
        contributor,
        amount,
        campaign_active,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Contribution below minimum"));
}

#[test]
fn test_validate_contribution_inactive_campaign() {
    let env = Env::default();
    let contributor = Address::generate(&env);
    let amount = 10_000_000;
    let campaign_active = false;

    let result = InputValidator::validate_contribution(
        env.clone(),
        contributor,
        amount,
        campaign_active,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Campaign is not active"));
}

#[test]
fn test_validate_withdrawal_creator_success() {
    let env = Env::default();
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

    assert!(result.is_valid);
}

#[test]
fn test_validate_withdrawal_creator_goal_not_reached() {
    let env = Env::default();
    let requester = Address::generate(&env);
    let is_creator = true;
    let goal_reached = false;
    let deadline_passed = true;

    let result = InputValidator::validate_withdrawal(
        env.clone(),
        requester,
        is_creator,
        goal_reached,
        deadline_passed,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Goal not reached"));
}

#[test]
fn test_validate_withdrawal_contributor_success() {
    let env = Env::default();
    let requester = Address::generate(&env);
    let is_creator = false;
    let goal_reached = false;
    let deadline_passed = true;

    let result = InputValidator::validate_withdrawal(
        env.clone(),
        requester,
        is_creator,
        goal_reached,
        deadline_passed,
    );

    assert!(result.is_valid);
}

#[test]
fn test_validate_withdrawal_contributor_goal_reached() {
    let env = Env::default();
    let requester = Address::generate(&env);
    let is_creator = false;
    let goal_reached = true;
    let deadline_passed = true;

    let result = InputValidator::validate_withdrawal(
        env.clone(),
        requester,
        is_creator,
        goal_reached,
        deadline_passed,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Cannot refund successful campaign"));
}

#[test]
fn test_validate_withdrawal_contributor_deadline_not_passed() {
    let env = Env::default();
    let requester = Address::generate(&env);
    let is_creator = false;
    let goal_reached = false;
    let deadline_passed = false;

    let result = InputValidator::validate_withdrawal(
        env.clone(),
        requester,
        is_creator,
        goal_reached,
        deadline_passed,
    );

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Deadline not yet passed"));
}

#[test]
fn test_validate_address_success() {
    let env = Env::default();
    let address = Address::generate(&env);

    let result = InputValidator::validate_address(env.clone(), address);

    assert!(result.is_valid);
}

#[test]
fn test_validate_batch_size_success() {
    let env = Env::default();
    let batch_size = 50;

    let result = InputValidator::validate_batch_size(env.clone(), batch_size);

    assert!(result.is_valid);
}

#[test]
fn test_validate_batch_size_zero() {
    let env = Env::default();
    let batch_size = 0;

    let result = InputValidator::validate_batch_size(env.clone(), batch_size);

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Batch cannot be empty"));
}

#[test]
fn test_validate_batch_size_excessive() {
    let env = Env::default();
    let batch_size = 150;

    let result = InputValidator::validate_batch_size(env.clone(), batch_size);

    assert!(!result.is_valid);
    assert_eq!(result.error_message, String::from_str(&env, "Batch size exceeds maximum"));
}

#[test]
fn test_validate_batch_size_boundary() {
    let env = Env::default();
    
    // Test exactly at maximum
    let result = InputValidator::validate_batch_size(env.clone(), 100);
    assert!(result.is_valid);
    
    // Test just over maximum
    let result = InputValidator::validate_batch_size(env.clone(), 101);
    assert!(!result.is_valid);
}
