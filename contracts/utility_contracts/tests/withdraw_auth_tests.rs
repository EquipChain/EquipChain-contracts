//! Regression tests for Issue #49: `withdraw_continuous` must require
//! authorization from the stream's provider before mutating the
//! accumulated balance.

use soroban_sdk::testutils::{Address as _, EnvTestConfig};
use soroban_sdk::{Address, BytesN, Env};
use utility_contracts::{
    ContinuousFlow, DataKey, StreamStatus, UtilityContract, UtilityContractClient,
};

/// Store a minimal continuous flow directly in contract storage.
///
/// The flow has a zero flow rate so no time-based accumulation occurs,
/// keeping the accumulated balance exactly at `balance` for assertions.
fn store_flow(
    env: &Env,
    contract_id: &Address,
    stream_id: u64,
    provider: &Address,
    payer: &Address,
    balance: i128,
) {
    let ts = env.ledger().timestamp();
    let flow = ContinuousFlow {
        stream_id,
        flow_rate_per_second: 0,
        accumulated_balance: balance,
        last_flow_timestamp: ts,
        created_timestamp: ts,
        status: StreamStatus::Active,
        paused_at: 0,
        provider: provider.clone(),
        buffer_balance: 0,
        buffer_warning_sent: false,
        payer: payer.clone(),
        priority_tier: 1,
        grid_epoch_seen: 0,
        device_mac_pubkey: BytesN::from_array(env, &[0; 32]),
        is_unreliable: false,
    };
    env.as_contract(contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::ContinuousFlow(stream_id), &flow);
    });
}

/// Create a test environment that does not write test-snapshot files.
fn test_env() -> Env {
    Env::new_with_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    })
}

/// Invoke a fallible client operation and capture whether it panicked.
///
/// The Soroban `Env` is not `UnwindSafe`, so the closure is wrapped in
/// `AssertUnwindSafe` to allow catching expected authorization panics.
fn catches_panic<F: FnOnce()>(f: F) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
}

#[test]
fn test_withdraw_continuous_unauthorized_rejected() {
    let env = test_env();
    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let payer = Address::generate(&env);
    let stream_id = 1u64;
    let initial_balance = 5000i128;

    store_flow(
        &env,
        &contract_id,
        stream_id,
        &provider,
        &payer,
        initial_balance,
    );

    // No authorization is provided, so the provider's `require_auth` must fail.
    assert!(
        catches_panic(|| {
            client.withdraw_continuous(&stream_id, &1000i128);
        }),
        "Unauthorized withdrawal should be rejected"
    );

    // Accumulated balance must remain unchanged.
    let flow = client.get_continuous_flow(&stream_id).unwrap();
    assert_eq!(flow.accumulated_balance, initial_balance);
}

#[test]
fn test_withdraw_continuous_authorized_succeeds() {
    let env = test_env();
    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let payer = Address::generate(&env);
    let stream_id = 1u64;
    let initial_balance = 5000i128;

    store_flow(
        &env,
        &contract_id,
        stream_id,
        &provider,
        &payer,
        initial_balance,
    );

    env.mock_all_auths();
    let withdrawn = client.withdraw_continuous(&stream_id, &2000i128);
    assert_eq!(withdrawn, 2000i128);

    let flow = client.get_continuous_flow(&stream_id).unwrap();
    assert_eq!(flow.accumulated_balance, initial_balance - 2000i128);
}

#[test]
fn test_withdraw_continuous_full_balance_drain_protected() {
    let env = test_env();
    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let payer = Address::generate(&env);
    let stream_id = 1u64;
    let initial_balance = 5000i128;

    store_flow(
        &env,
        &contract_id,
        stream_id,
        &provider,
        &payer,
        initial_balance,
    );

    // An unauthorized account attempts to drain the full balance.
    assert!(
        catches_panic(|| {
            client.withdraw_continuous(&stream_id, &initial_balance);
        }),
        "Full-balance drain without authorization should be rejected"
    );

    let flow = client.get_continuous_flow(&stream_id).unwrap();
    assert_eq!(flow.accumulated_balance, initial_balance);
}

#[test]
fn test_withdraw_continuous_existing_validation_preserved() {
    let env = test_env();
    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let payer = Address::generate(&env);
    let stream_id = 1u64;
    let initial_balance = 5000i128;

    store_flow(
        &env,
        &contract_id,
        stream_id,
        &provider,
        &payer,
        initial_balance,
    );

    env.mock_all_auths();

    // Authorized provider, but a zero amount must still be rejected.
    assert!(
        catches_panic(|| {
            client.withdraw_continuous(&stream_id, &0i128);
        }),
        "Zero withdrawal amount should be rejected"
    );

    // Authorized provider, but an amount exceeding the balance must be rejected.
    assert!(
        catches_panic(|| {
            client.withdraw_continuous(&stream_id, &(initial_balance + 1i128));
        }),
        "Withdrawal exceeding accumulated balance should be rejected"
    );

    let flow = client.get_continuous_flow(&stream_id).unwrap();
    assert_eq!(flow.accumulated_balance, initial_balance);
}
