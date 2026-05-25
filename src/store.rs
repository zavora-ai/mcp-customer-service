use crate::domain::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Store {
    pub conversations: Arc<RwLock<Vec<Conversation>>>,
    pub customers: Arc<RwLock<Vec<Customer>>>,
    pub agents: Arc<RwLock<Vec<Agent>>>,
    pub kb_articles: Vec<KbArticle>,
    pub canned_responses: Vec<CannedResponse>,
}

impl Store {
    pub fn seeded() -> Self {
        let customers = vec![
            Customer { id: "cust-1".into(), name: "Sarah Mitchell".into(), email: "sarah@acme.com".into(), plan: "Enterprise".into(), lifetime_value: 48000.0, health_score: 85, sentiment_trend: "stable".into(), tickets_last_30d: 1, member_since: "2023-03-15".into() },
            Customer { id: "cust-2".into(), name: "James Park".into(), email: "james@startup.io".into(), plan: "Pro".into(), lifetime_value: 12000.0, health_score: 45, sentiment_trend: "declining".into(), tickets_last_30d: 5, member_since: "2024-01-10".into() },
            Customer { id: "cust-3".into(), name: "Maria Garcia".into(), email: "maria@bigcorp.com".into(), plan: "Enterprise".into(), lifetime_value: 96000.0, health_score: 92, sentiment_trend: "improving".into(), tickets_last_30d: 0, member_since: "2022-06-01".into() },
            Customer { id: "cust-4".into(), name: "Tom Wilson".into(), email: "tom@freelance.dev".into(), plan: "Free".into(), lifetime_value: 0.0, health_score: 30, sentiment_trend: "declining".into(), tickets_last_30d: 8, member_since: "2025-11-20".into() },
            Customer { id: "cust-5".into(), name: "Lisa Chen".into(), email: "lisa@midsize.co".into(), plan: "Pro".into(), lifetime_value: 7200.0, health_score: 72, sentiment_trend: "stable".into(), tickets_last_30d: 2, member_since: "2024-08-05".into() },
        ];
        let agents = vec![
            Agent { id: "agent-1".into(), name: "Alice Nguyen".into(), team: "General Support".into(), status: "available".into(), active_conversations: 3 },
            Agent { id: "agent-2".into(), name: "Bob Martinez".into(), team: "Technical Support".into(), status: "busy".into(), active_conversations: 5 },
            Agent { id: "agent-3".into(), name: "Carol Thompson".into(), team: "General Support".into(), status: "available".into(), active_conversations: 2 },
        ];
        let conversations = vec![
            Conversation { id: "conv-1".into(), customer_id: "cust-2".into(), subject: "Cannot export data to CSV".into(), channel: "chat".into(), status: "open".into(), priority: "high".into(), assigned_agent: Some("agent-2".into()), assigned_team: Some("Technical Support".into()), tags: vec!["export".into(), "bug".into()], created_at: "2026-05-25T01:30:00Z".into(), updated_at: "2026-05-25T02:15:00Z".into(), messages: vec![
                Message { id: "m1".into(), sender: "customer".into(), body: "I've been trying to export my data for the past hour. The CSV button just spins and nothing downloads. This is blocking my quarterly report.".into(), internal: false, timestamp: "2026-05-25T01:30:00Z".into() },
                Message { id: "m2".into(), sender: "agent".into(), body: "I'm sorry about the trouble, James. Let me look into this right away. Can you tell me which browser you're using?".into(), internal: false, timestamp: "2026-05-25T01:35:00Z".into() },
                Message { id: "m3".into(), sender: "customer".into(), body: "Chrome 125 on Mac. I've tried Firefox too, same issue.".into(), internal: false, timestamp: "2026-05-25T01:38:00Z".into() },
            ]},
            Conversation { id: "conv-2".into(), customer_id: "cust-1".into(), subject: "SSO configuration help".into(), channel: "email".into(), status: "waiting".into(), priority: "medium".into(), assigned_agent: Some("agent-1".into()), assigned_team: Some("General Support".into()), tags: vec!["sso".into(), "setup".into()], created_at: "2026-05-24T14:00:00Z".into(), updated_at: "2026-05-24T16:00:00Z".into(), messages: vec![
                Message { id: "m1".into(), sender: "customer".into(), body: "We're setting up SSO with Okta. The docs mention a metadata URL but I can't find where to enter it.".into(), internal: false, timestamp: "2026-05-24T14:00:00Z".into() },
                Message { id: "m2".into(), sender: "agent".into(), body: "Hi Sarah! Go to Settings → Security → SSO. You'll see the metadata URL field under 'Identity Provider Configuration'. I've attached a screenshot.".into(), internal: false, timestamp: "2026-05-24T14:30:00Z".into() },
            ]},
            Conversation { id: "conv-3".into(), customer_id: "cust-4".into(), subject: "Why was I charged? I'm on free plan".into(), channel: "chat".into(), status: "open".into(), priority: "urgent".into(), assigned_agent: None, assigned_team: None, tags: vec!["billing".into(), "complaint".into()], created_at: "2026-05-25T03:00:00Z".into(), updated_at: "2026-05-25T03:00:00Z".into(), messages: vec![
                Message { id: "m1".into(), sender: "customer".into(), body: "I just got charged $49 on my credit card but I never upgraded from the free plan. This is unacceptable. I want a refund immediately.".into(), internal: false, timestamp: "2026-05-25T03:00:00Z".into() },
            ]},
            Conversation { id: "conv-4".into(), customer_id: "cust-5".into(), subject: "Feature request: dark mode".into(), channel: "email".into(), status: "resolved".into(), priority: "low".into(), assigned_agent: Some("agent-3".into()), assigned_team: Some("General Support".into()), tags: vec!["feature-request".into()], created_at: "2026-05-20T10:00:00Z".into(), updated_at: "2026-05-20T11:00:00Z".into(), messages: vec![
                Message { id: "m1".into(), sender: "customer".into(), body: "Would love to see a dark mode option. The white background is harsh at night.".into(), internal: false, timestamp: "2026-05-20T10:00:00Z".into() },
                Message { id: "m2".into(), sender: "agent".into(), body: "Great suggestion, Lisa! I've added this to our feature request board. Dark mode is actually on our Q3 roadmap. I'll notify you when it ships!".into(), internal: false, timestamp: "2026-05-20T11:00:00Z".into() },
            ]},
            Conversation { id: "conv-5".into(), customer_id: "cust-3".into(), subject: "API rate limit increase request".into(), channel: "email".into(), status: "waiting".into(), priority: "medium".into(), assigned_agent: Some("agent-2".into()), assigned_team: Some("Technical Support".into()), tags: vec!["api".into(), "enterprise".into()], created_at: "2026-05-23T09:00:00Z".into(), updated_at: "2026-05-24T10:00:00Z".into(), messages: vec![
                Message { id: "m1".into(), sender: "customer".into(), body: "We're hitting the 1000 req/min limit during peak hours. Can we get this increased to 5000 for our enterprise plan?".into(), internal: false, timestamp: "2026-05-23T09:00:00Z".into() },
                Message { id: "m2".into(), sender: "agent".into(), body: "Hi Maria, I've escalated this to our infrastructure team. Enterprise plans can go up to 10,000 req/min. I'll have an update within 24 hours.".into(), internal: false, timestamp: "2026-05-23T09:30:00Z".into() },
                Message { id: "m3".into(), sender: "agent".into(), body: "Internal: Checked with infra — approved for 5000. Waiting on deployment.".into(), internal: true, timestamp: "2026-05-24T10:00:00Z".into() },
            ]},
        ];
        let kb_articles = vec![
            KbArticle { id: "kb-1".into(), title: "How to export data to CSV".into(), summary: "Step-by-step guide to exporting your data. Go to Reports → Export → Select CSV format.".into(), category: "Data".into(), relevance_score: 0.0 },
            KbArticle { id: "kb-2".into(), title: "Setting up SSO with Okta/Azure AD".into(), summary: "Configure SAML SSO: Settings → Security → SSO → Enter metadata URL from your IdP.".into(), category: "Security".into(), relevance_score: 0.0 },
            KbArticle { id: "kb-3".into(), title: "Understanding your billing".into(), summary: "Billing FAQ: plan changes, refunds, invoices. Free plan users are never charged.".into(), category: "Billing".into(), relevance_score: 0.0 },
            KbArticle { id: "kb-4".into(), title: "API rate limits by plan".into(), summary: "Free: 100/min, Pro: 1000/min, Enterprise: 5000/min (upgradeable to 10,000).".into(), category: "API".into(), relevance_score: 0.0 },
            KbArticle { id: "kb-5".into(), title: "Troubleshooting export issues".into(), summary: "If exports hang: clear cache, try incognito, check if dataset exceeds 1M rows.".into(), category: "Data".into(), relevance_score: 0.0 },
        ];
        let canned_responses = vec![
            CannedResponse { id: "cr-1".into(), name: "Acknowledge & Investigate".into(), body: "Thank you for reaching out. I'm looking into this now and will have an update for you shortly.".into(), category: "General".into() },
            CannedResponse { id: "cr-2".into(), name: "Escalation Notice".into(), body: "I've escalated this to our specialist team for a faster resolution. You'll hear back within [timeframe].".into(), category: "Escalation".into() },
            CannedResponse { id: "cr-3".into(), name: "Resolution + CSAT".into(), body: "This should now be resolved. Please let me know if you need anything else! How would you rate your experience today?".into(), category: "Closing".into() },
            CannedResponse { id: "cr-4".into(), name: "Refund Processed".into(), body: "I've processed a full refund to your original payment method. It should appear within 5-10 business days.".into(), category: "Billing".into() },
            CannedResponse { id: "cr-5".into(), name: "Feature Request Logged".into(), body: "Great suggestion! I've added this to our product roadmap. I'll notify you when it's available.".into(), category: "Feature".into() },
        ];
        Self {
            conversations: Arc::new(RwLock::new(conversations)),
            customers: Arc::new(RwLock::new(customers)),
            agents: Arc::new(RwLock::new(agents)),
            kb_articles,
            canned_responses,
        }
    }
}
