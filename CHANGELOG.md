# Changelog

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
