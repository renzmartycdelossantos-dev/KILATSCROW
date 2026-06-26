#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    Active = 0,
    WorkCompleted = 1,
    Settled = 2,
    Refunded = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub client: Address,
    pub freelancer: Address,
    pub token: Address,
    pub amount: i128,
    pub status: EscrowStatus,
}

#[contracttype]
pub enum DataKey {
    Escrow(u64),
    NextEscrowId,
}

#[contract]
pub struct FlowEscrowContract;

#[contractimpl]
impl FlowEscrowContract {
    /// Creates a new milestone escrow agreement, increments the global tracker ID, and locks funds.
    pub fn create_escrow(
        env: Env,
        client: Address,
        freelancer: Address,
        token: Address,
        amount: i128,
    ) -> u64 {
        client.require_auth();
        if amount <= 0 {
            panic!("Escrow amount must be positive");
        }

        // Fetch and increment our global incremental counter key
        let mut next_id: u64 = env.storage().instance().get(&DataKey::NextEscrowId).unwrap_or(1);
        
        let new_escrow = Escrow {
            client: client.clone(),
            freelancer,
            token: token.clone(),
            amount,
            status: EscrowStatus::Active,
        };

        // Put escrow contract record into persistent instance state metadata layout
        env.storage().instance().set(&DataKey::Escrow(next_id), &new_escrow);
        env.storage().instance().set(&DataKey::NextEscrowId, &(next_id + 1));

        // Pull payment down into the contract space context natively
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&client, &env.current_contract_address(), &amount);

        // Emit trace tracking events layout patterns
        env.events().publish((Symbol::new(&env, "escrow_created"), next_id), amount);

        next_id
    }

    /// Fetches the structural details of a specific escrow deployment instance context records.
    pub fn get_escrow(env: Env, escrow_id: u64) -> Escrow {
        env.storage().instance().get(&DataKey::Escrow(escrow_id)).expect("Escrow does not exist")
    }

    /// Executed solely by the Freelancer to signal programmatic completion of work milestones.
    pub fn mark_complete(env: Env, escrow_id: u64) {
        let mut escrow = Self::get_escrow(env.clone(), escrow_id);
        escrow.freelancer.require_auth();

        if escrow.status != EscrowStatus::Active {
            panic!("Escrow is not active");
        }

        escrow.status = EscrowStatus::WorkCompleted;
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);
    }

    /// Executed by the Client to release the escrowed USDC stablecoins out to the freelancer destination.
    pub fn release_funds(env: Env, escrow_id: u64) {
        let mut escrow = Self::get_escrow(env.clone(), escrow_id);
        escrow.client.require_auth();

        if escrow.status != EscrowStatus::WorkCompleted && escrow.status != EscrowStatus::Active {
            panic!("Escrow cannot be settled from its current status state");
        }

        escrow.status = EscrowStatus::Settled;
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(&env.current_contract_address(), &escrow.freelancer, &escrow.amount);
    }

    /// Executed by the Client to perform asset recovery fallback procedures if conditions aren't met.
    pub fn refund(env: Env, escrow_id: u64) {
        let mut escrow = Self::get_escrow(env.clone(), escrow_id);
        escrow.client.require_auth();

        if escrow.status == EscrowStatus::Settled || escrow.status == EscrowStatus::Refunded {
            panic!("Funds already moved out of escrow");
        }

        escrow.status = EscrowStatus::Refunded;
        env.storage().instance().set(&DataKey::Escrow(escrow_id), &escrow);

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(&env.current_contract_address(), &escrow.client, &escrow.amount);
    }
}