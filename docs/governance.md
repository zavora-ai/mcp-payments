# Governance Model

`mcp-payments` enforces a governance model that prevents AI agents from executing financial operations without appropriate oversight. This document describes the policy evaluation, approval flows, execution gates, and audit requirements.

## Core Principle

**Agents propose, policy decides, humans approve, system executes.**

No financial action can bypass the governance pipeline. The system is designed so that even a compromised or hallucinating agent cannot move money without passing through policy evaluation and (where required) human approval.

---

## Policy Evaluation

Every intent creation triggers automatic policy evaluation. The Policy Engine examines the intent and returns one of three decisions:

| Decision | Meaning | Next Step |
|----------|---------|-----------|
| `Allowed` | Intent is within safe parameters | Can proceed to execution |
| `RequiresApproval` | Intent needs human sign-off | Must go through approval queue |
| `Blocked` | Intent violates hard rules | Cannot proceed |

### Current Policy Rules

| Rule | Trigger | Decision | Required Approvers |
|------|---------|----------|-------------------|
| Checkout threshold | `amount_minor > 10000` | RequiresApproval | finance_manager |
| Refund (any) | All refund intents | RequiresApproval | finance_manager |
| Payout (any) | All payout intents | RequiresApproval | finance_manager, compliance_officer |
| Low-value checkout | `amount_minor ≤ 10000` | Allowed | — |

### Policy Decision Structure

```rust
pub struct PolicyDecision {
    pub decision: PolicyDecisionResult,  // Allowed | RequiresApproval | Blocked
    pub reasons: Vec<String>,            // Human-readable explanation
    pub required_approvals: Vec<String>, // Role identifiers needed
}
```

The policy decision is attached to the `PaymentIntent` at creation time and is immutable — it cannot be changed after the fact.

---

## Approval Flows

### Single Approval (Refunds, High-Value Checkouts)

```
Agent creates intent
  → Policy: RequiresApproval [finance_manager]
  → Agent calls request_payment_approval
  → Status: PendingApproval
  → Finance manager reviews and approves (external to MCP)
  → Status: Approved
  → Agent calls execute_approved_intent
  → Status: Captured
```

### Dual Approval (Payouts)

```
Agent creates payout intent
  → Policy: RequiresApproval [finance_manager, compliance_officer]
  → Agent calls request_payment_approval
  → Status: PendingApproval
  → Finance manager approves
  → Compliance officer approves
  → Status: Approved
  → Agent calls execute_approved_intent
  → Status: Captured
```

### No Approval Needed (Low-Value Checkouts)

```
Agent creates checkout intent (≤ 10000 minor units)
  → Policy: Allowed
  → Status: Draft
  → Agent calls execute_approved_intent
  → ERROR: Only Approved intents can be executed
```

Note: Even "Allowed" intents must go through the approval step to reach `Approved` status. The policy decision indicates whether human review is *required*, but the status machine still enforces the flow.

---

## Execution Gates

The `execute_approved_intent` tool enforces a hard gate:

```
if intent.status != Approved {
    return Error("Only Approved intents can be executed")
}
```

This means:
- `Draft` intents cannot be executed (must request approval first)
- `PendingApproval` intents cannot be executed (must be approved first)
- `Executing` intents cannot be re-executed (already in progress)
- `Captured`/`Settled` intents cannot be re-executed (already complete)
- `Failed`/`Cancelled` intents cannot be executed (terminal states)

### Status Machine

```
Draft
  → PendingApproval (via request_payment_approval)
  → Cancelled (via external cancellation)

PendingApproval
  → Approved (via external approval)
  → Cancelled (via external rejection)

Approved
  → Executing (via execute_approved_intent)

Executing
  → Captured (on success)
  → Failed (on failure)

Captured
  → Settled (on settlement confirmation)
```

---

## Amount Thresholds

| Threshold | Value | Effect |
|-----------|-------|--------|
| Auto-approval ceiling | 10,000 minor units | Checkouts above this require finance_manager approval |
| Refund threshold | Any amount | All refunds require approval regardless of amount |
| Payout threshold | Any amount | All payouts require dual approval regardless of amount |

Thresholds are evaluated at intent creation time. The policy decision is recorded and cannot be retroactively changed.

---

## Audit Requirements

### Required Fields on Every Write

| Field | Purpose |
|-------|---------|
| `actor` | Identifies who or what initiated the action (e.g., "agent:billing-bot", "user:james@co.com") |
| `reason` | Business justification (e.g., "Customer requested refund for defective item") |
| `idempotency_key` | Prevents duplicate operations from retries or agent loops |

### Audit Trail

Every `PaymentIntent` maintains:

- `created_by` — the actor who created the intent
- `created_at` — ISO 8601 timestamp
- `approvals[]` — list of approval decisions with approver, decision, and timestamp
- `evidence[]` — attached artifacts (receipts, invoices, policy logs)
- `policy` — the original policy decision (immutable)

### Evidence Artifacts

Evidence can be attached at any point in the intent lifecycle:

| Type | When to Attach |
|------|---------------|
| `receipt` | After execution (proof of payment) |
| `invoice` | At creation (source document) |
| `order_snapshot` | At creation (order state at time of intent) |
| `ledger_snapshot` | At reconciliation (ledger state) |
| `policy_log` | At policy evaluation (decision rationale) |
| `support_transcript` | For refunds (customer interaction record) |
| `contract_evidence` | For payouts (contractual basis) |

---

## Reconciliation

After execution, payments should be reconciled against their source records:

```
reconcile_payment(
    payment_id: "pay_123",
    reference_id: "ORD-2025-001",
    reference_amount_minor: 5000,
    ...
)
```

The system compares `intent.amount.amount_minor` against `reference_amount_minor`:

| Comparison | Result |
|-----------|--------|
| Amounts match | `Matched` |
| Amounts differ | `Discrepancy` |

Discrepancies flag the payment for manual review. Unreconciled payments remain in `Pending` status until reconciled.

---

## Security Considerations

- **No credential exposure** — payment provider credentials are vault references, never passed through MCP tools
- **No PII in responses** — counterparty details use IDs and display names, not sensitive data
- **Idempotency prevents replay** — duplicate calls with the same key are safe
- **Status machine prevents bypass** — there is no path from Draft to Captured without going through Approved
- **Policy is immutable** — once attached to an intent, the policy decision cannot be altered
