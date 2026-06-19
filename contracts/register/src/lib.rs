#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String};
use scout_off_shared::{
    errors::Error,
    storage::{bump_instance, is_initialized, set_initialized},
};

#[contract]
pub struct RegisterContract;

#[contractimpl]
impl RegisterContract {
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

    /// Register a new player profile, storing the IPFS metadata URI on-chain.
    pub fn register_player(
        env: Env,
        wallet: Address,
        metadata_uri: String,
        position: String,
        region: String,
    ) -> Result<(), Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        wallet.require_auth();
        bump_instance(&env);
        // TODO: implement full registration logic (issue #197)
        let _ = (metadata_uri, position, region);
        Ok(())
    }

    /// Update an existing player's IPFS metadata URI.
    pub fn update_profile(
        env: Env,
        wallet: Address,
        metadata_uri: String,
    ) -> Result<(), Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        wallet.require_auth();
        bump_instance(&env);
        // TODO: implement profile update logic (issue #197)
        let _ = metadata_uri;
        Ok(())
    }
}

use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
}
