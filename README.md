# Customer Service MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-customer-service.svg)](https://crates.io/crates/mcp-customer-service)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Customer service platform for AI agents — conversations, customer intelligence, knowledge base, response suggestions, churn prediction, routing, and satisfaction tracking. 20 tools with realistic demo data.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-customer-service/main/docs/assets/architecture.svg" alt="MCP Customer Service Architecture" width="850"/>
</p>

## Tools (20)

### Conversations (5)

| Tool | Purpose | Risk |
|------|---------|------|
| `list_conversations` | Filter by status, priority, agent | read_only |
| `get_conversation` | Full thread with messages | read_only |
| `start_conversation` | Start new conversation | internal_write |
| `reply_conversation` | Public reply to customer | external_write |
| `add_internal_note` | Internal note (not visible) | internal_write |

### Customer Intelligence (4)

| Tool | Purpose | Risk |
|------|---------|------|
| `get_customer_profile` | Plan, LTV, health, history | read_only |
| `get_customer_health` | Health score + contributing factors | read_only |
| `get_interaction_history` | All past conversations | read_only |
| `assess_churn_risk` | Churn prediction with signals | read_only |

### Resolution (4)

| Tool | Purpose | Risk |
|------|---------|------|
| `search_knowledge_base` | Find relevant help articles | read_only |
| `suggest_response` | AI-suggested reply based on context | read_only |
| `get_canned_responses` | Approved response templates | read_only |
| `resolve_conversation` | Mark resolved + trigger CSAT | internal_write |

### Routing & Escalation (3)

| Tool | Purpose | Risk |
|------|---------|------|
| `assign_agent` | Assign to agent or team | internal_write |
| `escalate` | Escalate with reason | internal_write |
| `get_queue_status` | Queue depth, wait times | read_only |

### Metrics & Management (4)

| Tool | Purpose | Risk |
|------|---------|------|
| `get_satisfaction_scores` | CSAT, NPS, effort scores | read_only |
| `get_service_metrics` | Response time, FCR, volume | read_only |
| `list_agents` | Agent availability | read_only |
| `merge_conversations` | Merge duplicates | destructive |

## Installation

```bash
cargo install mcp-customer-service
```

## Configuration

The MCP server is a thin API client. It requires a backend that implements the [Customer Service API Spec](docs/api-spec.md).

| Env Var | Required | Description |
|---------|:---:|-------------|
| `CUSTOMER_SERVICE_API_URL` | ✅ | Base URL of your backend (e.g. `http://localhost:8080/api/v1`) |
| `CUSTOMER_SERVICE_API_KEY` | ❌ | Bearer token for auth |

### Supported Backends

| Backend | How |
|---------|-----|
| **Your own API** | Implement the [API spec](docs/api-spec.md) in any language |
| **Zendesk** | Build an adapter that maps Zendesk API → this spec |
| **Freshdesk** | Build an adapter that maps Freshdesk API → this spec |
| **Intercom** | Build an adapter that maps Intercom API → this spec |

The MCP server doesn't store any data — it's a pure passthrough to your backend.

## Client Configuration

```json
{
  "mcpServers": {
    "customer-service": {
      "command": "mcp-customer-service",
      "args": [],
      "env": {
        "CUSTOMER_SERVICE_API_URL": "http://localhost:8080/api/v1",
        "CUSTOMER_SERVICE_API_KEY": "your-api-key"
      }
    }
  }
}
```

## AI Agent Workflow

```
1. Triage    → list_conversations(status="open") → classify priority
2. Understand → get_customer_profile + get_customer_health → context
3. Resolve   → search_knowledge_base + suggest_response → draft reply
4. Respond   → reply_conversation (or escalate if complex)
5. Close     → resolve_conversation → CSAT survey sent
```

## Usage Examples

### Handle an angry customer
```
"There's an urgent billing complaint"
→ list_conversations(status="open", priority="urgent")
→ get_conversation(id="conv-3")
→ get_customer_profile(id="cust-4") — Free plan, health 30, declining
→ assess_churn_risk(id="cust-4") — HIGH risk, 5 signals
→ suggest_response(id="conv-3") — "I understand your concern about the charge..."
→ reply_conversation(id="conv-3", body="...")
```

### Proactive churn prevention
```
"Which customers are at risk of churning?"
→ assess_churn_risk for each customer
→ start_conversation for high-risk ones with proactive outreach
```

## Demo Data Highlights

| Customer | Plan | Health | Sentiment | Churn Risk |
|----------|------|:------:|-----------|:----------:|
| Sarah Mitchell | Enterprise | 85 | Stable | Low |
| James Park | Pro | 45 | Declining | High |
| Maria Garcia | Enterprise | 92 | Improving | Low |
| Tom Wilson | Free | 30 | Declining | High |
| Lisa Chen | Pro | 72 | Stable | Medium |

## MCP Server Manifest

```toml
server_id = "mcp_customer_service"
display_name = "Customer Service"
version = "1.0.0"
domain = "customer-service"
risk_level = "medium"
writes_allowed = "gated"
```

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)
