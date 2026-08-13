# Changelog

## [2.1.0] - 2026-08-13

### Changed
- Upgraded to rmcp 3.1.2 and raised the minimum supported Rust version to 1.94.1.
- Added MCP 2026-07-28 stateless request handling while retaining MCP 2025-11-25 initialization compatibility.

### Added
- Per-request identity and protocol metadata, on-demand discovery/cache hints, and the configured Tasks and sealed MRTR approval policies.

## [2.0.0] - 2025-05-24

### Changed
- Complete rewrite from Stripe wrapper to governed financial operations MCP
- Intent-based model replaces direct payment execution

### Added
- 10 tools: lookup, list, status, checkout/refund/payout intents, approval, execute, reconcile, evidence
- Policy engine with amount thresholds and approval gates
- PaymentIntent domain model with Money (i64 minor units)
- Evidence artifacts and reconciliation
- Idempotency keys on all writes
- mcp-server.toml registry manifest

### Removed
- Direct Stripe API wrapper (replaced by intent-based governance)
