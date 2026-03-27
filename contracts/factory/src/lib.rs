#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, IntoVal, Symbol, Vec,
};

#[cfg(test)]
mod test;

pub mod batch_contribute;
use batch_contribute::ContributeEntry;
#[cfg(test)]
mod batch_contribute_tests;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Total number of deployed campaigns — used as the next index key.
    /// Replaces iterating over an unbounded Vec for on-chain logic.
    CampaignCount,
    /// Per-index campaign address: O(1) lookup by index.
    Campaign(u32),
    /// Full ordered list kept for off-chain indexers / the `campaigns()` view.
    /// Never iterated on-chain in core logic.
    Campaigns,
}

#[contract]
pub struct FactoryContract;

#[contractimpl]
impl FactoryContract {
    /// Deploy a new crowdfund campaign contract.
    ///
    /// # Arguments
    /// * `creator`   – The campaign creator's address.
    /// * `token`     – The token contract address used for contributions.
    /// * `goal`      – The funding goal (in the token's smallest unit).
    /// * `deadline`  – The campaign deadline as a ledger timestamp.
    /// * `wasm_hash` – The hash of the crowdfund contract WASM to deploy.
    ///
    /// # Returns
    /// The address of the newly deployed campaign contract.
    pub fn create_campaign(
        env: Env,
        creator: Address,
        token: Address,
        goal: i128,
        deadline: u64,
        wasm_hash: BytesN<32>,
    ) -> Address {
        creator.require_auth();

        // Deploy the crowdfund contract from the WASM hash.
        let salt = BytesN::from_array(&env, &[0; 32]);
        let deployed_address = env
            .deployer()
            .with_address(creator.clone(), salt)
            .deploy_v2(wasm_hash, ());

        // Initialize the deployed contract.
        // Keep factory API stable: use default min contribution and no platform config.
        let min_contribution: i128 = 1_000;
        let no_platform_config: Option<soroban_sdk::Val> = None;
        let no_bonus_goal: Option<i128> = None;
        let no_bonus_description: Option<soroban_sdk::String> = None;
        let _: () = env.invoke_contract(
            &deployed_address,
            &Symbol::new(&env, "initialize"),
            soroban_sdk::vec![
                &env,
                creator.clone().into_val(&env),
                creator.into_val(&env),
                token.into_val(&env),
                goal.into_val(&env),
                deadline.into_val(&env),
                min_contribution.into_val(&env),
                no_platform_config.into_val(&env),
                no_bonus_goal.into_val(&env),
                no_bonus_description.into_val(&env)
            ],
        );

        // Add to registry — both the ordered Vec (for indexers) and the
        // O(1) index map (for on-chain lookups).
        let mut campaigns: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Campaigns)
            .unwrap_or(Vec::new(&env));

        let idx: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CampaignCount)
            .unwrap_or(0);

        campaigns.push_back(deployed_address.clone());
        env.storage()
            .instance()
            .set(&DataKey::Campaigns, &campaigns);

        // Mapping-style storage: Campaign(idx) → address, O(1) read.
        env.storage()
            .instance()
            .set(&DataKey::Campaign(idx), &deployed_address);
        env.storage()
            .instance()
            .set(&DataKey::CampaignCount, &(idx + 1));

        deployed_address
    }

    /// Returns the list of all deployed campaign addresses.
    pub fn campaigns(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Campaigns)
            .unwrap_or(Vec::new(&env))
    }

    /// Returns the total number of deployed campaigns.
    pub fn campaign_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CampaignCount)
            .unwrap_or(0)
    }

    /// O(1) campaign lookup by index — use this on-chain instead of
    /// iterating over the full `campaigns()` list.
    ///
    /// # Panics
    /// If `index >= campaign_count`.
    pub fn campaign_by_index(env: Env, index: u32) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Campaign(index))
            .expect("index out of bounds")
    }

    /// Contribute to multiple campaigns in a single transaction.
    ///
    /// Delegates to [`batch_contribute::batch_contribute`].
    /// Capped at [`batch_contribute::MAX_BATCH_SIZE`] entries.
    ///
    /// # Arguments
    /// * `contributor` – The address funding all campaigns; must authorize.
    /// * `entries`     – List of `(campaign, amount)` pairs (max 10).
    pub fn batch_contribute(env: Env, contributor: Address, entries: Vec<ContributeEntry>) {
        batch_contribute::batch_contribute(&env, &contributor, entries);
    }
}
