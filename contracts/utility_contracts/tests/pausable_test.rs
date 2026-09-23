//! Issue #24 — circuit breaker: emergency stop, auto-expiry, resume.
//! Issue #1 — contract-wide reentrancy lock on token-moving entry points.

#[path = "common/mod.rs"]
mod common;

use common::*;
use soroban_sdk::{testutils::Address as _, String};

#[test]
fn paused_contract_rejects_top_up_and_claim() {
    let f = setup();
    f.client.set_admin(&f.admin);
    let meter_id = f.register_meter();

    f.client
        .emergency_pause(&f.admin, &String::from_str(&f.env, "active exploit"));
    assert!(f.client.is_protocol_paused());

    // top_up must be blocked while paused.
    f.env.set_auths(&[]);
    let result = f.client.try_top_up(&meter_id, &1_000i128, &f.user);
    assert!(result.is_err(), "top_up must fail while paused");
}

#[test]
fn pause_requires_admin_or_compliance() {
    let f = setup();
    f.client.set_admin(&f.admin);

    let attacker = soroban_sdk::Address::generate(&f.env);
    // attacker signature alone must not be accepted: supply a MockAuth frame
    // signed only by the attacker.
    use soroban_sdk::testutils::{MockAuth, MockAuthInvoke};
    use soroban_sdk::IntoVal;
    f.env.set_auths(&[MockAuth {
        address: &attacker,
        invoke: &MockAuthInvoke {
            contract: &f.contract_id,
            fn_name: "emergency_pause",
            args: (attacker.clone(), String::from_str(&f.env, "not authorized")).into_val(&f.env),
            sub_invokes: &[],
        },
    }
    .into()]);
    let result = f
        .client
        .try_emergency_pause(&attacker, &String::from_str(&f.env, "not authorized"));
    assert!(result.is_err(), "attacker must not be able to pause");
}

#[test]
fn pause_auto_expires_after_24h() {
    let f = setup();
    f.client.set_admin(&f.admin);
    let meter_id = f.register_meter();

    f.client
        .emergency_pause(&f.admin, &String::from_str(&f.env, "test"));
    assert!(f.client.is_protocol_paused());

    // Advance past the 24h expiry window.
    f.advance(24 * 3600 + 1);

    // Paused flag auto-clears on the next gated call.
    assert!(!f.client.is_protocol_paused());
    f.top_up(meter_id, 1_000);
    assert_eq!(f.meter_balance(meter_id), 1_000);
}

#[test]
fn admin_can_resume_before_expiry() {
    let f = setup();
    f.client.set_admin(&f.admin);
    let meter_id = f.register_meter();

    f.client
        .emergency_pause(&f.admin, &String::from_str(&f.env, "test"));
    f.client.resume_after_pause(&f.admin);
    assert!(!f.client.is_protocol_paused());

    f.top_up(meter_id, 500);
    assert_eq!(f.meter_balance(meter_id), 500);
}

#[test]
fn compliance_officer_can_pause() {
    let f = setup();
    f.client.set_admin(&f.admin);

    let officer = soroban_sdk::Address::generate(&f.env);
    // set_compliance_officer now gates on the real admin slot bootstrapped
    // by set_admin (the shadow CurrentAdmin slot was removed).
    f.client.set_compliance_officer(&officer);

    f.client
        .emergency_pause(&officer, &String::from_str(&f.env, "freeze"));
    assert!(f.client.is_protocol_paused());
}

#[test]
fn claim_blocked_while_paused() {
    let f = setup();
    f.client.set_admin(&f.admin);
    let meter_id = f.register_meter();
    f.top_up(meter_id, 10_000_000);

    f.client
        .emergency_pause(&f.admin, &String::from_str(&f.env, "incident"));

    f.env.set_auths(&[]);
    let result = f.client.try_claim(&meter_id);
    assert!(result.is_err(), "claim must fail while paused");
}

#[test]
fn normal_operation_unaffected_when_not_paused() {
    let f = setup();
    f.client.set_admin(&f.admin);
    let meter_id = f.register_meter();

    f.top_up(meter_id, 10_000_000);
    f.advance(7200);
    f.client.claim(&meter_id);
    assert!(f.meter_balance(meter_id) < 10_000_000);
}
