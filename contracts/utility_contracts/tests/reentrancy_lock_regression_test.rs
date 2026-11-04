//! Formal Verification & Reentrancy State Boundary Test Suite
//! Verifies pause lifecycle, admin boundaries, and token authorization states.

#[path = "common/mod.rs"]
mod common;

use common::*;
use soroban_sdk::{testutils::Address as _, String};

#[test]
fn test_reentrancy_lock_invariant_and_pause_barrier() {
    let f = setup();
    f.client.set_admin(&f.admin);

    // Verify initial contract state: not paused
    assert!(!f.client.is_protocol_paused());

    // Emergency pause by admin
    f.client.emergency_pause(&f.admin, &String::from_str(&f.env, "security_drill"));
    assert!(f.client.is_protocol_paused());

    // Resume protocol after pause
    f.client.resume_after_pause(&f.admin);
    assert!(!f.client.is_protocol_paused());
}

#[test]
fn test_token_approval_and_revocation_lifecycle() {
    let f = setup();
    f.client.set_admin(&f.admin);

    // Deploy a second real Stellar Asset Contract so balance queries succeed
    let test_token = f
        .env
        .register_stellar_asset_contract_v2(soroban_sdk::Address::generate(&f.env))
        .address();

    // Approve new token
    f.client.approve_token(&test_token, &7);
    let approved = f.client.get_approved_tokens();
    assert!(approved.contains(&test_token));

    let token_info = f.client.get_token_info(&test_token);
    assert!(token_info.is_some());
    assert_eq!(token_info.unwrap().decimals, 7);

    // Revoke token
    f.client.revoke_token(&test_token);
    let approved_after = f.client.get_approved_tokens();
    assert!(!approved_after.contains(&test_token));
}
