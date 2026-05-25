use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::Value;

/// HTTP client that calls the Customer Service backend API.
/// Configure via CUSTOMER_SERVICE_API_URL (required).
/// Optional: CUSTOMER_SERVICE_API_KEY for auth.
#[derive(Clone)]
pub struct ApiClient {
    http: Client,
    base_url: String,
    api_key: Option<String>,
}

impl ApiClient {
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("CUSTOMER_SERVICE_API_URL")
            .map_err(|_| anyhow::anyhow!("CUSTOMER_SERVICE_API_URL is required"))?;
        let api_key = std::env::var("CUSTOMER_SERVICE_API_KEY").ok();
        Ok(Self { http: Client::new(), base_url: base_url.trim_end_matches('/').to_string(), api_key })
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, &url);
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        req.header("Content-Type", "application/json")
    }

    pub async fn get(&self, path: &str) -> Result<Value> {
        let resp = self.request(reqwest::Method::GET, path).send().await?;
        if !resp.status().is_success() {
            bail!("API error {}: {}", resp.status(), resp.text().await.unwrap_or_default());
        }
        Ok(resp.json().await?)
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.request(reqwest::Method::POST, path).json(body).send().await?;
        if !resp.status().is_success() {
            bail!("API error {}: {}", resp.status(), resp.text().await.unwrap_or_default());
        }
        Ok(resp.json().await?)
    }

    pub async fn patch(&self, path: &str, body: &Value) -> Result<Value> {
        let resp = self.request(reqwest::Method::PATCH, path).json(body).send().await?;
        if !resp.status().is_success() {
            bail!("API error {}: {}", resp.status(), resp.text().await.unwrap_or_default());
        }
        Ok(resp.json().await?)
    }

    pub async fn delete(&self, path: &str) -> Result<Value> {
        let resp = self.request(reqwest::Method::DELETE, path).send().await?;
        if !resp.status().is_success() {
            bail!("API error {}: {}", resp.status(), resp.text().await.unwrap_or_default());
        }
        Ok(resp.json().await?)
    }
}
