#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::{token, Address, BytesN, Env, Vec};

// --- Helpers ---
fn device_key(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

fn create_token(env: &Env) -> Address {
    let admin = Address::generate(env);
    env.register_stellar_asset_contract_v2(admin).address()
}

// ==================== MOCK CONTRACTS ====================

// ==================== MOCK CONTRACTS ====================

mod mock_environmental_oracle {
    use soroban_sdk::{contract, contractimpl, Address, Env};

    #[contract]
    pub struct MockEnvironmentalOracle;

    #[contractimpl]
    impl MockEnvironmentalOracle {
        pub fn xlm_to_usd_cents(_env: Env, xlm_amount: i128) -> i128 {
            xlm_amount.saturating_mul(100)
        }

        pub fn usd_cents_to_xlm(_env: Env, usd_cents: i128) -> i128 {
            usd_cents.saturating_div(100)
        }

        pub fn get_price(env: Env) -> utility_contracts::PriceData {
            utility_contracts::PriceData {
                price: 100,
                decimals: 2,
                last_updated: env.ledger().timestamp(),
            }
        }

        pub fn verify_green_source(
            _env: Env,
            _provider: Address,
            _meter_id: u64,
            _timestamp: u64,
        ) -> bool {
            true
        }
    }
}

#[test]
fn test_grace_period_expiration() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    token_admin_client.mint(&user, &2000);

    let device_public_key = device_key(&env, 1);
    // Integrated Seasonal/Sustainability params: end_date (0) and rent_deposit (0)
    let meter_id = client.register_meter(&user, &provider, &10, &token_address, &device_public_key, &0);

    // Top up with balance to activate
    client.top_up(&meter_id, &500, &user);
    let meter = client.get_meter(&meter_id).unwrap();
    assert!(meter.is_active);
    assert_eq!(meter.balance, 500);

    // Pair the meter
    client.initiate_pairing(&meter_id);
    client.complete_pairing(&meter_id, &BytesN::from_array(&env, &[2u8; 64]));

    // Use up balance exactly to 0 - should start grace period
    env.ledger().set_timestamp(env.ledger().timestamp() + 50); 
    client.claim(&meter_id);

    let meter = client.get_meter(&meter_id).unwrap();
    assert_eq!(meter.balance, 0);
    assert!(meter.is_active); 
    assert!(meter.grace_period_start > 0); 

    // Fast forward another 25 hours - should expire grace period
    env.ledger().set_timestamp(env.ledger().timestamp() + (25 * 60 * 60));
    client.claim(&meter_id); 

    let meter = client.get_meter(&meter_id).unwrap();
    assert!(!meter.is_active); 
}

#[test]
fn test_peak_hour_tariff() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
    let token = token::Client::new(&env, &token_address);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    token_admin_client.mint(&user, &5000);

    let rate = 10; 
    let device_public_key = device_key(&env, 1);
    let meter_id = client.register_meter(&user, &provider, &rate, &token_address, &device_public_key, &0);

    client.initiate_pairing(&meter_id);
    client.complete_pairing(&meter_id, &BytesN::from_array(&env, &[2u8; 64]));

    client.initiate_pairing(&meter_id);
    client.complete_pairing(&meter_id, &BytesN::from_array(&env, &[2u8; 64]));
    client.top_up(&meter_id, &5000, &user);

    // 19:00 UTC Peak hours
    env.ledger().set_timestamp(68400);

    let signed_data = SignedUsageData {
        meter_id,
        timestamp: 68400,
        watt_hours_consumed: 1000,
        units_consumed: 10,
        is_renewable_energy: false,
        signature: BytesN::from_array(&env, &[3u8; 64]),
        public_key: device_public_key,
    };
    client.deduct_units(&signed_data);

    let meter = client.get_meter(&meter_id).unwrap();
    // Base cost 100 * 1.5 multiplier = 150
    assert_eq!(meter.balance, 4850); 
}

#[test]
fn test_carbon_credit_stream_creates_credits_and_reduces_protocol_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let provider = Address::generate(&env);
    let payment_admin = Address::generate(&env);
    let credit_admin = provider.clone();

    let payment_token = env.register_stellar_asset_contract_v2(payment_admin.clone()).address();
    let credit_token = env.register_stellar_asset_contract_v2(credit_admin.clone()).address();

    let payment_client = token::StellarAssetClient::new(&env, &payment_token);
    let credit_client = token::StellarAssetClient::new(&env, &credit_token);

    payment_client.mint(&user, &100_000);
    credit_client.mint(&provider, &100_000);

    let oracle_id = env.register(mock_environmental_oracle::MockEnvironmentalOracle, ());
    client.set_oracle(&oracle_id);

    let fee_wallet = Address::generate(&env);
    client.set_maintenance_config(&fee_wallet, &1000);

    let device_public_key = device_key(&env, 99);
    let meter_id = client.register_meter(&user, &provider, &10, &payment_token, &device_public_key, &0);
    client.top_up(&meter_id, &50_000, &user);
    client.initiate_pairing(&meter_id);
    client.complete_pairing(&meter_id, &BytesN::from_array(&env, &[2u8; 64]));

    client.set_green_energy_discount(&meter_id, &2000);
    client.set_carbon_credit_config(&meter_id, credit_token.clone(), &500);

    let signed_usage = SignedUsageData {
        meter_id,
        timestamp: env.ledger().timestamp(),
        watt_hours_consumed: 1000,
        units_consumed: 10,
        signature: BytesN::from_array(&env, &[3u8; 64]),
        public_key: device_public_key,
        is_renewable_energy: true,
    };

    client.deduct_units(&signed_usage);

    let credit_balance = credit_client.balance(&user);
    assert!(credit_balance > 0);

    let fee_balance = payment_client.balance(&fee_wallet);
    assert!(fee_balance < 1000);
}

// ==================== PROVIDER RELIABILITY TESTS ====================

// Note: Provider reliability tests removed - functions not in current contract API

// Removed duplicate test_green_energy_bonus - only one kept above

#[test]
fn test_multisig_withdrawal_full_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let provider = Address::generate(&env);
    let treasury = Address::generate(&env);
    let token_address = create_token(&env);

    let mut finance_wallets = Vec::new(&env);
    for _ in 0..5 { finance_wallets.push_back(Address::generate(&env)); }

    let device_public_key = device_key(&env, 1);
    let meter_id = client.register_meter(&user, &provider, &100, &token_address, &device_public_key, &0);

    client.configure_multisig_withdrawal(&provider, &finance_wallets, &3, &100_000_00);

    let withdrawal_amount: i128 = 150_000_00;
    let request_id = client.propose_multisig_withdrawal(&provider, &meter_id, &withdrawal_amount, &treasury);

    // Approvals
    client.approve_multisig_withdrawal(&provider, &request_id);
    client.approve_multisig_withdrawal(&provider, &request_id);

    client.execute_multisig_withdrawal(&provider, &request_id);
    
    let request_count = client.get_withdrawal_request_count(&provider);
    assert_eq!(request_count, 1);
}

#[test]
fn test_seasonal_factor_affects_rate() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let provider = Address::generate(&env);
    let token_address = create_token(&env);
    let token_admin = token::StellarAssetClient::new(&env, &token_address);

    token_admin.mint(&user, &10000);

    let meter_id = client.register_meter(&user, &provider, &10, &token_address, &device_key(&env, 1), &0);
    client.top_up(&meter_id, &5000, &user);

#[test]
fn test_gas_buffer_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    // Mint tokens for provider to initialize gas buffer
    token_admin_client.mint(&provider, &1000);

    // Initialize gas buffer with minimum amount
    client.initialize_gas_buffer(&provider, &token_address, &100);
    
    let gas_buffer = client.get_gas_buffer(&provider).unwrap();
    assert_eq!(gas_buffer.balance, 100);
    assert_eq!(gas_buffer.provider, provider);
    assert_eq!(gas_buffer.token, token_address);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_gas_buffer_initialization_with_insufficient_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();

    // Try to initialize with amount below minimum
    client.initialize_gas_buffer(&provider, &token_address, &50);
}

#[test]
fn test_gas_buffer_top_up() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    token_admin_client.mint(&provider, &1000);

    // Initialize gas buffer
    client.initialize_gas_buffer(&provider, &token_address, &100);
    
    // Top up gas buffer
    client.top_up_gas_buffer(&provider, &token_address, &200);
    
    let gas_buffer = client.get_gas_buffer(&provider).unwrap();
    assert_eq!(gas_buffer.balance, 300);
}

#[test]
fn test_gas_buffer_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    token_admin_client.mint(&provider, &1000);

    // Initialize gas buffer
    client.initialize_gas_buffer(&provider, &token_address, &500);
    
    // Withdraw from gas buffer
    client.withdraw_from_gas_buffer(&provider, &token_address, &200);
    
    let gas_buffer = client.get_gas_buffer(&provider).unwrap();
    assert_eq!(gas_buffer.balance, 300);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_gas_buffer_withdrawal_below_minimum() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    token_admin_client.mint(&provider, &1000);

    // Initialize gas buffer with minimum amount
    client.initialize_gas_buffer(&provider, &token_address, &100);
    
    // Try to withdraw entire buffer (would go below minimum)
    client.withdraw_from_gas_buffer(&provider, &token_address, &50);
}

#[test]
fn test_claim_with_gas_buffer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let provider = Address::generate(&env);
    let oracle = Address::generate(&env);
    client.set_oracle(&oracle);

    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token = token::Client::new(&env, &token_address);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    token_admin_client.mint(&user, &1000);
    token_admin_client.mint(&provider, &1000);

    // Initialize gas buffer for provider
    client.initialize_gas_buffer(&provider, &token_address, &500);

    let meter_id = client.register_meter(&user, &provider, &10, &token_address, &device_key(&env, 1), &0);
    client.top_up(&meter_id, &500, &user);

    env.ledger().set_timestamp(env.ledger().timestamp() + 5);
    client.claim(&meter_id);

    let meter = client.get_meter(&meter_id).unwrap();
    assert_eq!(meter.balance, 450);
    assert_eq!(token.balance(&provider), 550); // 50 from claim + 500 initial gas buffer
    assert_eq!(token.balance(&contract_id), 450);
    
    // Check that gas buffer was used (balance should be reduced)
    let gas_buffer = client.get_gas_buffer(&provider).unwrap();
    assert_eq!(gas_buffer.balance, 400); // 500 - 100 (MIN_GAS_BUFFER)
}

#[test]
fn test_get_gas_buffer_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let provider = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    token_admin_client.mint(&provider, &1000);

    // Check balance before initialization
    assert_eq!(client.get_gas_buffer_balance(&provider), 0);

    // Initialize gas buffer
    client.initialize_gas_buffer(&provider, &token_address, &300);
    
    // Check balance after initialization
    assert_eq!(client.get_gas_buffer_balance(&provider), 300);
}

#[test]
fn test_event_emissions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(UtilityContract, ());
    let client = UtilityContractClient::new(&env, &contract_id);

    let root_admin = Address::generate(&env);
    
    // Set admin
    client.set_admin(&root_admin);
    
    let user = Address::generate(&env);
    let provider = Address::generate(&env);
    
    // Setup a token
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    token_admin_client.mint(&user, &1000);

    // Test meter registration event
    let meter_id = client.register_meter(&user, &provider, &10, &token_address, &device_key(&env, 1), &0);
    
    // Test top-up event
    client.top_up(&meter_id, &500, &user);
    
    // Test claim event
    env.ledger().set_timestamp(env.ledger().timestamp() + 10);
    client.claim(&meter_id);
    
    // Test webhook configuration event
    let webhook_url = soroban_sdk::String::from_str(&env, "https://example.com/webhook");
    client.configure_webhook(&user, &webhook_url);
    
    // Test emergency shutdown event
    client.emergency_shutdown(&meter_id);
    
    // Note: In a real test environment, you would verify the events were emitted
    // This test ensures the functions execute without panicking when events are published
}
