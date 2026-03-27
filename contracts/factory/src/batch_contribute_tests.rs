#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

use crate::batch_contribute::{batch_contribute, ContributeEntry, MAX_BATCH_SIZE};

// ── Minimal mock campaign ────────────────────────────────────────────────────
// We test batch_contribute in isolation using a mock campaign that records
// calls without needing the full crowdfund WASM.

use soroban_sdk::{contract, contractimpl, contracttype};

#[derive(Clone)]
#[contracttype]
enum MockKey {
    Received,
}

#[contract]
struct MockCampaign;

#[contractimpl]
impl MockCampaign {
    /// Minimal `contribute` stub — records (contributor, amount) pairs.
    pub fn contribute(env: Env, _contributor: Address, amount: i128) {
        if amount <= 0 {
            panic!("zero amount");
        }
        let prev: i128 = env
            .storage()
            .instance()
            .get(&MockKey::Received)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&MockKey::Received, &(prev + amount));
    }

    pub fn total(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&MockKey::Received)
            .unwrap_or(0)
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn setup() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contributor = Address::generate(&env);
    (env, contributor)
}

fn register_campaign(env: &Env) -> Address {
    env.register(MockCampaign, ())
}

fn entry(campaign: Address, amount: i128) -> ContributeEntry {
    ContributeEntry { campaign, amount }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn batch_single_entry_succeeds() {
    let (env, contributor) = setup();
    let c1 = register_campaign(&env);

    let mut entries = Vec::new(&env);
    entries.push_back(entry(c1.clone(), 1_000));

    batch_contribute(&env, &contributor, entries);

    let client = MockCampaignClient::new(&env, &c1);
    assert_eq!(client.total(), 1_000);
}

#[test]
fn batch_multiple_campaigns_all_funded() {
    let (env, contributor) = setup();
    let c1 = register_campaign(&env);
    let c2 = register_campaign(&env);
    let c3 = register_campaign(&env);

    let mut entries = Vec::new(&env);
    entries.push_back(entry(c1.clone(), 500));
    entries.push_back(entry(c2.clone(), 1_500));
    entries.push_back(entry(c3.clone(), 2_000));

    batch_contribute(&env, &contributor, entries);

    assert_eq!(MockCampaignClient::new(&env, &c1).total(), 500);
    assert_eq!(MockCampaignClient::new(&env, &c2).total(), 1_500);
    assert_eq!(MockCampaignClient::new(&env, &c3).total(), 2_000);
}

#[test]
fn batch_at_max_size_succeeds() {
    let (env, contributor) = setup();
    let mut entries = Vec::new(&env);
    for _ in 0..MAX_BATCH_SIZE {
        let c = register_campaign(&env);
        entries.push_back(entry(c, 100));
    }
    // Must not panic.
    batch_contribute(&env, &contributor, entries);
}

#[test]
#[should_panic(expected = "batch exceeds MAX_BATCH_SIZE")]
fn batch_over_max_size_panics() {
    let (env, contributor) = setup();
    let mut entries = Vec::new(&env);
    for _ in 0..MAX_BATCH_SIZE + 1 {
        let c = register_campaign(&env);
        entries.push_back(entry(c, 100));
    }
    batch_contribute(&env, &contributor, entries);
}

#[test]
#[should_panic(expected = "batch is empty")]
fn batch_empty_panics() {
    let (env, contributor) = setup();
    let entries: Vec<ContributeEntry> = Vec::new(&env);
    batch_contribute(&env, &contributor, entries);
}

#[test]
#[should_panic(expected = "batch entry amount must be positive")]
fn batch_zero_amount_entry_panics() {
    let (env, contributor) = setup();
    let c1 = register_campaign(&env);
    let mut entries = Vec::new(&env);
    entries.push_back(entry(c1, 0));
    batch_contribute(&env, &contributor, entries);
}

#[test]
#[should_panic(expected = "batch entry amount must be positive")]
fn batch_negative_amount_entry_panics() {
    let (env, contributor) = setup();
    let c1 = register_campaign(&env);
    let mut entries = Vec::new(&env);
    entries.push_back(entry(c1, -500));
    batch_contribute(&env, &contributor, entries);
}
