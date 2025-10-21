---
sidebar_position: 8
---

# Cost Tracking & Monitoring

OpenSpec includes comprehensive AI cost tracking to help you monitor usage, optimize spending, and stay within budget. This guide covers the cost dashboard, usage analytics, and optimization strategies.

## Overview

Every AI operation in OpenSpec is tracked:
- **What**: Operation type (PRD analysis, task generation, etc.)
- **When**: Timestamp of request
- **Who**: Project association
- **How Much**: Input/output tokens and estimated cost
- **Which Model**: Provider and model used
- **Success/Failure**: Error tracking

All data stored locally in SQLite for privacy and offline access.

## Cost Dashboard

Access the cost dashboard:

**Via UI:**
1. Navigate to project **Specs** tab
2. Click **"AI Usage"** tab
3. View comprehensive cost analytics

**Dashboard Sections:**

### 1. Summary Cards

Four key metrics at a glance:

**Total Cost**
- All-time AI spending
- Trend indicator (↑ or ↓ from last period)
- Example: $12.45 (+$3.20 this week)

**Total Tokens**
- Input + output tokens consumed
- Breakdown by type
- Example: 450k (300k input, 150k output)

**Total Requests**
- Number of AI operations
- Success rate
- Example: 145 requests (98% success)

**Average Duration**
- Mean response time
- Slowest operation type
- Example: 2.3s (PRD analysis: 4.5s)

### 2. By Operation Tab

Breakdown by operation type:

| Operation | Requests | Tokens | Cost | Avg Duration |
|-----------|----------|--------|------|--------------|
| PRD Analysis | 12 | 180k | $3.60 | 4.5s |
| Task Generation | 45 | 120k | $2.40 | 1.8s |
| Spec Suggestions | 23 | 80k | $1.60 | 2.1s |
| Task Validation | 38 | 60k | $1.20 | 1.5s |
| Spec Refinement | 8 | 40k | $0.80 | 3.2s |

**Visual progress bars** show relative costs.

### 3. By Model Tab

Compare costs across models:

| Model | Provider | Requests | Cost | Avg Cost/Request |
|-------|----------|----------|------|------------------|
| claude-3-5-sonnet | Anthropic | 85 | $8.50 | $0.10 |
| gpt-4-turbo | OpenAI | 42 | $3.15 | $0.075 |
| claude-3-haiku | Anthropic | 18 | $0.80 | $0.044 |

**Insights:**
- Which models are most cost-effective
- Usage distribution across providers
- Cost per operation type by model

### 4. By Provider Tab

Provider-level breakdown:

| Provider | Requests | Total Cost | Input Tokens | Output Tokens |
|----------|----------|------------|--------------|---------------|
| Anthropic | 103 | $9.30 | 280k | 110k |
| OpenAI | 42 | $3.15 | 120k | 40k |

**Features:**
- Compare provider pricing
- Identify primary vs secondary providers
- Track provider-specific usage patterns

### 5. Recent Logs Tab

Detailed log viewer with filters:

**Columns:**
- Timestamp
- Operation type
- Model used
- Tokens (input/output)
- Cost
- Duration
- Status (success/error)

**Filters:**
- Date range
- Operation type
- Model
- Provider
- Status

**Actions:**
- Export to CSV
- View request details
- Retry failed requests

## Database Schema

### ai_usage_logs Table

```sql
CREATE TABLE ai_usage_logs (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    request_id TEXT,              -- Vercel AI Gateway request ID
    operation TEXT NOT NULL,       -- Operation type
    model TEXT NOT NULL,           -- e.g., 'claude-3-5-sonnet'
    provider TEXT NOT NULL,        -- 'anthropic', 'openai', etc.
    input_tokens INTEGER,
    output_tokens INTEGER,
    total_tokens INTEGER,
    estimated_cost REAL,           -- In USD
    duration_ms INTEGER,           -- Response time
    error TEXT,                    -- NULL if success
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX idx_ai_usage_logs_project ON ai_usage_logs(project_id);
CREATE INDEX idx_ai_usage_logs_created ON ai_usage_logs(created_at);
CREATE INDEX idx_ai_usage_logs_operation ON ai_usage_logs(operation);
CREATE INDEX idx_ai_usage_logs_model ON ai_usage_logs(model);
```

## Cost Calculation

### Pricing Model

OpenSpec uses official provider pricing:

**Anthropic (as of 2025-01)**

| Model | Input (per 1M tokens) | Output (per 1M tokens) |
|-------|----------------------|------------------------|
| Claude 3.5 Sonnet | $3.00 | $15.00 |
| Claude 3 Opus | $15.00 | $75.00 |
| Claude 3 Haiku | $0.25 | $1.25 |

**OpenAI (as of 2025-01)**

| Model | Input (per 1M tokens) | Output (per 1M tokens) |
|-------|----------------------|------------------------|
| GPT-4 Turbo | $10.00 | $30.00 |
| GPT-4 | $30.00 | $60.00 |
| GPT-3.5 Turbo | $0.50 | $1.50 |

### Calculation Formula

```typescript
function calculateCost(
  model: string,
  inputTokens: number,
  outputTokens: number
): number {
  const pricing = MODEL_PRICING[model];

  const inputCost = (inputTokens / 1_000_000) * pricing.input;
  const outputCost = (outputTokens / 1_000_000) * pricing.output;

  return inputCost + outputCost;
}
```

**Example:**
```
Model: claude-3-5-sonnet
Input: 5,000 tokens
Output: 2,000 tokens

Input Cost: (5,000 / 1,000,000) * $3.00 = $0.015
Output Cost: (2,000 / 1,000,000) * $15.00 = $0.030
Total: $0.045
```

## Usage Analytics

### API Endpoints

**Get Usage Logs**
```bash
GET /api/ai-usage/logs?project_id=PROJECT_ID&limit=100&offset=0

Query Parameters:
- project_id: Filter by project
- operation: Filter by operation type
- model: Filter by model
- provider: Filter by provider
- start_date: From date (ISO 8601)
- end_date: To date (ISO 8601)
- limit: Results per page (default: 100)
- offset: Pagination offset
```

**Response:**
```json
{
  "success": true,
  "data": {
    "logs": [
      {
        "id": "log_123",
        "project_id": "proj_abc",
        "operation": "prd_analysis",
        "model": "claude-3-5-sonnet",
        "provider": "anthropic",
        "input_tokens": 5000,
        "output_tokens": 2000,
        "total_tokens": 7000,
        "estimated_cost": 0.045,
        "duration_ms": 4500,
        "error": null,
        "created_at": "2025-01-20T10:30:00Z"
      }
    ],
    "total": 145,
    "limit": 100,
    "offset": 0
  }
}
```

**Get Usage Statistics**
```bash
GET /api/ai-usage/stats?project_id=PROJECT_ID&group_by=operation

Query Parameters:
- project_id: Filter by project
- group_by: operation, model, provider, date
- start_date: From date
- end_date: To date
```

**Response:**
```json
{
  "success": true,
  "data": {
    "summary": {
      "total_cost": 12.45,
      "total_tokens": 450000,
      "total_requests": 145,
      "success_rate": 0.98,
      "average_duration_ms": 2300
    },
    "by_operation": [
      {
        "operation": "prd_analysis",
        "count": 12,
        "total_tokens": 180000,
        "total_cost": 3.60,
        "average_cost": 0.30,
        "average_duration_ms": 4500
      }
    ],
    "by_model": [...],
    "by_provider": [...]
  }
}
```

### Export Data

Export usage logs to CSV:

```bash
# Via API
GET /api/ai-usage/export?project_id=PROJECT_ID&format=csv

# Via UI
Click "Export" button in Recent Logs tab
```

**CSV Format:**
```csv
Timestamp,Operation,Model,Provider,Input Tokens,Output Tokens,Cost,Duration (ms),Status
2025-01-20 10:30:00,prd_analysis,claude-3-5-sonnet,anthropic,5000,2000,$0.045,4500,success
2025-01-20 10:35:00,task_generation,gpt-4-turbo,openai,3000,1500,$0.075,1800,success
```

## Budget Management

### Setting Budgets

Configure spending limits:

```typescript
// Project-level budget
const budget = {
  daily: 5.00,      // $5 per day
  weekly: 25.00,    // $25 per week
  monthly: 100.00   // $100 per month
};
```

### Budget Alerts

Configure alerts when thresholds reached:

```typescript
const alerts = [
  {
    type: 'daily_budget',
    threshold: 0.8,  // 80% of daily budget
    action: 'email',
    recipients: ['team@example.com']
  },
  {
    type: 'per_request',
    threshold: 1.00,  // $1 per request
    action: 'log',
    severity: 'warning'
  },
  {
    type: 'monthly_budget',
    threshold: 0.9,  // 90% of monthly budget
    action: 'slack',
    channel: '#ai-costs'
  }
];
```

### Rate Limiting

Prevent runaway costs:

```typescript
const rateLimits = {
  perUser: {
    requests: 10,
    window: '1 hour'
  },
  perProject: {
    requests: 100,
    window: '1 day'
  },
  global: {
    cost: 50.00,
    window: '1 day'
  }
};
```

## Cost Optimization

### 1. Model Selection

Choose the right model for each task:

**PRD Analysis** → Claude 3.5 Sonnet
- Needs: Nuanced understanding, long context
- Cost: Moderate ($0.30 per analysis)
- Justification: Quality worth the cost

**Task Generation** → GPT-4 Turbo
- Needs: Structured output, fast response
- Cost: Lower ($0.08 per generation)
- Justification: Simpler task, cheaper model works well

**Task Validation** → Claude Haiku
- Needs: Binary pass/fail decision
- Cost: Very low ($0.02 per validation)
- Justification: Simple task, fastest/cheapest model

### 2. Caching Strategy

Implement aggressive caching:

```typescript
// Cache PRD analysis for 7 days
const cacheKey = `prd-analysis-${prdId}-${prdVersion}`;
const ttl = 7 * 24 * 60 * 60; // 7 days

const cached = await cache.get(cacheKey);
if (cached) {
  // Saved ~$0.30 per hit
  return cached;
}

const result = await analyzePRD(prd);
await cache.set(cacheKey, result, ttl);
return result;
```

**What to Cache:**
- PRD analysis (rarely changes)
- Spec generation (stable requirements)
- Task suggestions (based on hash of requirements)

**What NOT to Cache:**
- Task validation (changes with implementation)
- Orphan suggestions (context changes)
- Real-time operations

### 3. Batching

Batch similar operations:

```typescript
// Instead of 10 separate requests ($0.08 each = $0.80)
for (const task of tasks) {
  await suggestSpec(task);
}

// Batch into 1 request ($0.15 total)
await suggestSpecsBatch(tasks);
```

**Savings:** ~80% reduction for batch operations

### 4. Token Optimization

Reduce token usage:

**Minimize Prompt Tokens:**
```typescript
// Bad: Send entire PRD every time (5000 tokens)
const prompt = `Analyze this PRD: ${prdContent}`;

// Good: Send only relevant sections (1000 tokens)
const prompt = `Analyze these requirements: ${relevantSections}`;
```

**Limit Response Tokens:**
```typescript
// Set maxTokens based on expected output
maxTokens: 2000  // For task generation
maxTokens: 500   // For validation
```

**Use Structured Outputs:**
```typescript
// JSON schema limits response to required fields
schema: TaskSuggestionSchema  // Only returns what's needed
```

### 5. Smart Retry Logic

Avoid expensive retries:

```typescript
// Don't retry expensive operations automatically
if (cost > 0.50) {
  // Log error, notify user, don't retry
  throw new Error('Operation too expensive to auto-retry');
}

// Retry cheap operations with exponential backoff
if (cost < 0.10) {
  await retryWithBackoff(operation, { maxRetries: 3 });
}
```

## Cost Benchmarks

### Typical Usage Patterns

**Small Project** (1-10 developers)
- PRDs: 2-5 per month
- Specs: 10-20 per month
- Task generations: 50-100 per month
- Validations: 100-200 per month
- **Monthly Cost:** $15-30

**Medium Project** (10-50 developers)
- PRDs: 10-20 per month
- Specs: 50-100 per month
- Task generations: 200-500 per month
- Validations: 500-1000 per month
- **Monthly Cost:** $60-120

**Large Project** (50+ developers)
- PRDs: 30-50 per month
- Specs: 200-500 per month
- Task generations: 1000-2000 per month
- Validations: 2000-5000 per month
- **Monthly Cost:** $200-400

### Cost per Operation

Average costs by operation:

| Operation | Input Tokens | Output Tokens | Cost Range |
|-----------|--------------|---------------|------------|
| PRD Analysis | 3000-8000 | 2000-5000 | $0.20-0.50 |
| Spec Generation | 1000-3000 | 1500-3000 | $0.08-0.20 |
| Task Generation | 800-2000 | 500-1500 | $0.05-0.15 |
| Orphan Analysis | 500-1500 | 800-2000 | $0.03-0.10 |
| Task Validation | 600-1200 | 300-800 | $0.02-0.06 |
| Spec Refinement | 2000-5000 | 1500-3000 | $0.10-0.25 |
| PRD Regeneration | 5000-15000 | 3000-8000 | $0.25-0.60 |

## Monitoring & Alerts

### Real-Time Monitoring

Dashboard updates every 5 seconds when active:

```typescript
// Auto-refresh cost stats
const { data, refetch } = useQuery({
  queryKey: ['ai-usage-stats', projectId],
  queryFn: () => fetchAIUsageStats(projectId),
  refetchInterval: 5000  // 5 seconds
});
```

### Alert Configuration

Set up email/Slack alerts:

```typescript
const alertConfig = {
  dailyBudget: {
    threshold: 10.00,
    notify: ['team@example.com'],
    action: 'warn'  // warn, block, notify
  },
  unusualActivity: {
    spikeThreshold: 2.0,  // 2x normal usage
    notify: ['admin@example.com'],
    action: 'block'
  },
  failureRate: {
    threshold: 0.1,  // 10% failure rate
    notify: ['tech@example.com'],
    action: 'notify'
  }
};
```

### Cost Anomaly Detection

Automatically detect unusual spending:

```typescript
// Check for spending spikes
const lastWeekAvg = getAverageDailyCost(7);
const today = getTodayCost();

if (today > lastWeekAvg * 2) {
  sendAlert({
    type: 'cost_spike',
    message: `Today's cost ($${today}) is 2x normal ($${lastWeekAvg})`,
    severity: 'high'
  });
}
```

## Best Practices

### Cost-Conscious Development

✅ **Do:**
- Cache aggressively
- Batch similar operations
- Use cheapest model that works
- Set budgets and alerts
- Monitor costs weekly
- Optimize prompts for brevity

❌ **Don't:**
- Send full PRDs when excerpts work
- Retry expensive operations automatically
- Use high-end models for simple tasks
- Skip validation on large operations
- Ignore cost trends

### Team Practices

**Weekly Reviews:**
- Check cost dashboard
- Identify expensive operations
- Optimize high-cost workflows
- Review caching effectiveness

**Monthly Planning:**
- Set next month's budget
- Analyze trends
- Adjust model selection
- Update cost policies

## Troubleshooting

### Unexpected High Costs

**Problem:** Cost spike without obvious cause

**Investigation:**
1. Check Recent Logs for large operations
2. Filter by cost (descending)
3. Identify operation type
4. Review prompt/input size
5. Check for retry loops

**Solutions:**
- Optimize expensive prompts
- Add caching where missing
- Set per-operation cost limits
- Block problematic operations

### Missing Cost Data

**Problem:** Usage logs not recording

**Solutions:**
1. Verify AI operations are using logging wrapper
2. Check database for errors
3. Ensure `ai_usage_logs` table exists
4. Review application logs for errors

### Inaccurate Cost Estimates

**Problem:** Estimated costs don't match actual

**Solutions:**
1. Update pricing data (models change)
2. Verify token counting is accurate
3. Check for billing discrepancies
4. Compare with provider dashboard

## API Reference

### Get Usage Logs
```bash
GET /api/ai-usage/logs
```

### Get Usage Statistics
```bash
GET /api/ai-usage/stats
```

### Export Usage Data
```bash
GET /api/ai-usage/export
```

### Get Cost Breakdown
```bash
GET /api/ai-usage/breakdown?group_by=operation&period=week
```

## Next Steps

- [Optimize AI prompts](./ai-features.md#prompt-engineering)
- [Configure caching](./ai-features.md#context-management)
- [Set up alerts](/docs/configuration/environment-variables#ai-configuration)
- [Review workflows](./workflows.md)

---

**Related**: [AI Features](./ai-features.md) | [Configuration](/docs/configuration/environment-variables) | [Workflows](./workflows.md)
