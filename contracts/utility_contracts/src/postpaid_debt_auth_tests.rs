#[cfg(test)]
mod postpaid_debt_auth_tests {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Symbol};

    fn setup_postpaid_meter(
        env: &Env,
        client: &UtilityContractClient,
        user: &Address,
        provider: &Address,
        token: &Address,
    ) -> u64 {
        let device_public_key = BytesN::from_array(env, &[1u8; 32]);
        env.mock_all_auths();
        client.register_meter_with_mode(
            user,
            provider,
            &1000,
            token,
            &BillingType::PostPaid,
            &device_public_key,
        )
    }

    fn lock_deposit(
        env: &Env,
        client: &UtilityContractClient,
        owner: &Address,
        token: &Address,
        amount: i128,
    ) {
        env.mock_all_auths();
        let token_admin_client = soroban_sdk::token::StellarAssetClient::new(env, token);
        token_admin_client.mint(owner, &amount);
        client.lock_guarantor_deposit(owner, token, &amount);
    }

    #[test]
    fn test_accrue_postpaid_debt_requires_provider_auth() {
        let env = Env::default();
        let contract_id = env.register(UtilityContract, ());
        let client = UtilityContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let provider = Address::generate(&env);
        let attacker = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let token = env
            .register_stellar_asset_contract_v2(token_admin.clone())
            .address();

        let meter_id = setup_postpaid_meter(&env, &client, &user, &provider, &token);
        lock_deposit(&env, &client, &user, &token, 10_000);

        // An unrelated caller must not be able to accrue debt against the user's deposit.
        env.mock_auths(&[(&attacker, &Symbol::new(&env, "accrue_postpaid_debt"))]);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.accrue_postpaid_debt(&meter_id, &1_000);
        }));
        assert!(result.is_err(), "unauthorized caller must not accrue debt");

        // The meter's own provider is authorized to accrue debt.
        env.mock_auths(&[(&provider, &Symbol::new(&env, "accrue_postpaid_debt"))]);
        client.accrue_postpaid_debt(&meter_id, &1_000);

        let deposit = client.get_guarantor_deposit(&user).unwrap();
        assert_eq!(deposit.accrued_debt, 1_000);
    }

    #[test]
    fn test_accrue_postpaid_debt_resolves_provider_by_meter_id() {
        let env = Env::default();
        let contract_id = env.register(UtilityContract, ());
        let client = UtilityContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let provider_a = Address::generate(&env);
        let provider_b = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let token = env
            .register_stellar_asset_contract_v2(token_admin.clone())
            .address();

        // Same user has two active post-paid meters from two different providers.
        let meter_a = setup_postpaid_meter(&env, &client, &user, &provider_a, &token);
        let meter_b = setup_postpaid_meter(&env, &client, &user, &provider_b, &token);
        lock_deposit(&env, &client, &user, &token, 10_000);

        // provider_a must not be able to accrue debt via meter_b's id.
        env.mock_auths(&[(&provider_a, &Symbol::new(&env, "accrue_postpaid_debt"))]);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.accrue_postpaid_debt(&meter_b, &500);
        }));
        assert!(
            result.is_err(),
            "provider_a must not authorize debt on provider_b's meter"
        );

        // provider_b, the meter's actual provider, is authorized.
        env.mock_auths(&[(&provider_b, &Symbol::new(&env, "accrue_postpaid_debt"))]);
        client.accrue_postpaid_debt(&meter_b, &500);

        let deposit = client.get_guarantor_deposit(&user).unwrap();
        assert_eq!(deposit.accrued_debt, 500);
        let _ = meter_a;
    }
}
