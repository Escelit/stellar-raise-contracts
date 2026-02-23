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
// Factory contract for batch campaign initialization
// Implements Issue #68 and extends Issue #23

use soroban_sdk::{contractimpl, contracttype, BytesN, Address, Env, Symbol, String, Vec};

// Registry key for storing deployed campaigns
const REGISTRY_KEY: &str = "campaign_registry";

// The WASM hash for the crowdfund contract (should be set to the correct value in production)
const CROWDFUND_WASM_HASH: [u8; 32] = [0u8; 32]; // TODO: Replace with actual hash

#[contracttype]
pub struct BatchCreatedEvent {
    pub count: u32,
    pub addresses: Vec<Address>,
}
#[derive(Clone)]
pub struct CampaignConfig {
    pub creator: Address,
    pub token: Address,
    pub goal: i128,
    pub deadline: u64,
    pub title: String,
    pub description: String,
}

#[derive(Clone)]
pub struct FactoryContract;

#[derive(Debug, PartialEq)]
pub enum ContractError {
    EmptyBatch,
    InvalidConfig { index: usize },
    // ...other errors
}

#[contractimpl]
impl FactoryContract {
    pub fn create_campaigns_batch(
        env: Env,
        configs: Vec<CampaignConfig>,
    ) -> Result<Vec<Address>, ContractError> {
        if configs.is_empty() {
            return Err(ContractError::EmptyBatch);
        }
        let mut deployed = Vec::new(&env);
        // Validate all configs first
        for (i, config) in configs.iter().enumerate() {
            if config.goal <= 0 || config.title.is_empty() || config.description.is_empty() {
                return Err(ContractError::InvalidConfig { index: i });
            }
        }
        // Deploy and initialize all campaigns
        for config in configs.iter() {
            let campaign_addr = deploy_and_init_campaign(&env, config);
            deployed.push_back(campaign_addr);
        }
        // Store all deployed addresses in the factory registry
        let mut registry: Vec<Address> = env
            .storage()
            .persistent()
            .get(&REGISTRY_KEY.into())
            .unwrap_or(Vec::new(&env));
        for addr in deployed.iter() {
            registry.push_back(addr.clone());
        }
        env.storage().persistent().set(&REGISTRY_KEY.into(), &registry);
        // Emit batch_campaigns_created event
        let event = BatchCreatedEvent {
            count: deployed.len() as u32,
            addresses: deployed.clone(),
        };
        env.events().publish(("factory", "batch_campaigns_created"), event);
        Ok(deployed)
    }
}

fn deploy_and_init_campaign(env: &Env, config: &CampaignConfig) -> Address {
    // Deploy the crowdfund contract
    let wasm_hash = BytesN::from_array(env, &CROWDFUND_WASM_HASH);
    let campaign_addr = env
        .deployer()
        .with_current_contract(env.current_contract_address())
        .deploy_contract(wasm_hash);
    // Call initialize on the deployed contract
    // NOTE: Hard cap, min_contribution, platform_config are set to defaults for this example
    let hard_cap = config.goal;
    let min_contribution = 1i128;
    let platform_config: Option<()> = None;
    env.invoke_contract(
        &campaign_addr,
        &Symbol::short("initialize"),
        (
            config.creator.clone(),
            config.token.clone(),
            config.goal,
            hard_cap,
            config.deadline,
            min_contribution,
            platform_config,
        ),
    );
    campaign_addr
}
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

    #[test]
    fn test_batch_deploys_campaigns() {
        let env = Env::default();
        let configs = Vec::from_array(
            &env,
            [
                CampaignConfig {
                    creator: Address::random(&env),
                    token: Address::random(&env),
                    goal: 1000,
                    deadline: 123456,
                    title: "Campaign 1".to_string(),
                    description: "Desc 1".to_string(),
                },
                CampaignConfig {
                    creator: Address::random(&env),
                    token: Address::random(&env),
                    goal: 2000,
                    deadline: 223456,
                    title: "Campaign 2".to_string(),
                    description: "Desc 2".to_string(),
                },
                CampaignConfig {
                    creator: Address::random(&env),
                    token: Address::random(&env),
                    goal: 3000,
                    deadline: 323456,
                    title: "Campaign 3".to_string(),
                    description: "Desc 3".to_string(),
                },
            ],
        );
        let result = FactoryContract::create_campaigns_batch(env.clone(), configs.clone());
        assert!(result.is_ok());
        let deployed = result.unwrap();
        assert_eq!(deployed.len(), 3);
        // TODO: Check registry and returned addresses
    }

    #[test]
    fn test_empty_batch_rejected() {
        let env = Env::default();
        let configs = Vec::new(&env);
        let result = FactoryContract::create_campaigns_batch(env, configs);
        assert_eq!(result, Err(ContractError::EmptyBatch));
    }

    #[test]
    fn test_invalid_config_rolls_back_batch() {
        let env = Env::default();
        let configs = Vec::from_array(
            &env,
            [
                CampaignConfig {
                    creator: Address::random(&env),
                    token: Address::random(&env),
                    goal: 1000,
                    deadline: 123456,
                    title: "Valid".to_string(),
                    description: "Valid".to_string(),
                },
                CampaignConfig {
                    creator: Address::random(&env),
                    token: Address::random(&env),
                    goal: -1, // Invalid goal
                    deadline: 223456,
                    title: "Invalid".to_string(),
                    description: "Invalid".to_string(),
                },
            ],
        );
        let result = FactoryContract::create_campaigns_batch(env, configs);
        assert_eq!(result, Err(ContractError::InvalidConfig { index: 1 }));
    }
}

// TODO: Add tests for batch deployment and error handling
