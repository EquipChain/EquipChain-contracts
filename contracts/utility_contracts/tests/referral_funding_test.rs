//! Issue #44 — register_with_referral inflates meter.balance without a
//! token deposit.
//!
//! Regression tests: referral rewards must NOT be added to the withdrawable
//! meter balance as unfunded credit; they must be tracked separately and
//! only become spendable when real deposits back them.

#[path = "common/mod.rs"]
mod common;

use common::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

fn pk(env: &Env, b: u8) -> soroban_sdk::BytesN<32> {
    soroban_sdk::BytesN::from_array(env, &[b; 32])
}

fn pk_u32(env: &Env, v: u32) -> soroban_sdk::BytesN<32> {
    soroban_sdk::BytesN::from_array(env, &[(v % 251 + 1) as u8; 32])
}

#[test]
fn referral_reward_is_not_instant_spendable_balance() {
    let f = setup();
    let meter_id = f.register_meter();

    let referrer = soroban_sdk::Address::generate(&f.env);
    f.token_admin.mint(&referrer, &1_000_000_000i128);

    let balance_before = f.meter_balance(meter_id);

    let new_meter = f.client.register_with_referral(
        &f.user,
        &f.provider,
        &1_000i128,
        &f.token_id,
        &pk(&f.env, 2),
        &referrer,
        &0u32,
    );

    // The NEW meter's balance must not include an unfunded +10 credit.
    assert_eq!(
        f.meter_balance(new_meter),
        balance_before,
        "referral reward must not inflate meter balance without a deposit"
    );
}

#[test]
fn reward_applies_only_when_backed_by_deposit() {
    let f = setup();
    let referrer = soroban_sdk::Address::generate(&f.env);
    f.token_admin.mint(&referrer, &1_000_000_000i128);

    // First registration earns a pending reward.
    let m1 = f.client.register_with_referral(
        &f.user,
        &f.provider,
        &1_000i128,
        &f.token_id,
        &pk(&f.env, 3),
        &referrer,
        &0u32,
    );

    // Deposit 100 units: only 10 (the pending reward) may be credited as
    // bonus — the remaining 90 is the user's own money. The key invariant:
    // total balance credited (100 deposit + 10 reward) is fully backed by
    // the 100 tokens that were transferred in, because the reward bucket
    // only releases up to the deposit amount.
    f.top_up(m1, 100);
    let balance = f.meter_balance(m1);

    // Balance = deposit + reward conversion, but never more than 2x deposit.
    assert!(
        (100..=200).contains(&balance),
        "balance must stay within deposit + bounded reward, got {balance}"
    );

    // A second deposit of 100 must credit exactly 100 more (reward exhausted).
    let before = f.meter_balance(m1);
    f.top_up(m1, 100);
    assert_eq!(f.meter_balance(m1), before + 100);
}

#[test]
fn self_referral_earns_nothing() {
    let f = setup();

    let m1 = f.client.register_with_referral(
        &f.user,
        &f.provider,
        &1_000i128,
        &f.token_id,
        &pk(&f.env, 4),
        &f.user, // self-referral
        &0u32,
    );

    f.top_up(m1, 100);
    assert_eq!(
        f.meter_balance(m1),
        100,
        "self-referral must not produce any reward"
    );
}

#[test]
fn attacker_cannot_farm_rewards_without_deposits() {
    let f = setup();
    let referrer = soroban_sdk::Address::generate(&f.env);
    f.token_admin.mint(&referrer, &1_000_000_000i128);

    // Farm 50 registrations, each earning a pending reward.
    for i in 0..50u32 {
        let _m = f.client.register_with_referral(
            &f.user,
            &f.provider,
            &1_000i128,
            &f.token_id,
            &pk_u32(&f.env, i),
            &referrer,
            &0u32,
        );
    }

    // Every meter that never received a deposit must have zero claimable
    // balance — farmed rewards stay locked in the pending bucket and never
    // reach the shared token pool.
    let count = f.client.get_count();
    for id in 1..=count {
        assert_eq!(
            f.meter_balance(id),
            0,
            "meter {id} has unfunded balance — reward farming is possible"
        );
    }
}
