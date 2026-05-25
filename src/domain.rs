use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub customer_id: String,
    pub subject: String,
    pub channel: String,       // "email", "chat", "phone", "social"
    pub status: String,        // "open", "waiting", "resolved", "closed"
    pub priority: String,      // "low", "medium", "high", "urgent"
    pub assigned_agent: Option<String>,
    pub assigned_team: Option<String>,
    pub messages: Vec<Message>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub sender: String,        // "customer", "agent", "system"
    pub body: String,
    pub internal: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub email: String,
    pub plan: String,
    pub lifetime_value: f64,
    pub health_score: u8,      // 0-100
    pub sentiment_trend: String, // "improving", "stable", "declining"
    pub tickets_last_30d: u32,
    pub member_since: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub team: String,
    pub status: String,        // "available", "busy", "offline"
    pub active_conversations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbArticle {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CannedResponse {
    pub id: String,
    pub name: String,
    pub body: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    pub open_conversations: u32,
    pub avg_first_response_min: f64,
    pub avg_resolution_hours: f64,
    pub first_contact_resolution_pct: f64,
    pub csat_score: f64,
    pub nps: i32,
    pub volume_today: u32,
}
