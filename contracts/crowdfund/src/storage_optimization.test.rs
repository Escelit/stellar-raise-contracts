#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Map, Vec};

#[test]
fn test_create_compact_campaign() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let goal = 1_000_000_000;
    let deadline = env.ledger().timestamp() + 86400;

    let campaign = StorageOptimizer::create_compact_campaign(
        env.clone(),
        creator.clone(),
        goal,
        deadline,
    );

    assert_eq!(campaign.creator, creator);
    assert_eq!(campaign.goal, goal);
    assert_eq!(campaign.deadline, deadline);
    assert_eq!(campaign.raised, 0);
    assert_eq!(campaign.flags, FLAG_ACTIVE);
}

#[test]
fn test_set_flag() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let mut campaign = StorageOptimizer::create_compact_campaign(
        env.clone(),
        creator,
        1_000_000_000,
        env.ledger().timestamp() + 86400,
    );

    campaign = StorageOptimizer::set_flag(campaign, FLAG_GOAL_REACHED);

    assert!(StorageOptimizer::has_flag(&campaign, FLAG_ACTIVE));
    assert!(StorageOptimizer::has_flag(&campaign, FLAG_GOAL_REACHED));
}

#[test]
fn test_clear_flag() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let mut campaign = StorageOptimizer::create_compact_campaign(
        env.clone(),
        creator,
        1_000_000_000,
        env.ledger().timestamp() + 86400,
    );

    campaign = StorageOptimizer::clear_flag(campaign, FLAG_ACTIVE);

    assert!(!StorageOptimizer::has_flag(&campaign, FLAG_ACTIVE));
}

#[test]
fn test_has_flag() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let campaign = StorageOptimizer::create_compact_campaign(
        env.clone(),
        creator,
        1_000_000_000,
        env.ledger().timestamp() + 86400,
    );

    assert!(StorageOptimizer::has_flag(&campaign, FLAG_ACTIVE));
    assert!(!StorageOptimizer::has_flag(&campaign, FLAG_GOAL_REACHED));
    assert!(!StorageOptimizer::has_flag(&campaign, FLAG_WITHDRAWN));
}

#[test]
fn test_multiple_flags() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let mut campaign = StorageOptimizer::create_compact_campaign(
        env.clone(),
        creator,
        1_000_000_000,
        env.ledger().timestamp() + 86400,
    );

    campaign = StorageOptimizer::set_flag(campaign, FLAG_GOAL_REACHED);
    campaign = StorageOptimizer::set_flag(campaign, FLAG_WITHDRAWN);

    assert!(StorageOptimizer::has_flag(&campaign, FLAG_ACTIVE));
    assert!(StorageOptimizer::has_flag(&campaign, FLAG_GOAL_REACHED));
    assert!(StorageOptimizer::has_flag(&campaign, FLAG_WITHDRAWN));
}

#[test]
fn test_create_compact_contribution() {
    let env = Env::default();
    let contributor = Address::generate(&env);
    let amount = 10_000_000;

    let contribution = StorageOptimizer::create_compact_contribution(
        env.clone(),
        contributor.clone(),
        amount,
    );

    assert_eq!(contribution.contributor, contributor);
    assert_eq!(contribution.amount, amount);
    assert!(contribution.timestamp > 0);
}

#[test]
fn test_batch_update_contributions() {
    let env = Env::default();
    let mut contributions = Vec::new(&env);

    for i in 0..5 {
        let contributor = Address::generate(&env);
        let contribution = StorageOptimizer::create_compact_contribution(
            env.clone(),
            contributor,
            1_000_000 * (i as i128 + 1),
        );
        contributions.push_back(contribution);
    }

    let count = StorageOptimizer::batch_update_contributions(env.clone(), contributions, 1);

    assert_eq!(count, 5);
}

#[test]
fn test_generate_storage_key() {
    let prefix = 42u32;
    let id = 12345u64;

    let key = StorageOptimizer::generate_storage_key(prefix, id);

    assert!(key > 0);
}

#[test]
fn test_unpack_storage_key() {
    let prefix = 42u32;
    let id = 12345u64;

    let key = StorageOptimizer::generate_storage_key(prefix, id);
    let (unpacked_prefix, unpacked_id) = StorageOptimizer::unpack_storage_key(key);

    assert_eq!(unpacked_prefix, prefix);
    assert_eq!(unpacked_id, id);
}

#[test]
fn test_storage_key_roundtrip() {
    let test_cases = vec![
        (0u32, 0u64),
        (1u32, 1u64),
        (255u32, 65535u64),
        (u32::MAX, u64::MAX),
    ];

    for (prefix, id) in test_cases {
        let key = StorageOptimizer::generate_storage_key(prefix, id);
        let (unpacked_prefix, unpacked_id) = StorageOptimizer::unpack_storage_key(key);
        assert_eq!(unpacked_prefix, prefix);
        assert_eq!(unpacked_id, id);
    }
}

#[test]
fn test_estimate_storage_cost() {
    let small_data = 100u64;
    let large_data = 10000u64;

    let small_cost = StorageOptimizer::estimate_storage_cost(small_data);
    let large_cost = StorageOptimizer::estimate_storage_cost(large_data);

    assert!(small_cost > 0);
    assert!(large_cost > small_cost);
}

#[test]
fn test_estimate_storage_cost_zero() {
    let cost = StorageOptimizer::estimate_storage_cost(0);
    assert_eq!(cost, 1000); // Base cost only
}

#[test]
fn test_compact_map_storage() {
    let env = Env::default();
    let mut map = Map::new(&env);

    // Add some entries with zero values
    map.set(1, 100);
    map.set(2, 0);
    map.set(3, 200);
    map.set(4, 0);
    map.set(5, 300);

    let removed = StorageOptimizer::compact_map_storage(env.clone(), &mut map);

    assert_eq!(removed, 2);
    assert_eq!(map.len(), 3);
}

#[test]
fn test_compact_map_storage_no_zeros() {
    let env = Env::default();
    let mut map = Map::new(&env);

    map.set(1, 100);
    map.set(2, 200);
    map.set(3, 300);

    let removed = StorageOptimizer::compact_map_storage(env.clone(), &mut map);

    assert_eq!(removed, 0);
    assert_eq!(map.len(), 3);
}

#[test]
fn test_batch_storage_update() {
    let env = Env::default();
    let mut updates = Vec::new(&env);

    updates.push_back((1u64, 100i128));
    updates.push_back((2u64, 200i128));
    updates.push_back((3u64, 300i128));

    let count = StorageOptimizer::batch_storage_update(env.clone(), updates);

    assert_eq!(count, 3);
}

#[test]
fn test_batch_storage_update_empty() {
    let env = Env::default();
    let updates = Vec::new(&env);

    let count = StorageOptimizer::batch_storage_update(env.clone(), updates);

    assert_eq!(count, 0);
}

#[test]
fn test_calculate_optimal_batch_size() {
    let total_items = 100u32;
    let max_gas = 50_000u64;

    let batch_size = StorageOptimizer::calculate_optimal_batch_size(total_items, max_gas);

    assert!(batch_size > 0);
    assert!(batch_size <= total_items);
}

#[test]
fn test_calculate_optimal_batch_size_limited_gas() {
    let total_items = 1000u32;
    let max_gas = 5_000u64; // Only enough for 5 items

    let batch_size = StorageOptimizer::calculate_optimal_batch_size(total_items, max_gas);

    assert_eq!(batch_size, 5);
}

#[test]
fn test_calculate_optimal_batch_size_unlimited_gas() {
    let total_items = 10u32;
    let max_gas = 1_000_000u64; // More than enough

    let batch_size = StorageOptimizer::calculate_optimal_batch_size(total_items, max_gas);

    assert_eq!(batch_size, total_items);
}

#[test]
fn test_compress_campaign_data() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let campaign = StorageOptimizer::create_compact_campaign(
        env.clone(),
        creator,
        1_000_000_000,
        env.ledger().timestamp() + 86400,
    );

    let compressed = StorageOptimizer::compress_campaign_data(&campaign);

    // Placeholder returns empty vec
    assert_eq!(compressed.len(), 0);
}

#[test]
fn test_decompress_campaign_data() {
    let env = Env::default();
    let compressed = Vec::new(&env);

    let result = StorageOptimizer::decompress_campaign_data(env.clone(), compressed);

    // Placeholder returns None
    assert!(result.is_none());
}

#[test]
fn test_is_optimization_beneficial_true() {
    let current_size = 10000u64;
    let optimized_size = 5000u64;
    let optimization_cost = 1000i128;

    let beneficial = StorageOptimizer::is_optimization_beneficial(
        current_size,
        optimized_size,
        optimization_cost,
    );

    assert!(beneficial);
}

#[test]
fn test_is_optimization_beneficial_false() {
    let current_size = 1000u64;
    let optimized_size = 900u64;
    let optimization_cost = 10000i128; // Cost exceeds savings

    let beneficial = StorageOptimizer::is_optimization_beneficial(
        current_size,
        optimized_size,
        optimization_cost,
    );

    assert!(!beneficial);
}

#[test]
fn test_is_optimization_beneficial_equal() {
    let current_size = 1000u64;
    let optimized_size = 1000u64;
    let optimization_cost = 0i128;

    let beneficial = StorageOptimizer::is_optimization_beneficial(
        current_size,
        optimized_size,
        optimization_cost,
    );

    assert!(!beneficial); // No savings, so not beneficial
}

#[test]
fn test_flag_operations_comprehensive() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let mut campaign = StorageOptimizer::create_compact_campaign(
        env.clone(),
        creator,
        1_000_000_000,
        env.ledger().timestamp() + 86400,
    );

    // Set all flags
    campaign = StorageOptimizer::set_flag(campaign, FLAG_GOAL_REACHED);
    campaign = StorageOptimizer::set_flag(campaign, FLAG_WITHDRAWN);

    assert!(StorageOptimizer::has_flag(&campaign, FLAG_ACTIVE));
    assert!(StorageOptimizer::has_flag(&campaign, FLAG_GOAL_REACHED));
    assert!(StorageOptimizer::has_flag(&campaign, FLAG_WITHDRAWN));

    // Clear one flag
    campaign = StorageOptimizer::clear_flag(campaign, FLAG_ACTIVE);

    assert!(!StorageOptimizer::has_flag(&campaign, FLAG_ACTIVE));
    assert!(StorageOptimizer::has_flag(&campaign, FLAG_GOAL_REACHED));
    assert!(StorageOptimizer::has_flag(&campaign, FLAG_WITHDRAWN));

    // Clear all flags
    campaign = StorageOptimizer::clear_flag(campaign, FLAG_GOAL_REACHED);
    campaign = StorageOptimizer::clear_flag(campaign, FLAG_WITHDRAWN);

    assert!(!StorageOptimizer::has_flag(&campaign, FLAG_ACTIVE));
    assert!(!StorageOptimizer::has_flag(&campaign, FLAG_GOAL_REACHED));
    assert!(!StorageOptimizer::has_flag(&campaign, FLAG_WITHDRAWN));
}
