//! Provider daily withdrawal window: at most 10% of the provider's total
//! pool may be withdrawn per 24h window across deduct_units settlement and
//! withdraw_earnings.

#[path = "common/mod.rs"]
mod common;

use common::*;

#[test]
fn withdraw_earnings_within_cap_succeeds() {
    let f = setup();
    let meter_id = f.register_meter();
    f.top_up(meter_id, 10_000_000);

    // Pool = 10_000_000 after top_up; 10% cap = 1_000_000.
    f.client.withdraw_earnings(&meter_id, &900_000i128);
    assert_eq!(f.meter_balance(meter_id), 10_000_000 - 900_000);
}

#[test]
fn withdraw_earnings_above_daily_cap_rejected() {
    let f = setup();
    let meter_id = f.register_meter();
    f.top_up(meter_id, 10_000_000);

    // 2_000_000 > 10% of 10_000_000 pool.
    f.env.set_auths(&[]);
    let result = f.client.try_withdraw_earnings(&meter_id, &2_000_000i128);
    assert!(
        result.is_err(),
        "withdrawal above daily cap must be rejected"
    );
    // Balance untouched.
    assert_eq!(f.meter_balance(meter_id), 10_000_000);
}

#[test]
fn daily_cap_is_cumulative_across_calls() {
    let f = setup();
    let meter_id = f.register_meter();
    f.top_up(meter_id, 10_000_000);

    // First withdrawal: 600k (under 1M cap).
    f.client.withdraw_earnings(&meter_id, &600_000i128);

    // Second same-day withdrawal of 500k would push the total to 1.1M > cap.
    f.env.set_auths(&[]);
    let result = f.client.try_withdraw_earnings(&meter_id, &500_000i128);
    assert!(
        result.is_err(),
        "cumulative daily withdrawal must be capped"
    );
}

#[test]
fn window_resets_after_24h() {
    let f = setup();
    let meter_id = f.register_meter();
    f.top_up(meter_id, 10_000_000);

    f.client.withdraw_earnings(&meter_id, &900_000i128);

    // Advance past the 24h window: cap resets.
    f.advance(24 * 3600 + 1);
    f.client.withdraw_earnings(&meter_id, &900_000i128);
    assert_eq!(f.meter_balance(meter_id), 10_000_000 - 1_800_000);
}
