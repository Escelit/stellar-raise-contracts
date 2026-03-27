//! Tests for bounded `withdraw()` event emission.
//!
//! Covers:
//! - NFT minting cap (below, at, and above `MAX_NFT_MINT_BATCH`)
//! - Single `nft_batch_minted` summary event (not one per contributor)
//! - `withdrawn` event emitted exactly once with correct payload
//! - `fee_transferred` event emitted with correct payload
//! - No `nft_batch_minted` event when NFT contract is not configured
//! - `withdrawn` event payout reflects platform fee deduction
//! - Double-withdraw is blocked (status guard)
//! - Security: `emit_fee_transferred` panics on zero/negative fee
//! - Security: `emit_nft_batch_minted` panics on zero count
//! - Security: `emit_withdrawn` panics on zero/negative payout

extern crate std;

use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    token, Address, Env, TryFromVal, Val,
};

use crate::{
    withdraw_event_emission::{emit_fee_transferred, emit_nft_batch_minted, emit_withdrawn},
    CrowdfundContract, CrowdfundContractClient, PlatformConfig, MAX_NFT_MINT_BATCH,
};

// ── Mock NFT contract ────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
enum MockNftKey {
    Count,
}

#[contract]
struct MockNft;

#[contractimpl]
impl MockNft {
    pub fn mint(env: Env, _to: Address, _token_id: u64) {
        let n: u32 = env
            .storage()
            .instance()
            .get(&MockNftKey::Count)
            .unwrap_or(0);
        env.storage().instance().set(&MockNftKey::Count, &(n + 1));
    }
    pub fn count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&MockNftKey::Count)
            .unwrap_or(0)
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Set up a campaign with `contributor_count` contributors and an NFT contract.
/// Advances past the deadline and calls `finalize()` so `withdraw()` is ready.
fn setup_with_nft(
    contributor_count: u32,
) -> (
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
    let token_addr = token_id.address();
    let sac = token::StellarAssetClient::new(&env, &token_addr);

    let creator = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 3_600;
    let goal = contributor_count as i128 * 100;

    client.initialize(
        &creator,
        &creator,
        &token_addr,
        &goal,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
    );

    let nft_id = env.register(MockNft, ());
    client.set_nft_contract(&creator, &nft_id);

    for _ in 0..contributor_count {
        let c = Address::generate(&env);
        sac.mint(&c, &100);
        client.contribute(&c, &100);
    }

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();

    (env, client, creator, token_addr, nft_id)
}

/// Set up a campaign without an NFT contract.
fn setup_no_nft(contribution: i128) -> (Env, CrowdfundContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let sac = token::StellarAssetClient::new(&env, &token_addr);

    let creator = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 3_600;

    client.initialize(
        &creator,
        &creator,
        &token_addr,
        &contribution,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
    );

    let c = Address::generate(&env);
    sac.mint(&c, &contribution);
    client.contribute(&c, &contribution);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();

    (env, client, creator, token_addr)
}

/// Count events whose first two topics match the given string pair.
fn count_events(env: &Env, t1: &str, t2: &str) -> usize {
    env.events()
        .all()
        .iter()
        .filter(|(_, topics, _)| {
            topics.len() >= 2
                && topics
                    .get(0)
                    .map(|v| v == soroban_sdk::Symbol::new(env, t1).into())
                    .unwrap_or(false)
                && topics
                    .get(1)
                    .map(|v| v == soroban_sdk::Symbol::new(env, t2).into())
                    .unwrap_or(false)
        })
        .count()
}

/// Return the data Val of the first matching event.
fn event_data(env: &Env, t1: &str, t2: &str) -> Option<Val> {
    env.events()
        .all()
        .iter()
        .find(|(_, topics, _)| {
            topics.len() >= 2
                && topics
                    .get(0)
                    .map(|v| v == soroban_sdk::Symbol::new(env, t1).into())
                    .unwrap_or(false)
                && topics
                    .get(1)
                    .map(|v| v == soroban_sdk::Symbol::new(env, t2).into())
                    .unwrap_or(false)
        })
        .map(|(_, _, data)| data)
}

// ── NFT minting cap ───────────────────────────────────────────────────────────

#[test]
fn test_withdraw_mints_all_when_within_cap() {
    let count = MAX_NFT_MINT_BATCH - 1;
    let (env, client, _creator, _token, nft_id) = setup_with_nft(count);
    client.withdraw();
    assert_eq!(MockNftClient::new(&env, &nft_id).count(), count);
}

#[test]
fn test_withdraw_caps_minting_at_max_batch() {
    let count = MAX_NFT_MINT_BATCH + 5;
    let (env, client, _creator, _token, nft_id) = setup_with_nft(count);
    client.withdraw();
    assert_eq!(
        MockNftClient::new(&env, &nft_id).count(),
        MAX_NFT_MINT_BATCH
    );
}

#[test]
fn test_withdraw_mints_exactly_at_cap_boundary() {
    let (env, client, _creator, _token, nft_id) = setup_with_nft(MAX_NFT_MINT_BATCH);
    client.withdraw();
    assert_eq!(
        MockNftClient::new(&env, &nft_id).count(),
        MAX_NFT_MINT_BATCH
    );
}

#[test]
fn test_withdraw_mints_single_contributor() {
    let (env, client, _creator, _token, nft_id) = setup_with_nft(1);
    client.withdraw();
    assert_eq!(MockNftClient::new(&env, &nft_id).count(), 1);
}

// ── nft_batch_minted event ────────────────────────────────────────────────────

#[test]
fn test_withdraw_emits_single_batch_event() {
    let (env, client, _creator, _token, _nft) = setup_with_nft(5);
    client.withdraw();
    assert_eq!(count_events(&env, "campaign", "nft_batch_minted"), 1);
}

#[test]
fn test_withdraw_no_batch_event_without_nft_contract() {
    let (env, client, _creator, _token) = setup_no_nft(1_000);
    client.withdraw();
    assert_eq!(count_events(&env, "campaign", "nft_batch_minted"), 0);
}

#[test]
fn test_withdraw_batch_event_data_equals_minted_count() {
    let count: u32 = 3;
    let (env, client, _creator, _token, _nft) = setup_with_nft(count);
    client.withdraw();
    let data = event_data(&env, "campaign", "nft_batch_minted").expect("event not found");
    let minted: u32 = u32::try_from_val(&env, &data).expect("not u32");
    assert_eq!(minted, count);
}

#[test]
fn test_withdraw_batch_event_data_capped_at_max() {
    let (env, client, _creator, _token, _nft) = setup_with_nft(MAX_NFT_MINT_BATCH + 5);
    client.withdraw();
    let data = event_data(&env, "campaign", "nft_batch_minted").expect("event not found");
    let minted: u32 = u32::try_from_val(&env, &data).expect("not u32");
    assert_eq!(minted, MAX_NFT_MINT_BATCH);
}

// ── withdrawn event ───────────────────────────────────────────────────────────

#[test]
fn test_withdraw_emits_withdrawn_event_once() {
    let (env, client, _creator, _token, _nft) = setup_with_nft(2);
    client.withdraw();
    assert_eq!(count_events(&env, "campaign", "withdrawn"), 1);
}

#[test]
fn test_withdraw_emits_withdrawn_event_without_nft() {
    let (env, client, _creator, _token) = setup_no_nft(1_000);
    client.withdraw();
    assert_eq!(count_events(&env, "campaign", "withdrawn"), 1);
}

#[test]
fn test_withdrawn_event_nft_count_zero_without_nft_contract() {
    let (env, client, _creator, _token) = setup_no_nft(1_000);
    client.withdraw();
    let data = event_data(&env, "campaign", "withdrawn").expect("event not found");
    let tuple: (Address, i128, u32) =
        <(Address, i128, u32)>::try_from_val(&env, &data).expect("shape mismatch");
    assert_eq!(tuple.2, 0u32);
}

/// `withdrawn` event payout equals total_raised when no platform fee.
#[test]
fn test_withdrawn_event_payout_equals_total_raised_no_fee() {
    let contribution: i128 = 5_000;
    let (env, client, creator, _token) = setup_no_nft(contribution);
    client.withdraw();
    let data = event_data(&env, "campaign", "withdrawn").expect("event not found");
    let tuple: (Address, i128, u32) =
        <(Address, i128, u32)>::try_from_val(&env, &data).expect("shape mismatch");
    assert_eq!(tuple.0, creator);
    assert_eq!(tuple.1, contribution);
}

/// `withdrawn` event payout equals total_raised minus platform fee.
#[test]
fn test_withdrawn_event_payout_reflects_fee_deduction() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let sac = token::StellarAssetClient::new(&env, &token_addr);

    let creator = Address::generate(&env);
    let platform_addr = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 1_000_000;

    client.initialize(
        &creator,
        &creator,
        &token_addr,
        &goal,
        &deadline,
        &1,
        &Some(PlatformConfig {
            address: platform_addr,
            fee_bps: 500,
        }), // 5%
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    let c = Address::generate(&env);
    sac.mint(&c, &goal);
    client.contribute(&c, &goal);
    env.ledger().set_timestamp(deadline + 1);
    client.finalize();
    client.withdraw();

    let data = event_data(&env, "campaign", "withdrawn").expect("event not found");
    let tuple: (Address, i128, u32) =
        <(Address, i128, u32)>::try_from_val(&env, &data).expect("shape mismatch");
    // 5% of 1_000_000 = 50_000 fee; creator payout = 950_000
    assert_eq!(tuple.1, 950_000);
}

// ── fee_transferred event ─────────────────────────────────────────────────────

#[test]
fn test_withdraw_emits_fee_transferred_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let sac = token::StellarAssetClient::new(&env, &token_addr);

    let creator = Address::generate(&env);
    let platform_addr = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 1_000_000;

    client.initialize(
        &creator,
        &creator,
        &token_addr,
        &goal,
        &deadline,
        &1,
        &Some(PlatformConfig {
            address: platform_addr,
            fee_bps: 200,
        }), // 2%
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    let c = Address::generate(&env);
    sac.mint(&c, &goal);
    client.contribute(&c, &goal);
    env.ledger().set_timestamp(deadline + 1);
    client.finalize();
    client.withdraw();

    assert_eq!(count_events(&env, "campaign", "fee_transferred"), 1);
}

#[test]
fn test_withdraw_no_fee_event_without_platform_config() {
    let (env, client, _creator, _token) = setup_no_nft(1_000);
    client.withdraw();
    assert_eq!(count_events(&env, "campaign", "fee_transferred"), 0);
}

// ── Double-withdraw guard ─────────────────────────────────────────────────────

/// Second withdraw() call panics because status is no longer Succeeded.
#[test]
#[should_panic]
fn test_double_withdraw_panics() {
    let (_, client, _creator, _token) = setup_no_nft(1_000);
    client.withdraw();
    client.withdraw(); // must panic — status is no longer Succeeded
}

// ── Security unit tests for emit helpers ──────────────────────────────────────

#[test]
#[should_panic(expected = "fee_transferred: fee must be positive")]
fn test_emit_fee_transferred_panics_on_zero_fee() {
    let env = Env::default();
    emit_fee_transferred(&env, &Address::generate(&env), 0);
}

#[test]
#[should_panic(expected = "fee_transferred: fee must be positive")]
fn test_emit_fee_transferred_panics_on_negative_fee() {
    let env = Env::default();
    emit_fee_transferred(&env, &Address::generate(&env), -1);
}

#[test]
fn test_emit_fee_transferred_succeeds_with_positive_fee() {
    let env = Env::default();
    emit_fee_transferred(&env, &Address::generate(&env), 1);
}

#[test]
#[should_panic(expected = "nft_batch_minted: minted_count must be positive")]
fn test_emit_nft_batch_minted_panics_on_zero_count() {
    let env = Env::default();
    emit_nft_batch_minted(&env, 0);
}

#[test]
fn test_emit_nft_batch_minted_succeeds_with_positive_count() {
    let env = Env::default();
    emit_nft_batch_minted(&env, 1);
}

#[test]
#[should_panic(expected = "withdrawn: creator_payout must be positive")]
fn test_emit_withdrawn_panics_on_zero_payout() {
    let env = Env::default();
    emit_withdrawn(&env, &Address::generate(&env), 0, 0);
}

#[test]
#[should_panic(expected = "withdrawn: creator_payout must be positive")]
fn test_emit_withdrawn_panics_on_negative_payout() {
    let env = Env::default();
    emit_withdrawn(&env, &Address::generate(&env), -100, 0);
}

#[test]
fn test_emit_withdrawn_succeeds_with_valid_args() {
    let env = Env::default();
    emit_withdrawn(&env, &Address::generate(&env), 1_000, 5);
}

#[test]
fn test_emit_withdrawn_allows_zero_nft_count() {
    let env = Env::default();
    emit_withdrawn(&env, &Address::generate(&env), 500, 0);
}

// ── Security: fee_bps in fee_transferred event ───────────────────────────────

/// `fee_transferred` event data includes fee_bps for independent verification.
#[test]
fn test_fee_transferred_event_data_includes_fee_bps() {
    use soroban_sdk::TryFromVal;

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let sac = token::StellarAssetClient::new(&env, &token_addr);

    let creator = Address::generate(&env);
    let platform_addr = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let fee_bps: u32 = 300; // 3%

    let config = PlatformConfig {
        address: platform_addr.clone(),
        fee_bps,
    };

    client.initialize(
        &creator,
        &creator,
        &token_addr,
        &goal,
        &deadline,
        &1,
        &Some(config),
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    sac.mint(&contributor, &goal);
    client.contribute(&contributor, &goal);
    env.ledger().set_timestamp(deadline + 1);
    client.withdraw();

    let data = first_event_data(&env, "campaign", "fee_transferred")
        .expect("fee_transferred event not found");
    let tuple: (Address, i128, u32) =
        <(Address, i128, u32)>::try_from_val(&env, &data).expect("data shape mismatch");

    // fee = 1_000_000 * 300 / 10_000 = 30_000
    assert_eq!(tuple.1, 30_000, "fee amount mismatch");
    assert_eq!(tuple.2, fee_bps, "fee_bps must be included in event data");
}

/// `fee_transferred` event fee_bps matches the configured platform fee.
#[test]
fn test_fee_transferred_event_fee_bps_matches_config() {
    use soroban_sdk::TryFromVal;

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let sac = token::StellarAssetClient::new(&env, &token_addr);

    let creator = Address::generate(&env);
    let platform_addr = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 500_000;
    let fee_bps: u32 = 100; // 1%

    let config = PlatformConfig {
        address: platform_addr.clone(),
        fee_bps,
    };

    client.initialize(
        &creator,
        &creator,
        &token_addr,
        &goal,
        &deadline,
        &1,
        &Some(config),
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    sac.mint(&contributor, &goal);
    client.contribute(&contributor, &goal);
    env.ledger().set_timestamp(deadline + 1);
    client.withdraw();

    let data = first_event_data(&env, "campaign", "fee_transferred")
        .expect("fee_transferred event not found");
    let tuple: (Address, i128, u32) =
        <(Address, i128, u32)>::try_from_val(&env, &data).expect("data shape mismatch");
    assert_eq!(tuple.2, fee_bps);
}

// ── Security: timestamp in withdrawn event ───────────────────────────────────

/// `withdrawn` event data includes the ledger timestamp.
#[test]
fn test_withdrawn_event_includes_ledger_timestamp() {
    use soroban_sdk::TryFromVal;

    let contribution: i128 = 2_000;
    let (env, client, _creator, _token) = setup_no_nft(contribution);

    let ts = env.ledger().timestamp();
    client.withdraw();

    let data = first_event_data(&env, "campaign", "withdrawn").expect("withdrawn event not found");
    let tuple: (Address, i128, u32, u64) =
        <(Address, i128, u32, u64)>::try_from_val(&env, &data).expect("data shape mismatch");

    assert_eq!(
        tuple.3, ts,
        "timestamp in event must match ledger at withdrawal time"
    );
}

/// Two withdrawals at different timestamps produce different timestamp fields.
/// (Replay detection: same creator + payout but different timestamp = distinct event.)
#[test]
fn test_withdrawn_event_timestamp_changes_between_calls() {
    use soroban_sdk::TryFromVal;

    // First campaign
    let (env1, client1, _creator1, _token1) = setup_no_nft(1_000);
    let ts1 = env1.ledger().timestamp();
    client1.withdraw();
    let data1 =
        first_event_data(&env1, "campaign", "withdrawn").expect("withdrawn event not found");
    let tuple1: (Address, i128, u32, u64) =
        <(Address, i128, u32, u64)>::try_from_val(&env1, &data1).unwrap();

    // Second campaign at a later timestamp
    let (env2, client2, _creator2, _token2) = setup_no_nft(1_000);
    env2.ledger().set_timestamp(ts1 + 100);
    client2.withdraw();
    let data2 =
        first_event_data(&env2, "campaign", "withdrawn").expect("withdrawn event not found");
    let tuple2: (Address, i128, u32, u64) =
        <(Address, i128, u32, u64)>::try_from_val(&env2, &data2).unwrap();

    assert_ne!(
        tuple1.3, tuple2.3,
        "timestamps must differ between withdrawals"
    );
}

// ── Security: emit helper — fee_bps boundary ─────────────────────────────────

/// `emit_fee_transferred` panics when fee_bps exceeds MAX_FEE_BPS.
#[test]
#[should_panic(expected = "fee_transferred: fee_bps exceeds MAX_FEE_BPS")]
fn test_emit_fee_transferred_panics_on_fee_bps_above_max() {
    use crate::withdraw_event_emission::MAX_FEE_BPS;
    let env = Env::default();
    let addr = Address::generate(&env);
    emit_fee_transferred(&env, &addr, 100, MAX_FEE_BPS + 1);
}

/// `emit_fee_transferred` accepts fee_bps == MAX_FEE_BPS (boundary).
#[test]
fn test_emit_fee_transferred_accepts_max_fee_bps_boundary() {
    use crate::withdraw_event_emission::MAX_FEE_BPS;
    let env = Env::default();
    let addr = Address::generate(&env);
    emit_fee_transferred(&env, &addr, 1_000, MAX_FEE_BPS);
}
