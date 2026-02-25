#![no_std]
#[allow(clippy::too_many_arguments)]
use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, token, Address, Env, String, Symbol, Vec,
};
#![allow(missing_docs)]
#![allow(clippy::too_many_arguments)]

// ── Modules ──────────────────────────────────────────────────────────────────
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, token, Address, Env, String, Symbol, Vec,
    contract, contractclient, contracterror, contractimpl, contracttype, token, Address, Env,
    String, Symbol, Vec,
};

pub mod access_control;
pub mod admin_upgrade_mechanism;
pub mod campaign_goal_minimum;
pub mod cargo_toml_rust;
pub mod contract_state_size;
pub mod contribute_error_handling;
pub mod crowdfund_initialize_function;
#[cfg(test)]
pub mod npm_package_lock;
pub mod proptest_generator_boundary;
pub mod refund_single_token;
pub mod soroban_sdk_minor;
pub mod stellar_token_minter;
pub mod stream_processing_optimization;
pub mod withdraw_event_emission;

// ── Imports from modules ──────────────────────────────────────────────────────

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
#[cfg(test)]
#[path = "npm_package_lock_test.rs"]
mod npm_package_lock_test;
#[cfg(test)]
mod test;

#[cfg(test)]
pub mod proptest_generator_boundary;
#[cfg(test)]
#[path = "proptest_generator_boundary.test.rs"]
mod proptest_generator_boundary_test;
#[cfg(test)]
#[path = "soroban_sdk_minor_test.rs"]
mod soroban_sdk_minor_test;
#[cfg(test)]
#[path = "stellar_token_minter.test.rs"]
mod stellar_token_minter_test_comprehensive;
#[cfg(test)]
#[path = "stream_processing_optimization.test.rs"]
mod stream_processing_optimization_test;

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
const CONTRACT_VERSION: u32 = 3;

#[derive(Clone, PartialEq)]
#[contracttype]
pub enum Status {
    Active,
    Successful,
    Refunded,
    Cancelled,
}

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
/// A reward tier with a name and minimum contribution amount to qualify.
#[derive(Clone)]
#[contracttype]
pub struct RewardTier {
    pub name: String,
    pub min_amount: i128,
}

/// Represents all storage keys used by the crowdfund contract.
#[derive(Clone)]
#[contracttype]
pub struct Contribution {
    pub amount: i128,
    pub is_early_bird: bool,
}

/// Represents a recurring subscription for patronage campaigns.
#[derive(Clone)]
#[contracttype]
pub struct Subscription {
    /// Amount to contribute per interval.
    pub amount: i128,
    /// Interval in seconds between contributions.
    pub interval: u64,
    /// Last time the subscription was processed (ledger timestamp).
    pub last_processed: u64,
}

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

#[derive(Clone)]
#[contracttype]
pub struct CampaignInfo {
    pub creator: Address,
    pub token: Address,
    pub goal: i128,
    pub deadline: u64,
    pub total_raised: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct PlatformConfig {
    pub address: Address,
    pub fee_bps: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct FeeTier {
    pub threshold: i128,
    pub fee_bps: u32,
}

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
    Token,
    Goal,
    Deadline,
    TotalRaised,
    Contribution(Address),
    Contributors,
    /// Current campaign status.
    Status,
    MinContribution,
    Pledge(Address),
    /// Total amount pledged but not yet collected.
    TotalPledged,
    StretchGoals,
    BonusGoal,
    BonusGoalDescription,
    BonusGoalReachedEmitted,
    Pledgers,
    /// Individual pledge by address (for conditional pledges).
    Pledge(Address),
    /// Total amount pledged but not yet claimed.
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
    Title,
    Description,
    SocialLinks,
    /// Platform fee configuration.
    PlatformConfig,
    NFTContract,
    /// Decimal precision of the campaign token (e.g. 7 for XLM, 6 for USDC).
    TokenDecimals,

    // ── Role-separation keys (access_control module) ──────────────────────
    /// Address with DEFAULT_ADMIN_ROLE — can upgrade, unpause, and transfer roles.
    DefaultAdmin,
    /// Address with PAUSER_ROLE — can pause in an emergency but cannot unpause.
    Pauser,
    /// Governance address (multisig / DAO) — the only address that may set platform fees.
    GovernanceAddress,
    /// Boolean flag — when true, contribute() and withdraw() are blocked.
    Paused,
    /// Individual pledge by address (not yet transferred).
    /// Whether the contract is paused.
    Paused,
    /// Individual pledge by address.
    Pledge(Address),
    /// Total amount pledged (not yet transferred).
    TotalPledged,
    /// List of all pledger addresses.
    Pledgers,
    /// List of stretch goal milestones above the primary goal.
    /// Maximum total amount that can be raised (hard cap).
    HardCap,
    /// List of reward tiers (name + min_amount).
    RewardTiers,
    /// Individual pledge by address.
    Pledge(Address),
    /// List of all pledger addresses.
    Pledgers,
    /// Total amount pledged (not yet collected).
    TotalPledged,
    /// List of stretch goal milestones.
    StretchGoals,
    /// Individual subscription by address (amount, interval, last_processed).
    Subscription(Address),
    /// List of all subscriber addresses.
    Subscribers,
    /// Campaign updates blog: Vec<(u64, String)> of (timestamp, update text).
    Updates,
    /// Whether whitelist is enabled for this campaign.
    WhitelistEnabled,
    /// Individual whitelist entry by address.
    Whitelist(Address),
    /// Total amount referred by each referrer address.
    ReferralTally(Address),
    /// Optional secondary bonus goal.
    BonusGoal,
    /// Optional bonus goal description.
    BonusGoalDescription,
    /// Whether a bonus-goal reached event was emitted.
    BonusGoalReachedEmitted,
}

// ── Rate Limiting ──────────────────────────────────────────────────────────
/// Minimum seconds required between contributions from the same address.
const CONTRIBUTION_COOLDOWN: u64 = 0;

// ── Contract Error ──────────────────────────────────────────────────────────

use soroban_sdk::contracterror;

/// Errors that can be returned by the crowdfund contract.
    NFTContract,
}

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

    /// Returned by `validate_goal_amount` when `goal_amount < MIN_GOAL_AMOUNT`.
    GoalTooLow = 18,

    /// Returned by `contribute` when `amount` is zero.
    ZeroAmount = 13,
    BelowMinimum = 14,
    CampaignNotActive = 15,
    /// Returned by `contribute` when `amount` is negative.
    NegativeAmount = 16,
    NegativeAmount = 11,
    Overflow = 6,
    ContractPaused = 7,
    InvalidHardCap = 7,
    HardCapExceeded = 8,
    /// Primary campaign category (e.g. Technology, Art).
    Category,
    /// Optional descriptive tags.
    Tags,
    RateLimitExceeded = 9,
    ContractPaused = 10,
    InvalidSubscriptionAmount = 11,
    InvalidSubscriptionInterval = 12,
    SubscriptionNotFound = 13,
    /// Platform configuration for fee handling.
    PlatformConfig,
    /// Fee tiers for dynamic fee calculation.
    FeeTiers,
    /// Platform administrator address.
    PlatformAdmin,
    /// Verified status for a creator address.
    Verified(Address),
}

#[derive(Clone, PartialEq)]
#[contracttype]
pub struct CampaignInfo {
    pub creator: Address,
    pub token: Address,
    pub goal: i128,
    pub deadline: u64,
    pub total_raised: i128,
    pub min_contribution: i128,
    pub status: Status,
    pub verified: bool,
    InvalidLimit = 11,
}

/// Interface for an external NFT contract used to mint contributor rewards.
#[contractclient(name = "NftContractClient")]
pub trait NftContract {
    /// Mints an NFT to the given address and returns the new token ID.
    fn mint(env: Env, to: Address) -> u128;
}

/// The main crowdfunding contract.
#[contractclient(name = "NftContractClient")]
pub trait NftContract {
    fn mint(env: Env, to: Address) -> u128;
}

#[contract]
pub struct CrowdfundContract;

#[contractimpl]
impl CrowdfundContract {
    // ── Multisig & DAO Compatibility ───────────────────────────────────────
    //
    // IMPORTANT: This contract fully supports multi-signature wallets and DAO
    // contracts as the campaign creator. The Soroban `Address` type can represent
    // both standard user accounts and contract addresses.
    //
    // When `creator.require_auth()` is called:
    // - For user accounts: Standard signature verification is performed
    // - For contract addresses: The contract's authorization logic is invoked,
    //   allowing multisig wallets, DAO governance contracts, or any custom
    //   authorization scheme to control the campaign
    //
    // All creator-restricted functions (withdraw, set_paused, update_metadata,
    // add_roadmap_item, add_stretch_goal, add_reward_tier, update_deadline,
    // post_update, add_to_whitelist) work seamlessly with multisig creators.
    //
    // Security Benefits:
    // - Eliminates single point of failure for high-value campaigns
    // - Enables decentralized governance of campaign operations
    // - Supports time-locked or threshold-based authorization schemes
    //
    // Example Multisig Patterns:
    // 1. M-of-N Threshold: Require M signatures out of N authorized signers
    // 2. DAO Governance: Require on-chain voting approval for admin actions
    // 3. Time-locked Admin: Require time delay before executing sensitive operations
    // 4. Hierarchical Control: Different authorization levels for different operations
    //
    // ───────────────────────────────────────────────────────────────────────

    /// Helper function to check if the creator address is a contract.
    ///
    /// This is useful for distinguishing between standard user accounts and
    /// contract-based creators (multisig wallets, DAOs, etc.) for logging,
    /// analytics, or conditional logic.
    ///
    /// # Arguments
    /// * `env` – The contract environment
    /// * `creator` – The address to check
    ///
    /// # Returns
    /// * `true` if the address is a contract, `false` if it's a user account
    ///
    /// # Note
    /// This function does not affect authorization - `require_auth()` works
    /// correctly for both user accounts and contracts. This is purely for
    /// informational purposes.
    ///
    /// # Implementation Note
    /// In Soroban, the Address type abstracts over both account and contract
    /// addresses. While there's no direct API to check the address type at
    /// runtime, the authorization mechanism (`require_auth()`) handles both
    /// cases transparently. This helper is provided for documentation purposes
    /// and potential future use cases where distinguishing address types is needed.
    #[allow(dead_code)]
    fn is_contract_creator(_env: &Env, _creator: &Address) -> bool {
        // Note: Soroban's Address type doesn't expose a direct method to check
        // if an address is a contract vs. an account at runtime. However, this
        // doesn't matter for authorization - require_auth() works correctly for
        // both types.
        //
        // For actual runtime checks, you would need to:
        // 1. Attempt to invoke a known contract function (will fail for accounts)
        // 2. Use external indexing/metadata to track contract addresses
        // 3. Rely on off-chain knowledge of the creator type
        //
        // Since authorization works transparently for both types, this function
        // is primarily for documentation and potential future enhancements.
        false // Placeholder - actual implementation would require additional context
    }

    /// Initializes a new crowdfunding campaign.
    ///
    /// Delegates all validation and storage logic to
    /// [`crowdfund_initialize_function::execute_initialize`].
    /// # Arguments
    /// * `creator`            – The campaign creator's address (can be a user account,
    ///                          multisig wallet, or DAO contract).
    /// * `token`              – The token contract address used for contributions.
    /// * `goal`               – The funding goal (in the token's smallest unit).
    /// * `hard_cap`           – Maximum total amount that can be raised (must be >= goal).
    /// * `deadline`           – The campaign deadline as a ledger timestamp.
    /// * `min_contribution`   – The minimum contribution amount.
    /// * `title`              – The campaign title.
    /// * `description`        – The campaign description.
    /// * `platform_config`    – Optional platform configuration (address and fee in basis points).
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
    /// # Multisig Support
    /// The `creator` parameter can be any valid Soroban address, including:
    /// - Standard user accounts (ed25519 public keys)
    /// - Multisig wallet contracts (requiring M-of-N signatures)
    /// - DAO governance contracts (requiring on-chain voting)
    /// - Custom authorization contracts (time-locks, hierarchical permissions, etc.)
    ///
    /// The `creator.require_auth()` call ensures that only the authorized entity
    /// (whether a single user or a multisig group) can initialize the campaign.
    ///
    /// # Panics
    /// * If already initialized.
    /// * If platform fee exceeds 10,000 (100%).
    /// * `admin`            – The platform administrator's address.
    /// * `creator`          – The campaign creator's address.
    /// * `token`            – The token contract address used for contributions.
    /// * `goal`             – The funding goal (in the token's smallest unit).
    /// * `deadline`         – The campaign deadline as a ledger timestamp.
    /// * `min_contribution` – The minimum contribution amount.
    /// * `category`         – Primary campaign category (e.g. Technology, Art).
    /// * `tags`             – Optional descriptive tags for the campaign.
    #[allow(clippy::too_many_arguments)]
    /// * `platform_config`  – Optional platform configuration (address and fee in basis points).
    /// * `fee_tiers`        – Optional fee tiers for dynamic fee calculation.
    pub fn initialize(
        env: Env,
        admin: Address,
        creator: Address,
        token: Address,
        goal: i128,
        hard_cap: i128,
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
        )
        category: soroban_sdk::String,
        tags: Vec<soroban_sdk::String>,
        platform_config: Option<PlatformConfig>,
        fee_tiers: Option<Vec<FeeTier>>,
    ) {
        // Prevent re-initialization.
        if env.storage().instance().has(&DataKey::Creator) {
            return Err(ContractError::AlreadyInitialized);
        }

        let eb_deadline = match early_bird_deadline {
            Some(eb) => {
                if eb >= deadline {
                    panic!("early bird deadline must be before campaign deadline");
                }
                eb
            }
            None => core::cmp::min(env.ledger().timestamp() + 86400, deadline.saturating_sub(1)),
        };
        if category.len() == 0 {
            panic!("category must not be empty");
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
        if hard_cap < goal {
            return Err(ContractError::InvalidHardCap);
        }

        // Validate and store fee tiers if provided.
        if let Some(ref tiers) = fee_tiers {
            if !tiers.is_empty() {
                // Validate each tier's fee_bps.
                for tier in tiers.iter() {
                    if tier.fee_bps > 10_000 {
                        panic!("fee tier fee_bps cannot exceed 10000");
                    }
                }

                // Validate tiers are ordered by threshold ascending.
                for i in 1..tiers.len() {
                    let prev = tiers.get(i - 1).unwrap();
                    let curr = tiers.get(i).unwrap();
                    if curr.threshold <= prev.threshold {
                        panic!("fee tiers must be ordered by threshold ascending");
                    }
                }

                env.storage().instance().set(&DataKey::FeeTiers, tiers);
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

        /// Returns the list of all contributor addresses.
        #[allow(dead_code)]
        pub fn contributors(env: Env) -> Vec<Address> {
            env.storage()
                .instance()
                .get(&DataKey::Contributors)
                .unwrap_or(Vec::new(&env))
        if hard_cap < goal {
            return Err(ContractError::InvalidHardCap);
        }

        env.storage().instance().set(&DataKey::Creator, &creator);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::HardCap, &hard_cap);
        env.storage().instance().set(&DataKey::PlatformAdmin, &admin);
        env.storage().instance().set(&DataKey::Creator, &creator);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Goal, &goal);
        env.storage().instance().set(&DataKey::HardCap, &hard_cap);
        env.storage().instance().set(&DataKey::Deadline, &deadline);
        env.storage()
            .instance()
            .set(&DataKey::MinContribution, &min_contribution);
        env.storage().instance().set(&DataKey::Title, &title);
        env.storage().instance().set(&DataKey::Description, &description);
        env.storage().instance().set(&DataKey::MinContribution, &min_contribution);
        env.storage().instance().set(&DataKey::Category, &category);
        env.storage().instance().set(&DataKey::Tags, &tags);
        env.storage()
            .instance()
            .set(&DataKey::Description, &description);
        if let Some(config) = platform_config {
            env.storage()
                .instance()
                .set(&DataKey::PlatformConfig, &config);
        }
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::BonusGoalReachedEmitted, &false);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Active);
        env.storage().instance().set(&DataKey::Paused, &false);

        // Store platform config if provided.
        if let Some(config) = platform_config {
            env.storage().instance().set(&DataKey::PlatformConfig, &config);
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

    /// Returns the list of all contributor addresses.
    pub fn contributors(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or(Vec::new(&env))
    /// Adds addresses to the campaign's whitelist.
    ///
    /// This function is restricted to the campaign creator and can only be
    /// called while the campaign is Active.
    pub fn add_to_whitelist(env: Env, addresses: Vec<Address>) {
        if addresses.is_empty() {
            panic!("addresses list must not be empty");
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

    pub fn contribute(env: Env, contributor: Address, amount: i128) -> Result<(), ContractError> {
        contributor.require_auth();

        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        if !env.storage().instance().has(&DataKey::WhitelistEnabled) {
            env.storage()
                .instance()
                .set(&DataKey::WhitelistEnabled, &true);
        }

        for address in addresses.iter() {
            env.storage()
                .instance()
                .set(&DataKey::Whitelist(address), &true);
        }
    }

    /// Contribute tokens to the campaign.
    ///
    /// The contributor must authorize the call. Contributions are rejected
    /// after the deadline has passed or if the campaign is not active.
    pub fn contribute(env: Env, contributor: Address, amount: i128) -> Result<(), ContractError> {
    /// after the deadline has passed.
    pub fn contribute(
        env: Env,
        contributor: Address,
        amount: i128,
        referral: Option<Address>,
    ) -> Result<(), ContractError> {
        // ── Rate limiting: enforce cooldown between contributions ──
        let now = env.ledger().timestamp();
        let last_time_key = DataKey::LastContributionTime(contributor.clone());
        if let Some(last_time) = env.storage().persistent().get::<_, u64>(&last_time_key) {
            if now < last_time + CONTRIBUTION_COOLDOWN {
                return Err(ContractError::RateLimitExceeded);
            }
        }

        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(ContractError::ContractPaused);
        }

        contributor.require_auth();

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

        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));
        let is_new_contributor = !contributors.contains(&contributor);
        if is_new_contributor {
            if let Err(err) =
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

        let pledgers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Pledgers)
            .unwrap_or_else(|| Vec::new(&env));
        let is_new_pledger = !pledgers.contains(&pledger);
        if is_new_pledger {
            if let Err(err) =
                contract_state_size::validate_pledger_capacity(pledger_stream.entries.len())
            {
                panic!("state size limit exceeded");
            }
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        let hard_cap: i128 = env.storage().instance().get(&DataKey::HardCap).unwrap();

        if total >= hard_cap {
            return Err(ContractError::HardCapExceeded);
        }

        let headroom = hard_cap - total;
        let effective_amount = if amount <= headroom { amount } else { headroom };

        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&contributor, &env.current_contract_address(), &amount);

        let contribution_key = DataKey::Contribution(contributor.clone());
        let previous_amount: i128 = env
            .storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&contribution_key, &(previous_amount + amount));
        env.storage()
            .persistent()
            .extend_ttl(&contribution_key, 100, 100);

        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &(total + amount));

        if new_total == hard_cap {
            env.events()
                .publish(("campaign", "hard_cap_reached"), hard_cap);
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

        // Track contributor address if new.
        let mut contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        if !contributors.contains(&contributor) {
            contributors.push_back(contributor);
            env.storage()
                .persistent()
                .set(&DataKey::Contributors, &contributors);
            env.storage()
                .persistent()
                .extend_ttl(&DataKey::Contributors, 100, 100);
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
        // Emit contribution event
        env.events()
            .publish(("campaign", "contributed"), (contributor, amount));
        env.events().publish(
            ("campaign", "contributed"),
            (contributor, effective_amount),
        );
            .publish(("campaign", "contributed"), (contributor, effective_amount));
            .publish(("campaign", "contributed"), (contributor.clone(), effective_amount));
        env.events().publish(
            ("campaign", "contributed"),
            (contributor.clone(), effective_amount),
        );

        // Update referral tally if referral provided
        if let Some(referrer) = referral {
            if referrer != contributor {
                let referral_key = DataKey::ReferralTally(referrer.clone());
                let current_tally: i128 =
                    env.storage().persistent().get(&referral_key).unwrap_or(0);

                let new_tally = current_tally
                    .checked_add(effective_amount)
                    .ok_or(ContractError::Overflow)?;

                env.storage().persistent().set(&referral_key, &new_tally);
                env.storage()
                    .persistent()
                    .extend_ttl(&referral_key, 100, 100);

                // Emit referral event
                env.events().publish(
                    ("campaign", "referral"),
                    (referrer, contributor, effective_amount),
                );
            }
        }

        // Update last contribution time for rate limiting
        env.storage().persistent().set(&last_time_key, &now);
        env.storage()
            .persistent()
            .extend_ttl(&last_time_key, 100, 100);

        Ok(())
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
        // Update global total raised.
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &(total - amount));
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
    ///
    /// # Multisig Support
    /// This function fully supports multisig and DAO creators. When the creator
    /// is a contract address, `creator.require_auth()` will invoke the contract's
    /// authorization logic, enabling:
    /// - M-of-N threshold signatures for withdrawal approval
    /// - DAO governance voting before fund withdrawal
    /// - Time-locked withdrawals for added security
    /// - Any custom authorization scheme implemented by the creator contract
    ///
    /// # Security Note
    /// For high-value campaigns, using a multisig or DAO as the creator significantly
    /// reduces the risk of unauthorized fund withdrawal, as multiple parties must
    /// approve the transaction.
        Ok(())
    }

    pub fn withdraw(env: Env) -> Result<(), ContractError> {
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
                config.fee_bps,
            );
            total.checked_sub(fee).expect("creator payout underflow")
        } else {
            total
        };

        token_client.transfer(&env.current_contract_address(), &creator, &creator_payout);

        env.storage().instance().set(&DataKey::TotalRaised, &0i128);

        // Bounded NFT minting: process at most MAX_NFT_MINT_BATCH contributors
        // per withdraw() call to cap event emission and gas consumption.
        let nft_contract: Option<Address> = env.storage().instance().get(&DataKey::NFTContract);
        let nft_minted_count = mint_nfts_in_batch(&env, &nft_contract);

        // Single withdrawal event carrying payout and mint count.
        emit_withdrawn(&env, &creator, creator_payout, nft_minted_count);

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
        execute_refund_single(&env, &contributor, amount)
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
        // Calculate and transfer platform fee if configured.
        let platform_config: Option<PlatformConfig> =
            env.storage().instance().get(&DataKey::PlatformConfig);

        let creator_payout = if let Some(config) = platform_config {
            let fee = total
                .checked_mul(config.fee_bps as i128)
                .expect("fee calculation overflow")
                .checked_div(10_000)
                .expect("fee division by zero");
        let platform_config: Option<PlatformConfig> = env.storage().instance().get(&DataKey::PlatformConfig);
        let fee_tiers: Option<Vec<FeeTier>> = env.storage().instance().get(&DataKey::FeeTiers);

        let creator_payout = if let Some(config) = platform_config {
            let fee = if let Some(tiers) = fee_tiers {
                // Use tiered fee calculation.
                Self::calculate_tiered_fee(&env, total, &tiers)
            } else {
                // Fall back to flat fee.
                total * config.fee_bps as i128 / 10_000
            };

            token_client.transfer(&env.current_contract_address(), &config.address, &fee);
            env.events()
                .publish(("campaign", "fee_transferred"), (&config.address, fee));
            total.checked_sub(fee).expect("creator payout underflow")
            total - fee
        } else {
            total
        };

        token_client.transfer(&env.current_contract_address(), &creator, &creator_payout);

        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Successful);

        // Mint one commemorative NFT per eligible contributor after successful payout.
        if let Some(nft_contract) = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::NFTContract)
        {
            let nft_client = NftContractClient::new(&env, &nft_contract);
            let contributors: Vec<Address> = env
                .storage()
                .persistent()
                .get(&DataKey::Contributors)
                .unwrap_or_else(|| Vec::new(&env));

            for contributor in contributors.iter() {
                let amount: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Contribution(contributor.clone()))
                    .unwrap_or(0);

                // Only mint for contributors with a non-zero stake.
                if amount > 0 {
                    let token_id = nft_client.mint(&contributor);
                    env.events().publish(
                        (
                            Symbol::new(&env, "campaign"),
                            Symbol::new(&env, "nft_minted"),
                        ),
                        (contributor, token_id),
                    );
                }
            }
        }

        env.events()
            .publish(("campaign", "withdrawn"), (creator.clone(), total));

        Ok(())
    }

    pub fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError> {
        contributor.require_auth();

        // Check campaign status is Active.
    /// Calculate tiered fee based on total raised and fee tiers.
    fn calculate_tiered_fee(_env: &Env, total: i128, tiers: &Vec<FeeTier>) -> i128 {
        let mut fee = 0i128;
        let mut prev_threshold = 0i128;

        for tier in tiers.iter() {
            if total <= prev_threshold {
                break;
            }

            let portion_end = if total < tier.threshold { total } else { tier.threshold };
            let portion = portion_end - prev_threshold;
            let portion_fee = portion * tier.fee_bps as i128 / 10_000;

            fee += portion_fee;
            prev_threshold = tier.threshold;
        }

        // Apply the last tier's rate to any amount above the highest threshold.
        if total > prev_threshold && !tiers.is_empty() {
            let last_tier = tiers.get(tiers.len() - 1).unwrap();
            let remaining = total - prev_threshold;
            let remaining_fee = remaining * last_tier.fee_bps as i128 / 10_000;
            fee += remaining_fee;
        }

        fee
    }

    /// Refund all contributors — callable by anyone after the deadline
    /// if the goal was **not** met.
    pub fn refund(env: Env) -> Result<(), ContractError> {
        let paused: bool = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        if paused {
            return Err(ContractError::ContractPaused);
        }

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

        let contribution_key = DataKey::Contribution(contributor.clone());
        let amount: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        if amount == 0 {
            return Ok(());
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
            .set(&DataKey::Status, &Status::Cancelled);
        // Transfer tokens back to the contributor.
        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &contributor, &amount);

        // Reset the contributor's contribution to 0.
        env.storage().persistent().set(&contribution_key, &0i128);
        env.storage()
            .persistent()
            .extend_ttl(&contribution_key, 100, 100);

        // Update total raised.
        let new_total = total - amount;
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &new_total);

        // Emit refund event
        env.events()
            .publish(("campaign", "refunded"), (contributor.clone(), amount));
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &(total - amount));

        if total - amount == 0 {
            env.storage()
                .instance()
                .set(&DataKey::Status, &Status::Refunded);
        }

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

    /// Pause or unpause the contract — creator-only.
    ///
    /// When paused, all contributions, withdrawals, and refunds are blocked.
    /// This is a security mechanism to halt operations in case of detected
    /// vulnerabilities or external threats.
    ///
    /// # Arguments
    /// * `paused` – True to pause, false to unpause.
    ///
    /// # Multisig Support
    /// This critical security function works seamlessly with multisig creators.
    /// When the creator is a multisig contract, pausing/unpausing requires
    /// approval from the multisig signers, preventing any single party from
    /// unilaterally halting the campaign. This is especially important for
    /// high-value campaigns where pause decisions should be collective.
    pub fn set_paused(env: Env, paused: bool) {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        env.storage().instance().set(&DataKey::Paused, &paused);

        let event_name = if paused { "paused" } else { "unpaused" };
        env.events().publish(("campaign", event_name), ());
    }

    /// Update campaign metadata — only callable by the creator while the
    /// campaign is still Active.
    ///
    /// # Arguments
    /// * `creator`     – The campaign creator's address (for authentication).
    /// * `title`       – Optional new title (None to keep existing).
    /// * `description` – Optional new description (None to keep existing).
    /// * `socials`    – Optional new social links (None to keep existing).
    ///
    /// # Multisig Support
    /// Metadata updates support multisig creators, ensuring that campaign
    /// information changes require collective approval when using a multisig
    /// or DAO creator. This prevents unauthorized modification of campaign
    /// details and maintains transparency.
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
        if let Err(err) = contract_state_size::validate_metadata_total_length(
            title_length,
            description_length,
            socials_length,
        ) {
            panic!("state size limit exceeded");
        }

        // Update title if provided.
        if let Some(new_title) = title {
            if let Err(err) = contract_state_size::validate_title(&new_title) {
                panic!("state size limit exceeded");
            }
            env.storage().instance().set(&DataKey::Title, &new_title);
            updated_fields.push_back(Symbol::new(&env, "title"));
        }

        // Update description if provided.
        if let Some(new_description) = description {
            if let Err(err) = contract_state_size::validate_description(&new_description) {
                panic!("state size limit exceeded");
            }
            env.storage()
                .instance()
                .set(&DataKey::Description, &new_description);
            updated_fields.push_back(Symbol::new(&env, "description"));
        }

        // Update social links if provided.
        if let Some(new_socials) = socials {
            if let Err(err) = contract_state_size::validate_social_links(&new_socials) {
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
        // Emit metadata_updated event with the list of updated field names.
        env.events().publish(
            (
                Symbol::new(&env, "campaign"),
                Symbol::new(&env, "metadata_updated"),
            ),
            updated_fields,
        );
    }

    /// Add a roadmap item — only callable by the creator.
    /// Update the campaign deadline — only callable by the creator while the
    /// campaign is still Active.
    ///
    /// # Arguments
    /// * `new_deadline` – The new deadline as a ledger timestamp (must be greater than current deadline).
    ///
    /// # Panics
    /// * If the campaign is not Active.
    /// * If new_deadline is less than or equal to the current deadline.
    pub fn update_deadline(env: Env, new_deadline: u64) {
        // Check campaign is active.
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        // Require creator authentication.
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        // Get the current deadline.
        let current_deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();

        // Ensure new_deadline is greater than current_deadline (only extensions allowed).
        if new_deadline <= current_deadline {
            panic!("new deadline must be after current deadline");
        }

        // Update the deadline.
        env.storage()
            .instance()
            .set(&DataKey::Deadline, &new_deadline);

        // Emit deadline_updated event with old and new deadline values.
        env.events().publish(
            ("campaign", "deadline_updated"),
            (current_deadline, new_deadline),
        );
    }

    // ── Verification Management ─────────────────────────────────────────

    /// Set the verified status for a creator address.
    /// Only callable by the platform admin.
    ///
    /// # Arguments
    /// * `admin`   – The platform admin address (must match stored admin).
    /// * `creator` – The creator address to verify/unverify.
    /// * `status`  – True to verify, false to unverify.
    pub fn set_verified(env: Env, admin: Address, creator: Address, status: bool) {
        admin.require_auth();

        let platform_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::PlatformAdmin)
            .unwrap();

        if admin != platform_admin {
            panic!("only platform admin can set verified status");
        }

        env.storage()
            .instance()
            .set(&DataKey::Verified(creator.clone()), &status);

        // Emit events for verification status changes
        if status {
            env.events()
                .publish(("platform", "creator_verified"), creator);
        } else {
            env.events()
                .publish(("platform", "creator_unverified"), creator);
        }
    }

    /// Check if a creator address is verified.
    ///
    /// # Arguments
    /// * `creator` – The creator address to check.
    ///
    /// # Returns
    /// True if the creator is verified, false otherwise.
    pub fn is_verified(env: Env, creator: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Verified(creator))
            .unwrap_or(false)
    }

    // ── View helpers ────────────────────────────────────────────────────

    /// Add a roadmap item to the campaign timeline.
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
        if let Err(err) = contract_state_size::validate_roadmap_capacity(roadmap.len()) {
            panic!("state size limit exceeded");
        }
        if let Err(err) = contract_state_size::validate_roadmap_description(&description) {
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

    /// Post a campaign update (creator only).
    ///
    /// Records the current timestamp and the provided text. Rejects empty text.
    /// Emits a (campaign, update_posted) event with timestamp and text.
    pub fn post_update(env: Env, text: String) {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        creator.require_auth();

        if text.is_empty() {
            panic!("update text cannot be empty");
        }

        let timestamp = env.ledger().timestamp();

        let mut updates: Vec<(u64, String)> = env
            .storage()
            .instance()
            .get(&DataKey::Updates)
            .unwrap_or_else(|| Vec::new(&env));

        updates.push_back((timestamp, text.clone()));

        env.storage().instance().set(&DataKey::Updates, &updates);
        env.events()
            .publish(("campaign", "update_posted"), (timestamp, text));
    }

    /// Returns the full ordered list of campaign updates.
    ///
    /// Each entry is a tuple of (timestamp, update text) in chronological order.
    pub fn get_updates(env: Env) -> Vec<(u64, String)> {
        env.storage()
            .instance()
            .get(&DataKey::Updates)
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
        if let Err(err) = contract_state_size::validate_stretch_goal_capacity(stretch_goals.len()) {
            panic!("state size limit exceeded");
        }

        stretch_goals.push_back(milestone);
        env.storage()
            .instance()
            .set(&DataKey::StretchGoals, &stretch_goals);
    }

    /// Add a reward tier (creator only). Rejects min_amount <= 0.
    pub fn add_reward_tier(env: Env, creator: Address, name: String, min_amount: i128) {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let stored_creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        if creator != stored_creator {
            panic!("not authorized");
        }
        creator.require_auth();

        if min_amount <= 0 {
            panic!("min_amount must be greater than 0");
        }

        let mut tiers: Vec<RewardTier> = env
            .storage()
            .instance()
            .get(&DataKey::RewardTiers)
            .unwrap_or_else(|| Vec::new(&env));

        tiers.push_back(RewardTier {
            name: name.clone(),
            min_amount,
        });
        env.storage().instance().set(&DataKey::RewardTiers, &tiers);

        env.events()
            .publish(("campaign", "reward_tier_added"), (name, min_amount));
    }

    /// Returns the full ordered list of reward tiers.
    pub fn reward_tiers(env: Env) -> Vec<RewardTier> {
        env.storage()
            .instance()
            .get(&DataKey::RewardTiers)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Returns the highest tier name the user's contribution qualifies for,
    /// or None if the user has not contributed or no tiers are defined.
    /// Tiers are evaluated by min_amount descending (highest qualifying tier wins).
    pub fn get_user_tier(env: Env, user: Address) -> Option<String> {
        let contribution: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Contribution(user))
            .unwrap_or(0);

        if contribution <= 0 {
            return None;
        }

        let tiers: Vec<RewardTier> = env
            .storage()
            .instance()
            .get(&DataKey::RewardTiers)
            .unwrap_or_else(|| Vec::new(&env));

        if tiers.is_empty() {
            return None;
        }

        let mut best: Option<RewardTier> = None;
        for tier in tiers.iter() {
            if contribution >= tier.min_amount {
                let is_better = match &best {
                    None => true,
                    Some(ref b) => tier.min_amount > b.min_amount,
                };
                if is_better {
                    best = Some(tier.clone());
                }
            }
        }

        best.map(|t| t.name)
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
        for milestone in stretch_goals.iter() {
            if total_raised < milestone {
                return milestone;
            }
        }

        0
    }
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
    /// Returns the optional secondary bonus goal.
    pub fn bonus_goal(env: Env) -> Option<i128> {
        env.storage().instance().get(&DataKey::BonusGoal)
    }

    /// Returns the optional bonus-goal description.
    /// Returns the optional secondary bonus goal description.
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
    /// Returns whether the bonus goal has been reached.
    pub fn bonus_goal_reached(env: Env) -> bool {
        let Some(bg) = env.storage().instance().get::<_, i128>(&DataKey::BonusGoal) else {
            return false;
        };
        let total = Self::total_raised(env);
        total >= bg
    }

    /// Returns bonus goal progress in basis points, capped at 10_000.
    pub fn bonus_goal_progress_bps(env: Env) -> u32 {
        let Some(bg) = env.storage().instance().get::<_, i128>(&DataKey::BonusGoal) else {
            return 0;
        };
        if bg <= 0 {
            return 0;
        }
        let total = Self::total_raised(env);
        let raw = (total * 10_000) / bg;
        if raw > 10_000 {
            10_000
        } else {
            raw as u32
        }
    }

    /// Returns the hard cap (maximum total that can be raised).
    pub fn hard_cap(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::HardCap).unwrap()
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
    /// Returns the pledge of a specific address.
    pub fn pledge_amount(env: Env, pledger: Address) -> i128 {
        let pledge_key = DataKey::Pledge(pledger);
        env.storage().persistent().get(&pledge_key).unwrap_or(0)
    }

    /// Returns the total amount pledged (not yet transferred).
    pub fn total_pledged(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalPledged)
            .unwrap_or(0)
    }

    /// Returns the minimum contribution amount.
    pub fn min_contribution(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap()
    }

    /// Returns the maximum individual contribution amount (if set).
    pub fn max_individual_contribution(env: Env) -> Option<i128> {
    /// Returns the campaign creator's address.
    pub fn creator(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Creator).unwrap()
    }

    pub fn nft_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::NFTContract)
    }

    pub fn get_campaign_info(env: Env) -> CampaignInfo {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);
        let title: String = env
            .storage()
            .instance()
            .get(&DataKey::Title)
            .unwrap_or_else(|| String::from_str(&env, ""));
        let description: String = env
            .storage()
            .instance()
            .get(&DataKey::Description)
            .unwrap_or_else(|| String::from_str(&env, ""));

        CampaignInfo {
            creator,
            token,
            goal,
            deadline,
            total_raised,
        }
    }

    /// Returns true if the address is whitelisted.
    pub fn is_whitelisted(env: Env, address: Address) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::MaxIndividualContribution)
    }

    /// Returns the primary campaign category.
    pub fn category(env: Env) -> soroban_sdk::String {
        env.storage().instance().get(&DataKey::Category).unwrap()
    }

    /// Returns the optional descriptive tags.
    pub fn tags(env: Env) -> Vec<soroban_sdk::String> {
        env.storage()
            .instance()
            .get(&DataKey::Tags)
            .unwrap_or(Vec::new(&env))
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

    pub fn title(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Title)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

    pub fn description(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Description)
            .unwrap_or_else(|| String::from_str(&env, ""))
    }

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
    /// Returns the list of all contributor addresses.
    pub fn contributors(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Contributors)
            .unwrap_or(Vec::new(&env))
    }

    // ── Subscription Model (Patronage Campaigns) ───────────────────────────

    /// Subscribe to recurring contributions (patronage model).
    ///
    /// Allows a user to commit to contributing a certain amount every X seconds.
    /// The subscription will be processed by calling `process_subscriptions`.
    ///
    /// # Arguments
    /// * `user` – The subscriber's address
    /// * `amount` – Amount to contribute per interval (must be > 0 and >= min_contribution)
    /// * `interval` – Time in seconds between contributions (must be > 0)
    ///
    /// # Errors
    /// * `InvalidSubscriptionAmount` – If amount <= 0 or < min_contribution
    /// * `InvalidSubscriptionInterval` – If interval <= 0
    pub fn subscribe(
        env: Env,
        user: Address,
        amount: i128,
        interval: u64,
    ) -> Result<(), ContractError> {
        user.require_auth();

        // Validate amount
        if amount <= 0 {
            return Err(ContractError::InvalidSubscriptionAmount);
        }

        // Validate interval
        if interval == 0 {
            return Err(ContractError::InvalidSubscriptionInterval);
        }

        // Check minimum contribution
        let min_contribution: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap_or(0);
        if amount < min_contribution {
            return Err(ContractError::InvalidSubscriptionAmount);
        }

        // Create subscription
        let subscription = Subscription {
            amount,
            interval,
            last_processed: env.ledger().timestamp(),
        };

        // Store subscription
        let sub_key = DataKey::Subscription(user.clone());
        env.storage().persistent().set(&sub_key, &subscription);
        env.storage().persistent().extend_ttl(&sub_key, 100, 100);

        // Add to subscribers list if not already present
        let mut subscribers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Subscribers)
            .unwrap_or_else(|| Vec::new(&env));

        if !subscribers.contains(&user) {
            subscribers.push_back(user.clone());
            env.storage()
                .persistent()
                .set(&DataKey::Subscribers, &subscribers);
            env.storage()
                .persistent()
                .extend_ttl(&DataKey::Subscribers, 100, 100);
        }

        // Emit event
        env.events().publish(
            ("campaign", "subscription_created"),
            (user, amount, interval),
        );

        Ok(())
    }

    /// Process all active subscriptions.
    ///
    /// Can be called by anyone to process subscriptions whose interval has elapsed.
    /// Transfers funds from subscribers to the campaign for each eligible subscription.
    ///
    /// # Returns
    /// Number of subscriptions processed
    pub fn process_subscriptions(env: Env) -> u32 {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            return 0;
        }

        let subscribers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Subscribers)
            .unwrap_or_else(|| Vec::new(&env));

        let current_time = env.ledger().timestamp();
        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        let mut processed_count: u32 = 0;

        for subscriber in subscribers.iter() {
            let sub_key = DataKey::Subscription(subscriber.clone());
            if let Some(mut subscription) =
                env.storage().persistent().get::<_, Subscription>(&sub_key)
            {
                // Check if interval has elapsed
                if current_time >= subscription.last_processed + subscription.interval {
                    // Transfer funds
                    token_client.transfer(
                        &subscriber,
                        &env.current_contract_address(),
                        &subscription.amount,
                    );

                    // Update total raised
                    let total: i128 = env
                        .storage()
                        .instance()
                        .get(&DataKey::TotalRaised)
                        .unwrap_or(0);
                    env.storage()
                        .instance()
                        .set(&DataKey::TotalRaised, &(total + subscription.amount));

                    // Update contribution tracking
                    let contrib_key = DataKey::Contribution(subscriber.clone());
                    let previous_amount: i128 =
                        env.storage().persistent().get(&contrib_key).unwrap_or(0);
                    env.storage()
                        .persistent()
                        .set(&contrib_key, &(previous_amount + subscription.amount));
                    env.storage()
                        .persistent()
                        .extend_ttl(&contrib_key, 100, 100);

                    // Add to contributors list if not already present
                    let mut contributors: Vec<Address> = env
                        .storage()
                        .persistent()
                        .get(&DataKey::Contributors)
                        .unwrap_or_else(|| Vec::new(&env));

                    if !contributors.contains(&subscriber) {
                        contributors.push_back(subscriber.clone());
                        env.storage()
                            .persistent()
                            .set(&DataKey::Contributors, &contributors);
                        env.storage()
                            .persistent()
                            .extend_ttl(&DataKey::Contributors, 100, 100);
                    }

                    // Update last_processed
                    subscription.last_processed = current_time;
                    env.storage().persistent().set(&sub_key, &subscription);
                    env.storage().persistent().extend_ttl(&sub_key, 100, 100);

                    // Emit event
                    env.events().publish(
                        ("campaign", "subscription_processed"),
                        (subscriber, subscription.amount),
                    );

                    processed_count += 1;
    /// Returns the top referrers sorted by total amount referred (descending).
    ///
    /// # Arguments
    /// * `limit` - Maximum number of referrers to return (must be > 0)
    ///
    /// # Returns
    /// Vec of (Address, i128) tuples sorted by amount descending
    ///
    /// # Errors
    /// * ContractError::InvalidLimit - if limit is 0
    pub fn top_referrers(env: Env, limit: u32) -> Result<Vec<(Address, i128)>, ContractError> {
        if limit == 0 {
            return Err(ContractError::InvalidLimit);
        }

        // Get all contributors to find potential referrers
    /// Returns the number of unique contributors.
    pub fn contributor_count(env: Env) -> u32 {
        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        let mut referrers: Vec<(Address, i128)> = Vec::new(&env);

        // Check each contributor's referral tally
        for contributor in contributors.iter() {
            let referral_key = DataKey::ReferralTally(contributor.clone());
            if let Some(tally) = env.storage().persistent().get::<_, i128>(&referral_key) {
                if tally > 0 {
                    referrers.push_back((contributor.clone(), tally));
                }
            }
        }

        processed_count
    }

    /// Cancel a subscription.
    ///
    /// Allows a subscriber to cancel their recurring contributions.
    ///
    /// # Arguments
    /// * `user` – The subscriber's address
    ///
    /// # Errors
    /// * `SubscriptionNotFound` – If no subscription exists for this user
    pub fn unsubscribe(env: Env, user: Address) -> Result<(), ContractError> {
        user.require_auth();

        let sub_key = DataKey::Subscription(user.clone());

        // Check if subscription exists
        if !env.storage().persistent().has(&sub_key) {
            return Err(ContractError::SubscriptionNotFound);
        }

        // Remove subscription
        env.storage().persistent().remove(&sub_key);

        // Remove from subscribers list
        let mut subscribers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Subscribers)
            .unwrap_or_else(|| Vec::new(&env));

        if let Some(index) = subscribers.first_index_of(&user) {
            subscribers.remove(index);
            env.storage()
                .persistent()
                .set(&DataKey::Subscribers, &subscribers);
            env.storage()
                .persistent()
                .extend_ttl(&DataKey::Subscribers, 100, 100);
        }

        // Emit event
        env.events()
            .publish(("campaign", "subscription_cancelled"), user);

        Ok(())
    }

    /// Get subscription details for a user.
    ///
    /// # Arguments
    /// * `user` – The subscriber's address
    ///
    /// # Returns
    /// * `Some(Subscription)` if subscription exists, `None` otherwise
    pub fn get_subscription(env: Env, user: Address) -> Option<Subscription> {
        env.storage().persistent().get(&DataKey::Subscription(user))
    }

    /// Get list of all subscribers.
    pub fn get_subscribers(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Subscribers)
            .unwrap_or_else(|| Vec::new(&env))
    /// Returns the configured fee tiers.
    pub fn fee_tiers(env: Env) -> Vec<FeeTier> {
        env.storage().instance().get(&DataKey::FeeTiers).unwrap_or_else(|| Vec::new(&env))
    }
    /// Returns comprehensive campaign information including verification status.
    pub fn campaign_info(env: Env) -> CampaignInfo {
        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        let verified = Self::is_verified(env.clone(), creator.clone());

        CampaignInfo {
            creator: creator.clone(),
            token: env.storage().instance().get(&DataKey::Token).unwrap(),
            goal: env.storage().instance().get(&DataKey::Goal).unwrap(),
            deadline: env.storage().instance().get(&DataKey::Deadline).unwrap(),
            total_raised: env
                .storage()
                .instance()
                .get(&DataKey::TotalRaised)
                .unwrap_or(0),
            min_contribution: env
                .storage()
                .instance()
                .get(&DataKey::MinContribution)
                .unwrap(),
            status: env.storage().instance().get(&DataKey::Status).unwrap(),
            verified,
        }
    }

    /// Returns true if the campaign is still active (before or at deadline).
    pub fn is_campaign_active(env: Env) -> bool {
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().timestamp() <= deadline
        // Sort by tally descending
        // Note: Soroban doesn't have built-in sorting, so we'll use a simple approach
        // In practice, you might want to implement a more efficient sorting algorithm
        let mut sorted = Vec::new(&env);
        
        // Simple bubble sort (for demonstration - in production use more efficient sort)
        let referrers_len = referrers.len();
        for i in 0..referrers_len {
            let mut max_idx = i;
            for j in (i + 1)..referrers_len {
                if referrers.get(j).unwrap().1 > referrers.get(max_idx).unwrap().1 {
                    max_idx = j;
                }
            }
            if max_idx != i {
                // Swap
                let temp = referrers.get(i).unwrap();
                referrers.set(i, referrers.get(max_idx).unwrap());
                referrers.set(max_idx, temp);
            }
        }

        // Take only the requested limit
        let result_limit = if referrers.len() < limit {
            referrers.len() as u32
        } else {
            limit
        };

        for i in 0..result_limit {
            sorted.push_back(referrers.get(i).unwrap());
        }

        Ok(sorted)
    }

    /// Returns the total amount referred by a specific address.
    ///
    /// # Arguments
    /// * `referrer` - The address to check referral tally for
    ///
    /// # Returns
    /// Total amount referred by the address (0 if none)
    pub fn referral_tally(env: Env, referrer: Address) -> i128 {
        let referral_key = DataKey::ReferralTally(referrer);
        env.storage()
            .persistent()
            .get(&referral_key)
            .unwrap_or(0)
    /// Returns the remaining amount needed to reach the goal.
    pub fn remaining_amount(env: Env) -> i128 {
        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let total_raised: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap_or(0);
        if goal > total_raised { goal - total_raised } else { 0 }
        contributors.len()
    }
}
