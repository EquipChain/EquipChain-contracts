//! Issue #51 — MEDIUM: `resume_continuous_flow` unauthorized flow-rate changes.
//!
//! Regression tests: only the stream provider may resume a paused stream
//! or set its flow rate; unauthenticated calls must be rejected.

#[path = "common/mod.rs"]
mod common;

use common::*;
use soroban_sdk::{testutils::Address as _, BytesN};

fn make_stream(f: &Fixture, stream_id: u64) {
    f.client.create_continuous_stream(
        &stream_id,
        &1, // meter_id
        &1_000i128,
        &10_000_000i128,
        &f.provider,
        &f.user,
        &0u32,
        &BytesN::from_array(&f.env, &[1u8; 32]),
    );
}

#[test]
fn provider_can_resume_and_set_rate() {
    let f = setup();
    let _meter = f.register_meter();
    let stream_id: u64 = 1;
    make_stream(&f, stream_id);

    // Pause then resume — provider authorized under mock auth.
    f.client.pause_continuous_flow(&stream_id);
    f.client.resume_continuous_flow(&stream_id, &500i128);

    let flow = f.client.get_continuous_flow(&stream_id).unwrap();
    assert_eq!(flow.flow_rate_per_second, 500);
}

#[test]
fn unauthenticated_caller_cannot_change_flow_rate() {
    let f = setup();
    let _meter = f.register_meter();
    let stream_id: u64 = 1;
    make_stream(&f, stream_id);

    f.client.pause_continuous_flow(&stream_id);

    f.env.set_auths(&[]);
    let result = f
        .client
        .try_resume_continuous_flow(&stream_id, &999_999_999i128);
    assert!(result.is_err(), "unauthenticated resume must be rejected");

    // Rate must remain paused (0) — attacker cannot accelerate depletion.
    let flow = f.client.get_continuous_flow(&stream_id).unwrap();
    assert_eq!(flow.flow_rate_per_second, 0);
}

#[test]
fn unauthenticated_pause_attempt_rejected() {
    let f = setup();
    let _meter = f.register_meter();
    let stream_id: u64 = 1;
    make_stream(&f, stream_id);

    f.env.set_auths(&[]);
    let result = f.client.try_pause_continuous_flow(&stream_id);
    assert!(result.is_err(), "unauthenticated pause must be rejected");
}

#[test]
fn zero_rate_rejected() {
    let f = setup();
    let _meter = f.register_meter();
    let stream_id: u64 = 1;
    make_stream(&f, stream_id);

    let result = f.client.try_resume_continuous_flow(&stream_id, &0i128);
    assert!(result.is_err(), "zero flow rate must be rejected");
}

#[test]
fn nonexistent_stream_rejected() {
    let f = setup();
    let _meter = f.register_meter();
    let result = f.client.try_resume_continuous_flow(&u64::MAX, &1_000i128);
    assert!(result.is_err());
}
