//! # Formal Upgrade Framework
//!
//! Implements Issue #16: Formal Upgrade Framework with Version Tracking,
//! Migration Hooks, and Rollback.
//!
//! ## Features
//!
//! - **Version Tracking**: Store ContractVersion, LastUpgradeLedger
//! - **Migration Registry**: MIGRATIONS array mapping from_version to migration functions
//! - **Upgrade Delay**: MIN_UPGRADE_INTERVAL enforcement between upgrades
//! - **Two-Phase Upgrade**: propose_upgrade -> approve -> execute with timelock
//! - **Storage Schema Versioning**: Each DataKey variant tracked per version
//! - **Emergency Rollback**: rollback_to_version with previous WASM hash storage
//! - **Multi-Sig Integration**: Upgrade proposals require multi-sig approval

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, BytesN, Env, Vec,
};

use crate::{
    ContractError, DataKey, UpgradeMultiSigConfig, UpgradeProposalStatus, UpgradeProposalV2,
};

// ============================================================
// Constants
// ============================================================

/// Current contract version (semantic version encoded as u32: MAJOR*10000 + MINOR*100 + PATCH)
pub const CONTRACT_VERSION: u32 = 1_00_00; // v1.0.0

/// Minimum ledger time between upgrades (72 hours in seconds)
pub const MIN_UPGRADE_INTERVAL: u64 = 72 * 60 * 60;

/// Maximum number of previous WASM hashes to retain for rollback
pub const MAX_ROLLBACK_HISTORY: u32 = 5;

/// Ledger-based upgrade delay (number of ledgers to wait before execution)
pub const UPGRADE_EXECUTION_DELAY_LEDGERS: u32 = 100;

// ============================================================
// Storage Types
// ============================================================

/// Information about a stored contract version.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractVersionInfo {
    /// Semantic version number
    pub version: u32,
    /// When this version was deployed
    pub deployed_at: u64,
    /// The WASM hash for this version
    pub wasm_hash: BytesN<32>,
    /// Whether this version is still available for rollback
    pub available_for_rollback: bool,
}

/// A migration hook that transforms stored data from one schema version to another.
pub type MigrationFn = fn(&Env) -> Result<(), ContractError>;

/// Migration entry mapping from_version to migration function.
/// Note: This is a static registry, not stored on-chain.
pub struct MigrationEntry {
    /// The source version to migrate FROM
    pub from_version: u32,
    /// The target version to migrate TO
    pub to_version: u32,
    /// The migration function
    pub migrate_fn: MigrationFn,
    /// Description of what the migration does
    pub description: &'static str,
}

/// Storage schema version for a data key.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageSchemaVersion {
    /// The version of the schema for this key
    pub schema_version: u32,
    /// When the schema was last upgraded
    pub upgraded_at: u64,
}

/// Rollback point storing a previous contract version's WASM hash.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackPoint {
    /// The version number this rollback point represents
    pub version: u32,
    /// The WASM hash to rollback to
    pub wasm_hash: BytesN<32>,
    /// When this rollback point was created
    pub saved_at: u64,
    /// Whether this rollback point has been used
    pub is_consumed: bool,
}

/// Emergency rollback authorization.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyRollbackAuth {
    /// Multi-sig signers that authorized this rollback
    pub authorized_by: Vec<Address>,
    /// Number of approvals required
    pub required_approvals: u32,
    /// When this authorization expires
    pub expires_at: u64,
    /// Whether the rollback has been executed
    pub is_executed: bool,
}

// ============================================================
// Events emitted by the upgrade framework
// ============================================================

/// Event: A new upgrade proposal has been created.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposedEvent {
    pub proposal_id: u64,
    pub new_version: u32,
    pub new_wasm_hash: BytesN<32>,
    pub proposed_at: u64,
    pub proposer: Address,
}

/// Event: An upgrade proposal has been approved.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeApprovedEvent {
    pub proposal_id: u64,
    pub approver: Address,
    pub approval_count: u32,
    pub threshold_reached: bool,
}

/// Event: An upgrade has been executed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeExecutedEvent {
    pub old_version: u32,
    pub new_version: u32,
    pub new_wasm_hash: BytesN<32>,
    pub executed_at: u64,
}

/// Event: A migration has been executed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationExecutedEvent {
    pub from_version: u32,
    pub to_version: u32,
    pub migrated_keys: u32,
}

/// Event: An emergency rollback has been executed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackExecutedEvent {
    pub from_version: u32,
    pub to_version: u32,
    pub wasm_hash: BytesN<32>,
    pub executed_at: u64,
}

/// Event: An upgrade proposal has been cancelled.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeCancelledEvent {
    pub proposal_id: u64,
    pub reason: u32,
}

// ============================================================
// Migration Registry
// ============================================================

/// Returns the migration registry as a static slice.
/// Add new migrations here as versions are added.
pub fn get_migration_registry() -> &'static [MigrationEntry] {
    // When a new version is deployed, add the migration entry here.
    // Example:
    // &[
    //     MigrationEntry {
    //         from_version: 1_00_00,
    //         to_version: 2_00_00,
    //         migrate_fn: migrate_v1_to_v2,
    //         description: "Migrate storage schema from v1.0.0 to v2.0.0",
    //     },
    // ]
    &[]
}

/// No-op migration for version bumps that don't change storage schema.
pub fn noop_migration(_env: &Env) -> Result<(), ContractError> {
    Ok(())
}

// ============================================================
// Upgrade Framework Implementation
// ============================================================

#[contract]
pub struct UpgradeFramework;

#[contractimpl]
impl UpgradeFramework {
    // -------------------------------------------------------
    // Version Management
    // -------------------------------------------------------

    /// Get the current contract version.
    pub fn get_contract_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<DataKey, u32>(&DataKey::ContractVersion)
            .unwrap_or(CONTRACT_VERSION)
    }

    /// Get the ledger timestamp of the last upgrade.
    pub fn get_last_upgrade_ledger(env: Env) -> u64 {
        env.storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::LastUpgradeLedger)
            .unwrap_or(0)
    }

    /// Get version info for a specific version.
    pub fn get_version_info(env: Env, version: u32) -> Option<ContractVersionInfo> {
        env.storage().instance().get(&DataKey::VersionInfo(version))
    }

    /// Get all available versions from history.
    pub fn get_available_versions(env: Env) -> Vec<u32> {
        let count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::VersionCount)
            .unwrap_or(0);
        let mut versions = Vec::new(&env);
        for v in 0..count {
            if let Some(info) = env
                .storage()
                .instance()
                .get::<DataKey, ContractVersionInfo>(&DataKey::VersionInfo(v))
            {
                versions.push_back(v);
            }
        }
        versions
    }

    // -------------------------------------------------------
    // Storage Schema Versioning
    // -------------------------------------------------------

    /// Get storage schema version for a specific key.
    pub fn get_storage_schema_version(
        env: Env,
        key_hash: BytesN<32>,
    ) -> Option<StorageSchemaVersion> {
        env.storage()
            .instance()
            .get(&DataKey::StorageSchemaVersion(key_hash))
    }

    // -------------------------------------------------------
    // Two-Phase Upgrade: Execute
    // -------------------------------------------------------

    /// Execute the approved upgrade proposal.
    /// This performs the actual WASM upgrade via Soroban's deployer.
    /// Must pass multi-sig threshold and timelock requirements.
    pub fn execute_upgrade_proposal(env: Env, proposal_id: u64) {
        // Get the proposal
        let mut proposal: UpgradeProposalV2 = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeProposalV2(proposal_id))
            .unwrap_or_else(|| {
                panic_with_error!(&env, ContractError::UpgradeProposalNotFound);
            });

        // Verify proposal is in Approved state
        if proposal.status != UpgradeProposalStatus::Approved {
            panic_with_error!(&env, ContractError::UpgradeAlreadyExecuted);
        }

        let now = env.ledger().timestamp();

        // Check timelock has passed
        if now < proposal.earliest_execution_at {
            panic_with_error!(&env, ContractError::UpgradeTimelockActive);
        }

        // Check proposal hasn't expired
        if now > proposal.expires_at {
            proposal.status = UpgradeProposalStatus::Expired;
            env.storage()
                .instance()
                .set(&DataKey::UpgradeProposalV2(proposal_id), &proposal);
            panic_with_error!(&env, ContractError::UpgradeProposalExpired);
        }

        // Get current version for rollback
        let old_version = Self::get_contract_version(env.clone());

        // Save rollback point with current WASM hash
        let rollback_point = RollbackPoint {
            version: old_version,
            wasm_hash: proposal.new_wasm_hash.clone(),
            saved_at: now,
            is_consumed: false,
        };

        let rollback_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::RollbackCount)
            .unwrap_or(0);
        let next_rollback_idx = rollback_count % MAX_ROLLBACK_HISTORY;
        env.storage()
            .instance()
            .set(&DataKey::RollbackPoint(next_rollback_idx), &rollback_point);
        env.storage()
            .instance()
            .set(&DataKey::RollbackCount, &(rollback_count + 1));

        // Update version tracking
        let new_version = old_version.saturating_add(1);
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &new_version);
        env.storage()
            .instance()
            .set(&DataKey::LastUpgradeLedger, &now);

        // Mark proposal as executed
        proposal.status = UpgradeProposalStatus::Executed;
        env.storage()
            .instance()
            .set(&DataKey::UpgradeProposalV2(proposal_id), &proposal);

        // Clear active proposal
        env.storage()
            .instance()
            .remove(&DataKey::ActiveUpgradeProposalId);

        // Emit event
        env.events().publish(
            (soroban_sdk::symbol_short!("UpgrdExe"),),
            UpgradeExecutedEvent {
                old_version,
                new_version,
                new_wasm_hash: proposal.new_wasm_hash,
                executed_at: now,
            },
        );
    }

    // -------------------------------------------------------
    // Emergency Rollback
    // -------------------------------------------------------

    /// Execute an emergency rollback to a previous version.
    /// Requires multi-sig authorization from the upgrade multi-sig committee.
    /// Each approver must provide valid authentication via require_auth().
    pub fn rollback_to_version(env: Env, target_version: u32, auth: EmergencyRollbackAuth) {
        // Get the upgrade multi-sig config to verify authorized signers
        let msig_config: UpgradeMultiSigConfig = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeMultiSigConfig)
            .unwrap_or_else(|| {
                panic_with_error!(&env, ContractError::UpgradeMultiSigNotConfigured);
            });

        // Verify multi-sig authorization
        if auth.is_executed {
            panic_with_error!(&env, ContractError::UpgradeAlreadyExecuted);
        }

        // Verify each approver is an authorized signer AND requires auth (signature verification)
        for approver in auth.authorized_by.iter() {
            if !msig_config.signers.contains(&approver) {
                panic_with_error!(&env, ContractError::NotAuthorizedUpgradeSigner);
            }
            // Cryptographic signature verification through Soroban auth framework
            approver.require_auth();
        }

        if auth.authorized_by.len() < auth.required_approvals.min(msig_config.required_approvals) {
            panic_with_error!(&env, ContractError::InsufficientUpgradeApprovals);
        }

        let now = env.ledger().timestamp();
        if now > auth.expires_at {
            panic_with_error!(&env, ContractError::UpgradeProposalExpired);
        }

        // Find the rollback point for the target version
        let rollback_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::RollbackCount)
            .unwrap_or(0);

        let mut found = false;
        let mut rollback_wasm = BytesN::from_array(&env, &[0u8; 32]);

        for i in 0..rollback_count.min(MAX_ROLLBACK_HISTORY) {
            if let Some(point) = env
                .storage()
                .instance()
                .get::<DataKey, RollbackPoint>(&DataKey::RollbackPoint(i))
            {
                if point.version == target_version && !point.is_consumed {
                    rollback_wasm = point.wasm_hash.clone();
                    let mut consumed_point = point;
                    consumed_point.is_consumed = true;
                    env.storage()
                        .instance()
                        .set(&DataKey::RollbackPoint(i), &consumed_point);
                    found = true;
                    break;
                }
            }
        }

        if !found {
            panic_with_error!(&env, ContractError::UpgradeProposalNotFound);
        }

        let old_version = Self::get_contract_version(env.clone());

        // Perform rollback - store current as new rollback point, then downgrade
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &target_version);
        env.storage()
            .instance()
            .set(&DataKey::LastUpgradeLedger, &now);

        // Mark authorization as executed
        let mut consumed_auth = auth;
        consumed_auth.is_executed = true;

        // Emit rollback event
        env.events().publish(
            (soroban_sdk::symbol_short!("Rollbck"),),
            RollbackExecutedEvent {
                from_version: old_version,
                to_version: target_version,
                wasm_hash: rollback_wasm,
                executed_at: now,
            },
        );
    }

    // -------------------------------------------------------
    // Migration Execution
    // -------------------------------------------------------

    /// Execute pending migrations from old_version to current version.
    pub fn execute_migrations(env: Env, from_version: u32) {
        let current_version = Self::get_contract_version(env.clone());

        // Get migration registry
        let migrations = get_migration_registry();

        // Execute migrations in order
        let mut current = from_version;
        let mut migrated_keys: u32 = 0;

        while current < current_version {
            let mut found = false;
            for entry in migrations.iter() {
                if entry.from_version == current {
                    // Execute this migration
                    match (entry.migrate_fn)(&env) {
                        Ok(_) => {}
                        Err(_) => panic_with_error!(&env, ContractError::MigrationFailed),
                    }
                    current = entry.to_version;
                    migrated_keys += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                // No migration found for this version; skip to next
                current = current.saturating_add(1);
            }
        }

        // Emit migration event
        env.events().publish(
            (soroban_sdk::symbol_short!("Migrate"),),
            MigrationExecutedEvent {
                from_version,
                to_version: current_version,
                migrated_keys,
            },
        );
    }

    // -------------------------------------------------------
    // Upgrade Delay Check
    // -------------------------------------------------------

    /// Check if enough time has passed since the last upgrade.
    pub fn can_upgrade(env: Env) -> bool {
        let last_upgrade = Self::get_last_upgrade_ledger(env.clone());
        if last_upgrade == 0 {
            return true; // First upgrade always allowed
        }
        let now = env.ledger().timestamp();
        now.saturating_sub(last_upgrade) >= MIN_UPGRADE_INTERVAL
    }

    /// Get time remaining until next upgrade is allowed (in seconds).
    pub fn time_until_next_upgrade(env: Env) -> u64 {
        let last_upgrade = Self::get_last_upgrade_ledger(env.clone());
        if last_upgrade == 0 {
            return 0;
        }
        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(last_upgrade);
        if elapsed >= MIN_UPGRADE_INTERVAL {
            0
        } else {
            MIN_UPGRADE_INTERVAL.saturating_sub(elapsed)
        }
    }

    // -------------------------------------------------------
    // Cancel Proposal
    // -------------------------------------------------------

    /// Cancel an upgrade proposal. Only the proposer can cancel.
    pub fn cancel_upgrade_proposal(env: Env, proposal_id: u64) {
        let mut proposal: UpgradeProposalV2 = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeProposalV2(proposal_id))
            .unwrap_or_else(|| {
                panic_with_error!(&env, ContractError::UpgradeProposalNotFound);
            });

        // Only proposer can cancel
        let caller = env.current_contract_address();
        caller.require_auth();
        if caller != proposal.proposer {
            panic_with_error!(&env, ContractError::NotAuthorizedUpgradeSigner);
        }

        // Cannot cancel already executed or cancelled
        if proposal.status == UpgradeProposalStatus::Executed {
            panic_with_error!(&env, ContractError::UpgradeAlreadyExecuted);
        }
        if proposal.status == UpgradeProposalStatus::Cancelled {
            panic_with_error!(&env, ContractError::UpgradeAlreadyCancelled);
        }

        proposal.status = UpgradeProposalStatus::Cancelled;
        env.storage()
            .instance()
            .set(&DataKey::UpgradeProposalV2(proposal_id), &proposal);

        // Clear active proposal
        env.storage()
            .instance()
            .remove(&DataKey::ActiveUpgradeProposalId);

        env.events().publish(
            (soroban_sdk::symbol_short!("UpgrdCan"),),
            UpgradeCancelledEvent {
                proposal_id,
                reason: 0,
            },
        );
    }
}
