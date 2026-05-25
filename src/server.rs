use crate::backend::Backend;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde_json::json;

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
pub struct CsServer { pub backend: Backend }

fn result_to_string(r: Result<serde_json::Value, anyhow::Error>) -> String {
    match r {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
        Err(e) => format!("Error: {}", e),
    }
}

#[tool_router(server_handler)]
impl CsServer {
    #[tool(description = "List conversations filtered by status, priority, or assigned agent")]
    async fn list_conversations(&self, Parameters(input): Parameters<FilterInput>) -> String {
        result_to_string(self.backend.list_conversations(input.status.as_deref(), input.priority.as_deref(), input.assigned_agent.as_deref()).await)
    }

    #[tool(description = "Get full conversation with message history")]
    async fn get_conversation(&self, Parameters(input): Parameters<IdInput>) -> String {
        result_to_string(self.backend.get_conversation(&input.id).await)
    }

    #[tool(description = "Start a new conversation from a customer")]
    async fn start_conversation(&self, Parameters(input): Parameters<StartConvInput>) -> String {
        result_to_string(self.backend.create_conversation(&json!({
            "customer_id": input.customer_id, "subject": input.subject,
            "channel": input.channel.unwrap_or("chat".into()),
            "message": input.message, "priority": input.priority.unwrap_or("medium".into())
        })).await)
    }

    #[tool(description = "Send a public reply to the customer")]
    async fn reply_conversation(&self, Parameters(input): Parameters<ReplyInput>) -> String {
        result_to_string(self.backend.add_message(&input.conversation_id, &json!({"body": input.body, "sender": "agent", "internal": false})).await)
    }

    #[tool(description = "Add an internal note (not visible to customer)")]
    async fn add_internal_note(&self, Parameters(input): Parameters<NoteInput>) -> String {
        result_to_string(self.backend.add_message(&input.conversation_id, &json!({"body": input.body, "sender": "agent", "internal": true})).await)
    }

    #[tool(description = "Get customer profile: plan, lifetime value, health score, history")]
    async fn get_customer_profile(&self, Parameters(input): Parameters<IdInput>) -> String {
        result_to_string(self.backend.get_customer(&input.id).await)
    }

    #[tool(description = "Get customer health score with contributing factors")]
    async fn get_customer_health(&self, Parameters(input): Parameters<IdInput>) -> String {
        result_to_string(self.backend.get_customer_health(&input.id).await)
    }

    #[tool(description = "Get all past interactions for a customer")]
    async fn get_interaction_history(&self, Parameters(input): Parameters<IdInput>) -> String {
        result_to_string(self.backend.get_interactions(&input.id).await)
    }

    #[tool(description = "Assess churn risk for a customer based on signals")]
    async fn assess_churn_risk(&self, Parameters(input): Parameters<IdInput>) -> String {
        result_to_string(self.backend.get_churn_risk(&input.id).await)
    }

    #[tool(description = "Search knowledge base for relevant articles")]
    async fn search_knowledge_base(&self, Parameters(input): Parameters<SearchInput>) -> String {
        result_to_string(self.backend.search_kb(&input.query).await)
    }

    #[tool(description = "Get AI-suggested response based on conversation context")]
    async fn suggest_response(&self, Parameters(input): Parameters<IdInput>) -> String {
        result_to_string(self.backend.suggest_response(&input.id).await)
    }

    #[tool(description = "List available canned/template responses")]
    async fn get_canned_responses(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        result_to_string(self.backend.get_canned_responses().await)
    }

    #[tool(description = "Resolve a conversation and request satisfaction rating")]
    async fn resolve_conversation(&self, Parameters(input): Parameters<IdInput>) -> String {
        result_to_string(self.backend.update_conversation(&input.id, &json!({"status": "resolved"})).await)
    }

    #[tool(description = "Assign conversation to an agent or team")]
    async fn assign_agent(&self, Parameters(input): Parameters<AssignInput>) -> String {
        result_to_string(self.backend.update_conversation(&input.conversation_id, &json!({"assigned_agent": input.agent_id, "assigned_team": input.team})).await)
    }

    #[tool(description = "Escalate to senior agent/manager with reason")]
    async fn escalate(&self, Parameters(input): Parameters<EscalateInput>) -> String {
        result_to_string(self.backend.escalate(&input.conversation_id, &json!({"reason": input.reason})).await)
    }

    #[tool(description = "Get queue status: depth, wait times, agent availability")]
    async fn get_queue_status(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        result_to_string(self.backend.get_queue_status().await)
    }

    #[tool(description = "Get satisfaction scores: CSAT, NPS, customer effort")]
    async fn get_satisfaction_scores(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        result_to_string(self.backend.get_satisfaction().await)
    }

    #[tool(description = "Get service metrics: response time, resolution time, FCR, volume")]
    async fn get_service_metrics(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        result_to_string(self.backend.get_service_metrics().await)
    }

    #[tool(description = "List available agents and their status")]
    async fn list_agents(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        result_to_string(self.backend.list_agents().await)
    }

    #[tool(description = "Merge duplicate conversations into one")]
    async fn merge_conversations(&self, Parameters(input): Parameters<MergeInput>) -> String {
        result_to_string(self.backend.merge_conversations(&input.primary_id, &json!({"secondary_id": input.secondary_id})).await)
    }
}
