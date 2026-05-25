# Customer Service Backend API Specification

The `mcp-customer-service` MCP server is a thin client. It requires a backend API that implements the following REST endpoints.

## Configuration

| Env Var | Required | Description |
|---------|:---:|-------------|
| `CUSTOMER_SERVICE_API_URL` | ✅ | Base URL of your backend (e.g. `http://localhost:8080/api/v1`) |
| `CUSTOMER_SERVICE_API_KEY` | ❌ | Bearer token for auth (sent as `Authorization: Bearer <key>`) |

## Endpoints

### Conversations

#### `GET /conversations`

List conversations. Supports query params:

| Param | Type | Description |
|-------|------|-------------|
| `status` | string | Filter: `open`, `waiting`, `resolved`, `closed` |
| `priority` | string | Filter: `low`, `medium`, `high`, `urgent` |
| `assigned_agent` | string | Filter by agent ID |

**Response:** `200 OK`
```json
[
  {
    "id": "conv-1",
    "customer_id": "cust-1",
    "subject": "Cannot export data",
    "channel": "chat",
    "status": "open",
    "priority": "high",
    "assigned_agent": "agent-1",
    "assigned_team": "Technical Support",
    "tags": ["export", "bug"],
    "messages": [...],
    "created_at": "2026-05-25T01:30:00Z",
    "updated_at": "2026-05-25T02:15:00Z"
  }
]
```

#### `GET /conversations/:id`

Get a single conversation with full message history.

**Response:** `200 OK` — same shape as above, single object.

#### `POST /conversations`

Create a new conversation.

**Request:**
```json
{
  "customer_id": "cust-1",
  "subject": "Need help with billing",
  "channel": "chat",
  "message": "I was charged incorrectly",
  "priority": "high"
}
```

**Response:** `201 Created`
```json
{ "id": "conv-new-123", "status": "open" }
```

#### `PATCH /conversations/:id`

Update conversation fields.

**Request:**
```json
{
  "status": "resolved",
  "priority": "urgent",
  "assigned_agent": "agent-2",
  "assigned_team": "Technical Support"
}
```

All fields optional. **Response:** `200 OK` with updated conversation.

#### `POST /conversations/:id/messages`

Add a message (public reply or internal note).

**Request:**
```json
{
  "body": "Thanks for reaching out. Let me look into this.",
  "sender": "agent",
  "internal": false
}
```

**Response:** `201 Created`
```json
{ "id": "msg-456", "timestamp": "2026-05-25T03:00:00Z" }
```

#### `POST /conversations/:id/escalate`

Escalate a conversation.

**Request:**
```json
{ "reason": "Customer threatening to cancel enterprise contract" }
```

**Response:** `200 OK`
```json
{ "escalated": true, "new_priority": "urgent" }
```

#### `POST /conversations/:id/merge`

Merge another conversation into this one.

**Request:**
```json
{ "secondary_id": "conv-5" }
```

**Response:** `200 OK`
```json
{ "merged": true, "messages_added": 3 }
```

#### `GET /conversations/:id/suggest`

Get an AI-suggested response for the conversation.

**Response:** `200 OK`
```json
{
  "suggested_response": "I understand your concern about the charge...",
  "confidence": 0.85,
  "sources": ["kb-3"]
}
```

---

### Customers

#### `GET /customers/:id`

Get customer profile.

**Response:** `200 OK`
```json
{
  "id": "cust-1",
  "name": "Sarah Mitchell",
  "email": "sarah@acme.com",
  "plan": "Enterprise",
  "lifetime_value": 48000.0,
  "health_score": 85,
  "sentiment_trend": "stable",
  "tickets_last_30d": 1,
  "member_since": "2023-03-15"
}
```

#### `GET /customers/:id/health`

Get customer health assessment.

**Response:** `200 OK`
```json
{
  "customer": "Sarah Mitchell",
  "health_score": 85,
  "churn_risk": "low",
  "sentiment_trend": "stable",
  "factors": {
    "usage_frequency": "high",
    "support_burden": "normal",
    "sentiment": "stable"
  }
}
```

#### `GET /customers/:id/interactions`

Get all past interactions.

**Response:** `200 OK`
```json
[
  { "id": "conv-2", "subject": "SSO help", "status": "resolved", "channel": "email", "created_at": "..." }
]
```

#### `GET /customers/:id/churn-risk`

Assess churn risk.

**Response:** `200 OK`
```json
{
  "customer": "James Park",
  "churn_risk_score": 55,
  "risk_level": "high",
  "signals": ["Declining sentiment", "High ticket volume", "Low engagement"],
  "recommendation": "Proactive outreach recommended"
}
```

---

### Agents

#### `GET /agents`

List agents with status.

**Response:** `200 OK`
```json
[
  { "id": "agent-1", "name": "Alice", "team": "General Support", "status": "available", "active_conversations": 3 }
]
```

---

### Knowledge Base

#### `GET /kb/search?q=:query`

Search knowledge base articles.

**Response:** `200 OK`
```json
[
  { "id": "kb-1", "title": "How to export data", "summary": "...", "category": "Data", "relevance_score": 0.92 }
]
```

---

### Canned Responses

#### `GET /canned-responses`

List approved response templates.

**Response:** `200 OK`
```json
[
  { "id": "cr-1", "name": "Acknowledge & Investigate", "body": "Thank you for reaching out...", "category": "General" }
]
```

---

### Queue

#### `GET /queue/status`

Get queue status.

**Response:** `200 OK`
```json
{
  "unassigned_conversations": 3,
  "agents_available": 2,
  "agents_busy": 1,
  "avg_wait_minutes": 4
}
```

---

### Metrics

#### `GET /metrics/satisfaction`

**Response:** `200 OK`
```json
{
  "csat": { "score": 4.2, "out_of": 5, "responses": 156 },
  "nps": { "score": 42, "promoters_pct": 58, "detractors_pct": 16 },
  "customer_effort": { "score": 2.1, "out_of": 5 }
}
```

#### `GET /metrics/service`

**Response:** `200 OK`
```json
{
  "open_conversations": 5,
  "avg_first_response_min": 4.2,
  "avg_resolution_hours": 6.8,
  "first_contact_resolution_pct": 68.0,
  "volume_today": 23
}
```

---

## Building a Local Backend

Implement these endpoints in any language/framework. Example with Express.js:

```bash
npm init -y && npm install express
```

```javascript
const express = require('express');
const app = express();
app.use(express.json());

let conversations = [/* your data */];
let customers = [/* your data */];

app.get('/api/v1/conversations', (req, res) => {
  let result = conversations;
  if (req.query.status) result = result.filter(c => c.status === req.query.status);
  res.json(result);
});

// ... implement remaining endpoints

app.listen(8080, () => console.log('Backend running on :8080'));
```

Then configure the MCP:
```bash
export CUSTOMER_SERVICE_API_URL=http://localhost:8080/api/v1
mcp-customer-service
```

Or use any backend: Django, FastAPI, Go, Rails — as long as it implements this spec.
