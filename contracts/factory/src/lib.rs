#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, IntoVal, Symbol, Vec,
};

#[cfg(test)]
mod test;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// List of all deployed campaign addresses.
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
        let _: () = env.invoke_contract(
            &deployed_address,
            &Symbol::new(&env, "initialize"),
            soroban_sdk::vec![
                &env,
                creator.into_val(&env),
                token.into_val(&env),
                goal.into_val(&env),
                deadline.into_val(&env)
            ],
        );

        // Add to registry.
        let mut campaigns: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Campaigns)
            .unwrap_or(Vec::new(&env));
        campaigns.push_back(deployed_address.clone());
        env.storage()
            .instance()
            .set(&DataKey::Campaigns, &campaigns);

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
        let campaigns: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Campaigns)
            .unwrap_or(Vec::new(&env));
        campaigns.len()
    }
}
