//! Stream creation must validate that the meter exists, is active, and is
//! operated by the signing provider — and that the per-second flow rate
//! respects the meter's hourly cap.

#[path = "common/mod.rs"]
mod common;

use common::*;
use soroban_sdk::{testutils::Address as _, BytesN};

fn mac(f: &Fixture, b: u8) -> BytesN<32> {
    BytesN::from_array(&f.env, &[b; 32])
}

#[test]
fn legitimate_provider_can_create_stream() {
    let f = setup();
    let meter_id = f.register_meter();

    f.client.create_continuous_stream(
        &1u64,
        &meter_id,
        &1_000i128,
        &10_000_000i128,
        &f.provider,
        &f.user,
        &0u32,
        &mac(&f, 1),
    );
    assert!(f.client.get_continuous_flow(&1).is_some());
}

#[test]
fn wrong_provider_cannot_anchor_stream_to_foreign_meter() {
    let f = setup();
    let meter_id = f.register_meter();

    let impostor = soroban_sdk::Address::generate(&f.env);
    f.token_admin.mint(&impostor, &1_000_000_000i128);

    // Impostor signs as provider but references a meter owned by f.provider.
    // Mock auth satisfies signature checks, so the binding check is what
    // must reject this call.
    let result = f.client.try_create_continuous_stream(
        &1u64,
        &meter_id,
        &1_000i128,
        &10_000_000i128,
        &impostor,
        &f.user,
        &0u32,
        &mac(&f, 2),
    );
    assert!(result.is_err(), "foreign provider must be rejected");

    // No stream may have been created.
    assert!(f.client.get_continuous_flow(&1).is_none());
}

#[test]
fn inactive_meter_cannot_host_new_stream() {
    let f = setup();
    let meter_id = f.register_meter();

    // Provider shuts the meter down.
    f.client.emergency_shutdown(&meter_id);

    let result = f.client.try_create_continuous_stream(
        &1u64,
        &meter_id,
        &1_000i128,
        &10_000_000i128,
        &f.provider,
        &f.user,
        &0u32,
        &mac(&f, 3),
    );
    assert!(result.is_err(), "inactive meter must not host streams");
}

#[test]
fn nonexistent_meter_rejected() {
    let f = setup();

    let result = f.client.try_create_continuous_stream(
        &1u64,
        &u64::MAX,
        &1_000i128,
        &10_000_000i128,
        &f.provider,
        &f.user,
        &0u32,
        &mac(&f, 4),
    );
    assert!(result.is_err(), "nonexistent meter must be rejected");
}

#[test]
fn flow_rate_above_meter_hourly_cap_rejected() {
    let f = setup();
    let meter_id = f.register_meter_rate(1_000i128); // cap = 1_000 * 3600 / 3600 = 1_000/s

    let result = f.client.try_create_continuous_stream(
        &1u64,
        &meter_id,
        &5_000i128, // 5x the cap
        &10_000_000i128,
        &f.provider,
        &f.user,
        &0u32,
        &mac(&f, 5),
    );
    assert!(result.is_err(), "rate above the meter cap must be rejected");
}
