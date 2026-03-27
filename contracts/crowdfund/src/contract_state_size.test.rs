//! Comprehensive tests for the ContractStateSize contract.
//!
//! @title   ContractStateSize Tests
//! @notice  Validates each constant exposure through the contract interface and validation logic.
//! @dev     Uses soroban-sdk's test utilities to mock the environment.

#[cfg(test)]
mod tests {
    use crate::contract_state_size::{
        ContractStateSize, ContractStateSizeClient, MAX_CONTRIBUTORS, MAX_DESCRIPTION_LENGTH,
        MAX_TITLE_LENGTH,
    };
    use soroban_sdk::{Env, String};

    /// Setup a fresh test environment with the state size contract registered.
    fn setup() -> (Env, ContractStateSizeClient<'static>) {
        let env = Env::default();
        let contract_id = env.register(ContractStateSize, ());
        let client = ContractStateSizeClient::new(&env, &contract_id);
        (env, client)
    }

    #[test]
    fn test_constants_return_correct_values() {
        let (_env, client) = setup();
        assert_eq!(client.max_title_length(), MAX_TITLE_LENGTH);
        assert_eq!(client.max_description_length(), MAX_DESCRIPTION_LENGTH);
        assert_eq!(client.max_contributors(), MAX_CONTRIBUTORS);
        assert_eq!(client.max_roadmap_items(), 32);
        assert_eq!(client.max_stretch_goals(), 32);
        assert_eq!(client.max_social_links_length(), 512);
    }

    #[test]
    fn test_validate_title() {
        let (env, client) = setup();
        let valid_title = String::from_str(&env, "A valid project title");
        let too_long_title = String::from_str(&env, &"A".repeat((MAX_TITLE_LENGTH + 1) as usize));

        assert!(client.validate_title(&valid_title));
        assert!(!client.validate_title(&too_long_title));
    }

    #[test]
    fn test_validate_description() {
        let (env, client) = setup();
        let valid_desc = String::from_str(&env, "A valid project description");
        let too_long_desc =
            String::from_str(&env, &"A".repeat((MAX_DESCRIPTION_LENGTH + 1) as usize));

        assert!(client.validate_description(&valid_desc));
        assert!(!client.validate_description(&too_long_desc));
    }

    #[test]
    fn test_validate_metadata_aggregate() {
        let (_env, client) = setup();
        let limit = 128 + 2048 + 512;
        assert!(client.validate_metadata_aggregate(&100));
        assert!(client.validate_metadata_aggregate(&limit));
        assert!(!client.validate_metadata_aggregate(&(limit + 1)));
    }

    // ... (rest of the code remains the same)

    // ── Pure helper tests ────────────────────────────────────────────────────────

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(MAX_CONTRIBUTORS, 128);
        assert_eq!(MAX_PLEDGERS, 128);
        assert_eq!(MAX_ROADMAP_ITEMS, 32);
        assert_eq!(MAX_STRETCH_GOALS, 32);
        assert_eq!(MAX_TITLE_LENGTH, 128);
        assert_eq!(MAX_DESCRIPTION_LENGTH, 2_048);
        assert_eq!(MAX_SOCIAL_LINKS_LENGTH, 512);
        assert_eq!(MAX_BONUS_GOAL_DESCRIPTION_LENGTH, 280);
        assert_eq!(MAX_ROADMAP_DESCRIPTION_LENGTH, 280);
        assert_eq!(MAX_METADATA_TOTAL_LENGTH, 2_304);
    }

    // ... (rest of the code remains the same)

    // ── Contract wiring tests ────────────────────────────────────────────────────

    #[test]
    fn initialize_accepts_bonus_goal_description_at_exact_limit() {
        let (env, client, creator, token_address, _admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        let description = soroban_string(&env, MAX_BONUS_GOAL_DESCRIPTION_LENGTH, 'b');

        client.initialize(
            &creator,
            &creator,
            &token_address,
            &1_000_000,
            &deadline,
            &1_000,
            &None,
            &Some(2_000_000),
            &Some(description.clone()),
        );

        assert_eq!(client.bonus_goal_description(), Some(description));
    }

    // ... (rest of the code remains the same)

    #[test]
    #[should_panic(expected = "stretch goal limit exceeded")]
    fn add_stretch_goal_rejects_when_capacity_full() {
        let (env, client, creator, token_address, _admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        default_init(&client, &creator, &token_address, deadline);

        env.as_contract(&client.address, || {
            let stretch_goals = filled_stretch_goals(&env, MAX_STRETCH_GOALS);
            env.storage()
                .instance()
                .set(&DataKey::StretchGoals, &stretch_goals);
        });

        client.add_stretch_goal(&9_999_999i128);
    }
}
