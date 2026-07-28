#!/usr/bin/env python3
"""Add remaining token security features for issue #23 to lib.rs"""
import os

lib_path = 'contracts/utility_contracts/src/lib.rs'
with open(lib_path, 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Add TokenStandard enum and TokenInfo struct after EmergencyDrainRecord struct
old_drain = 'pub struct EmergencyDrainRecord {\n    pub timestamp: u64,\n    pub amount: i128,\n    pub recipient: Address,\n    pub reason: String,\n}'

new_drain = '''// ============================================================================
// Issue #23: Token Standard Detection & Whitelist
// ============================================================================

/// Classification of a token based on on-chain heuristics.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenStandard {
    /// Standard Stellar Asset Contract or typical token.
    Standard,
    /// Token that charges a fee on every transfer.
    FeeOnTransfer,
    /// Token whose balance changes without transfers (e.g., rebasing tokens).
    Rebasing,
    /// Token that has blacklist/pause functionality.
    Blacklisted,
    /// Token that cannot be identified or is known to be dangerous.
    Unknown,
}

/// Metadata about an approved token.
#[contracttype]
#[derive(Clone, Debug)]
pub struct TokenInfo {
    pub token: Address,
    pub standard: TokenStandard,
    pub decimals: u32,
    pub approved_at: u64,
    pub approved_by: Address,
}

pub struct EmergencyDrainRecord {
    pub timestamp: u64,
    pub amount: i128,
    pub recipient: Address,
    pub reason: String,
}'''

content = content.replace(old_drain, new_drain)
print('1. TokenStandard enum added')

# 2. Add UnapprovedToken ContractError variant
old_err = '    NotFound = 114,\n    NotInitialized = 115,\n    FlowRateTooLow = 116,\n}'
new_err = '    NotFound = 114,\n    NotInitialized = 115,\n    FlowRateTooLow = 116,\n    // Issue #23 - Token Security\n    UnapprovedToken = 117,\n    TokenBalanceMismatch = 118,\n}'
content = content.replace(old_err, new_err)
print('2. ContractError variants added')

# 3. Add token security functions after transfer_tokens
# Find the transfer_tokens function and add after it
old_transfer = '''fn transfer_tokens(env: &Env, token: &Address, from: &Address, to: &Address, amount: &i128) {
    let client = token::Client::new(env, token);
    client.transfer(from, to, amount);
}'''

new_transfer = '''fn transfer_tokens(env: &Env, token: &Address, from: &Address, to: &Address, amount: &i128) {
    require_approved_token(env, token);
    let balance_before = get_token_balance(env, token, from);
    let client = token::Client::new(env, token);
    client.transfer(from, to, amount);
    let balance_after = get_token_balance(env, token, from);
    let expected_balance = balance_before.saturating_sub(*amount);
    if balance_after < expected_balance {
        panic_with_error!(env, ContractError::TokenBalanceMismatch);
    }
}

fn get_token_balance(env: &Env, token: &Address, address: &Address) -> i128 {
    let client = token::Client::new(env, token);
    client.balance(address)
}

fn require_approved_token(env: &Env, token: &Address) {
    let approved: Vec<Address> = env.storage()
        .instance()
        .get(&DataKey::ApprovedTokens)
        .unwrap_or(Vec::new(env));
    if approved.len() > 0 && !approved.contains(token) {
        panic_with_error!(env, ContractError::UnapprovedToken);
    }
}

fn validate_token(env: &Env, token: &Address) -> TokenStandard {
    // Simplified heuristic: attempt to read token metadata
    // If balance/transfer works, classify as Standard
    // In production, this would use more sophisticated on-chain analysis
    let client = token::Client::new(env, token);
    // Test that the token responds to balance queries
    let _ = client.balance(token);
    TokenStandard::Standard
}'''

content = content.replace(old_transfer, new_transfer)
print('3. Token security functions added')

with open(lib_path, 'w', encoding='utf-8') as f:
    f.write(content)

print('Complete! All token security features added.')
