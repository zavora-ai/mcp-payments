use adk_mcp_sdk::{HealthCheck, HealthStatus};
use crate::client::StripeClient;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListInput { #[serde(default = "default_20")] pub limit: u32 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetCustomerInput { pub customer_id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreatePaymentIntentInput {
    /// Amount in smallest currency unit (e.g. cents)
    pub amount: u64,
    /// Three-letter ISO currency code (e.g. usd)
    pub currency: String,
    #[serde(default)]
    pub customer: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPaymentIntentInput { pub payment_intent_id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateRefundInput {
    pub payment_intent: String,
    #[serde(default)]
    pub amount: Option<u64>,
}

fn default_20() -> u32 { 20 }

#[derive(Clone)]
pub struct PaymentsServer { pub client: Arc<StripeClient> }

#[tool_router(server_handler)]
impl PaymentsServer {
    #[tool(description = "List Stripe customers")]
    async fn list_customers(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.client.list_customers(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a Stripe customer by ID")]
    async fn get_customer(&self, Parameters(i): Parameters<GetCustomerInput>) -> String {
        match self.client.get_customer(&i.customer_id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create a Stripe payment intent")]
    async fn create_payment_intent(&self, Parameters(i): Parameters<CreatePaymentIntentInput>) -> String {
        match self.client.create_payment_intent(i.amount, &i.currency, i.customer.as_deref()).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List Stripe payment intents")]
    async fn list_payment_intents(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.client.list_payment_intents(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get a Stripe payment intent by ID")]
    async fn get_payment_intent(&self, Parameters(i): Parameters<GetPaymentIntentInput>) -> String {
        match self.client.get_payment_intent(&i.payment_intent_id).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "List Stripe invoices")]
    async fn list_invoices(&self, Parameters(i): Parameters<ListInput>) -> String {
        match self.client.list_invoices(i.limit).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Create a refund for a payment intent")]
    async fn create_refund(&self, Parameters(i): Parameters<CreateRefundInput>) -> String {
        match self.client.create_refund(&i.payment_intent, i.amount).await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Get Stripe account balance")]
    async fn get_balance(&self) -> String {
        match self.client.get_balance().await {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}"),
        }
    }
}

#[async_trait::async_trait]
impl HealthCheck for PaymentsServer {
    async fn check_health(&self) -> HealthStatus {
        HealthStatus { healthy: true, message: Some("operational".into()), latency_ms: Some(1) }
    }
}
