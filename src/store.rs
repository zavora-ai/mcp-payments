use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::PaymentIntent;

#[derive(Clone)]
pub struct PaymentStore {
    data: Arc<RwLock<HashMap<String, PaymentIntent>>>,
}

impl PaymentStore {
    pub fn new() -> Self {
        Self { data: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get(&self, id: &str) -> Option<PaymentIntent> {
        self.data.read().await.get(id).cloned()
    }

    pub async fn list_by_counterparty(&self, counterparty_id: &str) -> Vec<PaymentIntent> {
        self.data.read().await.values()
            .filter(|p| p.counterparty.id == counterparty_id)
            .cloned().collect()
    }

    pub async fn create(&self, intent: PaymentIntent) {
        self.data.write().await.insert(intent.id.clone(), intent);
    }

    pub async fn update(&self, id: &str, intent: PaymentIntent) {
        self.data.write().await.insert(id.to_string(), intent);
    }

    #[allow(dead_code)]
    pub async fn list_all(&self) -> Vec<PaymentIntent> {
        self.data.read().await.values().cloned().collect()
    }
}
