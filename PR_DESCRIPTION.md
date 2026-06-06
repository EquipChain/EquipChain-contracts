# docs: DAO emergency runbook â€” circuit breaker, Wasm upgrade & state migration

## Summary

Adds `EMERGENCY_RUNBOOK.md` â€” a comprehensive, actionable emergency operations guide for the Equipchain DAO covering every worst-case failure scenario with exact CLI commands.

## Changes

- `EMERGENCY_RUNBOOK.md` â€” new file (1,217 lines)

## What's included

| Section | Coverage |
|---|---|
| Roles & Responsibilities | DAO Admin, Compliance Officer, Finance Wallets, Oracle, Provider |
| Pre-Incident Checklist | Environment verification before any emergency action |
| Scenario A â€” Active Exploit | `challenge_service`, `emergency_shutdown`, velocity override revocation, cancel pending withdrawals |
| Scenario B â€” Protocol Pause | Per-meter and per-stream pause/resume, global velocity limiting |
| Scenario C â€” Wasm Hash Upgrade | Build â†’ upload â†’ propose â†’ veto window â†’ finalize â†’ verify â†’ rollback |
| Scenario D â€” State Migration | Pause â†’ dump â†’ deploy migration contract â†’ migrate â†’ diff verify â†’ transfer balances |
| Scenario E â€” Multi-Sig Freeze | Cancel request, revoke approval, reconfigure after wallet compromise |
| Scenario F â€” Legal Freeze | Freeze meter, verify, release with council multi-sig, rotate compliance officer |
| Scenario G â€” Gas Buffer Exhaustion | Check balance, top up, initialize, withdraw excess |
| Scenario H â€” Admin Key Compromise | Initiate transfer, DAO veto window, execute, rotate dependent keys |
| Scenario I â€” Oracle Failure | Diagnose, update oracle address, resolve downstream challenges |
| Scenario J â€” Velocity Limit Breach | Apply override, tighten limits, revoke override |
| Post-Incident Procedures | Evidence preservation, challenge resolution, key rotation, 72-hour post-mortem |
| Multi-Sig Signer Reference Card | Standalone guide for Finance Wallet holders â€” full lifecycle + pre-approval checklist |
| Contact Tree | P1â€“P4 escalation matrix with response time targets |

## Acceptance criteria

- [x] Actionable, step-by-step emergency procedures with exact `stellar contract invoke` commands
- [x] Multi-sig signers have a clear understanding of their technical duties (Section 14 â€” standalone reference card)
- [x] Covers all worst-case failure scenarios including admin key compromise, oracle failure, flash drain, and state migration

## Labels

`documentation` `security` `devops`

## Reviewers

Assign: DAO Admin, at least one Finance Wallet holder, Security Lead
