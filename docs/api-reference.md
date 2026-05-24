# API Reference

All 10 tools exposed by `mcp-payments`. Every write tool requires `actor`, `reason`, and `idempotency_key`.

---

## Read Operations

### `lookup_payment`

Look up a payment intent by ID.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `payment_id` | string | ✅ | The payment intent ID |

**Returns:** Full `PaymentIntent` object or error if not found.

**Risk class:** `read_only`

---

### `list_customer_payments`

List all payment intents for a customer.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `customer_id` | string | ✅ | The customer/counterparty ID |

**Returns:** Array of `PaymentIntent` objects.

**Risk class:** `read_only`

---

### `get_payment_status`

Get the current status and reconciliation state of a payment intent.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `payment_id` | string | ✅ | The payment intent ID |

**Returns:** `{ id, status, reconciliation }` or error if not found.

**Risk class:** `read_only`

---

## Intent Creation

### `create_checkout_intent`

Create an inbound checkout payment intent. Status starts as `Draft`. Amounts > 10,000 minor units automatically trigger `RequiresApproval` policy.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `amount_minor` | i64 | ✅ | Amount in minor units (e.g., 5000 = $50.00) |
| `currency` | string | ✅ | ISO 4217 currency code (e.g., "usd") |
| `customer_id` | string | ✅ | Customer identifier |
| `customer_name` | string | ✅ | Customer display name |
| `actor` | string | ✅ | Who/what initiated this action |
| `reason` | string | ✅ | Business justification |
| `idempotency_key` | string | ✅ | Unique key to prevent duplicates |

**Returns:** Full `PaymentIntent` with policy decision attached.

**Risk class:** `internal_write`

**Policy rules:**
- Amount ≤ 10,000 minor units → `Allowed`
- Amount > 10,000 minor units → `RequiresApproval` (finance_manager)

---

### `create_refund_intent`

Create an outbound refund intent for an existing payment. Always requires approval.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `original_payment_id` | string | ✅ | ID of the payment being refunded |
| `amount_minor` | i64 | ✅ | Refund amount in minor units |
| `currency` | string | ✅ | ISO 4217 currency code |
| `actor` | string | ✅ | Who/what initiated this action |
| `reason` | string | ✅ | Business justification for refund |
| `idempotency_key` | string | ✅ | Unique key to prevent duplicates |

**Returns:** Full `PaymentIntent` (direction=Outbound, type=Refund).

**Risk class:** `financial_action`

**Policy rules:**
- All refunds → `RequiresApproval` (finance_manager)

---

### `create_payout_intent`

Create an outbound payout intent. Always requires dual approval.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `amount_minor` | i64 | ✅ | Payout amount in minor units |
| `currency` | string | ✅ | ISO 4217 currency code |
| `counterparty_id` | string | ✅ | Recipient identifier |
| `counterparty_name` | string | ✅ | Recipient display name |
| `counterparty_type` | enum | ✅ | One of: `customer`, `vendor`, `partner`, `employee`, `marketplace_seller` |
| `actor` | string | ✅ | Who/what initiated this action |
| `reason` | string | ✅ | Business justification for payout |
| `idempotency_key` | string | ✅ | Unique key to prevent duplicates |

**Returns:** Full `PaymentIntent` (direction=Outbound, type=Payout).

**Risk class:** `financial_action`

**Policy rules:**
- All payouts → `RequiresApproval` (finance_manager + compliance_officer)

---

## Governance & Execution

### `request_payment_approval`

Submit a Draft payment intent for approval. Transitions status from `Draft` → `PendingApproval`.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `payment_id` | string | ✅ | The payment intent ID |
| `actor` | string | ✅ | Who is requesting approval |
| `reason` | string | ✅ | Why approval is being requested |
| `idempotency_key` | string | ✅ | Unique key to prevent duplicates |

**Returns:** Updated `PaymentIntent` with status=PendingApproval.

**Risk class:** `internal_write`

**Preconditions:**
- Intent must be in `Draft` status

---

### `execute_approved_intent`

Execute a payment intent that has been approved. Only works if status is `Approved`. Transitions `Approved` → `Executing` → `Captured`.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `payment_id` | string | ✅ | The payment intent ID |
| `actor` | string | ✅ | Who is triggering execution |
| `reason` | string | ✅ | Execution context |
| `idempotency_key` | string | ✅ | Unique key to prevent duplicates |

**Returns:** Updated `PaymentIntent` with status=Captured.

**Risk class:** `financial_action`

**Preconditions:**
- Intent must be in `Approved` status
- Fails with error if status is anything else

---

## Reconciliation & Evidence

### `reconcile_payment`

Reconcile a payment against an external reference (order, invoice). Compares amounts and sets reconciliation status.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `payment_id` | string | ✅ | The payment intent ID |
| `reference_id` | string | ✅ | External reference (order ID, invoice ID) |
| `reference_amount_minor` | i64 | ✅ | Expected amount from the reference |
| `actor` | string | ✅ | Who is performing reconciliation |
| `reason` | string | ✅ | Reconciliation context |
| `idempotency_key` | string | ✅ | Unique key to prevent duplicates |

**Returns:** `{ id, reconciliation, reference_id }` — reconciliation is `Matched` if amounts equal, `Discrepancy` otherwise.

**Risk class:** `internal_write`

---

### `attach_payment_evidence`

Attach an evidence artifact to a payment intent for audit trail.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `payment_id` | string | ✅ | The payment intent ID |
| `artifact_type` | enum | ✅ | One of: `receipt`, `invoice`, `order_snapshot`, `ledger_snapshot`, `policy_log`, `support_transcript`, `contract_evidence` |
| `uri` | string | ✅ | URI reference to the artifact (e.g., `s3://bucket/receipt.pdf`) |
| `actor` | string | ✅ | Who is attaching evidence |
| `reason` | string | ✅ | Why this evidence is relevant |
| `idempotency_key` | string | ✅ | Unique key to prevent duplicates |

**Returns:** The created `EvidenceArtifact` object.

**Risk class:** `internal_write`

---

## Enums Reference

### `IntentStatus`

```
Draft → PendingApproval → Approved → Executing → Captured → Settled
                                                → Failed
                       → Cancelled
```

### `Direction`

| Value | Description |
|-------|-------------|
| `inbound` | Money coming in (checkout, invoice) |
| `outbound` | Money going out (refund, payout) |

### `IntentType`

| Value | Description |
|-------|-------------|
| `checkout` | Customer payment capture |
| `refund` | Return funds to customer |
| `payout` | Send funds to counterparty |
| `invoice` | Invoice-based collection |
| `subscription` | Recurring charge |
| `settlement` | Marketplace settlement |
| `escrow` | Held funds |
| `credit` | Account credit |

### `CounterpartyType`

| Value | Description |
|-------|-------------|
| `customer` | End customer |
| `vendor` | Supplier/vendor |
| `partner` | Business partner |
| `employee` | Internal employee |
| `marketplace_seller` | Marketplace participant |

### `EvidenceType`

| Value | Description |
|-------|-------------|
| `receipt` | Payment receipt |
| `invoice` | Invoice document |
| `order_snapshot` | Order state at time of payment |
| `ledger_snapshot` | Ledger state snapshot |
| `policy_log` | Policy evaluation log |
| `support_transcript` | Customer support record |
| `contract_evidence` | Contract or agreement |

### `ReconciliationStatus`

| Value | Description |
|-------|-------------|
| `pending` | Not yet reconciled |
| `matched` | Amounts match reference |
| `discrepancy` | Amounts do not match |
| `unreconciled` | Cannot be reconciled |

### `PolicyDecisionResult`

| Value | Description |
|-------|-------------|
| `allowed` | Intent can proceed without approval |
| `requires_approval` | Human approval required before execution |
| `blocked` | Intent is not permitted |
