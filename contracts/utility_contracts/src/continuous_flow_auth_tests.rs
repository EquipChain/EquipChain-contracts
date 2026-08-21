#![cfg(test)]

use crate::{ContinuousFlow, DataKey, StreamStatus, UtilityContract, UtilityContractClient};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env, Symbol};

fn store_flow(env: &Env, contract_id: &Address, provider: Address, stream_id: u64) {
    let flow = ContinuousFlow {
        stream_id,
        flow_rate_per_second: 10,
        accumulated_balance: 1_000,
        last_flow_timestamp: 0,
        created_timestamp: 0,
        status: StreamStatus::Active,
        paused_at: 0,
        provider,
        buffer_balance: 1_000,
        buffer_warning_sent: false,
        payer: Address::generate(env),
        priority_tier: 0,
        grid_epoch_seen: 0,
        device_mac_pubkey: BytesN::from_array(env, &[1; 32]),
        is_unreliable: false,
    };

    env.as_contract(contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::ContinuousFlow(stream_id), &flow);
    });
}

#[test]
fn continuous_flow_rate_changes_require_provider_auth() {
    let env = Env::default();
    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);
    let provider = Address::generate(&env);
    let attacker = Address::generate(&env);
    let stream_id = 1;
    store_flow(&env, &contract_id, provider.clone(), stream_id);

    env.mock_auths(&[(&attacker, &Symbol::new(&env, "update_continuous_flow_rate"))]);
    let unauthorized_update = std::panic::catch_unwind(|| {
        client.update_continuous_flow_rate(&stream_id, &20);
    });
    assert!(unauthorized_update.is_err());
    assert_eq!(
        client
            .get_continuous_flow(&stream_id)
            .expect("flow should exist")
            .flow_rate_per_second,
        10
    );

    env.mock_auths(&[(&attacker, &Symbol::new(&env, "pause_continuous_flow"))]);
    let unauthorized_pause = std::panic::catch_unwind(|| {
        client.pause_continuous_flow(&stream_id);
    });
    assert!(unauthorized_pause.is_err());
    assert_eq!(
        client
            .get_continuous_flow(&stream_id)
            .expect("flow should exist")
            .status,
        StreamStatus::Active
    );

    env.mock_auths(&[(&provider, &Symbol::new(&env, "update_continuous_flow_rate"))]);
    client.update_continuous_flow_rate(&stream_id, &20);
    assert_eq!(
        client
            .get_continuous_flow(&stream_id)
            .expect("flow should exist")
            .flow_rate_per_second,
        20
    );

    env.mock_auths(&[(&provider, &Symbol::new(&env, "pause_continuous_flow"))]);
    client.pause_continuous_flow(&stream_id);
    let paused = client
        .get_continuous_flow(&stream_id)
        .expect("flow should exist");
    assert_eq!(paused.flow_rate_per_second, 0);
    assert_eq!(paused.status, StreamStatus::Paused);
}
