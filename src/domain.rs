use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Money {
    pub amount_minor: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction { Inbound, Outbound }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntentType { Checkout, Invoice, Subscription, Refund, Payout, Settlement, Escrow, Credit }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntentStatus { Draft, PendingApproval, Approved, Executing, Captured, Settled, Failed, Cancelled }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CounterpartyType { Customer, Vendor, Partner, Employee, MarketplaceSeller }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Counterparty {
    pub id: String,
    pub display_name: String,
    pub counterparty_type: CounterpartyType,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RailType { CardCapture, CardRefund, Ach, Wire, Invoice, Wallet, Escrow, AccountCredit }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Provider { Stripe, Adyen, Plaid, Dwolla, InternalLedger, Mock }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RailMode { Sandbox, Production }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PaymentRail {
    pub rail_type: RailType,
    pub provider: Provider,
    pub mode: RailMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecisionResult { Allowed, RequiresApproval, Blocked }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PolicyDecision {
    pub decision: PolicyDecisionResult,
    pub reasons: Vec<String>,
    pub required_approvals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType { Receipt, Invoice, OrderSnapshot, LedgerSnapshot, PolicyLog, SupportTranscript, ContractEvidence }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EvidenceArtifact {
    pub id: String,
    pub artifact_type: EvidenceType,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Approval {
    pub approver: String,
    pub decision: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationStatus { Pending, Matched, Discrepancy, Unreconciled }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PaymentIntent {
    pub id: String,
    pub direction: Direction,
    pub intent_type: IntentType,
    pub status: IntentStatus,
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
