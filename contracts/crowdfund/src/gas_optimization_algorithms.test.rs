#![cfg(test)]

use crate::gas_optimization_algorithms::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

#[test]
fn test_batch_read_contributions_empty() {
    let env = Env::default();
    let addresses = Vec::new(&env);
    
    let results = batch_read_contributions(&env, &addresses);
    assert_eq!(results.len(), 0);
}

#[test]
fn test_batch_read_contributions_with_data() {
    let env = Env::default();
    env.mock_all_auths();
    
    let addr1 = Address::generate(&env);
    let addr2 = Address::generate(&env);
    
    // Set up test data
    env.storage().persistent().set(&crate::DataKey::Contribution(addr1.clone()), &1000i128);
    env.storage().persistent().set(&crate::DataKey::Contribution(addr2.clone()), &2000i128);
    
    let mut addresses = Vec::new(&env);
    addresses.push_back(addr1);
    addresses.push_back(addr2);
    
    let results = batch_read_contributions(&env, &addresses);
    assert_eq!(results.len(), 2);
    assert_eq!(results.get(0).unwrap(), 1000);
    assert_eq!(results.get(1).unwrap(), 2000);
}

#[test]
fn test_batch_read_contributions_missing_data() {
    let env = Env::default();
    
    let addr1 = Address::generate(&env);
    let mut addresses = Vec::new(&env);
    addresses.push_back(addr1);
    
    let results = batch_read_contributions(&env, &addresses);
    assert_eq!(results.len(), 1);
    assert_eq!(results.get(0).unwrap(), 0); // Should return 0 for missing entries
}

#[test]
fn test_calculate_optimal_batch_size_edge_cases() {
    assert_eq!(calculate_optimal_batch_size(0, 50), 0);
    assert_eq!(calculate_optimal_batch_size(1, 50), 1);
    assert_eq!(calculate_optimal_batch_size(10, 50), 10);
}

#[test]
fn test_calculate_optimal_batch_size_large_operations() {
    let result = calculate_optimal_batch_size(1000, 50);
    assert!(result > 0 && result <= 50);
    
    let result = calculate_optimal_batch_size(2500, 100);
    assert!(result > 0 && result <= 100);
}

#[test]
fn test_calculate_optimal_batch_size_respects_max() {
    let result = calculate_optimal_batch_size(10000, 25);
    assert!(result <= 25);
}

#[test]
fn test_estimate_contribution_gas_base_case() {
    let gas = estimate_contribution_gas(false, false);
    assert_eq!(gas, 10_000);
}

#[test]
fn test_estimate_contribution_gas_new_contributor() {
    let gas = estimate_contribution_gas(true, false);
    assert_eq!(gas, 15_000);
}

#[test]
fn test_estimate_contribution_gas_with_platform_fee() {
    let gas = estimate_contribution_gas(false, true);
    assert_eq!(gas, 12_000);
}

#[test]
fn test_estimate_contribution_gas_all_features() {
    let gas = estimate_contribution_gas(true, true);
    assert_eq!(gas, 17_000);
}

#[test]
fn test_gas_optimization_config_default() {
    let config = GasOptimizationConfig::default();
    assert!(config.enable_batch_optimization);
    assert!(config.enable_storage_caching);
    assert_eq!(config.max_batch_size, 50);
}

#[test]
fn test_optimized_storage_read_with_cache() {
    let env = Env::default();
    let key = crate::DataKey::Goal;
    let cached_value = Some(1000i128);
    
    let result = optimized_storage_read(&env, &key, cached_value);
    assert_eq!(result, Some(1000i128));
}

#[test]
fn test_optimized_storage_read_without_cache() {
    let env = Env::default();
    env.storage().instance().set(&crate::DataKey::Goal, &5000i128);
    
    let result: Option<i128> = optimized_storage_read(&env, &crate::DataKey::Goal, None);
    assert_eq!(result, Some(5000i128));
}

#[test]
fn test_approximate_sqrt_comprehensive() {
    assert_eq!(approximate_sqrt(0), 0);
    assert_eq!(approximate_sqrt(1), 1);
    assert_eq!(approximate_sqrt(2), 1);
    assert_eq!(approximate_sqrt(3), 1);
    assert_eq!(approximate_sqrt(4), 2);
    assert_eq!(approximate_sqrt(15), 3);
    assert_eq!(approximate_sqrt(16), 4);
    assert_eq!(approximate_sqrt(24), 4);
    assert_eq!(approximate_sqrt(25), 5);
    assert_eq!(approximate_sqrt(99), 9);
    assert_eq!(approximate_sqrt(100), 10);
    assert_eq!(approximate_sqrt(144), 12);
    assert_eq!(approximate_sqrt(10000), 100);
}
