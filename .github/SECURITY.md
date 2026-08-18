# Security Policy

## Supported Versions

The following table shows which versions of EquipChain contracts currently receive security fixes.

| Version | Supported |
|---------|-----------|
| `main` (latest) | ✅ Yes |
| Older tags | ❌ No — please upgrade |

---

## Reporting a Vulnerability

**Do not open a public GitHub issue for security-sensitive findings.**

Public issues are visible to everyone, including potential attackers.  
Use one of the private reporting channels below instead.

### Option 1 — GitHub Private Security Advisory (preferred)

Open a private advisory directly in this repository:

👉 **[Report a vulnerability](https://github.com/EquipChain/EquipChain-contracts/security/advisories/new)**

GitHub keeps the advisory private until a fix is released. Only repository maintainers and the reporter have access.

### Option 2 — Email

Send your report to **security@equipchain.io**.

Use a descriptive subject line, e.g.:  
`[SECURITY] Critical issue in utility_contracts billing logic`

---

## What to Include in Your Report

Please provide enough information for maintainers to reproduce and assess the issue:

- **Affected contract(s) and function(s)** — e.g., `contracts/utility_contracts/src/lib.rs`, `claim()`
- **Impact** — describe the worst-case outcome (fund loss, access control bypass, denial of service, etc.)
- **Reproduction steps** — minimal steps or a proof-of-concept that demonstrates the issue
- **Suggested remediation** — if you have one (optional but appreciated)
- **Your contact information** — so we can follow up

Please **do not** include private keys, mnemonics, or credentials in your report.

---

## Bug Bounty

EquipChain operates a bug bounty program for the contracts in this repository.  
Reward ranges, response SLAs, and program scope are documented in the internal security policy.

**In scope:**
- `contracts/utility_contracts/src/` — core metering, billing, streaming, and governance logic
- `contracts/price_oracle/src/` — price feed contract

**Out of scope:**
- Third-party dependencies (Soroban SDK, Stellar network infrastructure)
- Phishing or social-engineering attacks
- Issues already publicly documented

Reward amounts are determined at maintainer discretion based on severity, impact, and report quality. Contact **security@equipchain.io** to confirm current reward ranges before submitting.

---

## Responsible Disclosure Policy

| Commitment | Details |
|------------|---------|
| Acknowledgement | Maintainers will acknowledge receipt of your report promptly |
| Initial assessment | An initial severity assessment will be provided as soon as possible |
| Fix timeline | Critical and High findings are prioritised for immediate remediation |
| Coordinated disclosure | We ask that you do not publish technical details until a fix is deployed and you have been notified |
| Credit | We will credit researchers in release notes unless anonymity is requested |

We follow responsible disclosure principles. We will not pursue legal action against researchers who:

- Report issues privately using one of the channels above
- Do not exploit the vulnerability beyond what is needed to demonstrate the issue
- Do not access, modify, or destroy data belonging to other users

---

## Severity Classification

| Severity | Definition | Examples |
|----------|------------|---------|
| **Critical** | Direct loss of funds, permanent loss of contract control | Drain of user/provider balances, admin key takeover |
| **High** | Temporary fund loss, service disruption, data corruption | Bypass of multi-sig, billing manipulation at scale |
| **Medium** | Limited financial impact, degraded functionality | Rate-limiting bypass, oracle staleness exploitation |
| **Low** | Minor inconvenience, information disclosure | Event spoofing with no financial impact |
| **Informational** | Best-practice recommendations | Code quality, gas optimisation |

---

## Contact

| Purpose | Contact |
|---------|---------|
| Security vulnerabilities | security@equipchain.io |
| General questions | GitHub Discussions or Issues |
| Audit inquiries | security@equipchain.io |

For the complete internal security documentation including the trust model, threat model, emergency runbooks, and role-based access control matrix, see [`docs/SECURITY.md`](../docs/SECURITY.md).
