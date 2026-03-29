use crate::{FactoryContract, FactoryContractClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env};
#![cfg(test)]
#![allow(clippy::too_many_arguments)]

use crate::{FactoryContract, FactoryContractClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env};
use soroban_sdk::{
    testutils::Address as _,
    token, Address, Env,
};

extern crate std;

// Import the crowdfund contract WASM.
#[allow(clippy::too_many_arguments)]
mod crowdfund_wasm {
    soroban_sdk::contractimport!(file = "../../contracts/factory/wasm/crowdfund.wasm");
}

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (Address, token::StellarAssetClient<'a>) {
mod crowdfund_wasm {
    soroban_sdk::contractimport!(file = "wasm/crowdfund.wasm");
    soroban_sdk::contractimport!(
        file = "../wasm/crowdfund.wasm"
    );
    soroban_sdk::contractimport!(file = "../wasm/crowdfund.wasm");
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/crowdfund.wasm");
    soroban_sdk::contractimport!(
        file = "../../contracts/factory/wasm/crowdfund.wasm"
    );
    soroban_sdk::contractimport!(file = "../../contracts/factory/wasm/crowdfund.wasm");
}

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (Address, token::StellarAssetClient<'a>) {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/crowdfund.wasm"
    );
}

fn create_token_contract<'a>(env: &Env, admin: &Address) -> (Address, token::StellarAssetClient<'a>) {
    let token_contract_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_contract_id.address();
    let token_client = token::StellarAssetClient::new(env, &token_address);
    (token_address, token_client)
}

#[test]
#[ignore = "requires wasm-opt for WASM optimization"]
fn test_create_single_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let factory_id = env.register(FactoryContract, ());
    let factory = FactoryContractClient::new(&env, &factory_id);

    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_address, _token_client) = create_token_contract(&env, &token_admin);

    // Upload the crowdfund WASM.
    let wasm_hash = env.deployer().upload_contract_wasm(crowdfund_wasm::WASM);

    let goal = 1000i128;
    let deadline = 100u64;

    let campaign_addr =
        factory.create_campaign(&creator, &token_address, &goal, &deadline, &wasm_hash);
    let campaign_addr = factory.create_campaign(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &wasm_hash,
    );

    // Verify campaign was added to registry.
    let campaigns = factory.campaigns();
    assert_eq!(campaigns.len(), 1);
    assert_eq!(campaigns.get(0).unwrap(), campaign_addr);

    // Verify count.
    assert_eq!(factory.campaign_count(), 1);
}

#[test]
#[ignore = "requires wasm-opt for WASM optimization"]
fn test_create_multiple_campaigns() {
    let env = Env::default();
    env.mock_all_auths();

    let factory_id = env.register(FactoryContract, ());
    let factory = FactoryContractClient::new(&env, &factory_id);

    let token_admin = Address::generate(&env);
    let (token_address, _token_client) = create_token_contract(&env, &token_admin);

    let wasm_hash = env.deployer().upload_contract_wasm(crowdfund_wasm::WASM);

    // Deploy 3 campaigns with different creators.
    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);
    let creator3 = Address::generate(&env);

    let campaign1 =
        factory.create_campaign(&creator1, &token_address, &1000i128, &100u64, &wasm_hash);

    let campaign2 =
        factory.create_campaign(&creator2, &token_address, &2000i128, &200u64, &wasm_hash);

    let campaign3 =
        factory.create_campaign(&creator3, &token_address, &3000i128, &300u64, &wasm_hash);
    let campaign1 = factory.create_campaign(
        &creator1,
        &token_address,
        &1000i128,
        &100u64,
        &wasm_hash,
    );

    let campaign2 =
        factory.create_campaign(&creator2, &token_address, &2000i128, &200u64, &wasm_hash);

    let campaign3 =
        factory.create_campaign(&creator3, &token_address, &3000i128, &300u64, &wasm_hash);
    let campaign2 = factory.create_campaign(
        &creator2,
        &token_address,
        &2000i128,
        &200u64,
        &wasm_hash,
    );

    let campaign3 = factory.create_campaign(
        &creator3,
        &token_address,
        &3000i128,
        &300u64,
        &wasm_hash,
    );

    // Verify all campaigns are in registry.
    let campaigns = factory.campaigns();
    assert_eq!(campaigns.len(), 3);
    assert_eq!(campaigns.get(0).unwrap(), campaign1);
    assert_eq!(campaigns.get(1).unwrap(), campaign2);
    assert_eq!(campaigns.get(2).unwrap(), campaign3);

    // Verify count.
    assert_eq!(factory.campaign_count(), 3);
}
use soroban_sdk::Env;

#[test]
fn test_empty_registry() {
    let env = Env::default();

    let factory_id = env.register(FactoryContract, ());
    let factory = FactoryContractClient::new(&env, &factory_id);

    // Verify empty state.
    let campaigns = factory.campaigns();
    assert_eq!(campaigns.len(), 0);
    assert_eq!(factory.campaign_count(), 0);
}
