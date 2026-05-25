use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct IntercomClient {
    http: Client,
    token: String,
}

impl IntercomClient {
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("INTERCOM_ACCESS_TOKEN").ok()?;
        Some(Self { http: Client::new(), token })
    }

    async fn get(&self, path: &str) -> Result<Value> {
        let resp = self.http.get(format!("https://api.intercom.io{}", path))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .send().await?;
        if !resp.status().is_success() { bail!("Intercom {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.post(format!("https://api.intercom.io{}", path))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("Intercom {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.put(format!("https://api.intercom.io{}", path))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("Intercom {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn list_conversations(&self, status: Option<&str>, _priority: Option<&str>, _agent: Option<&str>) -> Result<Value> {
        let open = status.unwrap_or("open") == "open";
        self.get(&format!("/conversations?open={}", open)).await
    }

    pub async fn get_conversation(&self, id: &str) -> Result<Value> {
        self.get(&format!("/conversations/{}", id)).await
    }

    pub async fn create_conversation(&self, body: &Value) -> Result<Value> {
        self.post("/conversations", &json!({
            "from": {"type": "user", "id": body["customer_id"]},
            "body": body["message"]
        })).await
    }

    pub async fn add_message(&self, id: &str, body: &Value) -> Result<Value> {
        let msg_type = if body["internal"].as_bool().unwrap_or(false) { "note" } else { "comment" };
        self.post(&format!("/conversations/{}/reply", id), &json!({
            "message_type": msg_type,
            "type": "admin",
            "body": body["body"]
        })).await
    }

    pub async fn update_conversation(&self, id: &str, body: &Value) -> Result<Value> {
        if body.get("status").and_then(|s| s.as_str()) == Some("resolved") {
            self.post(&format!("/conversations/{}/parts", id), &json!({"message_type": "close"})).await
        } else if let Some(a) = body.get("assigned_agent") {
            self.post(&format!("/conversations/{}/parts", id), &json!({"message_type": "assignment", "assignee_id": a})).await
        } else {
            self.put(&format!("/conversations/{}", id), body).await
        }
    }

    pub async fn escalate(&self, id: &str, body: &Value) -> Result<Value> {
        self.post(&format!("/conversations/{}/reply", id), &json!({
            "message_type": "note",
            "type": "admin",
            "body": format!("⚠️ ESCALATED: {}", body["reason"].as_str().unwrap_or(""))
        })).await
    }

    pub async fn search_kb(&self, query: &str) -> Result<Value> {
        self.get(&format!("/help_center/help_centers?q={}", urlencoding::encode(query))).await
    }

    pub async fn list_agents(&self) -> Result<Value> {
        self.get("/admins").await
    }

    pub async fn get_customer(&self, id: &str) -> Result<Value> {
        self.get(&format!("/contacts/{}", id)).await
    }

    pub async fn get_satisfaction(&self) -> Result<Value> {
        // Intercom doesn't have a direct satisfaction endpoint — use conversations with ratings
        self.get("/conversations?display_as=plaintext").await
    }
}
