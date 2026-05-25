use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct ZendeskClient {
    http: Client,
    base_url: String,
    auth: String, // base64(email/token:api_token)
}

impl ZendeskClient {
    pub fn from_env() -> Option<Self> {
        let subdomain = std::env::var("ZENDESK_SUBDOMAIN").ok()?;
        let email = std::env::var("ZENDESK_EMAIL").ok()?;
        let token = std::env::var("ZENDESK_API_TOKEN").ok()?;
        let auth = base64::engine::general_purpose::STANDARD.encode(format!("{}/token:{}", email, token));
        Some(Self {
            http: Client::new(),
            base_url: format!("https://{}.zendesk.com/api/v2", subdomain),
            auth,
        })
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        let resp = self.http.get(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Basic {}", self.auth))
            .send().await?;
        if !resp.status().is_success() { bail!("Zendesk {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.post(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Basic {}", self.auth))
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("Zendesk {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.put(format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Basic {}", self.auth))
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("Zendesk {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    // --- Mapped to our API spec ---

    pub async fn list_conversations(&self, status: Option<&str>, _priority: Option<&str>, _agent: Option<&str>) -> Result<Value> {
        let status_param = status.unwrap_or("open");
        self.get(&format!("/tickets.json?status={}", status_param)).await
    }

    pub async fn get_conversation(&self, id: &str) -> Result<Value> {
        let ticket = self.get(&format!("/tickets/{}.json", id)).await?;
        let comments = self.get(&format!("/tickets/{}/comments.json", id)).await?;
        Ok(json!({"ticket": ticket["ticket"], "messages": comments["comments"]}))
    }

    pub async fn create_conversation(&self, body: &Value) -> Result<Value> {
        let ticket = json!({
            "ticket": {
                "subject": body["subject"],
                "comment": {"body": body["message"]},
                "priority": body["priority"],
                "requester_id": body["customer_id"]
            }
        });
        self.post("/tickets.json", &ticket).await
    }

    pub async fn add_message(&self, id: &str, body: &Value) -> Result<Value> {
        let internal = body["internal"].as_bool().unwrap_or(false);
        let ticket = json!({
            "ticket": {
                "comment": {
                    "body": body["body"],
                    "public": !internal
                }
            }
        });
        self.put(&format!("/tickets/{}.json", id), &ticket).await
    }

    pub async fn update_conversation(&self, id: &str, body: &Value) -> Result<Value> {
        let mut update = json!({"ticket": {}});
        if let Some(s) = body.get("status") { update["ticket"]["status"] = s.clone(); }
        if let Some(a) = body.get("assigned_agent") { update["ticket"]["assignee_id"] = a.clone(); }
        if let Some(p) = body.get("priority") { update["ticket"]["priority"] = p.clone(); }
        self.put(&format!("/tickets/{}.json", id), &update).await
    }

    pub async fn escalate(&self, id: &str, body: &Value) -> Result<Value> {
        let update = json!({"ticket": {"priority": "urgent", "tags": ["escalated"], "comment": {"body": format!("Escalated: {}", body["reason"].as_str().unwrap_or("")), "public": false}}});
        self.put(&format!("/tickets/{}.json", id), &update).await
    }

    pub async fn search_kb(&self, query: &str) -> Result<Value> {
        self.get(&format!("/help_center/articles/search.json?query={}", urlencoding::encode(query))).await
    }

    pub async fn list_agents(&self) -> Result<Value> {
        self.get("/users.json?role=agent").await
    }

    pub async fn get_customer(&self, id: &str) -> Result<Value> {
        self.get(&format!("/users/{}.json", id)).await
    }

    pub async fn get_satisfaction(&self) -> Result<Value> {
        self.get("/satisfaction_ratings.json").await
    }
}

use base64::Engine as _;
