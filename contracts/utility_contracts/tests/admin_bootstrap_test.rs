//! Admin bootstrap — the admin slot was permanently uninitializable on a
//! deployed instance because set_admin required the contract's own
//! authorization, which no external caller can satisfy. This left
//! emergency_drain and all admin-gated functions dead (flagged in #38).

#[path = "common/mod.rs"]
mod common;

use common::*;
use soroban_sdk::testutils::Address as _;

#[test]
fn admin_slot_can_be_bootstrapped_once() {
    let f = setup();

    // get_admin not exposed; verify indirectly: admin-gated call succeeds
    // after bootstrap.
    f.client.set_admin(&f.admin);
    f.client.fund_gas_bounty(&1_000_000i128);
}

#[test]
fn bootstrap_cannot_be_hijacked_twice() {
    let f = setup();
    f.client.set_admin(&f.admin);

    let attacker = soroban_sdk::Address::generate(&f.env);

    // After bootstrap, an external caller cannot rotate the admin: only the
    // contract's own authorization (governance path) may. With empty auths
    // the rotation must fail.
    f.env.set_auths(&[]);
    let result = f.client.try_set_admin(&attacker);
    assert!(
        result.is_err(),
        "external rotation after bootstrap must fail"
    );
}

#[test]
fn uninitialized_contract_rejects_admin_functions() {
    let f = setup();

    // Without bootstrap, admin functions must fail (no admin set).
    f.env.set_auths(&[]);
    let result = f.client.try_fund_gas_bounty(&1_000_000i128);
    assert!(result.is_err(), "admin fn without bootstrap must fail");
}

#[test]
fn bootstrapped_admin_can_use_admin_functions() {
    let f = setup();
    f.client.set_admin(&f.admin);

    // Mock auth satisfies require_admin_auth for f.admin.
    f.client.fund_gas_bounty(&1_000_000i128);
    f.client.approve_token(&f.token_id, &7u32);
    let tokens = f.client.get_approved_tokens();
    assert_eq!(tokens.len(), 1);
}
