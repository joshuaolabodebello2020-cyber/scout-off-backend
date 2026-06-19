#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};
use scout_off_shared::{
    errors::Error,
    storage::{bump_instance, is_initialized, set_initialized},
};

#[contract]
pub struct SubscriptionContract;

#[contractimpl]
impl SubscriptionContract {
    /// One-time setup. Stores the admin address and marks the contract initialized.
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        platform_fee_bps: u32,
    ) -> Result<(), Error> {
        if is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::PlatformFeeBps, &platform_fee_bps);
        set_initialized(&env);
        bump_instance(&env);
        Ok(())
    }

    /// Purchase a scout subscription for the given tier and duration (in ledgers).
    pub fn subscribe(
        env: Env,
        scout: Address,
        tier: u32,
        duration_ledgers: u32,
    ) -> Result<(), Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        scout.require_auth();
        bump_instance(&env);
        // TODO: implement subscription payment logic (issue #200)
        let _ = (tier, duration_ledgers);
        Ok(())
    }

    /// Unlock direct contact with a player by paying the micro-fee.
    pub fn pay_to_contact(
        env: Env,
        scout: Address,
        player_id: Address,
    ) -> Result<(), Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        scout.require_auth();
        bump_instance(&env);
        // TODO: implement pay-to-contact logic (issue #200)
        let _ = player_id;
        Ok(())
    }

    /// Check whether a scout has an active subscription.
    pub fn is_subscribed(env: Env, scout: Address) -> bool {
        // TODO: implement subscription lookup (issue #200)
        let _ = scout;
        false
    }
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    PlatformFeeBps,
}
