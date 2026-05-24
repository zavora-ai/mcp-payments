use reqwest::Client;

#[derive(Clone)]
pub struct StripeClient {
    http: Client,
    key: String,
}

impl StripeClient {
    pub fn new(key: String) -> Self {
        Self { http: Client::new(), key }
    }

    async fn get(&self, path: &str, params: &[(&str, &str)]) -> anyhow::Result<serde_json::Value> {
        let resp = self.http
            .get(format!("https://api.stripe.com/v1/{path}"))
            .basic_auth(&self.key, None::<&str>)
            .query(params)
            .send().await?.error_for_status()?
            .json().await?;
        Ok(resp)
    }

    async fn post(&self, path: &str, form: &[(&str, &str)]) -> anyhow::Result<serde_json::Value> {
        let resp = self.http
            .post(format!("https://api.stripe.com/v1/{path}"))
            .basic_auth(&self.key, None::<&str>)
            .form(form)
            .send().await?.error_for_status()?
            .json().await?;
        Ok(resp)
    }

    pub async fn list_customers(&self, limit: u32) -> anyhow::Result<serde_json::Value> {
        self.get("customers", &[("limit", &limit.to_string())]).await
    }

    pub async fn get_customer(&self, id: &str) -> anyhow::Result<serde_json::Value> {
        self.get(&format!("customers/{id}"), &[]).await
    }

    pub async fn create_payment_intent(&self, amount: u64, currency: &str, customer: Option<&str>) -> anyhow::Result<serde_json::Value> {
        let mut params = vec![("amount", amount.to_string()), ("currency", currency.to_string())];
        if let Some(c) = customer { params.push(("customer", c.to_string())); }
        let form: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.post("payment_intents", &form).await
    }

    pub async fn list_payment_intents(&self, limit: u32) -> anyhow::Result<serde_json::Value> {
        self.get("payment_intents", &[("limit", &limit.to_string())]).await
    }

    pub async fn get_payment_intent(&self, id: &str) -> anyhow::Result<serde_json::Value> {
        self.get(&format!("payment_intents/{id}"), &[]).await
    }

    pub async fn list_invoices(&self, limit: u32) -> anyhow::Result<serde_json::Value> {
        self.get("invoices", &[("limit", &limit.to_string())]).await
    }

    pub async fn create_refund(&self, payment_intent: &str, amount: Option<u64>) -> anyhow::Result<serde_json::Value> {
        let mut params = vec![("payment_intent", payment_intent.to_string())];
        if let Some(a) = amount { params.push(("amount", a.to_string())); }
        let form: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.post("refunds", &form).await
    }

    pub async fn get_balance(&self) -> anyhow::Result<serde_json::Value> {
        self.get("balance", &[]).await
    }
}
