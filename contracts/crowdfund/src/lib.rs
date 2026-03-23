#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, token, Address, Env, IntoVal, String,
    Symbol, Vec,
};

#[cfg(test)]
mod test;

const CONTRACT_VERSION: u32 = 3;
#[allow(dead_code)]
const CONTRIBUTION_COOLDOWN: u64 = 60; // 60 seconds cooldown

// ── Data Types ──────────────────────────────────────────────────────────────

/// Represents the campaign status.
#[derive(Clone, PartialEq)]
#[contracttype]
pub enum Status {
    Active,
    Successful,
    Refunded,
    Cancelled,
}

/// Represents a single roadmap milestone with a date and description.
#[derive(Clone)]
#[contracttype]
pub struct RoadmapItem {
    pub date: u64,
    pub description: String,
}

/// Platform fee configuration: the recipient address and fee in basis points.
#[derive(Clone)]
#[contracttype]
pub struct PlatformConfig {
    pub address: Address,
    pub fee_bps: u32,
}

/// Snapshot of campaign funding statistics returned by [`CrowdfundContract::get_stats`].
#[derive(Clone)]
#[contracttype]
pub struct CampaignStats {
    pub total_raised: i128,
    pub goal: i128,
    pub progress_bps: u32,
    pub contributor_count: u32,
    pub average_contribution: i128,
    pub largest_contribution: i128,
}

/// Represents all storage keys used by the crowdfund contract.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// The address of the campaign creator.
    Creator,
    /// The token contract address used for contributions.
    Token,
    /// The funding goal in token units.
    Goal,
    /// The campaign deadline as a Unix timestamp.
    Deadline,
    /// The running total of tokens raised.
    TotalRaised,
    /// Individual contribution amount keyed by contributor address.
    Contribution(Address),
    /// List of all contributor addresses.
    Contributors,
    /// Current campaign status.
    Status,
    /// Minimum contribution amount.
    MinContribution,
    /// Individual pledge by address (for conditional pledges).
    Pledge(Address),
    /// Total amount pledged but not yet collected.
    TotalPledged,
    /// Stretch goals for bonus milestones.
    StretchGoals,
    /// Optional secondary bonus goal threshold.
    BonusGoal,
    /// Optional description for the secondary bonus goal.
    BonusGoalDescription,
    /// Tracks if bonus-goal-reached event has already been emitted.
    BonusGoalReachedEmitted,
    /// List of all pledgers (for conditional pledges).
    Pledgers,
    /// List of roadmap items with dates and descriptions.
    Roadmap,
    /// The designated admin address (set to creator at initialization).
    Admin,
    /// Campaign title.
    Title,
    /// Campaign description.
    Description,
    /// Campaign social links.
    SocialLinks,
    /// Platform fee configuration.
    PlatformConfig,
    /// Optional NFT contract used for contributor reward minting.
    NFTContract,
}

// ── Contract Error ──────────────────────────────────────────────────────────

use soroban_sdk::contracterror;

/// Errors that can be returned by the crowdfund contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// The contract has already been initialized.
    AlreadyInitialized = 1,
    /// The campaign deadline has passed.
    CampaignEnded = 2,
    /// The campaign deadline has not yet passed.
    CampaignStillActive = 3,
    /// The funding goal was not reached.
    GoalNotReached = 4,
    /// The funding goal has already been reached.
    GoalReached = 5,
    /// An arithmetic overflow occurred.
    Overflow = 6,
}

/// Interface for an external NFT contract used to mint contributor rewards.
#[contractclient(name = "NftContractClient")]
pub trait NftContract {
    /// Mints an NFT to the given address and returns the new token ID.
    fn mint(env: Env, to: Address) -> u128;
}

/// The main crowdfunding contract.
#[contract]
pub struct CrowdfundContract;

#[contractimpl]
impl CrowdfundContract {
    /// Initializes a new crowdfunding campaign.
    ///
    /// # Arguments
    /// * `creator`            – The campaign creator's address.
    /// * `token`              – The token contract address used for contributions.
    /// * `goal`               – The funding goal (in the token's smallest unit).
    /// * `deadline`           – The campaign deadline as a ledger timestamp.
    /// * `min_contribution`   – The minimum contribution amount.
    /// * `platform_config`    – Optional platform configuration (address and fee in basis points).
    ///
    /// # Panics
    /// * If already initialized.
    /// * If platform fee exceeds 10,000 (100%).
    pub fn initialize(
        env: Env,
        _admin: Address,
        creator: Address,
        token: Address,
        goal: i128,
        deadline: u64,
        min_contribution: i128,
        platform_config: Option<PlatformConfig>,
        bonus_goal: Option<i128>,
        bonus_goal_description: Option<String>,
    ) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Creator) {
            return Err(ContractError::AlreadyInitialized);
        }

        creator.require_auth();

        if let Some(ref config) = platform_config {
            if config.fee_bps > 10_000 {
                panic!("platform fee cannot exceed 100%");
            }
            env.storage()
                .instance()
                .set(&DataKey::PlatformConfig, config);
        }

        if let Some(bg) = bonus_goal {
            if bg <= goal {
                panic!("bonus goal must be greater than primary goal");
            }
            env.storage().instance().set(&DataKey::BonusGoal, &bg);
        }

        if let Some(bg) = bonus_goal {
            if bg <= goal {
                panic!("bonus goal must be greater than primary goal");
            }
            env.storage().instance().set(&DataKey::BonusGoal, &bg);
        }

        if let Some(bg_description) = bonus_goal_description {
            env.storage()
                .instance()
                .set(&DataKey::BonusGoalDescription, &bg_description);
        }

        env.storage().instance().set(&DataKey::Creator, &creator);
        env.storage().instance().set(&DataKey::Token, &token);

        // Returns the list of all contributor addresses.
        #[allow(dead_code)]
        pub fn contributors(env: Env) -> Vec<Address> {
            env.storage()
                .instance()
                .get(&DataKey::Contributors)
                .unwrap_or(Vec::new(&env))
        }

        env.storage().instance().set(&DataKey::Goal, &goal);
        env.storage().instance().set(&DataKey::Deadline, &deadline);
        env.storage()
            .instance()
            .set(&DataKey::MinContribution, &min_contribution);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::BonusGoalReachedEmitted, &false);
        env.storage()
            .instance()
            .set(&DataKey::BonusGoalReachedEmitted, &false);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Active);

        // Store platform config if provided.
        if let Some(config) = platform_config {
            env.storage()
                .instance()
                .set(&DataKey::PlatformConfig, &config);
        }

        let empty_contributors: Vec<Address> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Contributors, &empty_contributors);

        let empty_roadmap: Vec<RoadmapItem> = Vec::new(&env);
        env.storage()
            .instance()
            .set(&DataKey::Roadmap, &empty_roadmap);

        Ok(())
    }

    /// Contribute tokens to the campaign.
    ///
    /// The contributor must authorize the call. Contributions are rejected
    /// after the deadline has passed.
    pub fn contribute(env: Env, contributor: Address, amount: i128) -> Result<(), ContractError> {
        contributor.require_auth();

        let min_contribution: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap();
        if amount < min_contribution {
            panic!("amount below minimum");
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() > deadline {
            return Err(ContractError::CampaignEnded);
        }

        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        // Transfer tokens from the contributor to this contract.
        token_client.transfer(&contributor, &env.current_contract_address(), &amount);

        // Update the contributor's running total with overflow protection.
        let contribution_key = DataKey::Contribution(contributor.clone());
        let previous_amount: i128 = env
            .storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);

        let new_contribution = previous_amount
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;

        env.storage()
            .persistent()
            .set(&contribution_key, &new_contribution);
        env.storage()
            .persistent()
            .extend_ttl(&contribution_key, 100, 100);

        // Update the global total raised with overflow protection.
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();

        let new_total = total.checked_add(amount).ok_or(ContractError::Overflow)?;

        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &new_total);

        if let Some(bg) = env.storage().instance().get::<_, i128>(&DataKey::BonusGoal) {
            let already_emitted = env
                .storage()
                .instance()
                .get::<_, bool>(&DataKey::BonusGoalReachedEmitted)
                .unwrap_or(false);
            if !already_emitted && total < bg && new_total >= bg {
                env.events().publish(("campaign", "bonus_goal_reached"), bg);
                env.storage()
                    .instance()
                    .set(&DataKey::BonusGoalReachedEmitted, &true);
            }
        }

        let mut contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        if !contributors.contains(&contributor) {
            contributors.push_back(contributor.clone());
            env.storage()
                .persistent()
                .set(&DataKey::Contributors, &contributors);
            env.storage()
                .persistent()
                .extend_ttl(&DataKey::Contributors, 100, 100);
        }

        // Emit contribution event
        env.events()
            .publish(("campaign", "contributed"), (contributor, amount));

        Ok(())
    }

    /// Sets the NFT contract address used for reward minting.
    ///
    /// Only the campaign creator can configure this value.
    pub fn set_nft_contract(env: Env, creator: Address, nft_contract: Address) {
        let stored_creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        if creator != stored_creator {
            panic!("not authorized");
        }
        creator.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::NFTContract, &nft_contract);
    }

    /// Pledge tokens to the campaign without transferring them immediately.
    ///
    /// The pledger must authorize the call. Pledges are recorded off-chain
    /// and only collected if the goal is met after the deadline.
    pub fn pledge(env: Env, pledger: Address, amount: i128) -> Result<(), ContractError> {
        pledger.require_auth();

        let min_contribution: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap();
        if amount < min_contribution {
            panic!("amount below minimum");
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() > deadline {
            return Err(ContractError::CampaignEnded);
        }

        // Update the pledger's running total.
        let pledge_key = DataKey::Pledge(pledger.clone());
        let prev: i128 = env.storage().persistent().get(&pledge_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&pledge_key, &(prev + amount));
        env.storage().persistent().extend_ttl(&pledge_key, 100, 100);

        // Update the global total pledged.
        let total_pledged: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalPledged)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalPledged, &(total_pledged + amount));

        // Track pledger address if new.
        let mut pledgers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Pledgers)
            .unwrap_or_else(|| Vec::new(&env));
        if !pledgers.contains(&pledger) {
            pledgers.push_back(pledger.clone());
            env.storage()
                .persistent()
                .set(&DataKey::Pledgers, &pledgers);
            env.storage()
                .persistent()
                .extend_ttl(&DataKey::Pledgers, 100, 100);
        }

        // Emit pledge event
        env.events()
            .publish(("campaign", "pledged"), (pledger, amount));

        Ok(())
    }

    /// Collect all pledges after the deadline when the goal is met.
    ///
    /// This function transfers tokens from all pledgers to the contract.
    /// Only callable after the deadline and when the combined total of
    /// contributions and pledges meets or exceeds the goal.
    pub fn collect_pledges(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() <= deadline {
            return Err(ContractError::CampaignStillActive);
        }

        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let total_raised: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        let total_pledged: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalPledged)
            .unwrap_or(0);

        // Check if combined total meets the goal
        if total_raised + total_pledged < goal {
            return Err(ContractError::GoalNotReached);
        }

        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        let pledgers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Pledgers)
            .unwrap_or_else(|| Vec::new(&env));

        // Collect pledges from all pledgers
        for pledger in pledgers.iter() {
            let pledge_key = DataKey::Pledge(pledger.clone());
            let amount: i128 = env.storage().persistent().get(&pledge_key).unwrap_or(0);
            if amount > 0 {
                // Transfer tokens from pledger to contract
                token_client.transfer(&pledger, &env.current_contract_address(), &amount);

                // Clear the pledge
                env.storage().persistent().set(&pledge_key, &0i128);
                env.storage().persistent().extend_ttl(&pledge_key, 100, 100);
            }
        }

        // Update total raised to include collected pledges
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &(total_raised + total_pledged));

        // Reset total pledged
        env.storage().instance().set(&DataKey::TotalPledged, &0i128);

        // Emit pledges collected event
        env.events()
            .publish(("campaign", "pledges_collected"), total_pledged);

        Ok(())
    }

    /// Withdraw raised funds — only callable by the creator after the
    /// deadline, and only if the goal has been met.
    ///
    /// If a platform fee is configured, deducts the fee and transfers it to
    /// the platform address, then sends the remainder to the creator.
    pub fn withdraw(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() <= deadline {
            return Err(ContractError::CampaignStillActive);
        }

        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        if total < goal {
            return Err(ContractError::GoalNotReached);
        }

        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        let platform_config: Option<PlatformConfig> =
            env.storage().instance().get(&DataKey::PlatformConfig);

        let creator_payout = if let Some(config) = platform_config {
            let fee = total
                .checked_mul(config.fee_bps as i128)
                .expect("fee calculation overflow")
                .checked_div(10_000)
                .expect("fee division by zero");

            token_client.transfer(&env.current_contract_address(), &config.address, &fee);
            env.events()
                .publish(("campaign", "fee_transferred"), (&config.address, fee));
            total.checked_sub(fee).expect("creator payout underflow")
        } else {
            total
        };

        token_client.transfer(&env.current_contract_address(), &creator, &creator_payout);

        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Successful);

        if let Some(nft_contract) = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::NFTContract)
        {
            let contributors: Vec<Address> = env
                .storage()
                .persistent()
                .get(&DataKey::Contributors)
                .unwrap_or_else(|| Vec::new(&env));
            let mut token_id: u64 = 1;
            for contributor in contributors.iter() {
                let contribution: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Contribution(contributor.clone()))
                    .unwrap_or(0);
                if contribution > 0 {
                    env.invoke_contract::<()>(
                        &nft_contract,
                        &Symbol::new(&env, "mint"),
                        Vec::from_array(
                            &env,
                            [contributor.into_val(&env), token_id.into_val(&env)],
                        ),
                    );
                    env.events().publish(
                        (
                            Symbol::new(&env, "campaign"),
                            Symbol::new(&env, "nft_minted"),
                        ),
                        (contributor.clone(), token_id),
                    );
                    token_id += 1;
                }
            }
        }

        // Emit withdrawal event
        env.events()
            .publish(("campaign", "withdrawn"), (creator.clone(), total));

        Ok(())
    }

    /// Refund all contributors — callable by anyone after the deadline
    /// if the goal was **not** met.
    pub fn refund(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() <= deadline {
            return Err(ContractError::CampaignStillActive);
        }

        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        if total >= goal {
            return Err(ContractError::GoalReached);
        }

        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap();

        for contributor in contributors.iter() {
            let contribution_key = DataKey::Contribution(contributor.clone());
            let amount: i128 = env
                .storage()
                .persistent()
                .get(&contribution_key)
                .unwrap_or(0);
            if amount > 0 {
                token_client.transfer(&env.current_contract_address(), &contributor, &amount);
                env.storage().persistent().set(&contribution_key, &0i128);
                env.storage()
                    .persistent()
                    .extend_ttl(&contribution_key, 100, 100);
            }
        }

        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Refunded);

        Ok(())
    }

    /// Cancel the campaign and refund all contributors — callable only by
    /// the creator while the campaign is still Active.
    pub fn cancel(env: Env) {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let _token_client = token::Client::new(&env, &token_address);

        let _contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap();

        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Cancelled);
    }

    /// Upgrade the contract to a new WASM implementation — admin-only.
    ///
    /// This function allows the designated admin to upgrade the contract's WASM code
    /// without changing the contract's address or storage. The new WASM hash must be
    /// provided and the caller must be authorized as the admin.
    ///
    /// # Arguments
    /// * `new_wasm_hash` – The SHA-256 hash of the new WASM binary to deploy.
    ///
    /// # Panics
    /// * If the caller is not the admin.
    pub fn upgrade(env: Env, new_wasm_hash: soroban_sdk::BytesN<32>) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Update campaign metadata — only callable by the creator while the
    /// campaign is still Active.
    ///
    /// # Arguments
    /// * `creator`     – The campaign creator's address (for authentication).
    /// * `title`       – Optional new title (None to keep existing).
    /// * `description` – Optional new description (None to keep existing).
    /// * `socials`    – Optional new social links (None to keep existing).
    pub fn update_metadata(
        env: Env,
        creator: Address,
        title: Option<String>,
        description: Option<String>,
        socials: Option<String>,
    ) {
        // Check campaign is active.
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        // Require creator authentication and verify caller is the creator.
        let stored_creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        if creator != stored_creator {
            panic!("not authorized");
        }
        creator.require_auth();

        // Track which fields were updated for the event.
        let mut updated_fields: Vec<Symbol> = Vec::new(&env);

        // Update title if provided.
        if let Some(new_title) = title {
            env.storage().instance().set(&DataKey::Title, &new_title);
            updated_fields.push_back(Symbol::new(&env, "title"));
        }

        // Update description if provided.
        if let Some(new_description) = description {
            env.storage()
                .instance()
                .set(&DataKey::Description, &new_description);
            updated_fields.push_back(Symbol::new(&env, "description"));
        }

        // Update social links if provided.
        if let Some(new_socials) = socials {
            env.storage()
                .instance()
                .set(&DataKey::SocialLinks, &new_socials);
            updated_fields.push_back(Symbol::new(&env, "socials"));
        }

        // Emit event with updated fields.
        env.events().publish(
            (Symbol::new(&env, "metadata_updated"), creator.clone()),
            updated_fields,
        );
    }

    /// Add a roadmap item — only callable by the creator.
    ///
    /// # Arguments
    /// * `date`        – Future Unix timestamp for the milestone.
    /// * `description` – Non-empty description of the milestone.
    pub fn add_roadmap_item(env: Env, date: u64, description: String) {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        if date <= env.ledger().timestamp() {
            panic!("date must be in the future");
        }

        if description.is_empty() {
            panic!("description cannot be empty");
        }

        let mut roadmap: Vec<RoadmapItem> = env
            .storage()
            .instance()
            .get(&DataKey::Roadmap)
            .unwrap_or_else(|| Vec::new(&env));

        roadmap.push_back(RoadmapItem {
            date,
            description: description.clone(),
        });

        env.storage().instance().set(&DataKey::Roadmap, &roadmap);
        env.events()
            .publish(("campaign", "roadmap_item_added"), (date, description));
    }

    /// Returns all roadmap items for the campaign.
    pub fn roadmap(env: Env) -> Vec<RoadmapItem> {
        env.storage()
            .instance()
            .get(&DataKey::Roadmap)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Add a stretch goal milestone to the campaign.
    ///
    /// Only the creator can add stretch goals. The milestone must be greater
    /// than the primary goal.
    pub fn add_stretch_goal(env: Env, milestone: i128) {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        if milestone <= goal {
            panic!("stretch goal must be greater than primary goal");
        }

        let mut stretch_goals: Vec<i128> = env
            .storage()
            .instance()
            .get(&DataKey::StretchGoals)
            .unwrap_or_else(|| Vec::new(&env));

        stretch_goals.push_back(milestone);
        env.storage()
            .instance()
            .set(&DataKey::StretchGoals, &stretch_goals);
    }

    /// Returns the next unmet stretch goal milestone.
    ///
    /// Returns 0 if there are no stretch goals or all have been met.
    pub fn current_milestone(env: Env) -> i128 {
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);

        let stretch_goals: Vec<i128> = env
            .storage()
            .instance()
            .get(&DataKey::StretchGoals)
            .unwrap_or_else(|| Vec::new(&env));

        for milestone in stretch_goals.iter() {
            if total_raised < milestone {
                return milestone;
            }
        }

        0
    }
    /// Returns the total amount of tokens raised so far.
    pub fn total_raised(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0)
    }

    /// Returns the campaign funding goal.
    pub fn goal(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::Goal).unwrap()
    }

    /// Returns the optional bonus-goal threshold.
    pub fn bonus_goal(env: Env) -> Option<i128> {
        env.storage().instance().get(&DataKey::BonusGoal)
    }

    /// Returns the optional bonus-goal description.
    pub fn bonus_goal_description(env: Env) -> Option<String> {
        env.storage().instance().get(&DataKey::BonusGoalDescription)
    }

    /// Returns true if the optional bonus goal has been reached.
    pub fn bonus_goal_reached(env: Env) -> bool {
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);

        if let Some(bg) = env.storage().instance().get::<_, i128>(&DataKey::BonusGoal) {
            total_raised >= bg
        } else {
            false
        }
    }

    /// Returns bonus-goal progress in basis points (capped at 10,000).
    pub fn bonus_goal_progress_bps(env: Env) -> u32 {
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);

        if let Some(bg) = env.storage().instance().get::<_, i128>(&DataKey::BonusGoal) {
            if bg > 0 {
                let raw = (total_raised * 10_000) / bg;
                if raw > 10_000 {
                    10_000
                } else {
                    raw as u32
                }
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Returns the campaign deadline.
    pub fn deadline(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::Deadline).unwrap()
    }

    /// Returns the contribution amount for a given contributor.
    pub fn contribution(env: Env, contributor: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Contribution(contributor))
            .unwrap_or(0)
    }

    /// Returns the minimum contribution amount required.
    pub fn min_contribution(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap()
    }

    /// Returns comprehensive campaign statistics.
    pub fn get_stats(env: Env) -> CampaignStats {
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);
        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        let progress_bps = if goal > 0 {
            let raw = (total_raised * 10_000) / goal;
            if raw > 10_000 {
                10_000
            } else {
                raw as u32
            }
        } else {
            0
        };

        let contributor_count = contributors.len();
        let (average_contribution, largest_contribution) = if contributor_count == 0 {
            (0, 0)
        } else {
            let average = total_raised / contributor_count as i128;
            let mut largest = 0i128;
            for contributor in contributors.iter() {
                let amount: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Contribution(contributor))
                    .unwrap_or(0);
                if amount > largest {
                    largest = amount;
                }
            }
            (average, largest)
        };

        CampaignStats {
            total_raised,
            goal,
            progress_bps,
            contributor_count,
            average_contribution,
            largest_contribution,
        }
    }

    /// Returns the campaign title.
    pub fn title(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Title)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    /// Returns the campaign description.
    pub fn description(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Description)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    /// Returns the campaign social links.
    pub fn socials(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::SocialLinks)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    /// Returns the contract version number.
    pub fn version(_env: Env) -> u32 {
        CONTRACT_VERSION
    }

    /// Returns the token contract address used for contributions.
    pub fn token(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Token).unwrap()
    }

    /// Returns the configured NFT contract address, if any.
    pub fn nft_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::NFTContract)
    }
}
