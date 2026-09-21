# EquipChain Contracts

[![Soroban CI](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/ci.yml)
[![Utility Contract Tests](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/test.yml/badge.svg)](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/test.yml)
[![Test Coverage](https://img.shields.io/badge/coverage-%3E85%25-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()
[![Soroban](https://img.shields.io/badge/Soroban-23.2.4-blue)]()

A Soroban smart contract suite for decentralized utility metering, billing, and streaming on the Stellar network. Supports variable-rate tariffs, device nonce sync, ghost stream cleanup, zero-knowledge privacy proofs, and enterprise-grade multi-sig governance.

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Contract Addresses](#contract-addresses)
- [Core Features](#core-features)
- [Module Structure](#module-structure)
- [Key Concepts](#key-concepts)
- [Security Model](#security-model)
- [Incident Response](#incident-response)
- [Testing](#testing)
- [Documentation](#documentation)

---

## Overview

EquipChain is a decentralized utility management platform that enables:

- **Real-time streaming payments** for energy, water, and gas consumption
- **Variable-rate time-of-use pricing** with peak/off-peak billing
- **Prepaid and postpaid billing** models with collateral management
- **Zero-knowledge proofs** for privacy-preserving usage verification
- **Continuous flow streams** with buffer protection against dry streams
- **P2P energy exchange** with grid fee collection
- **Carbon credit integration** for renewable energy providers
- **Insurance pools** with governance proposals and risk assessment
- **Ghost stream cleanup** to maintain ledger efficiency
- **Hardware nonce synchronization** for IoT device liveness

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    EquipChain Contract                     │
│                                                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Metering  │  │Billing   │  │Streaming │  │Governance│  │
│  │ & IoT     │  │Engine    │  │Payments  │  │& MultiSig│  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       │              │              │              │        │
│  ┌────┴──────────────┴──────────────┴──────────────┴────┐  │
│  │              Core Contract (lib.rs)                    │  │
│  │  • Meter registration & lifecycle                     │  │
│  │  • Peak/off-peak rate calculation                     │  │
│  │  • Claim settlement with tax & fee splitting          │  │
│  │  • Buffer management & dust aggregation               │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │Enterprise│  │Tariff    │  │Insurance │  │ZK Privacy│    │
│  │Fleet Mgmt│  │Oracle    │  │Pool      │  │& Groth16 │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
└──────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Prerequisites: Rust, Stellar CLI
rustup target add wasm32-unknown-unknown

# Build contracts
cd contracts
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test --workspace

# Deploy (testnet)
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/utility_contracts.wasm \
  --network testnet
```

## Contract Addresses

| Contract | Network | Address |
|----------|---------|---------|
| UtilityContract | Testnet | `CB7PSJZALNWNX7NLOAM6LOEL4OJZMFPQZJMIYO522ZSACYWXTZIDEDSS` |
| PriceOracle | Testnet | *(Deploy separately via `price_oracle` package)* |
| UtilityContract | Mainnet | TBD |

## Core Features

### Metering & Billing
- **Prepaid & Postpaid billing** with collateral limits and debt tracking
- **Variable-rate time-of-use pricing** — peak hours (18:00–21:00 UTC) apply 1.5x rates
- **Tax compliance** — configurable tax rates with government vault splitting
- **Platform fees** with green energy discounts
- **SLA penalties** for provider downtime
- **Credit drip rates** for postpaid meter credit settlement

### Continuous Streaming
- **Buffer-protected streams** requiring 24-hour buffer deposits
- **Automatic buffer tapping** when main balance depletes
- **Amicable closure** with full buffer refunds
- **Platform fee deduction** on each accumulation cycle
- **Dust aggregation** for cost-efficient small-balance cleanup

### Enterprise Features
- **Fleet cap management** — per-provider active stream rate limits
- **P2P energy exchange** — peer-to-peer trading with grid fee collection
- **Liveness checking & slashing** — heartbeat-based device monitoring
- **Priority tier system** — Critical, High, Standard, Low for grid load shedding

### Security & Privacy
- **Zero-knowledge proofs** — Groth16 SNARKs for privacy-preserving usage verification
- **Hardware nonce sync** — replay attack protection for IoT devices
- **Ed25519 signature verification** — device-reported usage authentication
- **Reentrancy guards** on critical stream operations
- **Velocity limits** — flash drain protection with configurable thresholds
- **Emergency shutdown** — dual-authorization meter termination

### Governance
- **Multi-sig withdrawal** — finance wallet quorum for large provider payouts
- **Upgrade veto system** — time-locked WASM upgrade proposals
- **Insurance pool** — risk-scored membership with governance proposals
- **Conservation goals** — provider-funded water/energy savings targets
- **Asset voting** — community polling for supported tokens

## Module Structure

```
contracts/utility_contracts/src/
├── lib.rs                      # Core contract — meters, billing, streams, claims
├── enterprise.rs               # Fleet caps, P2P exchange, liveness/slashing, priority tiers
├── gas_estimator.rs            # Gas cost estimation for meter operations
├── insurance_pool.rs           # Insurance pool, risk scoring, governance proposals
├── energy_grid.rs              # Energy grid billing with peak/off-peak windows
├── tariff_oracle.rs            # Time-of-use tariff schedule management
├── grant_stream_listener.rs    # Conservation goal stream automation
├── temporary_storage.rs        # In-memory storage for cost optimization
├── optimized_flow_calculator.rs# High-performance flow accumulation math
├── sbt_minter.rs               # Soulbound token minting for achievements
└── [test files]                # Comprehensive test suites including fuzz tests
```

## Key Concepts

### Time-of-Use Tariff System

```
Peak Hours: 18:00–21:00 UTC (64,800–75,600 seconds)
Rate: off_peak_rate × 1.5 (stored as ×3/2 for integer math)

UTC Hour | Seconds  | Status
---------|----------|--------
00:00    | 0        | Off-peak
12:00    | 43,200   | Off-peak
18:00    | 64,800   | Peak ★
21:00    | 75,600   | Off-peak
23:59    | 86,399   | Off-peak
```

### Claim Settlement Flow

```
claim() → elapsed × effective_rate
  ├── allocate_to_maintenance_fund (1 bps)
  ├── calculate_tax_split (configurable rate)
  │   └── transfer to government vault
  ├── issue_carbon_credits (if renewable)
  ├── apply protocol fee (with green discount)
  └── transfer net payout to provider
```

### Buffer Protection

```
Stream Creation
  ├── Required: 24-hour buffer deposit
  ├── Buffer isolation (separate from balance)
  └── Warning at 1 hour remaining

During Streaming
  ├── Main balance consumed first
  ├── Automatic buffer activation
  └── Termination when buffer depleted

Amicable Closure
  ├── Full buffer refund to payer
  └── No refund after natural depletion
```

## Security Model

### Access Control Hierarchy

| Role | Authority |
|------|-----------|
| DAO Admin | System configuration, upgrade proposals, emergency controls |
| Compliance Officer | Legal freeze trigger/release |
| Finance Wallets (3–5) | Multi-sig provider withdrawals |
| Grid Administrator | Tariff schedule, load shedding |
| Provider | Meter lifecycle, stream management |
| User | Own meter operations, usage reporting |

### Security Measures

- **Saturating arithmetic** throughout — prevents integer overflow/underflow panics
- **Reentrancy guards** on stream create, pause, resume, and buffer operations
- **Input validation** on all public functions (zero-address, bounds, state checks)
- **Event emission** on all state-changing operations for audit trail
- **Token whitelist** enforcement (test mode bypass for development)
- **Gas buffer system** for provider transaction cost pre-payment

### Known Security Properties

1. All balance operations use `saturating_sub`/`saturating_add` to prevent panics
2. Admin functions require dual authorization (admin + role-specific)
3. Stream operations validate active status before state changes
4. Usage signatures reject both stale and future-dated submissions
5. Maximum flow rate caps prevent excessive hourly claims

## Incident Response

### Quick Reference

```bash
# Emergency freeze all streams
stellar contract invoke --id $CONTRACT --network testnet --source $ADMIN_KEY \
  -- emergency_freeze_all_streams

# Emergency shutdown a meter (dual auth required)
stellar contract invoke --id $CONTRACT --network testnet --source $ADMIN_KEY \
  -- emergency_shutdown --meter-id <ID>

# Pause nonce verification
stellar contract invoke --id $CONTRACT --network testnet --source $GRID_ADMIN_KEY \
  -- pause_nonce_verification

# Lock tariff oracle
stellar contract invoke --id $CONTRACT --network testnet --source $GRID_ADMIN_KEY \
  -- emergency_lock_tariff_oracle
```

### Full Runbook

See [`docs/SECURITY.md`](docs/SECURITY.md) for the complete audit-ready incident response runbook covering all emergency scenarios.

## Testing

```bash
# Run all tests
cargo test --workspace

# Run specific test suites
cargo test --package utility_contracts -- test
cargo test --package utility_contracts -- buffer_tests
cargo test --package utility_contracts -- fuzz_tests
```

### Test Coverage

| Area | Test File | Coverage |
|------|-----------|----------|
| Core billing | `test.rs` | Meter lifecycle, claim, top-up |
| Buffer system | `buffer_tests.rs` | Creation, depletion, refunds |
| Pause/Resume | `pause_resume_tests.rs` | Stream state transitions |
| Debt handling | `debt_fuzz_tests.rs` | Overflow/underflow edge cases |
| Gas metrics | `gas_metrics.rs` | Gas consumption benchmarks |
| Insurance | `insurance_pool_test.rs` | Pool, claims, governance |
| Temporary storage | `temporary_storage_tests.rs` | Flush, aggregation |
| Streaming invariants | `streaming_invariant_tests.rs` | Flow consistency |
| Stroop precision | `stroop_fuzz_tests.rs` | Fixed-point math |
| Security | `secure_call_tests.rs` | Auth enforcement |

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/CONTRACT_ARCHITECTURE.md`](docs/CONTRACT_ARCHITECTURE.md) | System architecture, data flow, storage layout |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Trust model, emergency procedures, bug bounty |
| [`docs/MIGRATION_GUIDE.md`](docs/MIGRATION_GUIDE.md) | Version history, migration paths, upgrade procedures |
| [`docs/AUDIT.md`](docs/AUDIT.md) | Audit readiness checklist, known issues |

## License

MIT
