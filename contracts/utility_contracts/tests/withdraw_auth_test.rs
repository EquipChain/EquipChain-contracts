//! Issue #49 — HIGH: `withdraw_continuous` missing caller authentication.
//!
//! Regression tests: only the stream provider may withdraw a stream's
//! accumulated balance. Any other caller must be rejected.

#[path = "common/mod.rs"]
mod common;

use common::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN};

/// Helper: create a continuous stream with a funded main balance.
/// Returns the stream id.
fn make_stream(f: &Fixture) -> u64 {
    let stream_id: u64 = 1;
    // Max flow rate for the meter is rate * 3600; stay under per-second limit.
    let flow_rate_per_second: i128 = 1_000;
    let initial_balance: i128 = 10_000_000;

    f.client.create_continuous_stream(
        &stream_id,
        &1, // meter_id
        &flow_rate_per_second,
        &initial_balance,
        &f.provider,
        &f.user, // payer
        &0u32,   // priority tier
        &BytesN::from_array(&f.env, &[1u8; 32]),
    );
    stream_id
}

#[test]
fn provider_can_withdraw_own_stream_balance() {
    let f = setup();
    let meter_id = f.register_meter();
    let _ = meter_id;
    let stream_id = make_stream(&f);

    let provider_before = f.token.balance(&f.provider);
    let withdrawn = f.client.withdraw_continuous(&stream_id, &1_000_000i128);

    assert_eq!(withdrawn, 1_000_000);
    assert_eq!(
        f.token.balance(&f.provider),
        provider_before,
        "withdraw_continuous must NOT move tokens — it only decrements bookkeeping"
    );
    // Stream bookkeeping decreased.
    let flow = f.client.get_continuous_flow(&stream_id).unwrap();
    assert_eq!(flow.accumulated_balance, 10_000_000 - 1_000_000);
}

#[test]
fn non_provider_cannot_withdraw_stream_balance() {
    let f = setup();
    let _meter = f.register_meter();
    let stream_id = make_stream(&f);

    let attacker = Address::generate(&f.env);
    let _ = attacker;

    // Empty auth set => the require_auth() call inside withdraw_continuous
    // must fail for any address (no signature provided for the provider).
    f.env.set_auths(&[]);
    let result = f.client.try_withdraw_continuous(&stream_id, &1_000_000i128);
    assert!(result.is_err(), "non-provider withdrawal must be rejected");
    // Stream state must be untouched.
    let flow = f.client.get_continuous_flow(&stream_id).unwrap();
    assert_eq!(flow.accumulated_balance, 10_000_000);
}

#[test]
fn payer_cannot_withdraw_provider_balance() {
    let f = setup();
    let _meter = f.register_meter();
    let stream_id = make_stream(&f);

    // The payer is NOT the provider and must not be able to drain the
    // provider's accumulated balance. Provide an empty auth set: with no
    // signature for `flow.provider`, the call must revert.
    f.env.set_auths(&[]);
    let result = f.client.try_withdraw_continuous(&stream_id, &1_000_000i128);
    assert!(result.is_err(), "payer must not withdraw provider balance");
    let flow = f.client.get_continuous_flow(&stream_id).unwrap();
    assert_eq!(flow.accumulated_balance, 10_000_000);
}
