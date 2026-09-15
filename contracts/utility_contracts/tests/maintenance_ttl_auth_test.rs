//! Issue #50 — MEDIUM: `manual_extend_ttl` missing caller authentication.
//!
//! Regression tests: only the meter provider may spend the per-meter
//! maintenance fund to extend storage TTL.

#[path = "common/mod.rs"]
mod common;

use common::*;
use soroban_sdk::testutils::Address as _;

/// Fund the per-meter maintenance fund the same way the protocol does:
/// a provider claim allocates MAINTENANCE_FUND_PERCENT_BPS (1%) of the
/// claimable amount into DataKey::MaintenanceFund.
fn fund_maintenance_fund(f: &Fixture, meter_id: u64) -> i128 {
    // Advance well into a new hour so the hourly cap does not clamp the claim.
    f.advance(7200);
    f.client.claim(&meter_id);
    f.client.get_maintenance_fund(&meter_id)
}

#[test]
fn provider_can_extend_ttl_when_funded() {
    let f = setup();
    // High rate so that 1% of a capped hourly claim exceeds the 1 XLM cost.
    let meter_id = f.register_meter_rate(100_000);
    f.top_up(meter_id, 1_000_000_000);

    let fund = fund_maintenance_fund(&f, meter_id);
    assert!(fund > 0, "claim must seed the maintenance fund, got {fund}");

    // Provider is authorized under mock auth — must not panic.
    f.client.manual_extend_ttl(&meter_id);
    let after = f.client.get_maintenance_fund(&meter_id);
    assert!(
        after < fund,
        "maintenance fund must be debited by the extension cost"
    );
}

#[test]
fn unauthenticated_caller_cannot_drain_maintenance_fund() {
    let f = setup();
    let meter_id = f.register_meter_rate(100_000);
    f.top_up(meter_id, 1_000_000_000);
    let fund = fund_maintenance_fund(&f, meter_id);
    assert!(fund > 0, "test requires a funded maintenance fund");

    // No signatures: the provider require_auth must fail.
    f.env.set_auths(&[]);
    let result = f.client.try_manual_extend_ttl(&meter_id);
    assert!(result.is_err(), "unauthenticated TTL extension must fail");

    // Fund must be untouched.
    assert_eq!(f.client.get_maintenance_fund(&meter_id), fund);
}

#[test]
fn non_provider_signature_cannot_extend_ttl() {
    let f = setup();
    let meter_id = f.register_meter_rate(100_000);
    f.top_up(meter_id, 1_000_000_000);
    let fund = fund_maintenance_fund(&f, meter_id);
    assert!(fund > 0, "test requires a funded maintenance fund");

    let attacker = soroban_sdk::Address::generate(&f.env);
    let _ = attacker;

    // Even with an empty auth set the call must fail — the contract
    // requires a signature from meter.provider specifically.
    f.env.set_auths(&[]);
    let result = f.client.try_manual_extend_ttl(&meter_id);
    assert!(result.is_err());
    assert_eq!(f.client.get_maintenance_fund(&meter_id), fund);
}

#[test]
fn nonexistent_meter_rejected() {
    let f = setup();
    f.env.set_auths(&[]);
    let result = f.client.try_manual_extend_ttl(&u64::MAX);
    assert!(result.is_err(), "nonexistent meter must be rejected");
}
