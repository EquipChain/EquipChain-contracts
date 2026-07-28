import os

lib_path = 'contracts/utility_contracts/src/lib.rs'
with open(lib_path, 'r') as f:
    content = f.read()

# 1. Add DataKey variants for token whitelist (before the closing brace of DataKey enum)
# Find the last DataKey variant and add our new ones before EmergencyDrainLastExecution
data_key_insert_point = '    EmergencyDrainLastExecution,'

new_data_keys = '''    // Issue #23 - Token Whitelist & Security
    ApprovedTokens,
    TokenInfo(Address),
'''

if new_data_keys not in content:
    content = content.replace(data_key_insert_point, new_data_keys + data_key_insert_point)

# 2. Add TokenStandard enum after DataKey
enums_insert_point = '// Issue #277 - Emergency Drain Recovery'
new_enums = '''
// ============================================================================
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

'''
if new_enums not in content:
    content = content.replace(enums_insert_point, new_enums + enums_insert_point)

with open(lib_path, 'w') as f:
    f.write(content)

print('DataKey variants and TokenStandard enum added successfully')
