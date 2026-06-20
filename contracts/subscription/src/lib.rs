#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};
use scout_off_shared::{
    errors::Error,
    storage::{bump_instance, is_initialized, set_initialized},
};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub struct SubscriptionRecord {
    pub tier: u32,
    pub expiry_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    PlatformFeeBps,
    Subscription(Address),
    ContactUnlock(Address, u64),
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct SubscriptionContract;

#[contractimpl]
impl SubscriptionContract {
    /// One-time setup. Stores admin, payment token, and platform contact fee.
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
    ///
    /// Required payment = tier × duration_ledgers × platform_fee_bps.
    /// Returns `InsufficientFee(7)` when the scout's balance is too low,
    /// or `Overflow(11)` when cost computation overflows i128.
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

        let unit_cost: i128 = env
            .storage()
            .instance()
            .get::<DataKey, u32>(&DataKey::PlatformFeeBps)
            .unwrap_or(0) as i128;

        // Overflow-safe cost: tier × duration × unit_cost.
        let required: i128 = (tier as i128)
            .checked_mul(duration_ledgers as i128)
            .and_then(|v| v.checked_mul(unit_cost))
            .ok_or(Error::Overflow)?;

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(Error::NotInitialized)?;
        let token_client = token::Client::new(&env, &token_addr);

        if token_client.balance(&scout) < required {
            return Err(Error::InsufficientFee);
        }

        token_client.transfer(&scout, &env.current_contract_address(), &required);

        let expiry_ledger = env.ledger().sequence() + duration_ledgers;
        env.storage().instance().set(
            &DataKey::Subscription(scout.clone()),
            &SubscriptionRecord { tier, expiry_ledger },
        );

        env.events().publish(
            (Symbol::new(&env, "scout_subscribed"), scout.clone()),
            (tier, duration_ledgers, expiry_ledger),
        );

        bump_instance(&env);
        Ok(())
    }

    /// Unlock direct contact with a player.
    ///
    /// Requires an active subscription. Transfers the platform contact fee from
    /// the scout to the contract. Returns `Unauthorized(9)` when the subscription
    /// is missing or expired, `InsufficientFee(7)` on low balance, or
    /// `Overflow(11)` on arithmetic overflow.
    pub fn pay_to_contact(env: Env, scout: Address, player_id: u64) -> Result<(), Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        scout.require_auth();

        // Subscription gate: subscription must exist and be unexpired.
        let sub: SubscriptionRecord = env
            .storage()
            .instance()
            .get(&DataKey::Subscription(scout.clone()))
            .ok_or(Error::Unauthorized)?;
        if env.ledger().sequence() >= sub.expiry_ledger {
            return Err(Error::Unauthorized);
        }

        // Read contact fee; u32 → i128 cast is always non-negative.
        let contact_fee: i128 = env
            .storage()
            .instance()
            .get::<DataKey, u32>(&DataKey::PlatformFeeBps)
            .unwrap_or(0) as i128;

        // Defensive overflow guard (future-proofs if the type ever changes).
        if contact_fee < 0 {
            return Err(Error::Overflow);
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(Error::NotInitialized)?;
        let token_client = token::Client::new(&env, &token_addr);

        if token_client.balance(&scout) < contact_fee {
            return Err(Error::InsufficientFee);
        }

        token_client.transfer(&scout, &env.current_contract_address(), &contact_fee);

        env.storage()
            .instance()
            .set(&DataKey::ContactUnlock(scout.clone(), player_id), &true);

        env.events().publish(
            (Symbol::new(&env, "contact_unlocked"), scout.clone()),
            (player_id,),
        );

        bump_instance(&env);
        Ok(())
    }

    /// Return true if the scout has an active (non-expired) subscription.
    pub fn is_subscribed(env: Env, scout: Address) -> bool {
        let sub: SubscriptionRecord =
            match env.storage().instance().get(&DataKey::Subscription(scout)) {
                Some(s) => s,
                None => return false,
            };
        env.ledger().sequence() < sub.expiry_ledger
    }

    /// Return the current platform contact fee in token base units.
    pub fn get_contact_fee(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::PlatformFeeBps)
            .unwrap_or(0)
    }

    /// Transfer all accumulated platform fees to `recipient`. Admin-only.
    ///
    /// The caller passes `admin` explicitly; the contract checks it equals the
    /// stored admin address before proceeding. This allows unauthorized access
    /// to be detected (returning `Unauthorized(9)`) even under `mock_all_auths`.
    pub fn withdraw_fees(
        env: Env,
        admin: Address,
        recipient: Address,
    ) -> Result<(), Error> {
        if !is_initialized(&env) {
            return Err(Error::NotInitialized);
        }
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::Unauthorized)?;

        // Address equality is checked before require_auth so that impostor
        // addresses fail deterministically in tests (mock_all_auths would
        // otherwise make any require_auth pass).
        if admin != stored_admin {
            return Err(Error::Unauthorized);
        }
        admin.require_auth();

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(Error::NotInitialized)?;
        let token_client = token::Client::new(&env, &token_addr);
        let balance = token_client.balance(&env.current_contract_address());

        if balance > 0 {
            token_client.transfer(&env.current_contract_address(), &recipient, &balance);
        }

        env.events().publish(
            (Symbol::new(&env, "fees_withdrawn"), recipient.clone()),
            (balance,),
        );

        bump_instance(&env);
        Ok(())
    }

    /// Return true if the scout has already unlocked this player via pay_to_contact.
    pub fn has_paid_contact(env: Env, scout: Address, player_id: u64) -> bool {
        env.storage()
            .instance()
            .has(&DataKey::ContactUnlock(scout, player_id))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger as _},
        token::{Client as TokenClient, StellarAssetClient},
        Env,
    };

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Deploy subscription contract + stellar-asset token, initialize both.
    /// Base fee = 100; subscription cost = tier × duration × 100.
    fn setup(env: &Env) -> (SubscriptionContractClient<'_>, Address, Address) {
        env.mock_all_auths();
        let admin = Address::generate(env);
        let token_addr = env.register_stellar_asset_contract(admin.clone());
        let contract_id = env.register_contract(None, SubscriptionContract);
        let client = SubscriptionContractClient::new(env, &contract_id);
        client.initialize(&admin, &token_addr, &100u32);
        (client, admin, token_addr)
    }

    fn mint(env: &Env, token: &Address, admin: &Address, to: &Address, amount: i128) {
        StellarAssetClient::new(env, token).mint(to, &amount);
    }

    // -----------------------------------------------------------------------
    // subscribe
    // -----------------------------------------------------------------------

    #[test]
    fn subscribe_with_exact_payment_succeeds() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        // tier=1, duration=5 → cost = 1 × 5 × 100 = 500
        mint(&env, &token, &admin, &scout, 500);
        client.subscribe(&scout, &1u32, &5u32);
        assert!(client.is_subscribed(&scout));
    }

    #[test]
    fn subscribe_deducts_exact_amount_leaving_surplus() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        // cost = 2 × 3 × 100 = 600; give 1000
        mint(&env, &token, &admin, &scout, 1_000);
        client.subscribe(&scout, &2u32, &3u32);
        assert!(client.is_subscribed(&scout));
        assert_eq!(TokenClient::new(&env, &token).balance(&scout), 400);
    }

    #[test]
    fn subscribe_insufficient_fee_returns_error() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        // cost = 500, give 499
        mint(&env, &token, &admin, &scout, 499);
        let result = client.try_subscribe(&scout, &1u32, &5u32);
        assert!(matches!(result, Err(Ok(Error::InsufficientFee))));
    }

    #[test]
    fn subscribe_zero_balance_returns_insufficient_fee() {
        let env = Env::default();
        let (client, _admin, _token) = setup(&env);
        let scout = Address::generate(&env);
        let result = client.try_subscribe(&scout, &1u32, &10u32);
        assert!(matches!(result, Err(Ok(Error::InsufficientFee))));
    }

    #[test]
    fn subscribe_overflow_returns_error() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        mint(&env, &token, &admin, &scout, i128::MAX);
        // tier=u32::MAX, duration=u32::MAX → tier × duration overflows i128
        let result = client.try_subscribe(&scout, &u32::MAX, &u32::MAX);
        assert!(matches!(result, Err(Ok(Error::Overflow))));
    }

    // -----------------------------------------------------------------------
    // is_subscribed / expiry window
    // -----------------------------------------------------------------------

    #[test]
    fn is_subscribed_true_within_duration() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        mint(&env, &token, &admin, &scout, 500);
        // Starts at sequence 0; expires at 5.
        client.subscribe(&scout, &1u32, &5u32);
        assert!(client.is_subscribed(&scout));

        env.ledger().with_mut(|li| li.sequence_number = 4);
        assert!(client.is_subscribed(&scout));
    }

    #[test]
    fn is_subscribed_false_at_exact_expiry() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        mint(&env, &token, &admin, &scout, 500);
        client.subscribe(&scout, &1u32, &5u32); // expiry_ledger = 5

        // At the expiry ledger the subscription has lapsed (sequence < expiry fails).
        env.ledger().with_mut(|li| li.sequence_number = 5);
        assert!(!client.is_subscribed(&scout));
    }

    #[test]
    fn is_subscribed_false_after_expiry() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        mint(&env, &token, &admin, &scout, 500);
        client.subscribe(&scout, &1u32, &5u32);

        env.ledger().with_mut(|li| li.sequence_number = 100);
        assert!(!client.is_subscribed(&scout));
    }

    #[test]
    fn is_subscribed_false_with_no_subscription() {
        let env = Env::default();
        let (client, _admin, _token) = setup(&env);
        assert!(!client.is_subscribed(&Address::generate(&env)));
    }

    // -----------------------------------------------------------------------
    // pay_to_contact
    // -----------------------------------------------------------------------

    #[test]
    fn pay_to_contact_succeeds_and_records_unlock() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        // sub cost=1000, contact fee=100 → need 1100 total
        mint(&env, &token, &admin, &scout, 1_100);
        client.subscribe(&scout, &1u32, &10u32);
        client.pay_to_contact(&scout, &42u64);

        assert!(client.has_paid_contact(&scout, &42u64));
        assert_eq!(TokenClient::new(&env, &token).balance(&scout), 0);
    }

    #[test]
    fn pay_to_contact_without_subscription_is_unauthorized() {
        let env = Env::default();
        let (client, _admin, _token) = setup(&env);
        let scout = Address::generate(&env);
        let result = client.try_pay_to_contact(&scout, &1u64);
        assert!(matches!(result, Err(Ok(Error::Unauthorized))));
    }

    #[test]
    fn pay_to_contact_with_expired_subscription_is_unauthorized() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        mint(&env, &token, &admin, &scout, 500);
        client.subscribe(&scout, &1u32, &5u32); // expiry = 5

        env.ledger().with_mut(|li| li.sequence_number = 5);
        let result = client.try_pay_to_contact(&scout, &1u64);
        assert!(matches!(result, Err(Ok(Error::Unauthorized))));
    }

    #[test]
    fn pay_to_contact_insufficient_fee_returns_error() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        // Exact cost for subscription; zero left for contact fee.
        mint(&env, &token, &admin, &scout, 1_000);
        client.subscribe(&scout, &1u32, &10u32);
        let result = client.try_pay_to_contact(&scout, &1u64);
        assert!(matches!(result, Err(Ok(Error::InsufficientFee))));
    }

    // -----------------------------------------------------------------------
    // get_contact_fee
    // -----------------------------------------------------------------------

    #[test]
    fn get_contact_fee_returns_stored_value() {
        let env = Env::default();
        let (client, _admin, _token) = setup(&env);
        assert_eq!(client.get_contact_fee(), 100u32);
    }

    // -----------------------------------------------------------------------
    // withdraw_fees
    // -----------------------------------------------------------------------

    #[test]
    fn withdraw_fees_transfers_accumulated_balance_to_recipient() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        let scout = Address::generate(&env);
        // sub=1000, contact=100 → contract accumulates 1100
        mint(&env, &token, &admin, &scout, 1_100);
        client.subscribe(&scout, &1u32, &10u32);
        client.pay_to_contact(&scout, &1u64);

        let recipient = Address::generate(&env);
        client.withdraw_fees(&admin, &recipient);

        assert_eq!(TokenClient::new(&env, &token).balance(&recipient), 1_100);
    }

    #[test]
    fn withdraw_fees_non_admin_returns_unauthorized() {
        let env = Env::default();
        let (client, _admin, _token) = setup(&env);
        let impostor = Address::generate(&env);
        let recipient = Address::generate(&env);
        // Address equality check fires before require_auth, so this fails
        // even with mock_all_auths active.
        let result = client.try_withdraw_fees(&impostor, &recipient);
        assert!(matches!(result, Err(Ok(Error::Unauthorized))));
    }

    #[test]
    fn withdraw_fees_zero_balance_is_noop() {
        let env = Env::default();
        let (client, admin, _token) = setup(&env);
        // No fees accumulated; should complete without panicking.
        client.withdraw_fees(&admin, &Address::generate(&env));
    }

    // -----------------------------------------------------------------------
    // Init guards
    // -----------------------------------------------------------------------

    #[test]
    fn double_initialize_fails() {
        let env = Env::default();
        let (client, admin, token) = setup(&env);
        assert!(client.try_initialize(&admin, &token, &100u32).is_err());
    }

    #[test]
    fn subscribe_before_initialize_fails() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, SubscriptionContract);
        let client = SubscriptionContractClient::new(&env, &id);
        assert!(client
            .try_subscribe(&Address::generate(&env), &1u32, &5u32)
            .is_err());
    }
}
