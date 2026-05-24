use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use rmcp::model::{CallToolResult, Content};

#[allow(unused_imports)]
use crate::domain::*;
use crate::store::PaymentStore;

// --- Input structs ---

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct LookupPaymentInput { pub payment_id: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListCustomerPaymentsInput { pub customer_id: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetPaymentStatusInput { pub payment_id: String }

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateCheckoutIntentInput {
    pub amount_minor: i64,
    pub currency: String,
    pub customer_id: String,
    pub customer_name: String,
    pub actor: String,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateRefundIntentInput {
    pub original_payment_id: String,
    pub amount_minor: i64,
    pub currency: String,
    pub actor: String,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreatePayoutIntentInput {
    pub amount_minor: i64,
    pub currency: String,
    pub counterparty_id: String,
    pub counterparty_name: String,
    pub counterparty_type: CounterpartyType,
    pub actor: String,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct RequestPaymentApprovalInput {
    pub payment_id: String,
    pub actor: String,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct ExecuteApprovedIntentInput {
    pub payment_id: String,
    pub actor: String,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct ReconcilePaymentInput {
    pub payment_id: String,
    pub reference_id: String,
    pub reference_amount_minor: i64,
    pub actor: String,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[allow(dead_code)]
pub struct AttachPaymentEvidenceInput {
    pub payment_id: String,
    pub artifact_type: EvidenceType,
    pub uri: String,
    pub actor: String,
    pub reason: String,
    pub idempotency_key: String,
}

// --- Server ---

#[derive(Clone)]
pub struct PaymentServer {
    pub store: PaymentStore,
}

fn now() -> String { chrono::Utc::now().to_rfc3339() }
fn new_id() -> String { uuid::Uuid::new_v4().to_string() }

fn ok_json(v: &impl serde::Serialize) -> Result<CallToolResult, rmcp::ErrorData> {
    Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(v).unwrap())]))
}

fn err_result(msg: &str) -> Result<CallToolResult, rmcp::ErrorData> {
    Ok(CallToolResult::error(vec![Content::text(msg)]))
}

#[tool_router(server_handler)]
impl PaymentServer {
    #[tool(description = "Look up a payment intent by ID")]
    async fn lookup_payment(&self, Parameters(input): Parameters<LookupPaymentInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        match self.store.get(&input.payment_id).await {
            Some(p) => ok_json(&p),
            None => err_result("Payment not found"),
        }
    }

    #[tool(description = "List all payments for a customer")]
    async fn list_customer_payments(&self, Parameters(input): Parameters<ListCustomerPaymentsInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let payments = self.store.list_by_counterparty(&input.customer_id).await;
        ok_json(&payments)
    }

    #[tool(description = "Get the current status of a payment intent")]
    async fn get_payment_status(&self, Parameters(input): Parameters<GetPaymentStatusInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        match self.store.get(&input.payment_id).await {
            Some(p) => ok_json(&serde_json::json!({"id": p.id, "status": p.status, "reconciliation": p.reconciliation})),
            None => err_result("Payment not found"),
        }
    }

    #[tool(description = "Create a checkout payment intent (Draft). Amounts > 10000 minor units require approval.")]
    async fn create_checkout_intent(&self, Parameters(input): Parameters<CreateCheckoutIntentInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let policy = if input.amount_minor > 10000 {
            PolicyDecision {
                decision: PolicyDecisionResult::RequiresApproval,
                reasons: vec!["Amount exceeds 10000 minor units threshold".into()],
                required_approvals: vec!["finance_manager".into()],
            }
        } else {
            PolicyDecision {
                decision: PolicyDecisionResult::Allowed,
                reasons: vec!["Within auto-approval threshold".into()],
                required_approvals: vec![],
            }
        };

        let intent = PaymentIntent {
            id: new_id(),
            direction: Direction::Inbound,
            intent_type: IntentType::Checkout,
            status: IntentStatus::Draft,
            amount: Money { amount_minor: input.amount_minor, currency: input.currency },
            counterparty: Counterparty { id: input.customer_id, display_name: input.customer_name, counterparty_type: CounterpartyType::Customer },
            rail: Some(PaymentRail { rail_type: RailType::CardCapture, provider: Provider::Mock, mode: RailMode::Sandbox }),
            policy: Some(policy),
            approvals: vec![],
            evidence: vec![],
            reconciliation: ReconciliationStatus::Pending,
            reason: input.reason,
            created_by: input.actor,
            created_at: now(),
            idempotency_key: input.idempotency_key,
        };
        self.store.create(intent.clone()).await;
        ok_json(&intent)
    }

    #[tool(description = "Create a refund intent for an existing payment. Direction=Outbound.")]
    async fn create_refund_intent(&self, Parameters(input): Parameters<CreateRefundIntentInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let original = match self.store.get(&input.original_payment_id).await {
            Some(p) => p,
            None => return err_result("Original payment not found"),
        };

        let intent = PaymentIntent {
            id: new_id(),
            direction: Direction::Outbound,
            intent_type: IntentType::Refund,
            status: IntentStatus::Draft,
            amount: Money { amount_minor: input.amount_minor, currency: input.currency },
            counterparty: original.counterparty,
            rail: Some(PaymentRail { rail_type: RailType::CardRefund, provider: Provider::Mock, mode: RailMode::Sandbox }),
            policy: Some(PolicyDecision {
                decision: PolicyDecisionResult::RequiresApproval,
                reasons: vec!["Refunds always require approval".into()],
                required_approvals: vec!["finance_manager".into()],
            }),
            approvals: vec![],
            evidence: vec![],
            reconciliation: ReconciliationStatus::Pending,
            reason: input.reason,
            created_by: input.actor,
            created_at: now(),
            idempotency_key: input.idempotency_key,
        };
        self.store.create(intent.clone()).await;
        ok_json(&intent)
    }

    #[tool(description = "Create a payout intent. Direction=Outbound. Always requires approval.")]
    async fn create_payout_intent(&self, Parameters(input): Parameters<CreatePayoutIntentInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let intent = PaymentIntent {
            id: new_id(),
            direction: Direction::Outbound,
            intent_type: IntentType::Payout,
            status: IntentStatus::Draft,
            amount: Money { amount_minor: input.amount_minor, currency: input.currency },
            counterparty: Counterparty { id: input.counterparty_id, display_name: input.counterparty_name, counterparty_type: input.counterparty_type },
            rail: Some(PaymentRail { rail_type: RailType::Ach, provider: Provider::Mock, mode: RailMode::Sandbox }),
            policy: Some(PolicyDecision {
                decision: PolicyDecisionResult::RequiresApproval,
                reasons: vec!["Payouts always require approval".into()],
                required_approvals: vec!["finance_manager".into(), "compliance_officer".into()],
            }),
            approvals: vec![],
            evidence: vec![],
            reconciliation: ReconciliationStatus::Pending,
            reason: input.reason,
            created_by: input.actor,
            created_at: now(),
            idempotency_key: input.idempotency_key,
        };
        self.store.create(intent.clone()).await;
        ok_json(&intent)
    }

    #[tool(description = "Request approval for a Draft payment intent. Moves Draft → PendingApproval.")]
    async fn request_payment_approval(&self, Parameters(input): Parameters<RequestPaymentApprovalInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut intent = match self.store.get(&input.payment_id).await {
            Some(p) => p,
            None => return err_result("Payment not found"),
        };
        if !matches!(intent.status, IntentStatus::Draft) {
            return err_result("Only Draft intents can be submitted for approval");
        }
        intent.status = IntentStatus::PendingApproval;
        self.store.update(&input.payment_id, intent.clone()).await;
        ok_json(&intent)
    }

    #[tool(description = "Execute an approved payment intent. ONLY works if status=Approved. Moves Approved → Executing → Captured.")]
    async fn execute_approved_intent(&self, Parameters(input): Parameters<ExecuteApprovedIntentInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut intent = match self.store.get(&input.payment_id).await {
            Some(p) => p,
            None => return err_result("Payment not found"),
        };
        if !matches!(intent.status, IntentStatus::Approved) {
            return err_result("Only Approved intents can be executed");
        }
        intent.status = IntentStatus::Executing;
        // Simulate execution success
        intent.status = IntentStatus::Captured;
        self.store.update(&input.payment_id, intent.clone()).await;
        ok_json(&intent)
    }

    #[tool(description = "Reconcile a payment against an order/invoice reference. Updates reconciliation status.")]
    async fn reconcile_payment(&self, Parameters(input): Parameters<ReconcilePaymentInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut intent = match self.store.get(&input.payment_id).await {
            Some(p) => p,
            None => return err_result("Payment not found"),
        };
        intent.reconciliation = if intent.amount.amount_minor == input.reference_amount_minor {
            ReconciliationStatus::Matched
        } else {
            ReconciliationStatus::Discrepancy
        };
        self.store.update(&input.payment_id, intent.clone()).await;
        ok_json(&serde_json::json!({"id": intent.id, "reconciliation": intent.reconciliation, "reference_id": input.reference_id}))
    }

    #[tool(description = "Attach evidence artifact to a payment intent for audit trail.")]
    async fn attach_payment_evidence(&self, Parameters(input): Parameters<AttachPaymentEvidenceInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut intent = match self.store.get(&input.payment_id).await {
            Some(p) => p,
            None => return err_result("Payment not found"),
        };
        let artifact = EvidenceArtifact { id: new_id(), artifact_type: input.artifact_type, uri: input.uri };
        intent.evidence.push(artifact.clone());
        self.store.update(&input.payment_id, intent).await;
        ok_json(&artifact)
    }
}
