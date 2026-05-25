use crate::client::ApiClient;
use crate::freshdesk::FreshdeskClient;
use crate::intercom::IntercomClient;
use crate::zendesk::ZendeskClient;
use anyhow::Result;
use serde_json::Value;

/// Unified backend — dispatches to whichever provider is configured.
#[derive(Clone)]
pub enum Backend {
    Zendesk(ZendeskClient),
    Freshdesk(FreshdeskClient),
    Intercom(IntercomClient),
    Custom(ApiClient),
}

impl Backend {
    /// Detect backend from environment variables.
    /// Priority: Zendesk > Freshdesk > Intercom > Custom API.
    pub fn from_env() -> Result<Self> {
        if let Some(z) = ZendeskClient::from_env() {
            tracing::info!("Using Zendesk backend");
            return Ok(Self::Zendesk(z));
        }
        if let Some(f) = FreshdeskClient::from_env() {
            tracing::info!("Using Freshdesk backend");
            return Ok(Self::Freshdesk(f));
        }
        if let Some(i) = IntercomClient::from_env() {
            tracing::info!("Using Intercom backend");
            return Ok(Self::Intercom(i));
        }
        let c = ApiClient::from_env()?;
        tracing::info!("Using custom API backend");
        Ok(Self::Custom(c))
    }

    pub async fn list_conversations(&self, status: Option<&str>, priority: Option<&str>, agent: Option<&str>) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.list_conversations(status, priority, agent).await,
            Self::Freshdesk(c) => c.list_conversations(status, priority, agent).await,
            Self::Intercom(c) => c.list_conversations(status, priority, agent).await,
            Self::Custom(c) => {
                let mut path = "/conversations?".to_string();
                if let Some(s) = status { path.push_str(&format!("status={}&", s)); }
                if let Some(p) = priority { path.push_str(&format!("priority={}&", p)); }
                if let Some(a) = agent { path.push_str(&format!("assigned_agent={}&", a)); }
                c.get(&path).await
            }
        }
    }

    pub async fn get_conversation(&self, id: &str) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.get_conversation(id).await,
            Self::Freshdesk(c) => c.get_conversation(id).await,
            Self::Intercom(c) => c.get_conversation(id).await,
            Self::Custom(c) => c.get(&format!("/conversations/{}", id)).await,
        }
    }

    pub async fn create_conversation(&self, body: &Value) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.create_conversation(body).await,
            Self::Freshdesk(c) => c.create_conversation(body).await,
            Self::Intercom(c) => c.create_conversation(body).await,
            Self::Custom(c) => c.post("/conversations", body).await,
        }
    }

    pub async fn add_message(&self, id: &str, body: &Value) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.add_message(id, body).await,
            Self::Freshdesk(c) => c.add_message(id, body).await,
            Self::Intercom(c) => c.add_message(id, body).await,
            Self::Custom(c) => c.post(&format!("/conversations/{}/messages", id), body).await,
        }
    }

    pub async fn update_conversation(&self, id: &str, body: &Value) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.update_conversation(id, body).await,
            Self::Freshdesk(c) => c.update_conversation(id, body).await,
            Self::Intercom(c) => c.update_conversation(id, body).await,
            Self::Custom(c) => c.patch(&format!("/conversations/{}", id), body).await,
        }
    }

    pub async fn escalate(&self, id: &str, body: &Value) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.escalate(id, body).await,
            Self::Freshdesk(c) => c.escalate(id, body).await,
            Self::Intercom(c) => c.escalate(id, body).await,
            Self::Custom(c) => c.post(&format!("/conversations/{}/escalate", id), body).await,
        }
    }

    pub async fn search_kb(&self, query: &str) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.search_kb(query).await,
            Self::Freshdesk(c) => c.search_kb(query).await,
            Self::Intercom(c) => c.search_kb(query).await,
            Self::Custom(c) => c.get(&format!("/kb/search?q={}", urlencoding::encode(query))).await,
        }
    }

    pub async fn list_agents(&self) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.list_agents().await,
            Self::Freshdesk(c) => c.list_agents().await,
            Self::Intercom(c) => c.list_agents().await,
            Self::Custom(c) => c.get("/agents").await,
        }
    }

    pub async fn get_customer(&self, id: &str) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.get_customer(id).await,
            Self::Freshdesk(c) => c.get_customer(id).await,
            Self::Intercom(c) => c.get_customer(id).await,
            Self::Custom(c) => c.get(&format!("/customers/{}", id)).await,
        }
    }

    pub async fn get_customer_health(&self, id: &str) -> Result<Value> {
        match self {
            Self::Custom(c) => c.get(&format!("/customers/{}/health", id)).await,
            _ => Ok(serde_json::json!({"error": "Health scores require custom backend or analytics integration"})),
        }
    }

    pub async fn get_interactions(&self, id: &str) -> Result<Value> {
        match self {
            Self::Custom(c) => c.get(&format!("/customers/{}/interactions", id)).await,
            _ => self.list_conversations(None, None, None).await, // fallback: list all
        }
    }

    pub async fn get_churn_risk(&self, id: &str) -> Result<Value> {
        match self {
            Self::Custom(c) => c.get(&format!("/customers/{}/churn-risk", id)).await,
            _ => Ok(serde_json::json!({"error": "Churn risk requires custom backend or analytics integration"})),
        }
    }

    pub async fn suggest_response(&self, id: &str) -> Result<Value> {
        match self {
            Self::Custom(c) => c.get(&format!("/conversations/{}/suggest", id)).await,
            _ => Ok(serde_json::json!({"error": "Response suggestions require custom backend with AI"})),
        }
    }

    pub async fn get_canned_responses(&self) -> Result<Value> {
        match self {
            Self::Custom(c) => c.get("/canned-responses").await,
            Self::Zendesk(c) => c.get("/macros.json").await,
            Self::Freshdesk(c) => c.get("/canned_responses").await,
            _ => Ok(serde_json::json!([])),
        }
    }

    pub async fn merge_conversations(&self, primary_id: &str, body: &Value) -> Result<Value> {
        match self {
            Self::Custom(c) => c.post(&format!("/conversations/{}/merge", primary_id), body).await,
            Self::Zendesk(c) => c.post(&format!("/tickets/{}/merge.json", primary_id), body).await,
            _ => Ok(serde_json::json!({"error": "Merge not supported on this backend"})),
        }
    }

    pub async fn get_queue_status(&self) -> Result<Value> {
        match self {
            Self::Custom(c) => c.get("/queue/status").await,
            _ => Ok(serde_json::json!({"error": "Queue status requires custom backend"})),
        }
    }

    pub async fn get_satisfaction(&self) -> Result<Value> {
        match self {
            Self::Zendesk(c) => c.get_satisfaction().await,
            Self::Freshdesk(c) => c.get_satisfaction().await,
            Self::Intercom(c) => c.get_satisfaction().await,
            Self::Custom(c) => c.get("/metrics/satisfaction").await,
        }
    }

    pub async fn get_service_metrics(&self) -> Result<Value> {
        match self {
            Self::Custom(c) => c.get("/metrics/service").await,
            _ => Ok(serde_json::json!({"error": "Service metrics require custom backend"})),
        }
    }
}
