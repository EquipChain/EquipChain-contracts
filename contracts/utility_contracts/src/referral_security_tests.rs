#![cfg(test)]

use crate::{UtilityContract, UtilityContractClient, REFERRAL_REWARD_UNITS};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, BytesN, Env};

#[test]
fn referral_reward_does_not_inflate_claimable_meter_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let provider = Address::generate(&env);
    let token = Address::generate(&env);
    let referrer = Address::generate(&env);
    let device_public_key = BytesN::from_array(&env, &[1; 32]);

    let meter_id = client.register_with_referral(
        &user,
        &provider,
        &10,
        &token,
        &device_public_key,
        &referrer,
        &0,
    );

    let meter = client.get_meter(&meter_id).expect("meter should exist");
    assert_eq!(meter.balance, 0, "referral registration must not mint funds");
    assert_eq!(
        client.get_referral_reward(&meter_id),
        REFERRAL_REWARD_UNITS
    );
}
