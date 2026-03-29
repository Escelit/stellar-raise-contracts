//! # batch_contribute
//!
//! @title   BatchContribute — gas-efficient multi-campaign contribution helper.
//!
//! @notice  Allows a single contributor to fund multiple campaigns in one
//!          transaction, avoiding the per-transaction overhead of N separate
//!          calls.
//!
//! ## Design decisions
//!
//! ### Bounded input
//! `MAX_BATCH_SIZE` caps the number of campaigns per call.  Unbounded loops
//! over caller-supplied arrays are a gas-exhaustion vector; the cap keeps
//! worst-case gas predictable and prevents accidental or malicious oversized
//! batches.
//!
//! ### Fail-fast vs. best-effort
//! The batch uses fail-fast semantics: if any single `contribute` call
//! returns an error the entire batch panics.  This is safer than
//! best-effort (silently skipping failures) because it prevents partial
//! state where the contributor paid for some campaigns but not others
//! without knowing which ones failed.
//!
//! ### No on-chain campaign registry loop
//! The caller supplies the exact campaign addresses.  The factory never
//! iterates over its full `Campaigns` vec on-chain — that list is for
//! off-chain indexers only.
//!
//! ### Mapping-style lookup
//! Per-campaign state (contributions, contributors) is keyed by
//! `DataKey::Contribution(address)` — a typed map — so reads are O(1)
//! regardless of how many campaigns exist.
//!
//! ## Security Assumptions
//! 1. `contributor.require_auth()` is enforced once here; each downstream
//!    `contribute()` call also enforces it via the crowdfund contract.
//! 2. Batch size is capped at `MAX_BATCH_SIZE` to bound gas consumption.
//! 3. Zero-amount entries are rejected before any cross-contract call.

#![allow(dead_code)]

use soroban_sdk::{Address, Env, IntoVal, Symbol, Vec};

/// Maximum number of campaigns that can be funded in a single batch call.
/// Keeps worst-case gas predictable and prevents oversized-array attacks.
pub const MAX_BATCH_SIZE: u32 = 10;

/// A single entry in a batch contribution request.
#[derive(Clone)]
#[soroban_sdk::contracttype]
pub struct ContributeEntry {
    /// The campaign contract address to fund.
    pub campaign: Address,
    /// The token amount to contribute (must be > 0).
    pub amount: i128,
}

/// @notice Contribute to multiple campaigns in a single transaction.
///
/// @dev    Iterates over a caller-supplied, bounded list of `(campaign, amount)`
///         pairs and invokes `contribute(contributor, amount)` on each campaign
///         contract.  Fails atomically — any single failure rolls back the
///         entire batch.
///
/// # Arguments
/// * `env`         – The Soroban environment (factory contract context).
/// * `contributor` – The address funding all campaigns; must authorize this call.
/// * `entries`     – Bounded list of `ContributeEntry` (max `MAX_BATCH_SIZE`).
///
/// # Panics
/// * If `entries.len() > MAX_BATCH_SIZE`.
/// * If any `entry.amount <= 0`.
/// * If any downstream `contribute()` call fails.
///
/// # Gas profile
/// O(n) cross-contract calls where n <= MAX_BATCH_SIZE (constant upper bound).
pub fn batch_contribute(env: &Env, contributor: &Address, entries: Vec<ContributeEntry>) {
    contributor.require_auth();

    let len = entries.len();
    if len == 0 {
        panic!("batch is empty");
    }
    if len > MAX_BATCH_SIZE {
        panic!("batch exceeds MAX_BATCH_SIZE");
    }

    for entry in entries.iter() {
        if entry.amount <= 0 {
            panic!("batch entry amount must be positive");
        }

        // Cross-contract call: invoke `contribute(contributor, amount)` on
        // each campaign.  Uses `invoke_contract` so the factory does not need
        // to import the crowdfund ABI — keeping the factory lean.
        let _: () = env.invoke_contract(
            &entry.campaign,
            &Symbol::new(env, "contribute"),
            soroban_sdk::vec![env, contributor.clone().into(), entry.amount.into(),],
            soroban_sdk::vec![
                env,
                contributor.clone().into(),
                entry.amount.into(),
            ],
            soroban_sdk::vec![env, contributor.clone().into_val(env), entry.amount.into_val(env),],
        );
    }

    // Emit a single summary event — cheaper than one event per campaign.
    env.events().publish(
        (Symbol::new(env, "batch"), Symbol::new(env, "contributed")),
        (
            Symbol::new(env, "batch"),
            Symbol::new(env, "contributed"),
        ),
        (contributor.clone(), len),
    );
}
