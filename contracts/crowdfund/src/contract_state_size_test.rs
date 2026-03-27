//! Tests for `contract_state_size` — state-size limit enforcement.
//!
//! Coverage targets (≥ 95 %):
//! - Every `check_*` helper returns `Ok` when below the limit.
//! - Every `check_*` helper returns the correct `Err` variant exactly at the limit.
//! - `check_string_len` accepts strings at the boundary and rejects strings one byte over.
//! - Constants are set to their documented values.
//! - All validation helpers return correct results.
//! - Edge cases: overflow protection, boundary conditions, empty states.

use soroban_sdk::{contract, contractimpl, testutils::Address as _, Address, Env, String, Vec};

use crate::{
    contract_state_size::{
        check_contributor_limit, check_pledger_limit, check_roadmap_limit,
        check_stretch_goal_limit, check_string_len, validate_bonus_goal_description,
        validate_contributor_capacity, validate_description, validate_metadata_total_length,
        validate_pledger_capacity, validate_roadmap_capacity, validate_roadmap_description,
        validate_social_links, validate_stretch_goal_capacity, validate_title, StateSizeError,
        MAX_BONUS_GOAL_DESCRIPTION_LENGTH, MAX_CONTRIBUTORS, MAX_DESCRIPTION_LENGTH,
        MAX_METADATA_TOTAL_LENGTH, MAX_PLEDGERS, MAX_ROADMAP_DESCRIPTION_LENGTH, MAX_ROADMAP_ITEMS,
        MAX_SOCIAL_LINKS_LENGTH, MAX_STRETCH_GOALS, MAX_STRING_LEN, MAX_TITLE_LENGTH,
    },
    DataKey, RoadmapItem,
};

// ── Minimal contract needed to access storage in tests ───────────────────────

#[contract]
struct TestContract;

#[contractimpl]
impl TestContract {}

// ── helpers ───────────────────────────────────────────────────────────────────

fn make_env() -> (Env, soroban_sdk::Address) {
    let env = Env::default();
    let contract_id = env.register(TestContract, ());
    (env, contract_id)
}

/// Build a `soroban_sdk::String` of exactly `n` bytes (ASCII 'a').
/// `n` must be ≤ 2304 for this helper (supports MAX_STRING_LEN + overflow cases).
fn str_of_len(env: &Env, n: u32) -> String {
    assert!(n <= 2304, "str_of_len: n too large for test helper");
    let mut b = soroban_sdk::Bytes::new(env);
    for _ in 0..n {
        b.push_back(b'a');
    }
    let mut buf = [0u8; 2304];
    b.copy_into_slice(&mut buf[..n as usize]);
    String::from_bytes(env, &buf[..n as usize])
}

// ── constant sanity checks ───────────────────────────────────────────────────

#[test]
fn constants_have_expected_values() {
    assert_eq!(MAX_CONTRIBUTORS, 128);
    assert_eq!(MAX_ROADMAP_ITEMS, 32);
    assert_eq!(MAX_STRETCH_GOALS, 32);
    assert_eq!(MAX_STRING_LEN, 256);
}

// ── error discriminants ───────────────────────────────────────────────────────

#[test]
fn error_discriminants_are_stable() {
    assert_eq!(StateSizeError::ContributorLimitExceeded as u32, 100);
    assert_eq!(StateSizeError::PledgerLimitExceeded as u32, 101);
    assert_eq!(StateSizeError::RoadmapLimitExceeded as u32, 102);
    assert_eq!(StateSizeError::StretchGoalLimitExceeded as u32, 103);
    assert_eq!(StateSizeError::StringTooLong as u32, 104);
}

// ── check_string_len ─────────────────────────────────────────────────────────

#[test]
fn string_len_empty_is_ok() {
    let (env, _) = make_env();
    let s = String::from_str(&env, "");
    assert_eq!(check_string_len(&s), Ok(()));
}

#[test]
fn string_len_at_limit_is_ok() {
    let (env, _) = make_env();
    let s = str_of_len(&env, MAX_STRING_LEN);
    assert_eq!(check_string_len(&s), Ok(()));
}

#[test]
fn string_len_one_over_limit_is_err() {
    let (env, _) = make_env();
    let s = str_of_len(&env, MAX_STRING_LEN + 1);
    assert_eq!(check_string_len(&s), Err(StateSizeError::StringTooLong));
}

#[test]
fn string_len_well_over_limit_is_err() {
    let (env, _) = make_env();
    let s = str_of_len(&env, MAX_STRING_LEN + 100);
    assert_eq!(check_string_len(&s), Err(StateSizeError::StringTooLong));
}

// ── check_contributor_limit ───────────────────────────────────────────────────

#[test]
fn contributor_limit_empty_list_is_ok() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        assert_eq!(check_contributor_limit(&env), Ok(()));
    });
}

#[test]
fn contributor_limit_below_max_is_ok() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        let mut list: Vec<Address> = Vec::new(&env);
        for _ in 0..MAX_CONTRIBUTORS - 1 {
            list.push_back(Address::generate(&env));
        }
        env.storage()
            .persistent()
            .set(&DataKey::Contributors, &list);
        assert_eq!(check_contributor_limit(&env), Ok(()));
    });
}

#[test]
fn contributor_limit_at_max_is_err() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        let mut list: Vec<Address> = Vec::new(&env);
        for _ in 0..MAX_CONTRIBUTORS {
            list.push_back(Address::generate(&env));
        }
        env.storage()
            .persistent()
            .set(&DataKey::Contributors, &list);
        assert_eq!(
            check_contributor_limit(&env),
            Err(StateSizeError::ContributorLimitExceeded)
        );
    });
}

#[test]
fn contributor_limit_over_max_is_err() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        let mut list: Vec<Address> = Vec::new(&env);
        for _ in 0..MAX_CONTRIBUTORS + 5 {
            list.push_back(Address::generate(&env));
        }
        env.storage()
            .persistent()
            .set(&DataKey::Contributors, &list);
        assert_eq!(
            check_contributor_limit(&env),
            Err(StateSizeError::ContributorLimitExceeded)
        );
    });
}

// ── check_pledger_limit ───────────────────────────────────────────────────────

#[test]
fn pledger_limit_empty_list_is_ok() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        assert_eq!(check_pledger_limit(&env), Ok(()));
    });
}

#[test]
fn pledger_limit_below_max_is_ok() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        let mut list: Vec<Address> = Vec::new(&env);
        // Use MAX_CONTRIBUTORS as proxy for MAX_PLEDGERS (both are 128)
        for _ in 0..crate::contract_state_size::MAX_PLEDGERS - 1 {
            list.push_back(Address::generate(&env));
        }
        env.storage().persistent().set(&DataKey::Pledgers, &list);
        assert_eq!(check_pledger_limit(&env), Ok(()));
    });
}

#[test]
fn pledger_limit_at_max_is_err() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        let mut list: Vec<Address> = Vec::new(&env);
        for _ in 0..crate::contract_state_size::MAX_PLEDGERS {
            list.push_back(Address::generate(&env));
        }
        env.storage().persistent().set(&DataKey::Pledgers, &list);
        assert_eq!(
            check_pledger_limit(&env),
            Err(StateSizeError::PledgerLimitExceeded)
        );
    });
}

// ── check_roadmap_limit ───────────────────────────────────────────────────────

#[test]
fn roadmap_limit_empty_list_is_ok() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        assert_eq!(check_roadmap_limit(&env), Ok(()));
    });
}

#[test]
fn roadmap_limit_below_max_is_ok() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        let mut list: Vec<RoadmapItem> = Vec::new(&env);
        for i in 0..MAX_ROADMAP_ITEMS - 1 {
            list.push_back(RoadmapItem {
                date: 1_000_000 + i as u64,
                description: String::from_str(&env, "milestone"),
            });
        }
        env.storage().instance().set(&DataKey::Roadmap, &list);
        assert_eq!(check_roadmap_limit(&env), Ok(()));
    });
}

#[test]
fn roadmap_limit_at_max_is_err() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        let mut list: Vec<RoadmapItem> = Vec::new(&env);
        for i in 0..MAX_ROADMAP_ITEMS {
            list.push_back(RoadmapItem {
                date: 1_000_000 + i as u64,
                description: String::from_str(&env, "milestone"),
            });
        }
        env.storage().instance().set(&DataKey::Roadmap, &list);
        assert_eq!(
            check_roadmap_limit(&env),
            Err(StateSizeError::RoadmapLimitExceeded)
        );
    });
}

// ── check_stretch_goal_limit ──────────────────────────────────────────────────

#[test]
fn stretch_goal_limit_empty_list_is_ok() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        assert_eq!(check_stretch_goal_limit(&env), Ok(()));
    });
}

#[test]
fn stretch_goal_limit_below_max_is_ok() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        let mut list: Vec<i128> = Vec::new(&env);
        for i in 0..MAX_STRETCH_GOALS - 1 {
            list.push_back(1_000 * (i as i128 + 1));
        }
        env.storage().instance().set(&DataKey::StretchGoals, &list);
        assert_eq!(check_stretch_goal_limit(&env), Ok(()));
    });
}

#[test]
fn stretch_goal_limit_at_max_is_err() {
    let (env, contract_id) = make_env();
    env.as_contract(&contract_id, || {
        let mut list: Vec<i128> = Vec::new(&env);
        for i in 0..MAX_STRETCH_GOALS {
            list.push_back(1_000 * (i as i128 + 1));
        }
        env.storage().instance().set(&DataKey::StretchGoals, &list);
        assert_eq!(
            check_stretch_goal_limit(&env),
            Err(StateSizeError::StretchGoalLimitExceeded)
        );
    });
}

// ── validate_title ────────────────────────────────────────────────────────────

#[test]
fn validate_title_empty_is_ok() {
    let (env, _) = make_env();
    let title = String::from_str(&env, "");
    assert_eq!(validate_title(&title), Ok(()));
}

#[test]
fn validate_title_at_limit_is_ok() {
    let (env, _) = make_env();
    let title = str_of_len(&env, MAX_TITLE_LENGTH);
    assert_eq!(validate_title(&title), Ok(()));
}

#[test]
fn validate_title_over_limit_is_err() {
    let (env, _) = make_env();
    let title = str_of_len(&env, MAX_TITLE_LENGTH + 1);
    assert_eq!(
        validate_title(&title),
        Err("title exceeds MAX_TITLE_LENGTH bytes")
    );
}

// ── validate_description ──────────────────────────────────────────────────────

#[test]
fn validate_description_empty_is_ok() {
    let (env, _) = make_env();
    let desc = String::from_str(&env, "");
    assert_eq!(validate_description(&desc), Ok(()));
}

#[test]
fn validate_description_at_limit_is_ok() {
    let (env, _) = make_env();
    let desc = str_of_len(&env, MAX_DESCRIPTION_LENGTH);
    assert_eq!(validate_description(&desc), Ok(()));
}

#[test]
fn validate_description_over_limit_is_err() {
    let (env, _) = make_env();
    let desc = str_of_len(&env, MAX_DESCRIPTION_LENGTH + 1);
    assert_eq!(
        validate_description(&desc),
        Err("description exceeds MAX_DESCRIPTION_LENGTH bytes")
    );
}

// ── validate_social_links ─────────────────────────────────────────────────────

#[test]
fn validate_social_links_empty_is_ok() {
    let (env, _) = make_env();
    let socials = String::from_str(&env, "");
    assert_eq!(validate_social_links(&socials), Ok(()));
}

#[test]
fn validate_social_links_at_limit_is_ok() {
    let (env, _) = make_env();
    let socials = str_of_len(&env, MAX_SOCIAL_LINKS_LENGTH);
    assert_eq!(validate_social_links(&socials), Ok(()));
}

#[test]
fn validate_social_links_over_limit_is_err() {
    let (env, _) = make_env();
    let socials = str_of_len(&env, MAX_SOCIAL_LINKS_LENGTH + 1);
    assert_eq!(
        validate_social_links(&socials),
        Err("social links exceed MAX_SOCIAL_LINKS_LENGTH bytes")
    );
}

// ── validate_bonus_goal_description ───────────────────────────────────────────

#[test]
fn validate_bonus_goal_description_empty_is_ok() {
    let (env, _) = make_env();
    let desc = String::from_str(&env, "");
    assert_eq!(validate_bonus_goal_description(&desc), Ok(()));
}

#[test]
fn validate_bonus_goal_description_at_limit_is_ok() {
    let (env, _) = make_env();
    let desc = str_of_len(&env, MAX_BONUS_GOAL_DESCRIPTION_LENGTH);
    assert_eq!(validate_bonus_goal_description(&desc), Ok(()));
}

#[test]
fn validate_bonus_goal_description_over_limit_is_err() {
    let (env, _) = make_env();
    let desc = str_of_len(&env, MAX_BONUS_GOAL_DESCRIPTION_LENGTH + 1);
    assert_eq!(
        validate_bonus_goal_description(&desc),
        Err("bonus goal description exceeds MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes")
    );
}

// ── validate_roadmap_description ──────────────────────────────────────────────

#[test]
fn validate_roadmap_description_empty_is_ok() {
    let (env, _) = make_env();
    let desc = String::from_str(&env, "");
    assert_eq!(validate_roadmap_description(&desc), Ok(()));
}

#[test]
fn validate_roadmap_description_at_limit_is_ok() {
    let (env, _) = make_env();
    let desc = str_of_len(&env, MAX_ROADMAP_DESCRIPTION_LENGTH);
    assert_eq!(validate_roadmap_description(&desc), Ok(()));
}

#[test]
fn validate_roadmap_description_over_limit_is_err() {
    let (env, _) = make_env();
    let desc = str_of_len(&env, MAX_ROADMAP_DESCRIPTION_LENGTH + 1);
    assert_eq!(
        validate_roadmap_description(&desc),
        Err("roadmap description exceeds MAX_ROADMAP_DESCRIPTION_LENGTH bytes")
    );
}

// ── validate_metadata_total_length ────────────────────────────────────────────

#[test]
fn validate_metadata_total_length_all_zero_is_ok() {
    assert_eq!(validate_metadata_total_length(0, 0, 0), Ok(()));
}

#[test]
fn validate_metadata_total_length_at_limit_is_ok() {
    // Exact budget: MAX_TITLE_LENGTH + MAX_DESCRIPTION_LENGTH + MAX_SOCIAL_LINKS_LENGTH = 2688
    assert_eq!(
        validate_metadata_total_length(
            MAX_TITLE_LENGTH,
            MAX_DESCRIPTION_LENGTH,
            MAX_SOCIAL_LINKS_LENGTH
        ),
        Ok(())
    );
}

#[test]
fn validate_metadata_total_length_over_limit_is_err() {
    // One byte over MAX_METADATA_TOTAL_LENGTH (2688): 128 + 2048 + 513 = 2689
    assert_eq!(
        validate_metadata_total_length(128, 2048, 513),
        Err("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes")
    );
}

#[test]
fn validate_metadata_total_length_overflow_is_err() {
    // Test overflow protection: u32::MAX + 1 + 1 would overflow
    assert_eq!(
        validate_metadata_total_length(u32::MAX, 1, 1),
        Err("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes")
    );
}

#[test]
fn validate_metadata_total_length_max_individual_fields_is_ok() {
    // Test with actual max values for each field
    assert_eq!(
        validate_metadata_total_length(
            MAX_TITLE_LENGTH,
            MAX_DESCRIPTION_LENGTH,
            MAX_SOCIAL_LINKS_LENGTH
        ),
        Ok(())
    );
}

// ── validate_contributor_capacity ─────────────────────────────────────────────

#[test]
fn validate_contributor_capacity_zero_is_ok() {
    assert_eq!(validate_contributor_capacity(0), Ok(()));
}

#[test]
fn validate_contributor_capacity_below_max_is_ok() {
    assert_eq!(validate_contributor_capacity(MAX_CONTRIBUTORS - 1), Ok(()));
}

#[test]
fn validate_contributor_capacity_at_max_is_err() {
    assert_eq!(
        validate_contributor_capacity(MAX_CONTRIBUTORS),
        Err("contributors exceed MAX_CONTRIBUTORS")
    );
}

#[test]
fn validate_contributor_capacity_over_max_is_err() {
    assert_eq!(
        validate_contributor_capacity(MAX_CONTRIBUTORS + 10),
        Err("contributors exceed MAX_CONTRIBUTORS")
    );
}

// ── validate_pledger_capacity ─────────────────────────────────────────────────

#[test]
fn validate_pledger_capacity_zero_is_ok() {
    assert_eq!(validate_pledger_capacity(0), Ok(()));
}

#[test]
fn validate_pledger_capacity_below_max_is_ok() {
    assert_eq!(validate_pledger_capacity(MAX_PLEDGERS - 1), Ok(()));
}

#[test]
fn validate_pledger_capacity_at_max_is_err() {
    assert_eq!(
        validate_pledger_capacity(MAX_PLEDGERS),
        Err("pledgers exceed MAX_PLEDGERS")
    );
}

#[test]
fn validate_pledger_capacity_over_max_is_err() {
    assert_eq!(
        validate_pledger_capacity(MAX_PLEDGERS + 10),
        Err("pledgers exceed MAX_PLEDGERS")
    );
}

// ── validate_roadmap_capacity ─────────────────────────────────────────────────

#[test]
fn validate_roadmap_capacity_zero_is_ok() {
    assert_eq!(validate_roadmap_capacity(0), Ok(()));
}

#[test]
fn validate_roadmap_capacity_below_max_is_ok() {
    assert_eq!(validate_roadmap_capacity(MAX_ROADMAP_ITEMS - 1), Ok(()));
}

#[test]
fn validate_roadmap_capacity_at_max_is_err() {
    assert_eq!(
        validate_roadmap_capacity(MAX_ROADMAP_ITEMS),
        Err("roadmap exceeds MAX_ROADMAP_ITEMS")
    );
}

#[test]
fn validate_roadmap_capacity_over_max_is_err() {
    assert_eq!(
        validate_roadmap_capacity(MAX_ROADMAP_ITEMS + 5),
        Err("roadmap exceeds MAX_ROADMAP_ITEMS")
    );
}

// ── validate_stretch_goal_capacity ────────────────────────────────────────────

#[test]
fn validate_stretch_goal_capacity_zero_is_ok() {
    assert_eq!(validate_stretch_goal_capacity(0), Ok(()));
}

#[test]
fn validate_stretch_goal_capacity_below_max_is_ok() {
    assert_eq!(
        validate_stretch_goal_capacity(MAX_STRETCH_GOALS - 1),
        Ok(())
    );
}

#[test]
fn validate_stretch_goal_capacity_at_max_is_err() {
    assert_eq!(
        validate_stretch_goal_capacity(MAX_STRETCH_GOALS),
        Err("stretch goals exceed MAX_STRETCH_GOALS")
    );
}

#[test]
fn validate_stretch_goal_capacity_over_max_is_err() {
    assert_eq!(
        validate_stretch_goal_capacity(MAX_STRETCH_GOALS + 5),
        Err("stretch goals exceed MAX_STRETCH_GOALS")
    );
}

// ── Additional edge case tests ────────────────────────────────────────────────

#[test]
fn all_constants_are_positive() {
    const { assert!(MAX_CONTRIBUTORS > 0) };
    const { assert!(MAX_PLEDGERS > 0) };
    const { assert!(MAX_ROADMAP_ITEMS > 0) };
    const { assert!(MAX_STRETCH_GOALS > 0) };
    const { assert!(MAX_TITLE_LENGTH > 0) };
    const { assert!(MAX_DESCRIPTION_LENGTH > 0) };
    const { assert!(MAX_SOCIAL_LINKS_LENGTH > 0) };
    const { assert!(MAX_BONUS_GOAL_DESCRIPTION_LENGTH > 0) };
    const { assert!(MAX_ROADMAP_DESCRIPTION_LENGTH > 0) };
    const { assert!(MAX_METADATA_TOTAL_LENGTH > 0) };
}

#[test]
fn metadata_total_budget_is_sufficient() {
    // Verify that MAX_METADATA_TOTAL_LENGTH >= sum of individual max lengths
    let sum = MAX_TITLE_LENGTH + MAX_DESCRIPTION_LENGTH + MAX_SOCIAL_LINKS_LENGTH;
    assert!(
        MAX_METADATA_TOTAL_LENGTH >= sum,
        "Total metadata budget should accommodate all individual max lengths"
    );
}

#[test]
fn error_discriminants_are_unique() {
    // Verify each discriminant is distinct from all others
    assert_ne!(
        StateSizeError::ContributorLimitExceeded as u32,
        StateSizeError::PledgerLimitExceeded as u32
    );
    assert_ne!(
        StateSizeError::ContributorLimitExceeded as u32,
        StateSizeError::RoadmapLimitExceeded as u32
    );
    assert_ne!(
        StateSizeError::ContributorLimitExceeded as u32,
        StateSizeError::StretchGoalLimitExceeded as u32
    );
    assert_ne!(
        StateSizeError::ContributorLimitExceeded as u32,
        StateSizeError::StringTooLong as u32
    );
    assert_ne!(
        StateSizeError::ContributorLimitExceeded as u32,
        StateSizeError::MetadataTotalExceeded as u32
    );
    assert_ne!(
        StateSizeError::PledgerLimitExceeded as u32,
        StateSizeError::RoadmapLimitExceeded as u32
    );
    assert_ne!(
        StateSizeError::PledgerLimitExceeded as u32,
        StateSizeError::StretchGoalLimitExceeded as u32
    );
    assert_ne!(
        StateSizeError::PledgerLimitExceeded as u32,
        StateSizeError::StringTooLong as u32
    );
    assert_ne!(
        StateSizeError::PledgerLimitExceeded as u32,
        StateSizeError::MetadataTotalExceeded as u32
    );
    assert_ne!(
        StateSizeError::RoadmapLimitExceeded as u32,
        StateSizeError::StretchGoalLimitExceeded as u32
    );
    assert_ne!(
        StateSizeError::RoadmapLimitExceeded as u32,
        StateSizeError::StringTooLong as u32
    );
    assert_ne!(
        StateSizeError::RoadmapLimitExceeded as u32,
        StateSizeError::MetadataTotalExceeded as u32
    );
    assert_ne!(
        StateSizeError::StretchGoalLimitExceeded as u32,
        StateSizeError::StringTooLong as u32
    );
    assert_ne!(
        StateSizeError::StretchGoalLimitExceeded as u32,
        StateSizeError::MetadataTotalExceeded as u32
    );
    assert_ne!(
        StateSizeError::StringTooLong as u32,
        StateSizeError::MetadataTotalExceeded as u32
    );
}
