//! Comprehensive tests for `contract_state_size` limits and contract wiring.
//!
//! Coverage:
//!   - Constant stability and invariants
//!   - String-length validators across exact-boundary and overflow cases
//!   - Collection-capacity validators across exact-boundary and full cases
//!   - Metadata aggregate-budget validation, including overflow hardening
//!   - Contract-level rejection of oversized metadata and full indexed lists
//!   - Contract-level acceptance when an update stays within budget

extern crate alloc;

use crate::contract_state_size::{
    validate_bonus_goal_description, validate_contributor_capacity, validate_description,
    validate_metadata_total_length, validate_pledger_capacity, validate_roadmap_capacity,
    validate_roadmap_description, validate_social_links, validate_stretch_goal_capacity,
    validate_title, MAX_BONUS_GOAL_DESCRIPTION_LENGTH, MAX_CONTRIBUTORS, MAX_DESCRIPTION_LENGTH,
    MAX_METADATA_TOTAL_LENGTH, MAX_PLEDGERS, MAX_ROADMAP_DESCRIPTION_LENGTH, MAX_ROADMAP_ITEMS,
    MAX_SOCIAL_LINKS_LENGTH, MAX_STRETCH_GOALS, MAX_TITLE_LENGTH,
};
use crate::{CrowdfundContract, CrowdfundContractClient, DataKey, RoadmapItem};
use soroban_sdk::{testutils::Address as _, token, Address, Env, String as SorobanString, Vec};

fn setup() -> (
    Env,
    CrowdfundContractClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_id.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    token_admin_client.mint(&creator, &10_000_000);

    (env, client, creator, token_address, token_admin)
}

fn default_init(
    client: &CrowdfundContractClient,
    creator: &Address,
    token_address: &Address,
    deadline: u64,
) {
    client.initialize(
        creator,
        creator,
        token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &None,
        &None,
        &None,
    );
}

fn mint_to(env: &Env, token_address: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token_address).mint(to, &amount);
}

fn soroban_string(env: &Env, len: u32, ch: char) -> SorobanString {
    let value = alloc::vec![ch as u8; len as usize];
    SorobanString::from_bytes(env, value.as_slice())
}

fn filled_addresses(env: &Env, count: u32) -> Vec<Address> {
    let mut values = Vec::new(env);
    for _ in 0..count {
        values.push_back(Address::generate(env));
    }
    values
}

fn filled_roadmap(env: &Env, count: u32) -> Vec<RoadmapItem> {
    let mut roadmap = Vec::new(env);
    for idx in 0..count {
        roadmap.push_back(RoadmapItem {
            date: 10_000 + idx as u64,
            description: soroban_string(env, 8, 'r'),
        });
    }
    roadmap
}

fn filled_stretch_goals(env: &Env, count: u32) -> Vec<i128> {
    let mut stretch_goals = Vec::new(env);
    for idx in 0..count {
        stretch_goals.push_back(2_000_000 + idx as i128);
    }
    stretch_goals
}

// ── Pure helper tests ────────────────────────────────────────────────────────

#[test]
fn constants_have_expected_values() {
    assert_eq!(MAX_CONTRIBUTORS, 128);
    assert_eq!(MAX_PLEDGERS, 128);
    assert_eq!(MAX_ROADMAP_ITEMS, 32);
    assert_eq!(MAX_STRETCH_GOALS, 32);
    assert_eq!(MAX_TITLE_LENGTH, 128);
    assert_eq!(MAX_DESCRIPTION_LENGTH, 2_048);
    assert_eq!(MAX_SOCIAL_LINKS_LENGTH, 512);
    assert_eq!(MAX_BONUS_GOAL_DESCRIPTION_LENGTH, 280);
    assert_eq!(MAX_ROADMAP_DESCRIPTION_LENGTH, 280);
    assert_eq!(MAX_METADATA_TOTAL_LENGTH, 2_304);
}

#[test]
fn validate_title_accepts_exact_limit() {
    let env = Env::default();
    let title = soroban_string(&env, MAX_TITLE_LENGTH, 't');
    assert!(validate_title(&title).is_ok());
}

#[test]
fn validate_title_rejects_one_over_limit() {
    let env = Env::default();
    let title = soroban_string(&env, MAX_TITLE_LENGTH + 1, 't');
    let err = validate_title(&title).unwrap_err();
    assert!(err.contains("MAX_TITLE_LENGTH"));
}

#[test]
fn validate_description_accepts_exact_limit() {
    let env = Env::default();
    let description = soroban_string(&env, MAX_DESCRIPTION_LENGTH, 'd');
    assert!(validate_description(&description).is_ok());
}

#[test]
fn validate_social_links_rejects_one_over_limit() {
    let env = Env::default();
    let socials = soroban_string(&env, MAX_SOCIAL_LINKS_LENGTH + 1, 's');
    let err = validate_social_links(&socials).unwrap_err();
    assert!(err.contains("MAX_SOCIAL_LINKS_LENGTH"));
}

#[test]
fn validate_bonus_goal_description_rejects_one_over_limit() {
    let env = Env::default();
    let description = soroban_string(&env, MAX_BONUS_GOAL_DESCRIPTION_LENGTH + 1, 'b');
    let err = validate_bonus_goal_description(&description).unwrap_err();
    assert!(err.contains("MAX_BONUS_GOAL_DESCRIPTION_LENGTH"));
}

#[test]
fn validate_roadmap_description_rejects_one_over_limit() {
    let env = Env::default();
    let description = soroban_string(&env, MAX_ROADMAP_DESCRIPTION_LENGTH + 1, 'r');
    let err = validate_roadmap_description(&description).unwrap_err();
    assert!(err.contains("MAX_ROADMAP_DESCRIPTION_LENGTH"));
}

#[test]
fn validate_metadata_total_length_accepts_exact_cap() {
    assert!(validate_metadata_total_length(128, 1_664, 512).is_ok());
}

#[test]
fn validate_metadata_total_length_rejects_total_over_cap() {
    let err = validate_metadata_total_length(128, 1_665, 512).unwrap_err();
    assert!(err.contains("MAX_METADATA_TOTAL_LENGTH"));
}

#[test]
fn validate_metadata_total_length_rejects_overflowed_sum() {
    let err = validate_metadata_total_length(u32::MAX, 1, 1).unwrap_err();
    assert!(err.contains("MAX_METADATA_TOTAL_LENGTH"));
}

#[test]
fn validate_contributor_capacity_accepts_one_below_max() {
    assert!(validate_contributor_capacity(MAX_CONTRIBUTORS - 1).is_ok());
}

#[test]
fn validate_contributor_capacity_rejects_when_full() {
    let err = validate_contributor_capacity(MAX_CONTRIBUTORS).unwrap_err();
    assert!(err.contains("MAX_CONTRIBUTORS"));
}

#[test]
fn validate_pledger_capacity_rejects_when_full() {
    let err = validate_pledger_capacity(MAX_PLEDGERS).unwrap_err();
    assert!(err.contains("MAX_PLEDGERS"));
}

#[test]
fn validate_roadmap_capacity_rejects_when_full() {
    let err = validate_roadmap_capacity(MAX_ROADMAP_ITEMS).unwrap_err();
    assert!(err.contains("MAX_ROADMAP_ITEMS"));
}

#[test]
fn validate_stretch_goal_capacity_rejects_when_full() {
    let err = validate_stretch_goal_capacity(MAX_STRETCH_GOALS).unwrap_err();
    assert!(err.contains("MAX_STRETCH_GOALS"));
}

// ── Contract wiring tests ────────────────────────────────────────────────────

#[test]
fn initialize_accepts_bonus_goal_description_at_exact_limit() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let description = soroban_string(&env, MAX_BONUS_GOAL_DESCRIPTION_LENGTH, 'b');

    client.initialize(
        &creator,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(2_000_000),
        &Some(description.clone()),
        &None,
    );

    assert_eq!(client.bonus_goal_description(), Some(description));
}

#[test]
#[should_panic(expected = "bonus goal description exceeds MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes")]
fn initialize_rejects_oversized_bonus_goal_description() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let description = soroban_string(&env, MAX_BONUS_GOAL_DESCRIPTION_LENGTH + 1, 'b');

    client.initialize(
        &creator,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(2_000_000),
        &Some(description),
        &None,
    );
}

#[test]
fn update_metadata_accepts_exact_total_budget() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token_address, deadline);

    let title = soroban_string(&env, 128, 't');
    let description = soroban_string(&env, 1_664, 'd');
    let socials = soroban_string(&env, 512, 's');

    client.update_metadata(
        &creator,
        &Some(title.clone()),
        &Some(description.clone()),
        &Some(socials.clone()),
    );

    assert_eq!(client.title(), title);
    assert_eq!(client.description(), description);
    assert_eq!(client.socials(), socials);
}

#[test]
#[should_panic(expected = "metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes")]
fn update_metadata_rejects_total_metadata_over_budget() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token_address, deadline);

    let description = soroban_string(&env, 1_900, 'd');
    let socials = soroban_string(&env, 300, 's');
    client.update_metadata(&creator, &None, &Some(description), &Some(socials));

    let title = soroban_string(&env, 128, 't');
    client.update_metadata(&creator, &Some(title), &None, &None);
}

#[test]
#[should_panic(expected = "contributors exceed MAX_CONTRIBUTORS")]
fn contribute_rejects_new_contributor_when_index_full() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token_address, deadline);

    env.as_contract(&client.address, || {
        let contributors = filled_addresses(&env, MAX_CONTRIBUTORS);
        env.storage()
            .persistent()
            .set(&DataKey::Contributors, &contributors);
    });

    let newcomer = Address::generate(&env);
    mint_to(&env, &token_address, &newcomer, 5_000);
    client.contribute(&newcomer, &1_000);
}

#[test]
fn contribute_allows_existing_contributor_when_index_full() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &contributor, 5_000);

    env.as_contract(&client.address, || {
        let mut contributors = filled_addresses(&env, MAX_CONTRIBUTORS - 1);
        contributors.push_back(contributor.clone());
        env.storage()
            .persistent()
            .set(&DataKey::Contributors, &contributors);
        env.storage()
            .persistent()
            .set(&DataKey::Contribution(contributor.clone()), &1_000i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &1_000i128);
    });

    client.contribute(&contributor, &1_000);

    assert_eq!(client.contribution(&contributor), 2_000);
    assert_eq!(client.total_raised(), 2_000);
}

#[test]
#[should_panic(expected = "pledgers exceed MAX_PLEDGERS")]
fn pledge_rejects_new_pledger_when_index_full() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token_address, deadline);

    env.as_contract(&client.address, || {
        let pledgers = filled_addresses(&env, MAX_PLEDGERS);
        env.storage()
            .persistent()
            .set(&DataKey::Pledgers, &pledgers);
    });

    let newcomer = Address::generate(&env);
    client.pledge(&newcomer, &1_000);
}

#[test]
#[should_panic(expected = "roadmap description exceeds MAX_ROADMAP_DESCRIPTION_LENGTH bytes")]
fn add_roadmap_item_rejects_oversized_description() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token_address, deadline);

    let future_date = env.ledger().timestamp() + 7_200;
    let description = soroban_string(&env, MAX_ROADMAP_DESCRIPTION_LENGTH + 1, 'r');
    client.add_roadmap_item(&future_date, &description);
}

#[test]
#[should_panic(expected = "roadmap exceeds MAX_ROADMAP_ITEMS")]
fn add_roadmap_item_rejects_when_capacity_full() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token_address, deadline);

    env.as_contract(&client.address, || {
        let roadmap = filled_roadmap(&env, MAX_ROADMAP_ITEMS);
        env.storage().instance().set(&DataKey::Roadmap, &roadmap);
    });

    let future_date = env.ledger().timestamp() + 7_200;
    let description = soroban_string(&env, 16, 'r');
    client.add_roadmap_item(&future_date, &description);
}

#[test]
#[should_panic(expected = "stretch goals exceed MAX_STRETCH_GOALS")]
fn add_stretch_goal_rejects_when_capacity_full() {
    let (env, client, creator, token_address, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token_address, deadline);

    env.as_contract(&client.address, || {
        let stretch_goals = filled_stretch_goals(&env, MAX_STRETCH_GOALS);
        env.storage()
            .instance()
            .set(&DataKey::StretchGoals, &stretch_goals);
    });

    client.add_stretch_goal(&9_999_999i128);
}
