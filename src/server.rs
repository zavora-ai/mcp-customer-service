use crate::domain::*;
use crate::store::Store;
use chrono::Utc;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};

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
pub struct CsServer { pub store: Store }

#[tool_router(server_handler)]
impl CsServer {
    // === Conversations (5) ===

    #[tool(description = "List conversations filtered by status, priority, or assigned agent")]
    async fn list_conversations(&self, Parameters(input): Parameters<FilterInput>) -> String {
        let convs = self.store.conversations.read().await;
        let filtered: Vec<serde_json::Value> = convs.iter()
            .filter(|c| input.status.as_ref().map_or(true, |s| c.status == *s))
            .filter(|c| input.priority.as_ref().map_or(true, |p| c.priority == *p))
            .filter(|c| input.assigned_agent.as_ref().map_or(true, |a| c.assigned_agent.as_ref() == Some(a)))
            .map(|c| serde_json::json!({"id": c.id, "subject": c.subject, "customer_id": c.customer_id, "status": c.status, "priority": c.priority, "channel": c.channel, "assigned_agent": c.assigned_agent, "messages": c.messages.len(), "updated_at": c.updated_at}))
            .collect();
        serde_json::to_string_pretty(&filtered).unwrap()
    }

    #[tool(description = "Get full conversation with message history")]
    async fn get_conversation(&self, Parameters(input): Parameters<IdInput>) -> String {
        let convs = self.store.conversations.read().await;
        match convs.iter().find(|c| c.id == input.id) {
            Some(c) => serde_json::to_string_pretty(c).unwrap(),
            None => format!("Conversation {} not found", input.id),
        }
    }

    #[tool(description = "Start a new conversation from a customer")]
    async fn start_conversation(&self, Parameters(input): Parameters<StartConvInput>) -> String {
        let id = format!("conv-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        let now = Utc::now().to_rfc3339();
        let conv = Conversation {
            id: id.clone(), customer_id: input.customer_id, subject: input.subject.clone(),
            channel: input.channel.unwrap_or("chat".into()), status: "open".into(),
            priority: input.priority.unwrap_or("medium".into()),
            assigned_agent: None, assigned_team: None, tags: vec![],
            messages: vec![Message { id: "m1".into(), sender: "customer".into(), body: input.message, internal: false, timestamp: now.clone() }],
            created_at: now.clone(), updated_at: now,
        };
        self.store.conversations.write().await.push(conv);
        format!("Started conversation '{}' (id: {})", input.subject, id)
    }

    #[tool(description = "Send a public reply to the customer")]
    async fn reply_conversation(&self, Parameters(input): Parameters<ReplyInput>) -> String {
        let mut convs = self.store.conversations.write().await;
        match convs.iter_mut().find(|c| c.id == input.conversation_id) {
            Some(c) => {
                let mid = format!("m{}", c.messages.len() + 1);
                c.messages.push(Message { id: mid, sender: "agent".into(), body: input.body, internal: false, timestamp: Utc::now().to_rfc3339() });
                c.updated_at = Utc::now().to_rfc3339();
                c.status = "waiting".into();
                format!("Reply sent on conversation {}", input.conversation_id)
            }
            None => format!("Conversation {} not found", input.conversation_id),
        }
    }

    #[tool(description = "Add an internal note (not visible to customer)")]
    async fn add_internal_note(&self, Parameters(input): Parameters<NoteInput>) -> String {
        let mut convs = self.store.conversations.write().await;
        match convs.iter_mut().find(|c| c.id == input.conversation_id) {
            Some(c) => {
                let mid = format!("m{}", c.messages.len() + 1);
                c.messages.push(Message { id: mid, sender: "agent".into(), body: input.body, internal: true, timestamp: Utc::now().to_rfc3339() });
                format!("Internal note added to {}", input.conversation_id)
            }
            None => format!("Conversation {} not found", input.conversation_id),
        }
    }

    // === Customer Intelligence (4) ===

    #[tool(description = "Get customer profile: plan, lifetime value, health score, history")]
    async fn get_customer_profile(&self, Parameters(input): Parameters<IdInput>) -> String {
        let customers = self.store.customers.read().await;
        match customers.iter().find(|c| c.id == input.id) {
            Some(c) => serde_json::to_string_pretty(c).unwrap(),
            None => format!("Customer {} not found", input.id),
        }
    }

    #[tool(description = "Get customer health score with contributing factors")]
    async fn get_customer_health(&self, Parameters(input): Parameters<IdInput>) -> String {
        let customers = self.store.customers.read().await;
        match customers.iter().find(|c| c.id == input.id) {
            Some(c) => {
                let risk = if c.health_score < 40 { "high" } else if c.health_score < 70 { "medium" } else { "low" };
                serde_json::to_string_pretty(&serde_json::json!({
                    "customer": c.name, "health_score": c.health_score, "sentiment_trend": c.sentiment_trend,
                    "churn_risk": risk, "tickets_last_30d": c.tickets_last_30d, "plan": c.plan,
                    "factors": {
                        "usage_frequency": if c.health_score > 70 { "high" } else { "low" },
                        "support_burden": if c.tickets_last_30d > 3 { "high" } else { "normal" },
                        "sentiment": c.sentiment_trend
                    }
                })).unwrap()
            }
            None => format!("Customer {} not found", input.id),
        }
    }

    #[tool(description = "Get all past interactions for a customer")]
    async fn get_interaction_history(&self, Parameters(input): Parameters<IdInput>) -> String {
        let convs = self.store.conversations.read().await;
        let history: Vec<serde_json::Value> = convs.iter()
            .filter(|c| c.customer_id == input.id)
            .map(|c| serde_json::json!({"id": c.id, "subject": c.subject, "status": c.status, "channel": c.channel, "created_at": c.created_at}))
            .collect();
        serde_json::to_string_pretty(&history).unwrap()
    }

    #[tool(description = "Assess churn risk for a customer based on signals")]
    async fn assess_churn_risk(&self, Parameters(input): Parameters<IdInput>) -> String {
        let customers = self.store.customers.read().await;
        match customers.iter().find(|c| c.id == input.id) {
            Some(c) => {
                let score = 100 - c.health_score as u32;
                let level = if score > 60 { "high" } else if score > 30 { "medium" } else { "low" };
                let mut signals = Vec::new();
                if c.sentiment_trend == "declining" { signals.push("Declining sentiment trend"); }
                if c.tickets_last_30d > 3 { signals.push("High support ticket volume"); }
                if c.health_score < 50 { signals.push("Low engagement score"); }
                if c.plan == "Free" { signals.push("No financial commitment"); }
                serde_json::to_string_pretty(&serde_json::json!({
                    "customer": c.name, "churn_risk_score": score, "risk_level": level,
                    "signals": signals,
                    "recommendation": if level == "high" { "Proactive outreach recommended — offer concession or success call" } else { "Monitor — no immediate action needed" }
                })).unwrap()
            }
            None => format!("Customer {} not found", input.id),
        }
    }

    // === Resolution (4) ===

    #[tool(description = "Search knowledge base for relevant articles")]
    async fn search_knowledge_base(&self, Parameters(input): Parameters<SearchInput>) -> String {
        let query = input.query.to_lowercase();
        let mut results: Vec<&KbArticle> = self.store.kb_articles.iter()
            .filter(|a| a.title.to_lowercase().contains(&query) || a.summary.to_lowercase().contains(&query) || a.category.to_lowercase().contains(&query))
            .collect();
        if results.is_empty() { results = self.store.kb_articles.iter().take(3).collect(); }
        serde_json::to_string_pretty(&results).unwrap()
    }

    #[tool(description = "Get AI-suggested response based on conversation context")]
    async fn suggest_response(&self, Parameters(input): Parameters<IdInput>) -> String {
        let convs = self.store.conversations.read().await;
        match convs.iter().find(|c| c.id == input.id) {
            Some(c) => {
                let last_msg = c.messages.iter().filter(|m| m.sender == "customer").last().map(|m| m.body.as_str()).unwrap_or("");
                let suggestion = if last_msg.contains("refund") || last_msg.contains("charged") {
                    "I understand your concern about the charge. Let me look into your billing history right away. If this was an error, I'll process a full refund within 24 hours."
                } else if last_msg.contains("export") || last_msg.contains("download") {
                    "I can see the export issue. As a workaround, try clearing your browser cache or using incognito mode. If that doesn't work, I can generate the export on our end and send it to you directly."
                } else if last_msg.contains("slow") || last_msg.contains("hang") || last_msg.contains("spin") {
                    "I'm sorry about the performance issue. Let me check our system status. In the meantime, could you try refreshing the page? I'll investigate on our end."
                } else {
                    "Thank you for reaching out. I'm looking into this now and will have an update for you shortly."
                };
                serde_json::to_string_pretty(&serde_json::json!({"conversation_id": c.id, "suggested_response": suggestion, "confidence": 0.85})).unwrap()
            }
            None => format!("Conversation {} not found", input.id),
        }
    }

    #[tool(description = "List available canned/template responses")]
    async fn get_canned_responses(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        serde_json::to_string_pretty(&self.store.canned_responses).unwrap()
    }

    #[tool(description = "Resolve a conversation and request satisfaction rating")]
    async fn resolve_conversation(&self, Parameters(input): Parameters<IdInput>) -> String {
        let mut convs = self.store.conversations.write().await;
        match convs.iter_mut().find(|c| c.id == input.id) {
            Some(c) => {
                c.status = "resolved".into();
                c.updated_at = Utc::now().to_rfc3339();
                format!("Conversation {} resolved. CSAT survey sent to customer.", input.id)
            }
            None => format!("Conversation {} not found", input.id),
        }
    }

    // === Routing & Escalation (3) ===

    #[tool(description = "Assign conversation to an agent or team")]
    async fn assign_agent(&self, Parameters(input): Parameters<AssignInput>) -> String {
        let mut convs = self.store.conversations.write().await;
        match convs.iter_mut().find(|c| c.id == input.conversation_id) {
            Some(c) => {
                if let Some(a) = input.agent_id { c.assigned_agent = Some(a.clone()); }
                if let Some(t) = input.team { c.assigned_team = Some(t.clone()); }
                format!("Conversation {} assigned", input.conversation_id)
            }
            None => format!("Conversation {} not found", input.conversation_id),
        }
    }

    #[tool(description = "Escalate to senior agent/manager with reason")]
    async fn escalate(&self, Parameters(input): Parameters<EscalateInput>) -> String {
        let mut convs = self.store.conversations.write().await;
        match convs.iter_mut().find(|c| c.id == input.conversation_id) {
            Some(c) => {
                c.priority = "urgent".into();
                c.tags.push("escalated".into());
                let mid = format!("m{}", c.messages.len() + 1);
                c.messages.push(Message { id: mid, sender: "system".into(), body: format!("Escalated: {}", input.reason), internal: true, timestamp: Utc::now().to_rfc3339() });
                format!("Conversation {} escalated: {}", input.conversation_id, input.reason)
            }
            None => format!("Conversation {} not found", input.conversation_id),
        }
    }

    #[tool(description = "Get queue status: depth, wait times, agent availability")]
    async fn get_queue_status(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let convs = self.store.conversations.read().await;
        let agents = self.store.agents.read().await;
        let unassigned = convs.iter().filter(|c| c.assigned_agent.is_none() && c.status == "open").count();
        let available = agents.iter().filter(|a| a.status == "available").count();
        serde_json::to_string_pretty(&serde_json::json!({
            "unassigned_conversations": unassigned,
            "agents_available": available,
            "agents_busy": agents.iter().filter(|a| a.status == "busy").count(),
            "avg_wait_minutes": if available > 0 { 3 } else { 12 },
            "queue_depth_by_priority": {"urgent": 1, "high": 1, "medium": 2, "low": 1}
        })).unwrap()
    }

    // === Metrics (2) ===

    #[tool(description = "Get satisfaction scores: CSAT, NPS, customer effort")]
    async fn get_satisfaction_scores(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "csat": {"score": 4.2, "out_of": 5, "responses": 156, "trend": "+0.1 vs last month"},
            "nps": {"score": 42, "promoters_pct": 58, "detractors_pct": 16, "trend": "+3 vs last month"},
            "customer_effort": {"score": 2.1, "out_of": 5, "lower_is_better": true}
        })).unwrap()
    }

    #[tool(description = "Get service metrics: response time, resolution time, FCR, volume")]
    async fn get_service_metrics(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let convs = self.store.conversations.read().await;
        let open = convs.iter().filter(|c| c.status == "open").count();
        serde_json::to_string_pretty(&serde_json::json!({
            "open_conversations": open,
            "avg_first_response_min": 4.2,
            "avg_resolution_hours": 6.8,
            "first_contact_resolution_pct": 68.0,
            "volume_today": 23,
            "volume_trend": "+12% vs last week",
            "busiest_channel": "chat",
            "top_categories": ["billing", "export", "api", "sso"]
        })).unwrap()
    }

    // === Extra ===

    #[tool(description = "List available agents and their status")]
    async fn list_agents(&self, Parameters(_): Parameters<EmptyInput>) -> String {
        let agents = self.store.agents.read().await;
        serde_json::to_string_pretty(&*agents).unwrap()
    }

    #[tool(description = "Merge duplicate conversations into one")]
    async fn merge_conversations(&self, Parameters(input): Parameters<MergeInput>) -> String {
        let mut convs = self.store.conversations.write().await;
        let secondary = convs.iter().find(|c| c.id == input.secondary_id).cloned();
        match secondary {
            Some(sec) => {
                if let Some(primary) = convs.iter_mut().find(|c| c.id == input.primary_id) {
                    primary.messages.extend(sec.messages);
                    primary.tags.push("merged".into());
                }
                convs.retain(|c| c.id != input.secondary_id);
                format!("Merged {} into {}", input.secondary_id, input.primary_id)
            }
            None => format!("Conversation {} not found", input.secondary_id),
        }
    }
}
