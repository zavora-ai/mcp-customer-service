use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct FreshdeskClient {
    http: Client,
    base_url: String,
    api_key: String,
}

impl FreshdeskClient {
    pub fn from_env() -> Option<Self> {
        let domain = std::env::var("FRESHDESK_DOMAIN").ok()?;
        let api_key = std::env::var("FRESHDESK_API_KEY").ok()?;
        Some(Self {
            http: Client::new(),
            base_url: format!("https://{}.freshdesk.com/api/v2", domain),
            api_key,
        })
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        let resp = self.http.get(format!("{}{}", self.base_url, path))
            .basic_auth(&self.api_key, Some("X"))
            .send().await?;
        if !resp.status().is_success() { bail!("Freshdesk {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.post(format!("{}{}", self.base_url, path))
            .basic_auth(&self.api_key, Some("X"))
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("Freshdesk {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.http.put(format!("{}{}", self.base_url, path))
            .basic_auth(&self.api_key, Some("X"))
            .json(body).send().await?;
        if !resp.status().is_success() { bail!("Freshdesk {}: {}", resp.status(), resp.text().await?); }
        Ok(resp.json().await?)
    }

    pub async fn list_conversations(&self, status: Option<&str>, priority: Option<&str>, _agent: Option<&str>) -> Result<Value> {
        // Freshdesk status: 2=open, 3=pending, 4=resolved, 5=closed
        let status_num = match status.unwrap_or("open") {
            "open" => "2", "waiting" | "pending" => "3", "resolved" => "4", "closed" => "5", _ => "2"
        };
        let mut filter = format!("\"status\":{}",status_num);
        if let Some(p) = priority {
            let p_num = match p { "low" => "1", "medium" => "2", "high" => "3", "urgent" => "4", _ => "2" };
            filter.push_str(&format!(",\"priority\":{}", p_num));
        }
        self.get(&format!("/tickets?filter={{{}}}", filter)).await
    }

    pub async fn get_conversation(&self, id: &str) -> Result<Value> {
        let ticket = self.get(&format!("/tickets/{}", id)).await?;
        let convos = self.get(&format!("/tickets/{}/conversations", id)).await?;
        Ok(json!({"ticket": ticket, "messages": convos}))
    }

    pub async fn create_conversation(&self, body: &Value) -> Result<Value> {
        let priority = match body["priority"].as_str().unwrap_or("medium") {
            "low" => 1, "medium" => 2, "high" => 3, "urgent" => 4, _ => 2
        };
        let ticket = json!({
            "subject": body["subject"],
            "description": body["message"],
            "priority": priority,
            "status": 2,
            "email": body["customer_id"]
        });
        self.post("/tickets", &ticket).await
    }

    pub async fn add_message(&self, id: &str, body: &Value) -> Result<Value> {
        let internal = body["internal"].as_bool().unwrap_or(false);
        if internal {
            self.post(&format!("/tickets/{}/notes", id), &json!({"body": body["body"], "private": true})).await
        } else {
            self.post(&format!("/tickets/{}/reply", id), &json!({"body": body["body"]})).await
        }
    }

    pub async fn update_conversation(&self, id: &str, body: &Value) -> Result<Value> {
        let mut update = json!({});
        if let Some(s) = body.get("status") {
            update["status"] = json!(match s.as_str().unwrap_or("") { "open" => 2, "resolved" => 4, "closed" => 5, _ => 3 });
        }
        if let Some(p) = body.get("priority") {
            update["priority"] = json!(match p.as_str().unwrap_or("") { "low" => 1, "high" => 3, "urgent" => 4, _ => 2 });
        }
        if let Some(a) = body.get("assigned_agent") { update["responder_id"] = a.clone(); }
        self.put(&format!("/tickets/{}", id), &update).await
    }

    pub async fn escalate(&self, id: &str, body: &Value) -> Result<Value> {
        let update = json!({"priority": 4, "tags": ["escalated"]});
        self.put(&format!("/tickets/{}", id), &update).await?;
        self.post(&format!("/tickets/{}/notes", id), &json!({"body": format!("Escalated: {}", body["reason"].as_str().unwrap_or("")), "private": true})).await
    }

    pub async fn search_kb(&self, query: &str) -> Result<Value> {
        self.get(&format!("/solutions/articles?search={}", urlencoding::encode(query))).await
    }

    pub async fn list_agents(&self) -> Result<Value> {
        self.get("/agents").await
    }

    pub async fn get_customer(&self, id: &str) -> Result<Value> {
        self.get(&format!("/contacts/{}", id)).await
    }

    pub async fn get_satisfaction(&self) -> Result<Value> {
        self.get("/surveys/satisfaction_ratings").await
    }
}
