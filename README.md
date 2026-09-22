# EquipChain Contracts

[![Rust Edition](https://img.shields.io/badge/rust-edition%202021-blue.svg)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)
[![Soroban SDK](https://img.shields.io/badge/soroban--sdk-23.2.4-6f26e0.svg)](https://crates.io/crates/soroban-sdk)
[![Stellar](https://img.shields.io/badge/Stellar-Soroban%20/%20wasm32-0f0f23.svg)](https://stellar.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](#license)
[![Soroban CI](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/ci.yml)
[![Utility Contract Tests](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/test.yml/badge.svg)](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/test.yml)
[![Test Coverage](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/test-coverage.yml/badge.svg)](https://github.com/EquipChain/EquipChain-contracts/actions/workflows/test-coverage.yml)

EquipChain is a decentralized equipment management platform that tracks registered metering assets across their full lifecycle — registration, verified usage reporting, ownership transfer, billing settlement, and per-asset maintenance accounting. The on-chain layer is a pair of Rust/Soroban smart contracts (`utility_contracts` + `price_oracle`) running on the Stellar network, with Ed25519-signed IoT payloads, streaming payments, and role-based access control enforced in-contract.

---

## Table of Contents

1. [Architecture & Modules](#architecture--modules)
2. [Prerequisites](#prerequisites)
3. [Getting Started & Development Workflow](#getting-started--development-workflow)
4. [Project Structure](#project-structure)
5. [Testing & CI](#testing--ci)
6. [Security & Audit Considerations](#security--audit-considerations)
7. [Documentation](#documentation)
8. [Contributing](#contributing)
9. [License](#license)

---

## Architecture & Modules

### System interaction flow

```
┌─────────────────────┐    ┌──────────────────────┐    ┌───────────────────────────┐
│  IoT device /        │    │  Client / Frontend    │    │  usage-dashboard          │
│  meter-simulator     │    │  (dApp, stellar-sdk)  │    │  (Next.js 14, read-only)  │
│  (Node.js, Ed25519)  │    │                       │    │                           │
└──────────┬──────────┘    └───────────┬──────────┘    └─────────────┬─────────────┘
           │ signed usage payload      │ invoke / simulate            │ query events,
           │ (update_usage, heartbeat) │ (register, claim, transfer)  │ ledger entries
           └─────────────┬────────────┴───────────────────────────────┘
                         ▼
           ┌───────────────────────────────┐
           │  Stellar RPC / Horizon        │  soroban-testnet.stellar.org
           │  (transaction submission,     │  soroban-rpc.stellar.org (pubnet)
           │   ledger reads, event stream) │  http://localhost:8000 (standalone)
           └───────────────┬───────────────┘
                           ▼
        ┌────────────────────────────────────────────────────────┐
        │  Soroban contracts (wasm32)                            │
        │                                                        │
        │  ┌──────────────────────────────┐  ┌─────────────────┐  │
        │  │ utility_contracts            │  │ price_oracle    │  │
        │  │ (equipment registry, usage,  │─▶│ (XLM/USD feed,  │  │
        │  │  billing, streams, govern.)  │  │  green source)  │  │
        │  └──────────────────────────────┘  └─────────────────┘  │
        └────────────────────────────────────────────────────────┘
```

### Equipment lifecycle flow

```
register_meter(user, provider, rate, token, device_public_key, priority)
   │      └─ user.require_auth(); validates rate > 0 and device key; returns meter_id
   │         (registration emits no event; on-chain activity events start with usage)
   ▼
update_usage / update_heartbeat ── Ed25519 verify + nonce sync (replay protection)
   │      └─ deltas buffered in temporary storage, flushed to ledger
   ▼
claim(meter_id) ── provider.require_auth()
   ├─ elapsed seconds × effective rate (time-of-use peak multiplier)
   ├─ 1 bps → per-meter maintenance fund        ← maintenance accounting
   ├─ tax split → government vault (if configured)
   ├─ carbon credits (renewable + verified green source)
   └─ net payout → provider (multi-sig for large withdrawals)
   ▼
transfer_meter_ownership(meter_id, new_user) ── user.require_auth()
   │      └─ lifecycle continues under new owner
   ▼
pause / resume / emergency_shutdown (dual authorization) / close
```

### Contract modules

| Concern | Module(s) | Key entrypoints |
|---|---|---|
| **Equipment (meter) registration** | `lib.rs` | `register_meter`, `register_meter_with_mode`, `register_with_referral` — binds a device Ed25519 public key, billing mode (prepaid/postpaid), tariff, and priority tier to a `meter_id` |
| **Verification & ownership transfer** | `lib.rs`, `nonce_sync.rs`, `tamper_detection.rs`, `zk_tests.rs` | `transfer_meter_ownership`, `assign_reseller`, hardware nonce sync, stale/future timestamp rejection, Groth16 ZK usage proofs |
| **Usage logging** | `lib.rs`, `multi_sensor.rs`, `temporary_storage.rs`, `energy_grid.rs` | `update_usage`, `update_heartbeat`, `record_consumption`, per-meter usage deltas with deferred (dust-aggregated) persistence |
| **Maintenance records** | `lib.rs` (`allocate_to_maintenance_fund`, `get_maintenance_fund_balance`) | 1 bps of every claim accrues to a per-meter maintenance fund; milestone-based payouts are covered by `tests/milestone_maintenance_fund_tests.rs` |
| **Billing & claims** | `lib.rs` | `claim`, `claim_with_alerts`, `submit_sla_report` — SLA penalties, credit drip rates, debt thresholds, hourly flow-rate caps |
| **Streaming payments** | `stream.rs`, `basket_stream.rs`, `auto_refill.rs`, `sep40_streaming.rs`, `ghost_sweeper.rs` | continuous streams with 24-hour buffer protection, SEP-40 streaming standard, ghost stream cleanup |
| **Pricing** | `tariff_oracle.rs`, `contracts/price_oracle` | time-of-use tariff schedules (peak 18:00–21:00 UTC at 1.5×), XLM/USD oracle with 5-minute staleness bound |
| **Enterprise & governance** | `enterprise.rs`, `Multi_Sig.rs`, `insurance_pool.rs`, `sbt_minter.rs`, `grant_stream_listener.rs` | fleet caps, P2P energy exchange, liveness slashing, multi-sig withdrawals, insurance proposals, soulbound impact tokens |
| **Hardening & ops** | `velocity_limit.rs`, `secure_call_interface.rs`, `gas_estimator.rs`, `gas_metrics.rs` | flash-drain protection, authenticated cross-contract calls, gas benchmarking |

Cross-contract interfaces are declared as `#[contractclient(...)]` traits in `lib.rs` (`PriceOracleClient`, `CarbonCreditMinterClient`), so the utility contract can be wired to a deployed oracle without code changes.

---

## Prerequisites

| Tool | Version / target | Purpose |
|---|---|---|
| [Rust](https://rustup.rs) + rustup | stable, target `wasm32-unknown-unknown` | compile contracts and the payload generator |
| [Stellar CLI](https://developers.stellar.org/docs/build/getting-started) | `stellar` ≥ v25.1.0 (what CI pins) | build, optimize, deploy, invoke (`soroban` CLI in older scripts is superseded by `stellar`) |
| [Node.js](https://nodejs.org) | ≥ 16 (dashboard: ≥ 18) | `meter-simulator` device mock and `usage-dashboard` |
| `jq` | any | used by `sanity_check.sh` and coverage checks |
| Docker | optional | `scripts/deploy.sh` pulls `stellar/quickstart` for local/testnet deployment |

Install the Rust target and CLI:

```sh
rustup target add wasm32-unknown-unknown

# Stellar CLI — any one of the official channels:
cargo install --locked stellar-cli            # Cargo (all platforms)
# curl -fsSL https://github.com/stellar/stellar-cli/install | sh   # installer
# brew install stellar-cli                    # macOS
```

---

## Getting Started & Development Workflow

### 1. Clone

```sh
git clone https://github.com/EquipChain/EquipChain-contracts.git
cd EquipChain-contracts
```

All contract commands run from the **`contracts/` workspace directory** (it has its own `Cargo.toml` workspace; the repo-root `Cargo.toml` is a separate mock IoT payload generator).

### 2. Compile

```sh
cd contracts

# Standard build (matches .github/workflows/ci.yml)
cargo build --target wasm32-unknown-unknown --release

# Or via the Stellar CLI (used by utility_contracts/Makefile)
stellar contract build
```

Artifacts:

| Build command | Output |
|---|---|
| `cargo build --target wasm32-unknown-unknown --release` | `contracts/target/wasm32-unknown-unknown/release/{utility_contracts,price_oracle}.wasm` |
| `stellar contract build` | `contracts/*/target/wasm32v1-none/release/*.wasm` |

Always optimize before deploying to a public network:

```sh
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/utility_contracts.wasm
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/price_oracle.wasm
```

### 3. Lint & test

```sh
cd contracts

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace                 # all tests, both crates
cargo test -p utility_contracts        # core contract suite only
cargo test -p price_oracle             # oracle suite only

# Filtered runs
cargo test -p utility_contracts -- buffer_tests
cargo test -p utility_contracts -- fuzz_tests
```

To reproduce the full CI pipeline locally from the repo root:

```sh
./validate-ci.sh        # fmt + clippy + wasm build + tests
```

### 4. Deploy

**Testnet** (get a funded identity first):

```sh
stellar keys generate admin --network testnet
stellar keys fund admin --network testnet

cd contracts
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/utility_contracts.wasm \
  --source admin --network testnet

stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/price_oracle.wasm \
  --source admin --network testnet
```

**Local standalone node** (via `stellar/quickstart`, or use `scripts/deploy.sh`):

```sh
stellar network add standalone \
  --rpc-url http://localhost:8000/soroban/rpc \
  --network-passphrase "Standalone Network ; February 2017"

cd contracts
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/utility_contracts.wasm \
  --source admin --network standalone
```

**Scripted deployment:**

```sh
./scripts/deploy.sh --network testnet      # or: --network mainnet --key "S..."
./sanity_check.sh                          # pre-flight load simulation (requires soroban/jq)
```

**Invoke an entrypoint:**

```sh
stellar contract invoke \
  --id <CONTRACT_ID> --network testnet --source admin \
  -- claim --meter_id 1
```

### 5. Contract addresses

| Contract | Network | Address |
|---|---|---|
| `utility_contracts` | Testnet | `CB7PSJZALNWNX7NLOAM6LOEL4OJZMFPQZJMIYO522ZSACYWXTZIDEDSS` |
| `price_oracle` | Testnet | *deploy separately from `contracts/price_oracle`* |

### 6. Off-chain tooling

```sh
# Mock IoT device: signs and submits usage payloads
cd meter-simulator && npm install && npm start

# Read-only monitoring dashboard
cd usage-dashboard && npm install && npm run dev
```

---

## Project Structure

```
EquipChain-contracts/
├── Cargo.toml                          # Root package: iot-payload-generator (mock Ed25519 signer)
├── main.rs                             # CLI that emits signed meter payload JSON
├── sanity_check.sh                     # Pre-flight load simulation (1k claims, pauses, admin rotation)
├── validate-ci.sh / validate-ci.ps1    # Run the CI pipeline locally
├── tarpaulin.toml                      # Coverage config (fail-under = 85%)
│
├── contracts/                          # *** Cargo workspace (resolver 2) ***
│   ├── Cargo.toml                      # members = [utility_contracts, price_oracle]; soroban-sdk = 23.2.4
│   ├── utility_contracts/
│   │   ├── Makefile                    # stellar contract build / test / fmt shortcuts
│   │   ├── src/
│   │   │   ├── lib.rs                  # Core contract (~9k lines): meters, billing, claims, transfers
│   │   │   ├── enterprise.rs           # Fleet caps, P2P exchange, liveness slashing
│   │   │   ├── tariff_oracle.rs        # Time-of-use tariff schedules
│   │   │   ├── energy_grid.rs          # Grid billing with peak/off-peak windows
│   │   │   ├── insurance_pool.rs       # Risk-scored pool, governance proposals
│   │   │   ├── Multi_Sig.rs            # Multi-sig withdrawals & config changes
│   │   │   ├── nonce_sync.rs           # Hardware nonce sync (replay protection)
│   │   │   ├── tamper_detection.rs     # Device tamper signals
│   │   │   ├── stream.rs / basket_stream.rs / auto_refill.rs
│   │   │   ├── sep40_streaming.rs      # SEP-40 streaming standard
│   │   │   ├── ghost_sweeper.rs        # Abandoned stream cleanup
│   │   │   ├── velocity_limit.rs       # Flash-drain protection
│   │   │   ├── secure_call_interface.rs# Authenticated cross-contract calls
│   │   │   ├── temporary_storage.rs    # In-memory usage deltas, dust aggregation
│   │   │   ├── gas_estimator.rs / gas_metrics.rs
│   │   │   ├── sbt_minter.rs           # Soulbound impact tokens
│   │   │   ├── grant_stream_listener.rs# Conservation-goal stream automation
│   │   │   └── *_tests.rs              # Unit / property / fuzz test modules
│   │   ├── tests/                      # Integration tests: e2e, SLA, ZK, firmware, maintenance milestones
│   │   ├── fuzz/                       # cargo-fuzz targets (velocity_limit_fuzz, …)
│   │   └── test_snapshots/             # soroban-cli test snapshots
│   └── price_oracle/                   # Standalone XLM/USD oracle (admin/updater roles, 5-min staleness)
│
├── meter-simulator/                    # Node.js ESP32 mock: signs & submits usage over MQTT/SDK
├── usage-dashboard/                    # Next.js 14 monitoring dashboard (read-only)
├── examples/                           # Standalone examples (insurance_pool_demo.rs)
├── scripts/                            # deploy.sh (stellar/quickstart), doc validation
├── docs/                               # Architecture, security, migration, audit docs
└── .github/workflows/                  # ci.yml, test.yml, test-coverage.yml
```

---

## Testing & CI

The workspace carries unit tests, property tests (`proptest`), and randomized/fuzz suites:

| Suite | Location | Focus |
|---|---|---|
| Core billing | `src/test.rs` | Meter lifecycle, claims, top-ups |
| Buffer system | `src/buffer_tests.rs` | Stream creation, depletion, refunds |
| Pause/resume | `src/pause_resume_tests.rs`, `src/pause_resume_fuzz_tests.rs` | State transitions |
| Debt & stroop fuzz | `src/debt_fuzz_tests.rs`, `src/stroop_fuzz_tests.rs` | Overflow/underflow, fixed-point math |
| Ghost sweeper | `src/ghost_sweeper_tests.rs` | Stream cleanup |
| Nonce sync | `src/nonce_sync_tests.rs` | Device replay protection |
| Insurance | `src/insurance_pool_test.rs` | Pool, claims, governance |
| Streaming invariants | `src/streaming_invariant_tests.rs`, `src/stream_balance_property_tests.rs` | Flow consistency, property-based |
| Tariff oracle | `src/tariff_oracle_tests.rs` | Schedule management |
| Temporary storage | `src/temporary_storage_tests.rs` | Flush & aggregation |
| Integration | `tests/e2e_integration_test.rs`, `tests/sla_penalty_tests.rs`, `tests/zk_privacy_tests.rs`, `tests/milestone_maintenance_fund_tests.rs`, `tests/firmware_update_tests.rs`, … | Cross-module scenarios |
| Fuzz targets | `fuzz/fuzz_targets/` | `cargo-fuzz` harnesses |

CI (`.github/workflows/`):

- **`ci.yml` — Soroban CI:** `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, then a `wasm32-unknown-unknown` release build uploaded as an artifact.
- **`test.yml` — Utility Contract Tests:** installs Stellar CLI v25.1.0, clippy against the wasm target, builds + optimizes both `.wasm` artifacts, runs unit tests.
- **`test-coverage.yml` — Test Coverage Check:** `cargo tarpaulin` with an **85% coverage gate** (`tarpaulin.toml`), report uploaded to Codecov.

---

## Security & Audit Considerations

### Authorization patterns

Every state-changing entrypoint gates on Soroban's `require_auth()` against the owning `Address`:

```rust
// claim: only the meter's provider may settle
meter.provider.require_auth();

// ownership transfer: only the current user
meter.user.require_auth();

// oracle updates: only the whitelisted updater
updater.require_auth();

// cross-contract invocation: contract-to-contract authorization
env.current_contract_address().require_auth();
```

- **Role-based access control:** DAO Admin, Compliance Officer, Grid Administrator, Finance Wallets (multi-sig), Provider, User — each administrative path checks the role-specific address (see `docs/SECURITY.md` §5).
- **Multi-sig & timelocks:** large provider withdrawals require finance-wallet quorum (`Multi_Sig.rs`); admin changes go through `initiate_admin_transfer` → `execute_admin_transfer` with a timelock; upgrades have a community veto window.
- **Dual authorization:** emergency meter shutdown requires admin **and** provider consent.

### Device data integrity

- **Ed25519 signature verification** of device-reported usage against the `device_public_key` bound at registration.
- **Hardware nonce sync** (`nonce_sync.rs`) rejects replayed heartbeats; submissions older than 24 h or future-dated are rejected.
- **Velocity limits** and hourly flow-rate caps bound extraction per ledger hour.

### Events for off-chain indexing

All state transitions emit topics via `env.events().publish(...)` across dozens of call sites, e.g.:

```rust
// verified from set_meter_pause in lib.rs:
env.events().publish(
    (symbol_short!("Paused"), meter_id), // topic
    paused,                              // payload (bool)
);
```

Indexers and the `usage-dashboard` subscribe to these topics rather than polling storage. Compact `u16` `IoTErrorCode` variants are also emitted/returned so firmware can automate recovery without parsing human-readable errors.

### Defensive arithmetic & failure modes

- Balance math uses `saturating_sub` / `saturating_add` — no overflow panics on ledger values.
- Self-referencing guards prevent the contract from being set as its own admin, oracle, or vault.
- Reentrancy guards on stream create/pause/resume/buffer operations.
- Typed `ContractError` panics instead of bare string panics in hot paths.

### Audit resources

| Document | Contents |
|---|---|
| [`docs/SECURITY.md`](docs/SECURITY.md) | Trust model, threat model, RBAC, emergency procedures, bug bounty |
| [`docs/AUDIT.md`](docs/AUDIT.md) | Audit readiness checklist, coverage, known issues |
| [`docs/CONTRACT_ARCHITECTURE.md`](docs/CONTRACT_ARCHITECTURE.md) | System architecture, data flow, storage layout |
| [`docs/MIGRATION_GUIDE.md`](docs/MIGRATION_GUIDE.md) | Version history and upgrade procedures |

Emergency response (entrypoints verified against `lib.rs`):

```sh
# Compliance freeze a single meter (requires the Compliance Officer key)
stellar contract invoke --id $CONTRACT_ID --network testnet --source $COMPLIANCE \
  -- legal_freeze --meter_id <ID> --reason "investigation"

# Pause / resume a meter (requires the meter user's auth)
stellar contract invoke --id $CONTRACT_ID --network testnet --source $USER \
  -- set_meter_pause --meter_id <ID> --paused true

# Emergency shutdown (requires BOTH provider and admin authorization)
stellar contract invoke --id $CONTRACT_ID --network testnet --source $ADMIN \
  -- emergency_shutdown --meter_id <ID>
```

---

## Documentation

See the [docs/](docs) directory, plus [`meter-simulator/`](meter-simulator) for device integration and [`usage-dashboard/`](usage-dashboard) for monitoring.

---

## Contributing

1. Fork and branch from `main`.
2. Keep changes scoped; follow existing module boundaries in `contracts/utility_contracts/src/`.
3. Before opening a PR, run the local pipeline:

   ```sh
   ./validate-ci.sh        # fmt + clippy (-D warnings) + wasm build + tests
   ```

4. Ensure `cargo test --workspace` passes and coverage stays ≥ 85%.
5. Open a pull request targeting `main`; CI must be green on all three workflows.

---

## License

Released under the MIT License (see the `License` field of [`meter-simulator/package.json`](meter-simulator/package.json) and each crate's manifest; an SPDX `LICENSE` file is pending addition).
