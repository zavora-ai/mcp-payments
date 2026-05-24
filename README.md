# MCP Payments

[![Crates.io](https://img.shields.io/crates/v/mcp-payments.svg)](https://crates.io/crates/mcp-payments)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Governed financial operations for AI agents. Payment intents, approval gates, evidence, reconciliation. **Agents propose — policy decides — humans approve — system executes.**

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-payments/main/docs/assets/architecture.svg" alt="MCP Payments Architecture" width="800"/>
</p>

## Key Principles

- **Intent-based** — agents create payment *intents*, never execute payments directly
- **No direct execution** — every financial action passes through policy evaluation and approval gates
- **i64 money** — all amounts in minor units (cents/pence), no floating point, no rounding errors
- **No secrets** — credentials stay in vault references, never in tool parameters or responses
- **Idempotency** — every write tool requires an `idempotency_key` to prevent duplicate operations
- **Audit trail** — every action records `actor`, `reason`, and timestamp for compliance

## Design Philosophy

### Why this isn't a Stripe wrapper

Traditional payment MCP servers expose raw API calls: `charge_card(amount, token)`. This is dangerous for AI agents because:

1. **No governance** — an agent hallucination can trigger a real charge
2. **No approval** — high-value transactions execute without human review
3. **No audit** — no record of *why* a payment was made or *who* authorized it
4. **No reconciliation** — payments exist in isolation, disconnected from business context

`mcp-payments` solves this with an **intent-based governance model**:

- Agents express *intent* ("I want to charge $50 for order #123")
- Policy engine evaluates the intent against configurable rules
- High-risk intents queue for human approval
- Only approved intents can be executed
- Every payment links to evidence and reconciles against source records

The agent never touches money directly. It proposes, the system governs.

## Tools (10)

### Read Operations (3)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `lookup_payment` | Look up a payment intent by ID | Read-only |
| `list_customer_payments` | List all payments for a customer | Read-only |
| `get_payment_status` | Get current status and reconciliation state | Read-only |

### Intent Creation (3)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `create_checkout_intent` | Create inbound checkout intent (Draft) | Internal write |
| `create_refund_intent` | Create outbound refund intent | Financial action |
| `create_payout_intent` | Create outbound payout intent | Financial action |

### Governance & Execution (2)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `request_payment_approval` | Submit Draft → PendingApproval | Internal write |
| `execute_approved_intent` | Execute only if status=Approved | Financial action |

### Reconciliation & Evidence (2)

| Tool | Purpose | Risk Class |
|------|---------|------------|
| `reconcile_payment` | Match payment against order/invoice reference | Internal write |
| `attach_payment_evidence` | Attach receipt, invoice, or audit artifact | Internal write |

## Governance Rules

Every financial operation enforces these controls:

- **Actor identification** — every write requires `actor` field identifying who/what initiated
- **Reason logging** — every write requires `reason` explaining business justification
- **Idempotency** — every write requires `idempotency_key` preventing duplicate execution
- **Amount thresholds** — checkout intents > 10,000 minor units automatically require approval
- **Refund approval** — all refunds require `finance_manager` approval
- **Payout approval** — all payouts require `finance_manager` + `compliance_officer` approval
- **Execution gate** — `execute_approved_intent` only works on `Approved` status intents
- **Status machine** — intents follow Draft → PendingApproval → Approved → Executing → Captured
- **Reconciliation** — payments must reconcile against source records (Matched/Discrepancy)
- **Evidence chain** — audit artifacts attach to intents for compliance review

## Data Model

```rust
pub struct PaymentIntent {
    pub id: String,
    pub direction: Direction,          // Inbound | Outbound
    pub intent_type: IntentType,       // Checkout | Refund | Payout | ...
    pub status: IntentStatus,          // Draft → PendingApproval → Approved → Captured
    pub amount: Money,
    pub counterparty: Counterparty,
    pub rail: Option<PaymentRail>,
    pub policy: Option<PolicyDecision>,
    pub approvals: Vec<Approval>,
    pub evidence: Vec<EvidenceArtifact>,
    pub reconciliation: ReconciliationStatus,
    pub reason: String,
    pub created_by: String,
    pub created_at: String,
    pub idempotency_key: String,
}

pub struct Money {
    pub amount_minor: i64,             // Always minor units (cents)
    pub currency: String,              // ISO 4217 (e.g., "usd")
}

pub struct PolicyDecision {
    pub decision: PolicyDecisionResult, // Allowed | RequiresApproval | Blocked
    pub reasons: Vec<String>,
    pub required_approvals: Vec<String>,
}

pub struct EvidenceArtifact {
    pub id: String,
    pub artifact_type: EvidenceType,   // Receipt | Invoice | OrderSnapshot | ...
    pub uri: String,                   // Reference to artifact storage
}
```

## Usage Examples

### Create a checkout (auto-approved, under threshold)

```
"Charge customer cust_123 $50 for their subscription renewal"
→ create_checkout_intent(amount_minor: 5000, currency: "usd", customer_id: "cust_123", ...)
→ Policy: Allowed (under 10000 threshold)
→ Status: Draft (ready for execution)
```

### High-value triggers approval

```
"Process a $500 equipment purchase for customer cust_456"
→ create_checkout_intent(amount_minor: 50000, currency: "usd", ...)
→ Policy: RequiresApproval (exceeds threshold)
→ request_payment_approval(payment_id: "...", ...)
→ Status: PendingApproval (waiting for finance_manager)
```

### Execute after approval

```
"The finance manager approved payment pay_789, execute it"
→ execute_approved_intent(payment_id: "pay_789", ...)
→ Status: Captured
```

### Reconcile against order

```
"Match payment pay_789 against order ORD-2025-001"
→ reconcile_payment(payment_id: "pay_789", reference_id: "ORD-2025-001", reference_amount_minor: 50000, ...)
→ Reconciliation: Matched
```

## Installation

```bash
cargo install mcp-payments
```

Or build from source:

```bash
git clone https://github.com/zavora-ai/mcp-payments
cd mcp-payments
cargo build --release
```

## Client Configuration

### Claude Desktop

```json
{
  "mcpServers": {
    "payments": {
      "command": "mcp-payments",
      "args": [],
      "env": {}
    }
  }
}
```

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "payments": {
      "command": "mcp-payments",
      "args": [],
      "env": {}
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "payments": {
      "command": "mcp-payments",
      "args": [],
      "env": {}
    }
  }
}
```

## Integration

`mcp-payments` is designed to work within the ADK-Rust Enterprise ecosystem:

| System | Integration |
|--------|-------------|
| **ADK-Payments** | Backend execution engine — `mcp-payments` creates intents, ADK-Payments processes them |
| **Credentials Vault** | Payment provider credentials stored as vault references, never in MCP context |
| **Governance Policy** | External policy rules feed into the Policy Engine for threshold and approval decisions |
| **Artifacts Store** | Evidence artifacts (receipts, invoices) stored externally, referenced by URI |

## Documentation

| Document | Description |
|----------|-------------|
| [API Reference](docs/api-reference.md) | All 10 tools with parameters, types, and examples |
| [Governance](docs/governance.md) | Policy evaluation, approval flows, execution gates |
| [Architecture](docs/assets/architecture.svg) | System diagram |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [mcp-server.toml](mcp-server.toml) | ADK-Rust Enterprise registry manifest |

## MCP Server Manifest

```toml
[server]
name = "mcp-payments"
version = "2.0.0"
description = "Governed Payment Operations MCP"
category = "finance"
risk_level = "high"
writes_allowed = "gated"
transports = ["stdio"]
credentials = ["vault://payment-credentials"]
governance_gates = ["amount_threshold", "refund_approval", "payout_approval"]
```

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and credentials
- **Structured tracing** — `RUST_LOG` env-filter for observability

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)
