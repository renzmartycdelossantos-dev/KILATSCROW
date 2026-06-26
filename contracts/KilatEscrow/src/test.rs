#[cfg(test)]
mod tests {
    use crate::{EscrowStatus, FlowEscrowContract, FlowEscrowContractClient};
    use soroban_sdk::{testutils::Address as _, Address, Env};

    struct TestFixture {
        env: Env,
        client: Address,
        freelancer: Address,
        token: soroban_sdk::token::Client<'static>,
        contract_client: FlowEscrowContractClient<'static>,
    }

    fn setup_test_fixture() -> TestFixture {
        let env = Env::default();
        env.mock_all_auths();

        let client = Address::generate(&env);
        let freelancer = Address::generate(&env);
        let token_admin = Address::generate(&env);

        let token_id = env.register_stellar_asset_contract(token_admin);
        let token = soroban_sdk::token::Client::new(&env, &token_id);
        token.mint(&client, &5000);

        let contract_id = env.register_contract(None, FlowEscrowContract);
        let contract_client = FlowEscrowContractClient::new(&env, &contract_id);

        TestFixture {
            env,
            client,
            freelancer,
            token,
            contract_client,
        }
    }

    #[test]
    fn test_happy_path_settlement() {
        let tf = setup_test_fixture();
        
        // 1. Create and Fund Escrow Engine Layout Pipeline Execution
        let id = tf.contract_client.create_escrow(&tf.client, &tf.freelancer, &tf.token.address, &1000);
        assert_eq!(tf.token.balance(&tf.client), 4000);
        assert_eq!(tf.token.balance(&tf.contract_client.address), 1000);

        // 2. Freelancer marks submission payload delivery clear tracking step parameters
        tf.contract_client.mark_complete(&id);
        
        // 3. Client processes release authorizations execution trigger
        tf.contract_client.release_funds(&id);
        assert_eq!(tf.token.balance(&tf.freelancer), 1000);
        assert_eq!(tf.token.balance(&tf.contract_client.address), 0);
    }

    #[test]
    #[should_panic(expected = "Escrow is not active")]
    fn test_duplicate_completion_edge_case() {
        let tf = setup_test_fixture();
        let id = tf.contract_client.create_escrow(&tf.client, &tf.freelancer, &tf.token.address, &1000);
        
        tf.contract_client.mark_complete(&id);
        // Expect panic: transaction validation catches overlapping status configuration modifications
        tf.contract_client.mark_complete(&id);
    }

    #[test]
    fn test_state_verification() {
        let tf = setup_test_fixture();
        let id = tf.contract_client.create_escrow(&tf.client, &tf.freelancer, &tf.token.address, &1000);
        
        let escrow_state = tf.contract_client.get_escrow(&id);
        assert_eq!(escrow_state.status, EscrowStatus::Active);
        assert_eq!(escrow_state.amount, 1000);
        assert_eq!(escrow_state.client, tf.client);
        assert_eq!(escrow_state.freelancer, tf.freelancer);
    }

    #[test]
    #[should_panic(expected = "Escrow cannot be settled from its current status state")]
    fn test_invalid_release_by_unauthorized_state() {
        let tf = setup_test_fixture();
        let id = tf.contract_client.create_escrow(&tf.client, &tf.freelancer, &tf.token.address, &1000);
        
        // Client tries processing programmatic payouts prior to milestone submission records
        tf.contract_client.refund(&id);
        tf.contract_client.release_funds(&id);
    }

    #[test]
    fn test_client_refund_recovery_flow() {
        let tf = setup_test_fixture();
        let id = tf.contract_client.create_escrow(&tf.client, &tf.freelancer, &tf.token.address, &1000);
        
        tf.contract_client.refund(&id);
        assert_eq!(tf.token.balance(&tf.client), 5000);
        assert_eq!(tf.token.balance(&tf.contract_client.address), 0);
        
        let state = tf.contract_client.get_escrow(&id);
        assert_eq!(state.status, EscrowStatus::Refunded);
    }
}