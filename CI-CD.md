# CI/CD Pipeline for Equipchain Contracts

This document describes the automated testing pipeline implemented for the Equipchain Contracts project.

## ðŸ”„ Workflow Overview

The GitHub Actions workflow (`.github/workflows/test.yml`) automatically runs on:
- **Push to main branch** - Ensures main branch is always tested
- **Pull Requests to main** - Prevents breaking changes from being merged

## âœ… Testing Stages

### 1. Environment Setup
- **Rust Toolchain**: Installs stable Rust with WASM target
- **Stellar CLI**: Installs Stellar CLI v25.1.0 for contract interaction
- **Dependency Caching**: Caches Cargo dependencies for faster builds

### 2. Code Quality Checks
- **Formatting**: `cargo fmt --all -- --check` ensures consistent code formatting
- **Linting**: `cargo clippy --target wasm32-unknown-unknown -- -D warnings` catches potential issues

### 3. Build & Test
- **WASM Build**: `cargo build --target wasm32-unknown-unknown --release` builds smart contract
- **Unit Tests**: `cargo test` runs all unit tests including fuzz tests
- **Fuzz Tests**: Detects and validates fuzz testing infrastructure

## ðŸ§ª Fuzz Testing Integration

The pipeline includes automatic detection of fuzz tests:
- Checks for `contracts/utility_contracts/fuzz/` directory
- Installs `cargo-fuzz` if fuzz tests are present
- Validates fuzz testing infrastructure availability

## ðŸ“Š Test Coverage

### Current Test Suites
1. **Unit Tests**: Standard contract functionality tests
2. **Fuzz Tests**: 
   - Debt calculation underflow protection
   - Extreme usage scenarios
   - Balance handling edge cases
   - Arithmetic overflow protection

### Acceptance Criteria Validation
- âœ… Workflow runs on push to main
- âœ… `cargo test` passes successfully  
- âœ… Code formatting validated
- âœ… Clippy linting passes
- âœ… WASM build succeeds
- âœ… Fuzz tests infrastructure available

## ðŸ”§ Pipeline Configuration

### Environment Variables
- `CARGO_TERM_COLOR: always` - Ensures colored output in logs

### Build Matrix
- **OS**: Ubuntu Latest (ubuntu-latest)
- **Target**: wasm32-unknown-unknown (for Soroban contracts)
- **Rust Version**: Stable with required components

## ðŸ“ˆ Pipeline Benefits

1. **Prevents Breaking Changes**: Every PR is automatically tested
2. **Code Quality**: Enforces formatting and linting standards
3. **Fast Feedback**: Caching and parallel execution provide quick results
4. **Comprehensive Testing**: Unit + fuzz testing coverage
5. **WASM Compatibility**: Ensures contracts build for target platform

## ðŸš€ Usage

### Automatic Execution
- No manual intervention required
- Tests run automatically on git events
- Results displayed in GitHub Actions UI

### Local Development
```bash
# Run same tests locally
cargo fmt --all -- --check
cargo clippy --target wasm32-unknown-unknown -- -D warnings
cargo build --target wasm32-unknown-unknown --release
cargo test

# Run fuzz tests (if available)
cd contracts/utility_contracts/fuzz
cargo fuzz run debt_calculation_fuzz -- -max_total_time 30
```

## ðŸ“‹ Test Results Summary

The pipeline generates a summary in GitHub Actions including:
- âœ… Unit tests status
- âœ… Clippy linting status  
- âœ… Code formatting status
- âœ… WASM build status
- âœ… Fuzz tests availability

This ensures every pull request maintains code quality and prevents regressions in smart contract logic.
