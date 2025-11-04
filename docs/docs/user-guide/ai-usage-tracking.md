# AI Usage Tracking

Orkee provides comprehensive tracking of all AI operations, including token usage, costs, and tool invocations. All tracking is automatic and completely local - no data is sent externally.

## Overview

Every AI operation in Orkee is automatically tracked with detailed metrics:

- **Token Usage**: Input, output, and total tokens for every request
- **Cost Tracking**: Real-time cost estimation based on provider pricing
- **Tool Analytics**: Which tools were called, success rates, and performance
- **Duration Metrics**: Actual request duration with millisecond precision
- **Model Information**: Which AI model and provider was used
- **Response Metadata**: Finish reasons, IDs, and provider-specific data

## Features

### Automatic Tracking

All AI SDK calls are tracked automatically with zero manual configuration:

- ✅ **Zero Configuration**: Works out of the box
- ✅ **No Manual Logging**: Telemetry happens transparently
- ✅ **Async Operation**: No performance impact on AI requests
- ✅ **Error Resilient**: Telemetry failures don't break AI operations

### Cost Monitoring

Track spending across different AI providers:

- **Multi-Provider Support**: Anthropic, OpenAI, Google, xAI, and more
- **Real-Time Costs**: Instant cost calculation based on token usage
- **Cost Breakdown**: See costs per operation, model, and provider
- **Historical Trends**: Track spending over time

### Tool Call Analytics

Understand how AI tools are being used:

- **Tool Invocation Tracking**: Which tools are called and how often
- **Success Rates**: Monitor tool call success vs. failure rates
- **Performance Metrics**: Average duration per tool
- **Arguments & Results**: Full JSON storage of tool calls (including arguments and results)

### Per-Operation Metrics

Track specific operations independently:

- **PRD Generation**: Token usage and costs for PRD creation
- **Chat Operations**: Interactive conversation metrics
- **Analysis Tasks**: PRD analysis and validation operations
- **Spec Generation**: Technical specification creation
- **Research**: Competitive analysis and research operations

## Usage Dashboard

Access comprehensive analytics in the **Usage** tab of the Orkee dashboard.

### Overview Tab

**Key Metrics Cards:**
- Total AI requests made
- Total tokens consumed (input + output)
- Total estimated costs
- Total tool calls executed

**Model Breakdown:**
- Token usage distribution across different AI models
- Visual breakdown showing which models consume the most tokens

**Provider Breakdown:**
- Cost distribution across providers (Anthropic, OpenAI, etc.)
- Identify your highest-cost provider

**Tool Analytics:**
- Most frequently used tools
- Success rates for each tool
- Average performance metrics

### Charts & Analytics Tab

**Time-Series Visualizations:**
- **Requests Over Time**: Track request volume trends
- **Token Usage Over Time**: Monitor token consumption patterns
- **Costs Over Time**: Identify spending trends

**Tool Analytics:**
- **Call Count Bar Chart**: Compare tool usage frequency
- **Success/Failure Rates**: Visual breakdown of tool reliability
- **Performance Chart**: Average duration per tool

**Distribution Charts:**
- **Model Distribution Pie Chart**: Token usage by model
- **Provider Distribution Pie Chart**: Costs by provider

## Data Storage

All usage data is stored locally in your SQLite database (`~/.orkee/orkee.db`):

### Database Table: `ai_usage_logs`

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT | Unique log entry ID |
| `project_id` | TEXT | Associated project (nullable) |
| `request_id` | TEXT | Request tracking ID |
| `operation` | TEXT | Operation name (e.g., "generate_prd") |
| `model` | TEXT | AI model used |
| `provider` | TEXT | AI provider (anthropic, openai, etc.) |
| `input_tokens` | INTEGER | Input token count |
| `output_tokens` | INTEGER | Output token count |
| `total_tokens` | INTEGER | Total tokens (input + output) |
| `estimated_cost` | REAL | Estimated cost in USD |
| `duration_ms` | INTEGER | Request duration in milliseconds |
| `tool_calls_count` | INTEGER | Number of tool calls |
| `tool_calls_json` | TEXT | JSON array of tool call details |
| `response_metadata` | TEXT | JSON response metadata |
| `error` | TEXT | Error message if failed (nullable) |
| `created_at` | TEXT | Timestamp |

### Tool Call JSON Structure

Tool calls are stored as JSON with full details:

```json
[
  {
    "name": "search",
    "arguments": {
      "query": "latest web frameworks 2024"
    },
    "result": {
      "results": ["Next.js 14", "Remix", "SvelteKit"]
    },
    "durationMs": 150
  },
  {
    "name": "calculate",
    "arguments": {
      "expression": "1024 * 1024"
    },
    "result": {
      "value": 1048576
    },
    "durationMs": 5,
    "error": null
  }
]
```

## API Endpoints

Access usage data programmatically:

### Get Aggregate Statistics

```http
GET /api/ai-usage/stats?projectId=<id>&startDate=<date>&endDate=<date>
```

Returns aggregate statistics including:
- Total requests
- Total tokens (input, output, total)
- Total estimated costs
- Requests by operation
- Tokens by model
- Costs by provider

### Get Tool Usage Statistics

```http
GET /api/ai-usage/tools?projectId=<id>&startDate=<date>&endDate=<date>
```

Returns tool-specific analytics:
- Call counts per tool
- Success/failure rates
- Average duration per tool

### Get Time-Series Data

```http
GET /api/ai-usage/time-series?projectId=<id>&startDate=<date>&endDate=<date>&granularity=day
```

Returns time-series data for charting:
- Requests over time
- Token usage over time
- Costs over time

Supported granularities: `hour`, `day`, `week`, `month`

## Privacy & Security

### Completely Local

All usage tracking is local:
- ✅ No data sent to external servers
- ✅ Stored in local SQLite database
- ✅ Full control over your data
- ✅ No third-party analytics

### Data Retention

You control data retention:

```bash
# Query usage data via SQLite
sqlite3 ~/.orkee/orkee.db "SELECT * FROM ai_usage_logs WHERE created_at > '2025-01-01'"

# Delete old data
sqlite3 ~/.orkee/orkee.db "DELETE FROM ai_usage_logs WHERE created_at < '2024-01-01'"

# Export data
sqlite3 ~/.orkee/orkee.db ".mode csv" ".output usage-export.csv" "SELECT * FROM ai_usage_logs"
```

## For Developers

### How It Works

Orkee uses telemetry wrappers that automatically capture AI SDK responses:

1. **Frontend Wrapper** (`packages/dashboard/src/lib/ai/telemetry.ts`):
   - Wraps all AI SDK calls
   - Captures tokens, duration, tool calls
   - Sends data to backend asynchronously

2. **Backend Endpoint** (`POST /api/ai-usage`):
   - Validates telemetry data
   - Stores in SQLite database
   - Non-blocking operation

3. **Dashboard Components**:
   - Query endpoints for stats
   - Visualize data with charts
   - Real-time updates

### Adding Telemetry to New AI Operations

All AI operations in `lib/ai/services.ts` are automatically tracked via the `sendAIResultTelemetry` helper. No manual tracking needed when using this pattern.

For custom AI operations, use the telemetry wrapper:

```typescript
import { trackAIOperation } from '@/lib/ai/telemetry';

const result = await trackAIOperation(
  'operation_name',        // e.g., 'generate_tasks'
  projectId,               // Project ID or null
  modelName,              // e.g., 'claude-sonnet-4-5'
  provider,               // e.g., 'anthropic'
  () => generateText({    // Your AI SDK call
    model: anthropic('claude-3-opus'),
    prompt: 'Your prompt here',
    tools: { search, calculate }
  })
);
```

The wrapper automatically:
- ✅ Measures duration with `performance.now()`
- ✅ Extracts token usage from response
- ✅ Identifies tool calls from response
- ✅ Calculates cost estimates
- ✅ Sends telemetry to backend
- ✅ Handles streaming responses
- ✅ Logs errors without breaking operation

## Troubleshooting

### No Data Showing in Dashboard

1. **Check database**: Verify `~/.orkee/orkee.db` exists and has `ai_usage_logs` table
2. **Check API endpoint**: Test `curl http://localhost:4001/api/ai-usage/stats`
3. **Check browser console**: Look for telemetry errors in dev tools
4. **Check server logs**: Look for validation errors from `/api/ai-usage` endpoint

### Inaccurate Cost Estimates

Cost estimates are based on published provider pricing. Actual costs may vary due to:
- Provider pricing changes
- Special discounts or credits
- Tool usage not properly accounted for
- Caching (some tokens may be cached by provider)

Update pricing constants in `packages/dashboard/src/lib/ai/utils.ts` if needed.

### Missing Tool Call Data

Ensure the AI SDK response includes tool call information:
- Check `experimental_toolCalls` field in response
- Verify tools are configured in AI SDK call
- Check browser console for extraction errors

## Related Documentation

- [AI Usage Implementation Plan](../../../ai-usage.md) - Technical implementation details
- [API Reference](../api-reference/) - Complete API endpoint documentation
- [Configuration](../configuration/) - Environment variable configuration
