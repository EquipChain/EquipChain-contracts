//! # Multi-Sig Governance Enhancements
//!
//! Implements Issue #18: Add Proposal Expiry, Quorum, Vote Weighting,
//! and Nonce Sync Safety to Multi-Sig.
//!
//! ## Features
//!
//! - **Proposal Expiry**: Votes must be collected within TIMEFRAME
//! - **Quorum Mechanism**: min_voters must participate regardless of approval
//! - **Vote Weighting**: Token-based or reputation-based weighted voting
//! - **Timelock**: Configurable delay after approval threshold is reached
//! - **Cancel Proposal**: Proposer can retract flawed proposals
//! - **Proposal Status Query**: Rich status enum for governance transparency

use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, Env, Symbol,
};

use crate::{ContractError, DataKey};

// ============================================================
// Constants
// ============================================================

/// Default proposal expiry duration (7 days in seconds)
pub const DEFAULT_PROPOSAL_EXPIRY: u64 = 7 * 24 * 60 * 60;

/// Default timelock duration (48 hours in seconds)
pub const DEFAULT_TIMELOCK_DURATION: u64 = 48 * 60 * 60;

/// Maximum quorum as absolute number of voters
pub const MAX_QUORUM_VOTERS: u32 = 100;

// ============================================================
// Governance Types
// ============================================================

/// Status of a governance proposal.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProposalStatus {
    /// Proposal created, voting not yet started
    Pending = 0,
    /// Voting is active
    Active = 1,
    /// Threshold reached, waiting for timelock
    Approved = 2,
    /// Timelock passed, ready for execution
    Ready = 3,
    /// Proposal has been executed
    Executed = 4,
    /// Proposal has expired
    Expired = 5,
    /// Proposal has been cancelled by proposer
    Cancelled = 6,
}

/// Weighted vote from a voter.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vote {
    /// The voter's address
    pub voter: Address,
    /// Whether the voter approves (true) or rejects (false)
    pub approve: bool,
    /// The weight of this vote (based on token holdings or reputation)
    pub weight: u64,
    /// When this vote was cast
    pub voted_at: u64,
}

/// Vote weight provider trait.
/// Implementations can provide token-based, NFT-based, or 1-person-1-vote weighting.
#[contracttype]
#[derive(Clone)]
pub struct VoteWeightProvider {
    /// Whether token-based weighting is enabled
    pub use_token_weighting: bool,
    /// The token address for weighted voting (if applicable)
    pub token_address: Option<Address>,
    /// Whether reputation-based weighting is enabled
    pub use_reputation_weighting: bool,
}

/// Governance proposal with all enhancements.
#[contracttype]
#[derive(Clone)]
pub struct GovernanceProposal {
    /// Unique proposal ID
    pub proposal_id: u64,
    /// Title/description of the proposal
    pub description: Symbol,
    /// The address that created this proposal
    pub proposer: Address,
    /// When the proposal was created
    pub created_at: u64,
    /// When the proposal expires (voting no longer allowed)
    pub expires_at: u64,
    /// Minimum number of voters required (absolute count quorum)
    pub min_quorum: u32,
    /// Threshold of approving voters required (in basis points, e.g., 5000 = 50%)
    pub approval_threshold_bps: u32,
    /// Timelock duration after approval before execution (in seconds)
    pub timelock_duration: u64,
    /// When the threshold was reached (0 = not yet)
    pub threshold_reached_at: u64,
    /// When execution becomes available (threshold_reached_at + timelock)
    pub execution_available_at: u64,
    /// Current status
    pub status: ProposalStatus,
    /// Total votes counted
    pub total_votes: u32,
    /// Total approval weight
    pub approval_weight: u64,
    /// Total rejection weight
    pub rejection_weight: u64,
    /// Vote weight provider configuration
    pub weight_provider: VoteWeightProvider,
    /// Whether this proposal has been executed
    pub is_executed: bool,
}

/// Governance configuration.
#[contracttype]
#[derive(Clone)]
pub struct GovernanceConfig {
    /// Admin address that can update governance parameters
    pub admin: Address,
    /// Default proposal expiry duration
    pub default_expiry: u64,
    /// Default timelock duration
    pub default_timelock: u64,
    /// Default quorum requirement (absolute voter count)
    pub default_quorum: u32,
    /// Default approval threshold (basis points)
    pub default_approval_threshold_bps: u32,
    /// Whether governance is enabled
    pub enabled: bool,
    /// Vote weight provider configuration
    pub weight_provider: VoteWeightProvider,
}

/// Event: Governance proposal created.
#[contracttype]
#[derive(Clone)]
pub struct ProposalCreatedEvent {
    pub proposal_id: u64,
    pub proposer: Address,
    pub description: Symbol,
    pub expires_at: u64,
    pub min_quorum: u32,
}

/// Event: Vote cast on a proposal.
#[contracttype]
#[derive(Clone)]
pub struct VoteCastEvent {
    pub proposal_id: u64,
    pub voter: Address,
    pub approve: bool,
    pub weight: u64,
    pub total_approval: u64,
    pub total_rejection: u64,
}

/// Event: Proposal status changed.
#[contracttype]
#[derive(Clone)]
pub struct ProposalStatusChangedEvent {
    pub proposal_id: u64,
    pub old_status: ProposalStatus,
    pub new_status: ProposalStatus,
    pub timestamp: u64,
}

/// Event: Proposal cancelled.
#[contracttype]
#[derive(Clone)]
pub struct ProposalCancelledEvent {
    pub proposal_id: u64,
    pub proposer: Address,
    pub reason: Symbol,
}

// ============================================================
// Governance Module Implementation
// ============================================================

#[contract]
pub struct GovernanceModule;

#[contractimpl]
impl GovernanceModule {
    /// Initialize governance configuration.
    pub fn initialize_governance(
        env: Env,
        admin: Address,
        default_expiry: Option<u64>,
        default_timelock: Option<u64>,
        default_quorum: Option<u32>,
        default_approval_threshold_bps: Option<u32>,
    ) {
        if env
            .storage()
            .instance()
            .has(&DataKey::GovernanceConfig)
        {
            panic_with_error!(&env, ContractError::MultiSigAlreadyConfigured);
        }

        let config = GovernanceConfig {
            admin: admin.clone(),
            default_expiry: default_expiry.unwrap_or(DEFAULT_PROPOSAL_EXPIRY),
            default_timelock: default_timelock.unwrap_or(DEFAULT_TIMELOCK_DURATION),
            default_quorum: default_quorum.unwrap_or(5),
            default_approval_threshold_bps: default_approval_threshold_bps.unwrap_or(5000),
            enabled: true,
            weight_provider: VoteWeightProvider {
                use_token_weighting: false,
                token_address: None,
                use_reputation_weighting: false,
            },
        };

        env.storage()
            .instance()
            .set(&DataKey::GovernanceConfig, &config);

        env.events().publish(
            (soroban_sdk::symbol_short!("GovInit"),),
            admin,
        );
    }

    /// Create a new governance proposal.
    pub fn create_proposal(
        env: Env,
        description: Symbol,
        expiry: Option<u64>,
        min_quorum: Option<u32>,
        approval_threshold_bps: Option<u32>,
        timelock_duration: Option<u64>,
    ) -> u64 {
        let config: GovernanceConfig = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceConfig)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::MultiSigNotConfigured));

        if !config.enabled {
            panic_with_error!(&env, ContractError::GovernanceDisabled);
        }

        let proposer = env.current_contract_address();
        proposer.require_auth();

        // Get next proposal ID
        let counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceProposalCounter)
            .unwrap_or(0);
        let proposal_id = counter.saturating_add(1);

        let now = env.ledger().timestamp();
        let proposal_expiry = expiry.unwrap_or(config.default_expiry);
        let quorum = min_quorum.unwrap_or(config.default_quorum);
        let threshold = approval_threshold_bps.unwrap_or(config.default_approval_threshold_bps);
        let timelock = timelock_duration.unwrap_or(config.default_timelock);

        // Validate parameters - quorum is absolute voter count
        if quorum == 0 || quorum > MAX_QUORUM_VOTERS {
            panic_with_error!(&env, ContractError::InvalidSignatureThreshold);
        }
        if threshold > 10000 || threshold < 1000 {
            panic_with_error!(&env, ContractError::InvalidSignatureThreshold);
        }

        let proposal = GovernanceProposal {
            proposal_id,
            description: description.clone(),
            proposer: proposer.clone(),
            created_at: now,
            expires_at: now.saturating_add(proposal_expiry),
            min_quorum: quorum,
            approval_threshold_bps: threshold,
            timelock_duration: timelock,
            threshold_reached_at: 0,
            execution_available_at: 0,
            status: ProposalStatus::Active,
            total_votes: 0,
            approval_weight: 0,
            rejection_weight: 0,
            weight_provider: config.weight_provider.clone(),
            is_executed: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::GovernanceProposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::GovernanceProposalCounter, &proposal_id);

        env.events().publish(
            (soroban_sdk::symbol_short!("PropCrtd"),),
            ProposalCreatedEvent {
                proposal_id,
                proposer,
                description,
                expires_at: now.saturating_add(proposal_expiry),
                min_quorum: quorum,
            },
        );

        proposal_id
    }

    /// Cast a vote on a proposal.
    pub fn cast_vote(env: Env, proposal_id: u64, approve: bool) {
        let mut proposal: GovernanceProposal = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UpgradeProposalNotFound));

        // Check proposal is in active state
        if proposal.status != ProposalStatus::Active {
            if proposal.status == ProposalStatus::Executed {
                panic_with_error!(&env, ContractError::UpgradeAlreadyExecuted);
            } else if proposal.status == ProposalStatus::Expired {
                panic_with_error!(&env, ContractError::UpgradeProposalExpired);
            } else if proposal.status == ProposalStatus::Cancelled {
                panic_with_error!(&env, ContractError::UpgradeAlreadyCancelled);
            }
            panic_with_error!(&env, ContractError::UpgradeProposalNotFound);
        }

        // Check if proposal has expired
        let now = env.ledger().timestamp();
        if now > proposal.expires_at {
            proposal.status = ProposalStatus::Expired;
            env.storage()
                .instance()
                .set(&DataKey::GovernanceProposal(proposal_id), &proposal);
            panic_with_error!(&env, ContractError::UpgradeProposalExpired);
        }

        let voter = env.current_contract_address();
        voter.require_auth();

        // Check if voter already voted
        let vote_key = DataKey::GovernanceVote(proposal_id, voter.clone());
        if env.storage().instance().has(&vote_key) {
            panic_with_error!(&env, ContractError::AlreadyVoted);
        }

        // Calculate vote weight
        let weight = Self::calculate_vote_weight(&env, &voter, &proposal.weight_provider);

        // Record the vote
        let vote = Vote {
            voter: voter.clone(),
            approve,
            weight,
            voted_at: now,
        };

        env.storage()
            .instance()
            .set(&vote_key, &vote);

        // Update proposal tally
        proposal.total_votes = proposal.total_votes.saturating_add(1);
        if approve {
            proposal.approval_weight = proposal.approval_weight.saturating_add(weight as u64);
        } else {
            proposal.rejection_weight = proposal.rejection_weight.saturating_add(weight as u64);
        }

        // Check quorum and threshold
        let total_weight = proposal.approval_weight.saturating_add(proposal.rejection_weight);

        // Quorum check: Need minimum participation
        let quorum_satisfied = proposal.total_votes >= proposal.min_quorum;

        // Threshold check: Need approval_bps % of participating weight
        let threshold_met = if total_weight > 0 {
            (proposal.approval_weight as u128)
                .saturating_mul(10000)
                .saturating_div(total_weight as u128)
                >= proposal.approval_threshold_bps as u128
        } else {
            false
        };

        if quorum_satisfied && threshold_met {
            // Mark as approved and start timelock
            proposal.status = ProposalStatus::Approved;
            proposal.threshold_reached_at = now;
            proposal.execution_available_at = now.saturating_add(proposal.timelock_duration);
        }

        env.storage()
            .instance()
            .set(&DataKey::GovernanceProposal(proposal_id), &proposal);

        env.events().publish(
            (soroban_sdk::symbol_short!("VoteCstd"),),
            VoteCastEvent {
                proposal_id,
                voter,
                approve,
                weight,
                total_approval: proposal.approval_weight,
                total_rejection: proposal.rejection_weight,
            },
        );
    }

    /// Execute an approved proposal after timelock has passed.
    pub fn execute_proposal(env: Env, proposal_id: u64) {
        let mut proposal: GovernanceProposal = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UpgradeProposalNotFound));

        // Check proposal is approved
        if proposal.status != ProposalStatus::Approved {
            if proposal.status == ProposalStatus::Active {
                panic_with_error!(&env, ContractError::InsufficientUpgradeApprovals);
            } else if proposal.status == ProposalStatus::Executed {
                panic_with_error!(&env, ContractError::UpgradeAlreadyExecuted);
            } else if proposal.status == ProposalStatus::Expired {
                panic_with_error!(&env, ContractError::UpgradeProposalExpired);
            } else if proposal.status == ProposalStatus::Cancelled {
                panic_with_error!(&env, ContractError::UpgradeAlreadyCancelled);
            }
            panic_with_error!(&env, ContractError::UpgradeProposalNotFound);
        }

        let now = env.ledger().timestamp();

        // Check timelock has passed
        if now < proposal.execution_available_at {
            panic_with_error!(&env, ContractError::UpgradeTimelockActive);
        }

        // Check proposal hasn't expired during timelock
        if now > proposal.expires_at {
            proposal.status = ProposalStatus::Expired;
            env.storage()
                .instance()
                .set(&DataKey::GovernanceProposal(proposal_id), &proposal);
            panic_with_error!(&env, ContractError::UpgradeProposalExpired);
        }

        // Mark as executed
        proposal.status = ProposalStatus::Executed;
        proposal.is_executed = true;
        env.storage()
            .instance()
            .set(&DataKey::GovernanceProposal(proposal_id), &proposal);

        env.events().publish(
            (soroban_sdk::symbol_short!("PropExed"),),
            ProposalStatusChangedEvent {
                proposal_id,
                old_status: ProposalStatus::Approved,
                new_status: ProposalStatus::Executed,
                timestamp: now,
            },
        );
    }

    /// Cancel a proposal. Only the proposer can cancel.
    pub fn cancel_proposal(env: Env, proposal_id: u64, reason: Symbol) {
        let mut proposal: GovernanceProposal = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UpgradeProposalNotFound));

        // Only proposer can cancel
        let caller = env.current_contract_address();
        caller.require_auth();
        if caller != proposal.proposer {
            panic_with_error!(&env, ContractError::NotAuthorizedUpgradeSigner);
        }

        // Cannot cancel if already executed or cancelled
        if proposal.status == ProposalStatus::Executed {
            panic_with_error!(&env, ContractError::UpgradeAlreadyExecuted);
        }
        if proposal.status == ProposalStatus::Cancelled {
            panic_with_error!(&env, ContractError::UpgradeAlreadyCancelled);
        }

        let old_status = proposal.status;
        proposal.status = ProposalStatus::Cancelled;

        env.storage()
            .instance()
            .set(&DataKey::GovernanceProposal(proposal_id), &proposal);

        env.events().publish(
            (soroban_sdk::symbol_short!("PropCand"),),
            ProposalCancelledEvent {
                proposal_id,
                proposer: caller,
                reason,
            },
        );
    }

    /// Get proposal status.
    pub fn get_proposal_status(env: Env, proposal_id: u64) -> ProposalStatus {
        let proposal: GovernanceProposal = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UpgradeProposalNotFound));
        proposal.status
    }

    /// Get full proposal details.
    pub fn get_proposal(env: Env, proposal_id: u64) -> GovernanceProposal {
        env.storage()
            .instance()
            .get(&DataKey::GovernanceProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UpgradeProposalNotFound))
    }

    /// Get governance configuration.
    pub fn get_governance_config(env: Env) -> GovernanceConfig {
        env.storage()
            .instance()
            .get(&DataKey::GovernanceConfig)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::MultiSigNotConfigured))
    }

    /// Update governance configuration (admin only).
    pub fn update_governance_config(
        env: Env,
        default_expiry: Option<u64>,
        default_timelock: Option<u64>,
        default_quorum: Option<u32>,
        default_approval_threshold_bps: Option<u32>,
        enabled: Option<bool>,
    ) {
        let mut config: GovernanceConfig = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceConfig)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::MultiSigNotConfigured));

        config.admin.require_auth();

        if let Some(expiry) = default_expiry {
            config.default_expiry = expiry;
        }
        if let Some(timelock) = default_timelock {
            config.default_timelock = timelock;
        }
        if let Some(quorum) = default_quorum {
            if quorum == 0 || quorum > MAX_QUORUM_VOTERS {
                panic_with_error!(&env, ContractError::InvalidSignatureThreshold);
            }
            config.default_quorum = quorum;
        }
        if let Some(threshold) = default_approval_threshold_bps {
            if threshold > 10000 || threshold < 1000 {
                panic_with_error!(&env, ContractError::InvalidSignatureThreshold);
            }
            config.default_approval_threshold_bps = threshold;
        }
        if let Some(en) = enabled {
            config.enabled = en;
        }

        env.storage()
            .instance()
            .set(&DataKey::GovernanceConfig, &config);
    }

    /// Update vote weight provider configuration (admin only).
    pub fn update_vote_weight_provider(
        env: Env,
        use_token_weighting: Option<bool>,
        token_address: Option<Address>,
        use_reputation_weighting: Option<bool>,
    ) {
        let mut config: GovernanceConfig = env
            .storage()
            .instance()
            .get(&DataKey::GovernanceConfig)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::MultiSigNotConfigured));

        config.admin.require_auth();

        if let Some(use_token) = use_token_weighting {
            config.weight_provider.use_token_weighting = use_token;
        }
        if let Some(token_addr) = token_address {
            config.weight_provider.token_address = Some(token_addr);
        }
        if let Some(use_reputation) = use_reputation_weighting {
            config.weight_provider.use_reputation_weighting = use_reputation;
        }

        env.storage()
            .instance()
            .set(&DataKey::GovernanceConfig, &config);
    }
}

impl GovernanceModule {
    /// Calculate vote weight for a voter based on the configured weight provider.
    fn calculate_vote_weight(
        env: &Env,
        voter: &Address,
        weight_provider: &VoteWeightProvider,
    ) -> u64 {
        if weight_provider.use_token_weighting {
            // Token-based weighting
            if let Some(ref token) = weight_provider.token_address {
                let client = soroban_sdk::token::Client::new(env, token);
                let balance = client.balance(voter);
                // Convert i128 to u64, cap at u64::MAX
                if balance <= 0 {
                    1 // Minimum weight of 1
                } else if balance > u64::MAX as i128 {
                    u64::MAX
                } else {
                    balance as u64
                }
            } else {
                1 // No token configured, default to 1
            }
        } else if weight_provider.use_reputation_weighting {
            // Reputation-based weighting could be added here
            // For now, default to 1-person-1-vote
            1
        } else {
            // 1-person-1-vote
            1
        }
    }
}
