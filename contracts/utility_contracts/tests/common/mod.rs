//! Shared test utilities for security hardening test suites.
//!
//! Deploys the real contract, wires a real SAC token,
//! registers meters and streams, and provides assertion helpers.
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient};
use soroban_sdk::{Address, BytesN, Env};

/// Deterministic starting timestamp for all tests (2026-01-01 UTC).
pub const START_TS: u64 = 1_767_225_600;

pub struct Fixture {
    pub env: Env,
    pub contract_id: Address,
    pub client: utility_contracts::UtilityContractClient<'static>,
    pub token_id: Address,
    pub token: TokenClient<'static>,
    pub token_admin: StellarAssetClient<'static>,
    pub admin: Address,
    pub provider: Address,
    pub user: Address,
}

/// Build the full fixture. Returns static-lifetime clients via leaking.
pub fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.timestamp = START_TS;
        li.protocol_version = 22;
        li.sequence_number = 10;
    });

    // Deploy a real Stellar Asset Contract with a mintable admin.
    let token_id = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();

    let contract_id = env.register(utility_contracts::UtilityContract, ());
    let client = utility_contracts::UtilityContractClient::new(&env, &contract_id);
    let token = TokenClient::new(&env, &token_id);
    let token_admin = StellarAssetClient::new(&env, &token_id);

    let admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let user_addr = Address::generate(&env);

    // Mint starting balances to participants so transfers can succeed.
    token_admin.mint(&user_addr, &10_000_000_000_000i128);
    token_admin.mint(&provider, &10_000_000_000_000i128);
    token_admin.mint(&admin, &10_000_000_000_000i128);

    // Fund the contract so provider payouts can be made from the pool.
    token.transfer(&user_addr, &contract_id, &5_000_000_000_000i128);

    Fixture {
        env,
        contract_id: contract_id.clone(),
        client: unsafe { std::mem::transmute(client) },
        token_id,
        token: unsafe { std::mem::transmute(token) },
        token_admin: unsafe { std::mem::transmute(token_admin) },
        admin,
        provider,
        user: user_addr,
    }
}

impl Fixture {
    pub fn advance(&self, secs: u64) {
        self.env.ledger().with_mut(|li| {
            li.timestamp += secs;
            li.sequence_number += 1;
        });
    }

    /// Register a default prepaid meter and return its id.
    pub fn register_meter(&self) -> u64 {
        self.register_meter_rate(1_000i128)
    }

    pub fn register_meter_rate(&self, rate: i128) -> u64 {
        self.client.register_meter(
            &self.user,
            &self.provider,
            &rate,
            &self.token_id,
            &BytesN::from_array(&self.env, &[1u8; 32]),
            &0u32,
        )
    }

    /// Top up the meter with `amount` units of balance funded by `self.user`.
    pub fn top_up(&self, meter_id: u64, amount: i128) {
        self.client.top_up(&meter_id, &amount, &self.user);
    }

    pub fn meter_balance(&self, meter_id: u64) -> i128 {
        self.client.get_meter(&meter_id).unwrap().balance
    }
}

/// Panic helper: assert that the closure panics.
#[track_caller]
pub fn assert_panics<T>(f: impl FnOnce() -> T) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    assert!(result.is_err(), "expected panic, but call succeeded");
}
