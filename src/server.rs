use crate::client::ApiClient;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde_json::{json, Value};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdInput { pub id: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FilterInput { pub status: Option<String>, pub priority: Option<String>, pub assigned_agent: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StartConvInput { pub customer_id: String, pub subject: String, pub channel: Option<String>, pub message: String, pub priority: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReplyInput { pub conversation_id: String, pub body: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct NoteInput { pub conversation_id: String, pub body: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AssignInput { pub conversation_id: String, pub agent_id: Option<String>, pub team: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EscalateInput { pub conversation_id: String, pub reason: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchInput { pub query: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MergeInput { pub primary_id: String, pub secondary_id: String }

#[derive(Clone)]
pub struct CsServer {
    pub api: ApiClient,
}

fn ok_or_err(result: Result<Value, anyhow::Error>) -> String {
    match result {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
        Err(e) => format!("Error: {}", e),
    }
}

#[tool_router(server_handler)]
impl CsServer {
    #[tool(description = "List conversations filtered by status, priority, or assigned agent")]
    async fn list_conversations(&self, Parameters(input): Parameters<FilterInput>) -> String {
        let mut path = "/conversations?".to_string();
        if let Some(s) = &input.status { path.push_str(&format!("status={}&", s)); }
        if let Some(p) = &input.priority { path.push_str(&format!("priority={}&", p)); }
        if let Some(a) = &input.assigned_agent { path.push_str(&format!("assigned_agent={}&", a)); }
        ok_or_err(self.api.get(&path).await)
    }

    #[tool(description = "Get full conversation with message history")]
    async fn get_conversation(&self, Parameters(input): Parameters<IdInput>) -> String {
        ok_or_err(self.api.get(&format!("/conversations/{}", input.id)).await)
    }

    #[tool(description = "Start a new conversation from a customer")]
    async fn start_conversation(&self, Parameters(input): Parameters<StartConvInput>) -> String {
        ok_or_err(self.api.post("/conversations", &json!({
            "customer_id": input.customer_id, "subject": input.subject,
            "channel": input.channel.unwrap_or("chat".into()),
            "message": input.message, "priority": input.priority.unwrap_or("medium".into())
        })).await)
    }

    #[tool(description = "Send a public reply to the customer")]
    async fn reply_conversation(&self, Parameters(input): Parameters<ReplyInput>) -> String {
        ok_or_err(self.api.post(&format!("/conversations/{}/messages", input.conversation_id), &json!({
            "body": input.body, "sender": "agent", "internal": false
        })).await)
    }

    #[tool(description = "Add an internal note (not visible to customer)")]
    async fn add_internal_note(&self, Parameters(input): Parameters<NoteInput>) -> String {
        ok_or_err(self.api.post(&format!("/conversations/{}/messages", input.conversation_id), &json!({
            "body": input.body, "sender": "agent", "internal": true
        })).await)
    }

    #[tool(description = "Get customer profile: plan, lifetime value, health score, history")]
    async fn get_customer_profile(&self, Parameters(input): Parameters<IdInput>) -> String {
        ok_or_err(self.api.get(&format!("/customers/{}", input.id)).await)
    }

    #[tool(description = "Get customer health score with contributing factors")]
    async fn get_customer_health(&self, Parameters(input): Parameters<IdInput>) -> String {
        ok_or_err(self.api.get(&format!("/customers/{}/health", input.id)).await)
    }

    #[tool(description = "Get all past interactions for a customer")]
    async fn get_interaction_history(&self, Parameters(input): Parameters<IdInput>) -> String {
        ok_or_err(self.api.get(&format!("/customers/{}/interactions", input.id)).await)
    }

    #[tool(description = "Assess churn risk for a customer based on signals")]
    async fn assess_churn_risk(&self, Parameters(input): Parameters<IdInput>) -> String {
        ok_or_err(self.api.get(&format!("/customers/{}/churn-risk", input.id)).await)
    }

    #[tool(description = "Search knowledge base for relevant articles")]
    async fn search_knowledge_base(&self, Parameters(input): Parameters<SearchInput>) -> String {
        ok_or_err(self.api.get(&format!("/kb/search?q={}", urlencoding::encode(&input.query))).await)
    }

    #[tool(description = "Get AI-suggested response based on conversation context")]
    async fn suggest_response(&self, Parameters(input): Parameters<IdInput>) -> String {
        ok_or_err(self.api.get(&format!("/conversations/{}/suggest", input.id)).await)
    }

    #[tool(description = "List available canned/template responses")]
    async fn get_canned_responses(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        ok_or_err(self.api.get("/canned-responses").await)
    }

    #[tool(description = "Resolve a conversation and request satisfaction rating")]
    async fn resolve_conversation(&self, Parameters(input): Parameters<IdInput>) -> String {
        ok_or_err(self.api.patch(&format!("/conversations/{}", input.id), &json!({"status": "resolved"})).await)
    }

    #[tool(description = "Assign conversation to an agent or team")]
    async fn assign_agent(&self, Parameters(input): Parameters<AssignInput>) -> String {
        ok_or_err(self.api.patch(&format!("/conversations/{}", input.conversation_id), &json!({
            "assigned_agent": input.agent_id, "assigned_team": input.team
        })).await)
    }

    #[tool(description = "Escalate to senior agent/manager with reason")]
    async fn escalate(&self, Parameters(input): Parameters<EscalateInput>) -> String {
        ok_or_err(self.api.post(&format!("/conversations/{}/escalate", input.conversation_id), &json!({
            "reason": input.reason
        })).await)
    }

    #[tool(description = "Get queue status: depth, wait times, agent availability")]
    async fn get_queue_status(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        ok_or_err(self.api.get("/queue/status").await)
    }

    #[tool(description = "Get satisfaction scores: CSAT, NPS, customer effort")]
    async fn get_satisfaction_scores(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        ok_or_err(self.api.get("/metrics/satisfaction").await)
    }

    #[tool(description = "Get service metrics: response time, resolution time, FCR, volume")]
    async fn get_service_metrics(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        ok_or_err(self.api.get("/metrics/service").await)
    }

    #[tool(description = "List available agents and their status")]
    async fn list_agents(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        ok_or_err(self.api.get("/agents").await)
    }

    #[tool(description = "Merge duplicate conversations into one")]
    async fn merge_conversations(&self, Parameters(input): Parameters<MergeInput>) -> String {
        ok_or_err(self.api.post(&format!("/conversations/{}/merge", input.primary_id), &json!({
            "secondary_id": input.secondary_id
        })).await)
    }
}
