#![no_std]
#![allow(clippy::too_many_arguments)]
use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, token, Address, Env, String, Symbol, Vec,
};

// ── Modules ──────────────────────────────────────────────────────────────────

pub mod access_control;
pub mod admin_upgrade_mechanism;
pub mod dependency_vulnerability_scanning;
pub mod emergency_stop;
pub mod pause_mechanism;
pub mod sharding_mechanism;
pub mod algorithm_optimization;
pub mod session_management;
pub mod campaign_goal_minimum;
pub mod cargo_toml_rust;
pub mod contract_state_size;
pub mod contribute_error_handling;
pub mod crowdfund_initialize_function;
pub mod role_based_access;
#[cfg(test)]
pub mod npm_package_lock;
pub mod proptest_generator_boundary;
pub mod refund_single_token;
pub mod soroban_sdk_minor;
pub mod stellar_token_minter;
pub mod stream_processing_optimization;
pub mod withdraw_event_emission;
pub mod loop_optimization;
pub mod security_compliance_automation;
pub mod security_compliance_validation;
pub mod security_analytics;
pub mod conditional_optimization;
pub mod batch_processing_optimization;
pub mod state_compression;

use crate::reentrancy_guard::{enter_transfer, exit_transfer, protected_transfer};

use crowdfund_initialize_function::{execute_initialize, InitParams};
use refund_single_token::{
    execute_refund_single, refund_single_transfer, validate_refund_preconditions,
};
use stream_processing_optimization::{
    bonus_goal_progress_bps as compute_bonus_goal_progress_bps, build_campaign_stats,
    load_address_stream_state, next_unmet_milestone, persist_address_stream_if_missing,
};
use withdraw_event_emission::{emit_fee_transferred, emit_withdrawn, mint_nfts_in_batch};

// ── Test Modules ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod access_control_tests;
#[cfg(test)]
#[path = "admin_upgrade_mechanism.test.rs"]
mod admin_upgrade_mechanism_test;
#[cfg(test)]
#[path = "dependency_vulnerability_scanning.test.rs"]
mod dependency_vulnerability_scanning_test;
#[cfg(test)]
#[path = "emergency_stop.test.rs"]
mod emergency_stop_test;
#[cfg(test)]
#[path = "pause_mechanism.test.rs"]
mod pause_mechanism_test;
#[cfg(test)]
#[path = "sharding_mechanism.test.rs"]
mod sharding_mechanism_test;
#[cfg(test)]
mod auth_tests;
#[cfg(test)]
#[path = "campaign_goal_minimum.test.rs"]
mod campaign_goal_minimum_test;
#[cfg(test)]
#[path = "cargo_toml_rust.test.rs"]
mod cargo_toml_rust_test;
#[cfg(test)]
#[path = "contract_state_size.test.rs"]
mod contract_state_size_test;
#[cfg(test)]
mod contribute_error_handling_tests;
#[cfg(all(test, feature = "legacy_crowdfund_tests"))]
#[path = "npm_package_lock_test.rs"]
mod npm_package_lock_test;

#[cfg(test)]
#[path = "proptest_generator_boundary.test.rs"]
mod proptest_generator_boundary_test;
#[cfg(test)]
#[path = "proptest_generator_boundary_tests.rs"]
mod proptest_generator_boundary_tests;
#[cfg(test)]
#[path = "soroban_sdk_minor_test.rs"]
mod soroban_sdk_minor_test;
#[cfg(all(test, feature = "legacy_crowdfund_tests"))]
#[path = "stellar_token_minter_test.rs"]
mod stellar_token_minter_test_original;
#[cfg(all(test, feature = "legacy_crowdfund_tests"))]
#[path = "stellar_token_minter.test.rs"]
mod stellar_token_minter_test_comprehensive;
#[cfg(test)]
#[path = "stream_processing_optimization.test.rs"]
mod stream_processing_optimization_test;
#[cfg(test)]
#[path = "security_compliance_automation.test.rs"]
mod security_compliance_automation_test;
#[cfg(test)]
#[path = "security_compliance_validation.test.rs"]
mod security_compliance_validation_test;
#[cfg(test)]
#[path = "role_based_access.test.rs"]
mod role_based_access_test;
#[cfg(test)]
#[path = "security_analytics.test.rs"]
mod security_analytics_test;
#[cfg(test)]
#[path = "conditional_optimization.test.rs"]
mod conditional_optimization_test;
#[path = "batch_processing_optimization.test.rs"]
mod batch_processing_optimization_test;
#[cfg(test)]
#[path = "state_compression.test.rs"]
mod state_compression_test;

// --- Constants ---
const CONTRACT_VERSION: u32 = 3;
#[allow(dead_code)]
const CONTRIBUTION_COOLDOWN: u64 = 60;

pub const MAX_NFT_MINT_BATCH: u32 = 50;

// ── Data Types ──────────────────────────────────────────────────────────────

/// Represents the campaign status.
///
/// Transitions:
///   `Active` → `Succeeded`  (via `finalize` when deadline passed and goal met)
///   `Active` → `Expired`    (via `finalize` when deadline passed and goal not met)
///   `Active` → `Cancelled`  (via `cancel`)
#[derive(Clone, PartialEq, Debug)]
#[contracttype]
pub enum Status {
    Active,
    Succeeded,
    Expired,
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

/// Security metric types tracked by the analytics system.
/// Used for threat detection and security monitoring.
#[derive(Clone, Copy, PartialEq, Debug)]
#[contracttype]
pub enum MetricType {
    /// Total contribution transactions.
    TotalContributions,
    /// Total withdrawal transactions.
    TotalWithdrawals,
    /// Total refund transactions.
    TotalRefunds,
    /// Failed transaction attempts.
    FailedTransactions,
    /// Unique active contributors.
    UniqueContributors,
    /// Average contribution size.
    AverageContributionSize,
    /// Large transaction count (above threshold).
    LargeTransactions,
    /// Reverted transactions.
    RevertedTransactions,
    /// Auth failures.
    AuthFailures,
    /// Rate limit triggers.
    RateLimitTriggers,
}

/// Represents all storage keys used by the crowdfund contract.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
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
    Pledge(Address),
    /// Total amount pledged but not yet collected.
    TotalPledged,
    StretchGoals,
    BonusGoal,
    BonusGoalDescription,
    BonusGoalReachedEmitted,
    Pledgers,
    Roadmap,
    /// The designated admin address (set to creator at initialization).
    Admin,
    /// Campaign title.
    Title,
    Description,
    /// Campaign social links.
    SocialLinks,
    /// Platform fee configuration.
    PlatformConfig,
    NFTContract,
    /// Decimal precision of the campaign token (e.g. 7 for XLM, 6 for USDC).
    TokenDecimals,
    /// Optional cap on the amount a single contributor may contribute.
    MaxIndividualContribution,

    // ── Role-separation keys (access_control module) ──────────────────────
    /// Address with DEFAULT_ADMIN_ROLE — can upgrade, unpause, and transfer roles.
    DefaultAdmin,
    /// Address with PAUSER_ROLE — can pause in an emergency but cannot unpause.
    Pauser,
    /// Governance address (multisig / DAO) — the only address that may set platform fees.
    GovernanceAddress,
    /// Boolean flag — when true, contribute() and withdraw() are blocked.
    Paused,

    // ── Security Analytics keys ─────────────────────────────────────────────
    /// Threat log storage key.
    ThreatLog,
    /// Access pattern storage keyed by address.
    AccessPattern(Address),
    /// Security metric storage keyed by address and metric type.
    SecurityMetric(Address, MetricType),
    /// Global security metric storage keyed by metric type.
    GlobalSecurityMetric(MetricType),

    // ── Sharding keys ───────────────────────────────────────────────────────
    /// Number of contributor shards currently allocated.
    ShardCount,
    /// Contributor address list for shard index `n`.
    ContributorShard(u32),

    // ── Emergency stop key ──────────────────────────────────────────────────
    /// Boolean flag — when true, all state-mutating entry points are permanently blocked.
    EmergencyStopped,
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
    NothingToRefund = 7,

    /// Returned by `initialize` when `goal < MIN_GOAL_AMOUNT`.
    InvalidGoal = 8,
    /// Returned by `initialize` when `min_contribution < MIN_CONTRIBUTION_AMOUNT`.
    InvalidMinContribution = 9,
    /// Returned by `initialize` when `deadline` is too soon.
    DeadlineTooSoon = 10,
    /// Returned by `initialize` when `platform_config.fee_bps > MAX_PLATFORM_FEE_BPS`.
    InvalidPlatformFee = 11,
    /// Returned by `initialize` when `bonus_goal <= goal`.
    InvalidBonusGoal = 12,
    /// Returned by `initialize` when `goal < MIN_GOAL_AMOUNT`.
    GoalTooLow = 13,
    /// Returned by `contribute` when `amount` is zero.
    ZeroAmount = 14,
    BelowMinimum = 15,
    CampaignNotActive = 16,
    /// Returned by `contribute` when `amount` is negative.
    NegativeAmount = 17,
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
    /// Delegates all validation and storage logic to
    /// [`crowdfund_initialize_function::execute_initialize`].
    ///
    /// # Arguments
    /// * `admin`                  – Address authorized to upgrade the contract.
    /// * `creator`                – The campaign creator's address (must authorize).
    /// * `token`                  – The SEP-41 token contract address.
    /// * `goal`                   – Funding goal in the token's smallest unit (>= 1).
    /// * `deadline`               – Campaign deadline as a Unix timestamp (>= now + 60s).
    /// * `min_contribution`       – Minimum contribution amount (>= 1).
    /// * `platform_config`        – Optional platform fee configuration (fee_bps <= 10_000).
    /// * `bonus_goal`             – Optional bonus goal threshold (must be > `goal`).
    /// * `bonus_goal_description` – Optional description for the bonus goal.
    ///
    /// # Errors
    /// * [`ContractError::AlreadyInitialized`]    – Contract was already initialized.
    /// * [`ContractError::InvalidGoal`]           – `goal < 1`.
    /// * [`ContractError::InvalidMinContribution`]– `min_contribution < 1`.
    /// * [`ContractError::DeadlineTooSoon`]       – `deadline < now + 60`.
    /// * [`ContractError::InvalidPlatformFee`]    – `fee_bps > 10_000`.
    /// * [`ContractError::InvalidBonusGoal`]      – `bonus_goal <= goal`.
    pub fn initialize(
        env: Env,
        admin: Address,
        creator: Address,
        token: Address,
        goal: i128,
        deadline: u64,
        min_contribution: i128,
        max_individual_contribution: Option<i128>,
        platform_config: Option<PlatformConfig>,
        bonus_goal: Option<i128>,
        bonus_goal_description: Option<String>,
    ) -> Result<(), ContractError> {
        execute_initialize(
            &env,
            InitParams {
                admin,
                creator,
                token,
                goal,
                deadline,
                min_contribution,
                platform_config,
                bonus_goal,
                bonus_goal_description,
            },
        )?;

        // Store optional max individual contribution cap.
        if let Some(max_contrib) = max_individual_contribution {
            env.storage()
                .instance()
                .set(&DataKey::MaxIndividualContribution, &max_contrib);
        }

        Ok(())
    }

    /// Returns the list of all contributor addresses.
    pub fn contributors(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or(Vec::new(&env))
    }

    /// Contribute tokens to the campaign.
    ///
    /// The contributor must authorize the call. Contributions are rejected
    /// after the deadline has passed or if the campaign is not active.
    pub fn contribute(env: Env, contributor: Address, amount: i128) -> Result<(), ContractError> {
        contributor.require_auth();

        // Guard: reject if emergency stop is active.
        emergency_stop::assert_not_stopped(&env);
        // Guard: reject if contract is paused.
        pause_mechanism::assert_not_paused(&env);

        // Guard: campaign must be active.
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            contribute_error_handling::log_contribute_error(&env, ContractError::CampaignNotActive);
            return Err(ContractError::CampaignNotActive);
        }

        if amount < 0 {
            return Err(ContractError::NegativeAmount);
        }

        if amount == 0 {
            contribute_error_handling::log_contribute_error(&env, ContractError::ZeroAmount);
            return Err(ContractError::ZeroAmount);
        }

        let min_contribution: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap();
        if amount < min_contribution {
            contribute_error_handling::log_contribute_error(&env, ContractError::BelowMinimum);
            return Err(ContractError::BelowMinimum);
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() > deadline {
            contribute_error_handling::log_contribute_error(&env, ContractError::CampaignEnded);
            return Err(ContractError::CampaignEnded);
        }

        let mut contributor_stream =
            load_address_stream_state(&env, &DataKey::Contributors, &contributor);
        let is_new_contributor = !contributor_stream.contains_target;
        if is_new_contributor {
            if let Err(_) =
                contract_state_size::validate_contributor_capacity(contributor_stream.entries.len())
            {
                panic!("state size limit exceeded");
            }
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

        let new_contribution = previous_amount.checked_add(amount).ok_or_else(|| {
            contribute_error_handling::log_contribute_error(&env, ContractError::Overflow);
            ContractError::Overflow
        })?;

        env.storage()
            .persistent()
            .set(&contribution_key, &new_contribution);
        env.storage()
            .persistent()
            .extend_ttl(&contribution_key, 100, 100);

        // Update the global total raised with overflow protection.
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();

        let new_total = total.checked_add(amount).ok_or_else(|| {
            contribute_error_handling::log_contribute_error(&env, ContractError::Overflow);
            ContractError::Overflow
        })?;

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

        if is_new_contributor {
            // Enforce contributor list size limit before appending.
            contract_state_size::check_contributor_limit(&env).expect("contributor limit exceeded");
            persist_address_stream_if_missing(
                &env,
                &DataKey::Contributors,
                &mut contributor_stream,
                &contributor,
            );
        }

        // Emit PledgeReceived event — includes total_raised for real-time progress bars.
        env.events().publish(
            ("campaign", "pledge_received"),
            (contributor, amount, new_total),
        );

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

        let mut pledger_stream = load_address_stream_state(&env, &DataKey::Pledgers, &pledger);
        let is_new_pledger = !pledger_stream.contains_target;
        if is_new_pledger {
            if let Err(_) =
                contract_state_size::validate_pledger_capacity(pledger_stream.entries.len())
            {
                panic!("state size limit exceeded");
            }
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
        if is_new_pledger {
            // Enforce pledger list size limit before appending.
            contract_state_size::check_pledger_limit(&env).expect("pledger limit exceeded");
            persist_address_stream_if_missing(
                &env,
                &DataKey::Pledgers,
                &mut pledger_stream,
                &pledger,
            );
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

    /// Finalize the campaign by transitioning it from `Active` to either
    /// `Succeeded` or `Expired` based on the deadline and total raised.
    ///
    /// - `Active → Succeeded`: deadline has passed **and** goal was met.
    /// - `Active → Expired`:   deadline has passed **and** goal was not met.
    ///
    /// Anyone may call this function — it is permissionless and idempotent
    /// in the sense that it will panic if the campaign is not `Active`.
    ///
    /// # Errors
    /// * Panics if the campaign is not `Active`.
    /// * Returns `ContractError::CampaignStillActive` if the deadline has not passed.
    pub fn finalize(env: Env) -> Result<Status, ContractError> {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() <= deadline {
            return Err(ContractError::CampaignStillActive);
        }

        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let total: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);

        let new_status = if total >= goal {
            Status::Succeeded
        } else {
            Status::Expired
        };

        env.storage().instance().set(&DataKey::Status, &new_status);
        env.events()
            .publish(("campaign", "finalized"), new_status.clone());

        Ok(new_status)
    }

    /// Returns the current stored campaign status.
    pub fn status(env: Env) -> Status {
        env.storage().instance().get(&DataKey::Status).unwrap()
    }

    /// Withdraw raised funds — only callable by the creator after the campaign
    /// has been finalized as `Succeeded`.
    ///
    /// Call `finalize()` first to transition the campaign from `Active` to
    /// `Succeeded` (deadline passed + goal met). This explicit two-step design
    /// prevents "state bleeding" where a creator could withdraw while the
    /// campaign is still technically active.
    ///
    /// If a platform fee is configured, deducts the fee and transfers it to
    /// the platform address, then sends the remainder to the creator.
    pub fn withdraw(env: Env) -> Result<(), ContractError> {
        // Guard: reject if emergency stop is active.
        emergency_stop::assert_not_stopped(&env);
        // Guard: reject if contract is paused.
        pause_mechanism::assert_not_paused(&env);

        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Succeeded {
            panic!("campaign must be in Succeeded state to withdraw");
        }

        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
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
            withdraw_event_emission::emit_fee_transferred(
                &env,
                &config.address,
                fee,
            );
            total.checked_sub(fee).expect("creator payout underflow")
        } else {
            0
        };

        protected_transfer(&env, || {
            // Platform fee transfer + emit (if applicable)
            if let Some(config) = platform_config {
                token_client.transfer(&env.current_contract_address(), &config.address, &creator_payout);
                withdraw_event_emission::emit_fee_transferred(&env, &config.address, creator_payout, config.fee_bps);
            }

            // Creator payout transfer
            let final_creator_payout = total.checked_sub(creator_payout).expect("creator payout underflow");
            token_client.transfer(&env.current_contract_address(), &creator, &final_creator_payout);

            // Bounded NFT minting
            let nft_contract: Option<Address> = env.storage().instance().get(&DataKey::NFTContract);
            let nft_minted_count = mint_nfts_in_batch(&env, &nft_contract);

            // Emit withdrawal event
            emit_withdrawn(&env, &creator, final_creator_payout, nft_minted_count);
        });

        // Clear TotalRaised AFTER transfers (CEI compliance)
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);

        Ok(())
    }

    /// Claim a refund for a single contributor (pull-based).
    ///
    /// Each contributor independently claims their own refund after the campaign
    /// deadline has passed and the goal was not met.
    ///
    /// # Errors
    /// * [`ContractError::CampaignStillActive`] – Deadline has not yet passed.
    /// * [`ContractError::GoalReached`]         – Goal was met; no refunds available.
    /// * [`ContractError::NothingToRefund`]     – Caller has no contribution on record.
    pub fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError> {
        contributor.require_auth();
        let amount = validate_refund_preconditions(&env, &contributor)?;
        protected_transfer(&env, || {
            execute_refund_single(&env, &contributor, amount)
        });
        Ok(())
    }

    /// Check if a refund is available for the given contributor.
    ///
    /// This is a view function that can be called to determine if `refund_single`
    /// would succeed for the given contributor. Useful for frontend UI to show
    /// refund buttons or status.
    ///
    /// Returns the amount that would be refunded if `refund_single` is called,
    /// or an error if no refund is available.
    ///
    /// @param contributor The address to check for refund availability.
    /// @return `Ok(amount)` if refund is available, `Err(ContractError)` otherwise.
    pub fn refund_available(env: Env, contributor: Address) -> Result<i128, ContractError> {
        validate_refund_preconditions(&env, &contributor)
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
        let token_client = token::Client::new(&env, &token_address);

        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        for contributor in contributors.iter() {
            let contribution_key = DataKey::Contribution(contributor.clone());
            let amount: i128 = env
                .storage()
                .persistent()
                .get(&contribution_key)
                .unwrap_or(0);
            if amount > 0 {
                env.storage().persistent().set(&contribution_key, &0i128);
                refund_single_transfer(
                    &token_client,
                    &env.current_contract_address(),
                    &contributor,
                    amount,
                );
            }
        }

        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Cancelled);
    }

    /// Upgrade the contract to a new WASM implementation — admin-only.
    ///
    /// Validation order (cheapest checks first for gas efficiency):
    /// 1. Reject zero hash — pure, no storage reads.
    /// 2. Load admin + enforce `require_auth()`.
    /// 3. Execute WASM swap.
    /// 4. Emit audit event.
    ///
    /// # Panics
    /// * `"zero wasm hash"` — if `new_wasm_hash` is all-zero bytes.
    /// * `"Admin not initialized"` — if `initialize()` was never called.
    /// * Auth error — if the caller is not the stored admin.
    pub fn upgrade(env: Env, new_wasm_hash: soroban_sdk::BytesN<32>) {
        // Gas-efficiency edge case: reject zero hash before any storage read.
        if !admin_upgrade_mechanism::validate_wasm_hash(&new_wasm_hash) {
            panic!("zero wasm hash");
        }
        let admin = admin_upgrade_mechanism::validate_admin_upgrade(&env);
        admin_upgrade_mechanism::validate_wasm_hash(&new_wasm_hash);
        admin_upgrade_mechanism::perform_upgrade(&env, new_wasm_hash.clone());

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "upgrade"), admin),
            new_wasm_hash,
        );
    }

    /// @notice Pause the contract — blocks `contribute()` and `withdraw()`.
    /// @dev    Callable by `PAUSER_ROLE` or `DEFAULT_ADMIN_ROLE`.
    ///         Emits `(access, paused)` event.
    ///
    /// # Arguments
    /// * `caller` — Must hold `PAUSER_ROLE` or `DEFAULT_ADMIN_ROLE`.
    ///
    /// # Panics
    /// * `"not authorized to pause"` if `caller` holds neither role.
    pub fn pause(env: Env, caller: Address) {
        pause_mechanism::pause(&env, &caller);
    }

    /// @notice Unpause the contract — re-enables `contribute()` and `withdraw()`.
    /// @dev    Only `DEFAULT_ADMIN_ROLE` may unpause (asymmetric by design).
    ///         Emits `(access, unpaused)` event.
    ///
    /// # Arguments
    /// * `caller` — Must be `DEFAULT_ADMIN_ROLE`.
    ///
    /// # Panics
    /// * `"only DEFAULT_ADMIN_ROLE can unpause"` if `caller` is not the admin.
    pub fn unpause(env: Env, caller: Address) {
        pause_mechanism::unpause(&env, &caller);
    }

    /// @notice Returns `true` if the contract is currently paused.
    pub fn paused(env: Env) -> bool {
        pause_mechanism::is_paused(&env)
    }

    /// @notice Permanently stop the contract — irreversible.
    /// @dev    Only `DEFAULT_ADMIN_ROLE` may call this.
    ///         Sets campaign status to `Cancelled` so contributors can refund.
    ///         Emits `(emergency, stopped)` event.
    ///
    /// # Arguments
    /// * `caller` — Must be `DEFAULT_ADMIN_ROLE`.
    ///
    /// # Panics
    /// * `"only DEFAULT_ADMIN_ROLE can trigger emergency stop"` if not admin.
    /// * `"already stopped"` if already triggered.
    pub fn emergency_stop(env: Env, caller: Address) {
        emergency_stop::trigger(&env, &caller);
    }

    /// @notice Returns `true` if the emergency stop has been triggered.
    pub fn is_stopped(env: Env) -> bool {
        emergency_stop::is_stopped(&env)
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

        let current_title = env.storage().instance().get::<_, String>(&DataKey::Title);
        let current_description = env
            .storage()
            .instance()
            .get::<_, String>(&DataKey::Description);
        let current_socials = env
            .storage()
            .instance()
            .get::<_, String>(&DataKey::SocialLinks);

        let title_length = title
            .as_ref()
            .map(|value| value.len())
            .or_else(|| current_title.as_ref().map(|value| value.len()))
            .unwrap_or(0);
        let description_length = description
            .as_ref()
            .map(|value| value.len())
            .or_else(|| current_description.as_ref().map(|value| value.len()))
            .unwrap_or(0);
        let socials_length = socials
            .as_ref()
            .map(|value| value.len())
            .or_else(|| current_socials.as_ref().map(|value| value.len()))
            .unwrap_or(0);
        if let Err(_) = contract_state_size::validate_metadata_total_length(
            title_length,
            description_length,
            socials_length,
        ) {
            panic!("state size limit exceeded");
        }

        // Update title if provided.
        if let Some(new_title) = title {
            if let Err(_) = contract_state_size::validate_title(&new_title) {
                panic!("state size limit exceeded");
            }
            env.storage().instance().set(&DataKey::Title, &new_title);
            updated_fields.push_back(Symbol::new(&env, "title"));
        }

        // Update description if provided.
        if let Some(new_description) = description {
            if let Err(_) = contract_state_size::validate_description(&new_description) {
                panic!("state size limit exceeded");
            }
            env.storage()
                .instance()
                .set(&DataKey::Description, &new_description);
            updated_fields.push_back(Symbol::new(&env, "description"));
        }

        // Update social links if provided.
        if let Some(new_socials) = socials {
            if let Err(_) = contract_state_size::validate_social_links(&new_socials) {
                panic!("state size limit exceeded");
            }
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

        // Enforce string length and roadmap list size limits.
        contract_state_size::check_string_len(&description).expect("description too long");
        contract_state_size::check_roadmap_limit(&env).expect("roadmap limit exceeded");

        let mut roadmap: Vec<RoadmapItem> = env
            .storage()
            .instance()
            .get(&DataKey::Roadmap)
            .unwrap_or_else(|| Vec::new(&env));
        if let Err(_) = contract_state_size::validate_roadmap_capacity(roadmap.len()) {
            panic!("state size limit exceeded");
        }
        if let Err(_) = contract_state_size::validate_roadmap_description(&description) {
            panic!("state size limit exceeded");
        }

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

        // Enforce stretch-goal list size limit.
        contract_state_size::check_stretch_goal_limit(&env).expect("stretch goal limit exceeded");

        let mut stretch_goals: Vec<i128> = env
            .storage()
            .instance()
            .get(&DataKey::StretchGoals)
            .unwrap_or_else(|| Vec::new(&env));
        if let Err(_) = contract_state_size::validate_stretch_goal_capacity(stretch_goals.len()) {
            panic!("state size limit exceeded");
        }

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

        next_unmet_milestone(total_raised, &stretch_goals)
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

        compute_bonus_goal_progress_bps(
            total_raised,
            env.storage().instance().get::<_, i128>(&DataKey::BonusGoal),
        )
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

    /// Returns the maximum individual contribution amount (if set).
    pub fn max_individual_contribution(env: Env) -> Option<i128> {
        env.storage()
            .instance()
            .get(&DataKey::MaxIndividualContribution)
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

        build_campaign_stats(&env, total_raised, goal, &contributors)
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

    /// Returns the decimal precision of the campaign token.
    ///
    /// All goal and contribution amounts are expressed in the token's smallest
    /// unit (e.g. stroops for XLM, micro-USDC for USDC). Use this value to
    /// convert raw amounts to human-readable form: `amount / 10^decimals`.
    pub fn token_decimals(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::TokenDecimals)
            .unwrap()
    }

    /// Returns the configured NFT contract address, if any.
    pub fn nft_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::NFTContract)
    }
}
