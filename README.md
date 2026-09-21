# EquipChain Contracts

[![Soroban CI](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/ci.yml)
[![Utility Contract Tests](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/test.yml/badge.svg)](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/test.yml)

A Soroban smart contract suite for decentralized utility metering, billing, and streaming payments on the Stellar network.

Built with [Soroban SDK v23.2.4](https://soroban.stellar.org).

---

## Overview

EquipChain enables decentralized utility management through on-chain metering and real-time payment streaming. Utility providers register IoT meters on-chain, and consumers pay for energy, water, or gas usage via continuous payment streams with time-of-use pricing.

**Key capabilities:**

- Continuous payment streams with 24-hour buffer protection
- Variable-rate time-of-use billing (peak hours 18:00–21:00 UTC at 1.5x rates)
- Prepaid and postpaid meter models with collateral management
- Zero-knowledge proof verification for privacy-preserving usage reporting
- P2P energy exchange with grid fee collection
- Enterprise fleet management with priority tiers and load shedding
- Multi-sig governance for provider withdrawals and upgrades
- Ghost stream cleanup for ledger efficiency

## Quick Start

```bash
# Prerequisites: Rust toolchain with wasm32 target, Stellar CLI
rustup target add wasm32-unknown-unknown

# Build
cd contracts
cargo build --target wasm32-unknown-unknown --release

# Test
cargo test --workspace

# Deploy to testnet
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/utility_contracts.wasm \
  --network testnet
```

## Contract Addresses

| Contract | Network | Address |
|----------|---------|---------|
| UtilityContract | Testnet | `CB7PSJZALNWNX7NLOAM6LOEL4OJZMFPQZJMIYO522ZSACYWXTZIDEDSS` |
| PriceOracle | Testnet | *Deploy separately from `contracts/price_oracle`* |

## Architecture

```
┌───────────────────────────────────────────────────────────┐
│                   UtilityContract (lib.rs)                  │
│  177 public functions — meters, billing, streams, claims   │
│                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Enterprise   │  │ Tariff       │  │ Insurance Pool   │  │
│  │ (fleet caps, │  │ Oracle       │  │ (risk scoring,   │  │
│  │  P2P, slash) │  │ (24h rates)  │  │  governance)     │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
│                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Nonce Sync   │  │ Ghost        │  │ Velocity Limit   │  │
│  │ (replay      │  │ Sweeper      │  │ (flash drain     │  │
│  │  protection) │  │ (cleanup)    │  │  protection)     │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
│                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Secure Call  │  │ Gas          │  │ Temporary        │  │
│  │ Interface    │  │ Estimator    │  │ Storage          │  │
│  └─────────────┘  └──────────────┘  └──────────────────┘  │
└───────────────────────────────────────────────────────────┘
```

## Project Structure

```
EquipChain-contracts/
├── contracts/
│   ├── Cargo.toml                          # Workspace: utility_contracts + price_oracle
│   ├── utility_contracts/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                      # Core contract (177 public functions)
│   │       ├── enterprise.rs               # Fleet caps, P2P exchange, liveness slashing
│   │       ├── tariff_oracle.rs            # Time-of-use tariff schedule management
│   │       ├── insurance_pool.rs           # Insurance pool, risk scoring, proposals
│   │       ├── energy_grid.rs              # Grid billing with peak/off-peak windows
│   │       ├── nonce_sync.rs               # Hardware nonce synchronization
│   │       ├── ghost_sweeper.rs            # Abandoned stream cleanup
│   │       ├── velocity_limit.rs           # Flash drain protection
│   │       ├── secure_call_interface.rs    # Authenticated cross-call interface
│   │       ├── gas_estimator.rs            # Gas cost estimation
│   │       ├── gas_metrics.rs              # Gas consumption benchmarking
│   │       ├── temporary_storage.rs        # In-memory storage optimization
│   │       ├── grant_stream_listener.rs    # Conservation goal automation
│   │       ├── sbt_minter.rs               # Soulbound token minting
│   │       ├── stream.rs                   # Stream data structures
│   │       ├── asset.rs                    # Asset management
│   │       ├── auto_refill.rs              # Auto-refill logic
│   │       ├── basket_stream.rs            # Basket stream types
│   │       ├── multi_sensor.rs             # Multi-sensor support
│   │       ├── oracle_flow.rs              # Oracle integration
│   │       ├── sep40_streaming.rs          # SEP-40 streaming standard
│   │       ├── tamper_detection.rs          # Tamper detection
│   │       ├── vault_interface.rs          # Vault interface
│   │       ├── Multi_Sig.rs                # Multi-signature operations
│   │       └── [14 test files]             # 178 test functions
│   └── price_oracle/
│       ├── Cargo.toml
│       └── src/                            # Price oracle contract
├── meter-simulator/                        # Mock IoT device for testing
├── examples/                               # Usage examples
├── scripts/                                # Deployment scripts
├── usage-dashboard/                        # Monitoring dashboard
├── docs/
│   ├── CONTRACT_ARCHITECTURE.md            # System architecture
│   ├── SECURITY.md                         # Trust model and emergency procedures
│   ├── MIGRATION_GUIDE.md                  # Version history and upgrades
│   └── AUDIT.md                            # Audit readiness checklist
└── .github/workflows/
    ├── ci.yml                              # Build and lint
    ├── test.yml                            # Unit and fuzz tests
    └── test-coverage.yml                   # Coverage reporting
```

## Features

### Metering & Billing

- **Prepaid meters** — balance deducted per second of usage at the configured rate
- **Postpaid meters** — collateral-capped debt with settlement on claim
- **Time-of-use pricing** — peak hours (18:00–21:00 UTC) apply a 1.5x rate multiplier
- **SLA penalties** — automated penalty application for provider downtime
- **Credit drip rates** — gradual debt settlement for postpaid consumers
- **Tax compliance** — configurable tax rates split to a government vault before provider payout
- **Carbon credits** — renewable energy providers earn credits on verified green usage

### Continuous Streaming

- **Buffer protection** — streams require a 24-hour buffer deposit to prevent dry streams
- **Automatic buffer tapping** — seamless transition from main balance to buffer when depleted
- **Amicable closure** — full buffer refund to the payer when streams are closed cooperatively
- **Platform fees** — configurable basis-point fee deducted per accumulation cycle
- **Dust aggregation** — deferred persistence for small-balance operations to reduce storage costs

### Enterprise

- **Fleet caps** — per-provider active stream rate limits to manage infrastructure load
- **P2P energy exchange** — peer-to-peer trading with configurable grid fee collection
- **Liveness slashing** — heartbeat-based monitoring with proportional buffer slashing
- **Priority tiers** — Critical / High / Standard / Low tiers for grid load shedding

### Security

- **Ed25519 signature verification** — device-reported usage is cryptographically authenticated
- **Hardware nonce sync** — replay attack protection for IoT device heartbeats
- **Zero-knowledge proofs** — Groth16 SNARKs for privacy-preserving usage verification
- **Reentrancy guards** — on stream create, pause, resume, and buffer operations
- **Velocity limits** — configurable thresholds for flash drain protection
- **Emergency shutdown** — dual-authorization (admin + provider) for meter termination

### Governance

- **Multi-sig withdrawals** — finance wallet quorum for large provider payouts
- **Upgrade veto system** — time-locked WASM upgrade proposals with community veto period
- **Insurance pool** — risk-scored membership with governance proposals and claims
- **Conservation goals** — provider-funded water/energy savings with automated stream grants
- **Asset voting** — community polling for supported token additions

## How It Works

### Claim Settlement

When a provider claims usage from a meter, the settlement follows this flow:

```
claim(meter_id)
  │
  ├─ Calculate elapsed consumption (seconds × effective_rate)
  ├─ Allocate 1 bps to maintenance fund
  ├─ Split tax to government vault (if configured)
  ├─ Issue carbon credits (if renewable + verified green source)
  ├─ Deduct platform fee (with green energy discount if applicable)
  └─ Transfer net payout to provider
```

### Time-of-Use Pricing

```
Peak:  18:00–21:00 UTC → off_peak_rate × 1.5
Off-peak: all other hours → off_peak_rate

Integer math: peak_rate = off_peak_rate × 3 / 2
```

### Buffer Protection

```
Stream creation → 24-hour buffer deposit required
During streaming → main balance consumed first, then buffer
Buffer warning → emitted when 1 hour of buffer remains
Stream ends → automatic termination when buffer depleted
Amicable close → full buffer refund to payer
```

## Security Model

### Roles

| Role | Authority |
|------|-----------|
| DAO Admin | System configuration, upgrade proposals, emergency controls |
| Compliance Officer | Legal freeze trigger and release |
| Finance Wallets (3–5) | Multi-sig approval for large provider withdrawals |
| Grid Administrator | Tariff schedule management, load shedding |
| Provider | Meter lifecycle management, stream control |
| User | Own meter operations, usage reporting |

### Safety Properties

- All balance arithmetic uses `saturating_sub`/`saturating_add` to prevent overflow panics
- Admin functions require role-specific authorization
- Stream operations validate active meter status before state changes
- Usage signatures reject both stale (>24h) and future-dated submissions
- Maximum flow rate caps prevent excessive hourly claims
- Self-referencing guards prevent the contract from being set as its own admin, oracle, or vault

### Incident Response

```bash
# Emergency freeze all streams
stellar contract invoke --id $CONTRACT --network testnet --source $ADMIN_KEY \
  -- emergency_freeze_all_streams

# Emergency shutdown a meter
stellar contract invoke --id $CONTRACT --network testnet --source $ADMIN_KEY \
  -- emergency_shutdown --meter-id <ID>
```

Full incident response runbook: [`docs/SECURITY.md`](docs/SECURITY.md)

## Testing

178 test functions across 14 test files, including fuzz tests for arithmetic edge cases.

```bash
# Run all tests
cargo test --workspace

# Run specific suites
cargo test --package utility_contracts -- test
cargo test --package utility_contracts -- buffer_tests
cargo test --package utility_contracts -- fuzz_tests
```

| Test Suite | File | Coverage |
|------------|------|----------|
| Core billing | `test.rs` | Meter lifecycle, claims, top-ups |
| Buffer system | `buffer_tests.rs` | Creation, depletion, refunds |
| Pause/Resume | `pause_resume_tests.rs` | Stream state transitions |
| Pause/Resume fuzz | `pause_resume_fuzz_tests.rs` | Randomized state testing |
| Debt handling | `debt_fuzz_tests.rs` | Overflow/underflow edge cases |
| Stroop precision | `stroop_fuzz_tests.rs` | Fixed-point math accuracy |
| General fuzz | `fuzz_tests.rs` | Randomized meter operations |
| Ghost sweeper | `ghost_sweeper_tests.rs` | Stream cleanup |
| Nonce sync | `nonce_sync_tests.rs` | Device nonce verification |
| Insurance | `insurance_pool_test.rs` | Pool, claims, governance |
| Streaming invariants | `streaming_invariant_tests.rs` | Flow consistency |
| Tariff oracle | `tariff_oracle_tests.rs` | Schedule management |
| Temporary storage | `temporary_storage_tests.rs` | Flush, aggregation |
| Stream balance | `stream_balance_property_tests.rs` | Property-based testing |

## Documentation

| Document | Description |
|----------|-------------|
| [`docs/CONTRACT_ARCHITECTURE.md`](docs/CONTRACT_ARCHITECTURE.md) | System architecture, data flow, storage layout |
| [`docs/SECURITY.md`](docs/SECURITY.md) | Trust model, emergency procedures, incident response |
| [`docs/MIGRATION_GUIDE.md`](docs/MIGRATION_GUIDE.md) | Version history, migration paths, upgrade procedures |
| [`docs/AUDIT.md`](docs/AUDIT.md) | Audit readiness checklist, test coverage, known issues |

## Contributing

1. Create a feature branch from `main`
2. Make changes with clear commit messages
3. Ensure `cargo test --workspace` passes
4. Submit a pull request targeting `main`

## License

MIT
