#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};
use scout_off_shared::{
    errors::Error,
    storage::{bump_instance, is_initialized, set_initialized},
};

#[contract]
pub struct ConnectionContract;

#[contractimpl]
impl ConnectionContract {
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

    /// Record a trial offer between a scout and player on-chain.
    pub fn log_trial_offer(
        env: Env,
        scout: Address,
        player_id: Address,
        details_uri: String,
    ) -> Result<(), Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        scout.require_auth();
        bump_instance(&env);
        // TODO: implement trial offer recording logic (issue #201)
        let _ = (player_id, details_uri);
        Ok(())
    }

    /// Retrieve the connection record between a scout and player.
    pub fn get_connection(
        env: Env,
        scout: Address,
        player_id: Address,
    ) -> Option<bool> {
        // TODO: implement connection lookup (issue #201)
        let _ = (env, scout, player_id);
        None
    }
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
}
