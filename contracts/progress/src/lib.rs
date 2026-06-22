#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};
use scout_off_shared::{
    errors::Error,
    storage::{bump_instance, is_initialized, set_initialized},
};

#[contract]
pub struct ProgressContract;

#[contractimpl]
impl ProgressContract {
    /// One-time setup. Stores the admin address and marks the contract initialized.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        set_initialized(&env);
        bump_instance(&env);
        Ok(())
    }

    /// Submit a new milestone for a player (validator auth required).
    pub fn submit_milestone(
        env: Env,
        validator: Address,
        player_id: Address,
        milestone_type: String,
        evidence_uri: String,
    ) -> Result<(), Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        validator.require_auth();
        bump_instance(&env);
        // TODO: implement milestone submission logic (issue #198)
        let _ = (player_id, milestone_type, evidence_uri);
        Ok(())
    }

    /// Approve a pending milestone, incrementing the player's progress tier.
    pub fn approve_milestone(
        env: Env,
        validator: Address,
        milestone_id: u64,
    ) -> Result<(), Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        validator.require_auth();
        bump_instance(&env);
        // TODO: implement milestone approval logic (issue #198)
        let _ = milestone_id;
        Ok(())
    }
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
}
