//! Issue #38 — CRITICAL: unauthenticated redirection of settlement payouts.
//!
//! Regression tests: set_government_vault and set_tax_rate must be
//! admin-gated; the vault must never be writable by the proposed address
//! itself, and the tax rate must be capped below 100%.

#[path = "common/mod.rs"]
mod common;

use common::*;
use soroban_sdk::testutils::Address as _;

/// Bootstrap the admin slot. set_admin requires the contract's own
/// authorization, which mock_all_auths satisfies on behalf of the fixture.
fn bootstrap_admin(f: &Fixture) {
    f.client.set_admin(&f.admin);
}

#[test]
fn admin_can_set_government_vault() {
    let f = setup();
    bootstrap_admin(&f);

    let vault = soroban_sdk::Address::generate(&f.env);
    f.client.set_government_vault(&vault);
}

#[test]
fn admin_can_set_tax_rate() {
    let f = setup();
    bootstrap_admin(&f);

    f.client.set_tax_rate(&500i128); // 5%
}

#[test]
fn unauthenticated_vault_takeover_rejected() {
    let f = setup();
    bootstrap_admin(&f);

    let attacker_vault = soroban_sdk::Address::generate(&f.env);

    // No signatures at all: the admin require_auth must fail.
    f.env.set_auths(&[]);
    let result = f.client.try_set_government_vault(&attacker_vault);
    assert!(result.is_err(), "unauthenticated vault takeover must fail");
}

#[test]
fn vault_auth_self_authorization_is_not_sufficient() {
    let f = setup();
    bootstrap_admin(&f);

    let attacker = soroban_sdk::Address::generate(&f.env);
    let attacker_vault = attacker.clone();

    // Even if a signature from the proposed vault address exists, the call
    // must fail without the admin's signature. Emulate by supplying an auth
    // set that authorizes only the attacker: the admin check fails first.
    use soroban_sdk::testutils::MockAuth;
    use soroban_sdk::testutils::MockAuthInvoke;
    f.env.set_auths(&[MockAuth {
        address: &attacker,
        invoke: &MockAuthInvoke {
            contract: &f.contract_id,
            fn_name: "set_government_vault",
            args: (attacker_vault.clone(),).into_val(&f.env),
            sub_invokes: &[],
        },
    }
    .into()]);
    let result = f.client.try_set_government_vault(&attacker_vault);
    assert!(
        result.is_err(),
        "self-authorized vault proposal must not be accepted"
    );
}

#[test]
fn unauthenticated_tax_rate_change_rejected() {
    let f = setup();
    bootstrap_admin(&f);

    f.env.set_auths(&[]);
    let result = f.client.try_set_tax_rate(&10_000i128);
    assert!(result.is_err(), "unauthenticated tax rate change must fail");
}

#[test]
fn tax_rate_above_cap_rejected_even_for_admin() {
    let f = setup();
    bootstrap_admin(&f);

    // 5,001 bps > 50% cap
    let result = f.client.try_set_tax_rate(&5_001i128);
    assert!(
        result.is_err(),
        "tax rate above the 50% cap must be rejected"
    );

    // Exactly 50% is allowed.
    f.client.set_tax_rate(&5_000i128);
}

#[test]
fn negative_tax_rate_rejected() {
    let f = setup();
    bootstrap_admin(&f);

    let result = f.client.try_set_tax_rate(&-1i128);
    assert!(result.is_err());
}

use soroban_sdk::IntoVal;
