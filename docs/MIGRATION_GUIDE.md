# EquipChain Contracts — Migration Guide

> **Last Updated:** 2026-07-25  
> **Audience:** Developers & DAO Administrators

---

## Table of Contents

1. [Why `lib_original.rs` vs `lib.rs`?](#1-why-lib_originalrs-vs-librs)
2. [Secure Call Interface Versions](#2-secure-call-interface-versions)
3. [Migration Paths](#3-migration-paths)
4. [Upgrade History Table](#4-upgrade-history-table)
5. [How to Deploy an Upgrade](#5-how-to-deploy-an-upgrade)
6. [State Migration Considerations](#6-state-migration-considerations)

---

## 1. Why `lib_original.rs` vs `lib.rs`?

The repository contains two library entry points:

### `lib_original.rs` — Original Implementation

This file represents the **baseline** contract implementation. It contains a minimal, clean version of `UtilityContract` with:

- Core `Meter` struct (11 fields)
- Basic `DataKey` enum (5 variants)
- Simple `ContractError` enum (15 errors)
- Foundational functions: `register_meter`, `register_meter_with_mode`, `claim`, `deduct_units`, `top_up`, `update_usage`
- Pairing system (`initiate_pairing`, `complete_pairing`)
- Withdrawal with XLM/USD conversion

### `lib.rs` — Current Implementation

This is the **active** contract entry point. It evolved from `lib_original.rs` through iterative feature additions:

- Greatly expanded `Meter` struct (45+ fields) with device-offline grace period, SLA penalties, billing groups, firmware updates, green energy credits
- Expanded `DataKey` enum (100+ variants) covering reputation, clawback, nonce sync, tariff oracle, ghost sweeper, emergency drain, upgrade multi-sig
- Expanded `ContractError` enum (116 errors)
- Added modules: `enterprise`, `ghost_sweeper`, `nonce_sync`, `tariff_oracle`, `secure_call_interface`, `velocity_limit`, `temporary_storage`
- ZK-proof structures, IoT error codes, post-paid escrow, conservation goals

**Key difference:** `lib_original.rs` is **not compiled** into the current WASM binary. It is preserved as a reference for understanding the evolution of the codebase and for audit trail purposes.

---

## 2. Secure Call Interface Versions

### Version 1: `secure_call_interface_old.rs` (Legacy)

The original implementation had the following characteristics:

- **Call depth type:** `u8` (max 255, but limited to 5 in practice)
- **Error handling:** Used `Status` enum with exhaustive (but redundant) error mapping via `match`
- **Admin detection:** Used `Address::generate(&env)` as a sentinel key — this was **flawed** because it generates a random address each time rather than storing the actual admin
- **Rate limiting:** Basic manual reset logic
- **Initialization check:** Checked for existence of the admin sentinel key

### Version 2: `secure_call_interface.rs` (Current)

The refactored implementation addressed these issues:

- **Call depth type:** `u32` (increased range, better aligned with Soroban conventions)
- **Error handling:** Removed the giant `Status` match — uses cleaner `Result<_, SecureCallError>` propagation
- **Admin detection:** Uses `require_initialized()` function that checks for `LastCallReset` key — more reliable
- **Rate limiting:** Cleaner logic with proper window resets
- **Generic return type:** `secure_call_inner<T>` with proper `TryFromVal` bound for type-safe returns
- **Removed `#[contractimpl]`:** The manager struct no longer requires contract macro, making it usable as a library module
- **Added admin config storage:** Stores admin config separately with proper tracking

### Migration from Old to New

```diff
- use secure_call_interface_old::SecureCallManager;
+ use secure_call_interface::SecureCallManager;
```

No breaking API changes — the trait interface remained identical. The internal improvements are fully backward-compatible.

---

## 3. Migration Paths

### 3.1 From Original → Current (Code)

If you are working from `lib_original.rs`:

1. **Import the module system:**
   ```rust
   pub mod enterprise;
   pub mod ghost_sweeper;
   pub mod nonce_sync;
   pub mod secure_call_interface;
   pub mod tariff_oracle;
   pub mod temporary_storage;
   pub mod velocity_limit;
   ```

2. **Expand data types** — Update `Meter`, `DataKey`, `ContractError` to include current variants

3. **Update helper functions** — Replace `get_effective_rate(meter, timestamp)` signature (now takes `env` as first param)

4. **Add new constants** — Buffer, ghost stream, emergency drain, upgrade timelock constants

5. **Update tests** — Test module structure changed to use sub-modules with `#[cfg(test)]`

### 3.2 State Migration (On-Chain)

When upgrading an already-deployed contract:

1. **Prepare a migration contract** that reads old state and writes to new keys
2. **Deploy the new WASM hash** via the upgrade multi-sig process
3. **Run the migration function** (if included) or use off-chain scripts to re-write state
4. **Verify all data** is correctly migrated before enabling new features

> **Important:** The `DataKey` enum variants must remain binary-compatible. Never reorder or delete variants. Only append new variants.

---

## 4. Upgrade History Table

| Version | Date | Key Changes | Author |
|---------|------|-------------|--------|
| 0.0.0 (Original) | 2026-Q1 | Initial `lib_original.rs`: basic metering, pairing, oracle conversion | Core Team |
| 0.1.0 | 2026-Q1 | Variable-rate tariffs: peak/off-peak rates, `is_peak_hour()` | Core Team |
| 0.2.0 | 2026-Q2 | Buffer vault system: 24h mandatory buffer, auto-depletion | Core Team |
| 0.3.0 | 2026-Q2 | Multi-sig provider withdrawal: 3-of-5 finance wallets | Core Team |
| 0.4.0 | 2026-Q2 | ZK-proof structures for private usage reporting | Core Team |
| 0.5.0 | 2026-Q2 | Firmware update authorization gate (Issue #178) | Core Team |
| 0.6.0 | 2026-Q3 | Fleet caps, P2P energy exchange (Issues #248-#251) | Core Team |
| 0.7.0 | 2026-Q3 | Carbon credit minter integration (Issue #252) | Core Team |
| 0.8.0 | 2026-Q3 | Post-paid multi-factor escrow (Issue #255) | Core Team |
| 0.9.0 | 2026-Q3 | SAC clawback reconciliation (Issue #256) | Core Team |
| 0.10.0 | 2026-Q3 | IoT error codes, byte array validation (Issues #257, #279) | Core Team |
| 0.11.0 | 2026-Q3 | Storage optimization & streaming invariant tests | Core Team |
| 0.12.0 | 2026-Q3 | Temporary storage optimization module | Core Team |
| 0.13.0 | 2026-Q3 | Energy-score reputation adapter (Issue #259) | Core Team |
| 0.14.0 | 2026-Q3 | Hardware nonce sync (Issue #260) | Core Team |
| 0.15.0 | 2026-Q3 | Utility-tariff oracle (Issue #261) | Core Team |
| 0.16.0 | 2026-Q3 | Ghost stream sweeper (Issue #262) | Core Team |
| 0.17.0 | 2026-Q3 | Secure call interface v2 refactor | Core Team |
| 0.18.0 | 2026-Q3 | Velocity limit circuit breaker | Core Team |
| 0.19.0 | 2026-Q3 | Upgrade multi-sig (48h timelock) | Core Team |
| 0.20.0 | 2026-Q3 | Emergency drain recovery (Issue #277) | Core Team |
| 0.21.0 | 2026-Q3 | Flow rate boundary validation (Issue #273) | Core Team |

---

## 5. How to Deploy an Upgrade

### 5.1 Multi-Sig Upgrade Process

The contract uses an **upgrade multi-sig** system for WASM upgrades:

1. **Propose:** Any authorized signer calls `propose_upgrade(new_wasm_hash)`
2. **Approve:** Other signers call `approve_upgrade(proposal_id)`
3. **Timelock:** After threshold is reached, a 48-hour timelock begins
4. **Execute:** After timelock expires, any signer can call `execute_upgrade(proposal_id)`

### 5.2 Pre-Upgrade Checklist

- [ ] All tests pass (`cargo test`)
- [ ] WASM builds successfully (`cargo build --target wasm32-unknown-unknown --release`)
- [ ] New `DataKey` variants are appended (never reordered)
- [ ] New `ContractError` variants are appended
- [ ] State migration tested on testnet first
- [ ] Multi-sig signers are available for quorum

### 5.3 Post-Upgrade Verification

```bash
# Verify contract is responsive
stellar contract invoke --id $CONTRACT --network testnet -- get_count

# Verify key state variables
stellar contract invoke --id $CONTRACT --network testnet -- get_meter --meter-id 1

# Check upgrade proposal was executed
stellar contract invoke --id $CONTRACT --network testnet -- get_upgrade_proposal --proposal-id 1
```

---

## 6. State Migration Considerations

### Storage Compatibility

The `DataKey` enum is serialized via Soroban SDK's built-in XDR encoding. **Do not reorder variants** — the contract reads storage by the ordinal position of each variant. Append new variants at the end.

### Persistent vs Temporary Storage

- **Persistent storage** (`env.storage().instance()`) survives upgrades
- **Temporary storage** (`env.storage().temporary()`) has TTL — may be lost between upgrades if TTL expires
- **Contract code** is replaced entirely on WASM upgrade; only storage persists

### Data Expiration

Soroban entries have TTL (time-to-live). After upgrades, ensure that:
- Critical persistent data has sufficient TTL remaining
- New entries are created with adequate TTL
- The auto-extend mechanism is configured

---

*This document is part of the EquipChain Contracts technical documentation suite.*
