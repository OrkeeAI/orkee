---
sidebar_position: 7
---

# AI-Powered Features

OpenSpec leverages AI to analyze requirements, generate tasks, suggest specifications, and validate implementations. This guide covers all AI-powered features, configuration, and best practices.

## AI Architecture

### Vercel AI SDK Integration

OpenSpec uses the Vercel AI SDK for production-ready AI features:

- **Provider Abstraction** - Works with OpenAI, Anthropic, and more
- **Structured Outputs** - Type-safe responses with Zod schemas
- **Streaming Support** - Real-time progress updates
- **Error Handling** - Graceful degradation and retries
- **Cost Tracking** - Built-in usage monitoring

### Supported Providers

| Provider | Models | Best For |
|----------|--------|----------|
| **Anthropic** (Recommended) | Claude 3.5 Sonnet, Claude 3 Opus | PRD analysis, requirement extraction |
| **OpenAI** | GPT-4 Turbo, GPT-4 | Task generation, validation |
| **Perplexity** | Online models | Research, context gathering |

## AI Configuration

### Environment Variables

```bash
# ~/.orkee/.env

# Primary provider (recommended)
ANTHROPIC_API_KEY=sk-ant-api03-...

# Optional providers
OPENAI_API_KEY=sk-proj-...
PERPLEXITY_API_KEY=pplx-...

# Provider selection
VITE_AI_PROVIDER=anthropic  # or openai

# Model selection
VITE_AI_MODEL=claude-3-5-sonnet-20241022

# Generation parameters
AI_TEMPERATURE=0.7      # 0.0-1.0, higher = more creative
AI_MAX_TOKENS=4096      # Maximum response length
AI_TOP_P=1.0            # Nucleus sampling parameter
```

### Model Selection Guide

**Claude 3.5 Sonnet** (Default)
- Best for: PRD analysis, requirement extraction
- Strengths: Long context, nuanced understanding
- Cost: Moderate ($3/1M input tokens)

**GPT-4 Turbo**
- Best for: Task generation, quick analysis
- Strengths: Fast, structured outputs
- Cost: Lower ($10/1M input tokens)

**Claude 3 Opus**
- Best for: Complex analysis, edge cases
- Strengths: Highest capability model
- Cost: Higher ($15/1M input tokens)

## AI Operations

### 1. PRD Analysis

Extract capabilities and requirements from PRD documents.

**How it works:**

1. User uploads PRD markdown
2. AI receives full PRD content
3. AI identifies functional areas (capabilities)
4. AI extracts requirements with WHEN/THEN scenarios
5. AI suggests implementation tasks

**Input:**
```markdown
# User Authentication System

## Requirements
Users can log in with email and password.
System must support password reset via email.
Account lockout after 3 failed attempts.
```

**AI Output:**
```json
{
  "summary": "Secure authentication system with login and password recovery",
  "capabilities": [
    {
      "id": "user-authentication",
      "name": "User Authentication",
      "purpose": "Secure user login and session management",
      "requirements": [
        {
          "name": "Email/Password Login",
          "content": "Users authenticate with email and password",
          "scenarios": [
            {
              "name": "Valid credentials",
              "when": "user submits valid email and password",
              "then": "user is authenticated and session created",
              "and": ["login event logged", "user redirected to dashboard"]
            },
            {
              "name": "Invalid credentials",
              "when": "user submits incorrect password",
              "then": "error message displayed",
              "and": ["failed attempt logged", "account lockout check triggered"]
            }
          ]
        },
        {
          "name": "Password Reset",
          "content": "Users can reset forgotten password via email",
          "scenarios": [...]
        },
        {
          "name": "Account Lockout",
          "content": "Account locked after failed attempts",
          "scenarios": [...]
        }
      ]
    }
  ],
  "suggestedTasks": [
    {
      "title": "Implement login API endpoint",
      "description": "POST /api/auth/login with email/password validation",
      "capabilityId": "user-authentication",
      "requirementName": "Email/Password Login",
      "complexity": 5,
      "estimatedHours": 6
    }
  ],
  "dependencies": ["Email service", "Session storage"]
}
```

**API Endpoint:**
```bash
POST /api/projects/:project_id/prds/:prd_id/analyze

Response: PRDAnalysisSchema (Zod validated)
```

**Cost**: ~$0.10-0.30 per PRD (depending on length)

### 2. Spec Generation

Generate complete spec from high-level requirements.

**How it works:**

1. User provides capability name and purpose
2. AI generates detailed requirements
3. AI creates WHEN/THEN scenarios
4. AI validates completeness

**Input:**
```json
{
  "capabilityName": "Product Search",
  "purpose": "Full-text search across product catalog",
  "context": "E-commerce site with 10k products"
}
```

**AI Output:**
```json
{
  "capability": {
    "id": "product-search",
    "name": "Product Search",
    "purpose": "Full-text search across product catalog with filtering",
    "requirements": [
      {
        "name": "Search Query Processing",
        "content": "Process user search queries with fuzzy matching",
        "scenarios": [
          {
            "name": "Exact match",
            "when": "user searches for exact product name",
            "then": "product appears first in results",
            "and": ["search term is highlighted", "related products shown"]
          },
          {
            "name": "Fuzzy match",
            "when": "user misspells product name",
            "then": "closest matches are returned",
            "and": ["'Did you mean' suggestion shown"]
          },
          {
            "name": "No results",
            "when": "search query matches nothing",
            "then": "helpful message displayed",
            "and": ["popular products suggested"]
          }
        ]
      }
    ]
  }
}
```

**API Endpoint:**
```bash
POST /api/ai/generate-spec

Response: SpecCapabilitySchema (Zod validated)
```

**Cost**: ~$0.05-0.15 per spec

### 3. Task Generation

Generate implementation tasks from spec requirements.

**How it works:**

1. User selects requirements
2. AI analyzes scenarios
3. AI suggests implementation tasks
4. AI estimates complexity

**Input:**
```json
{
  "requirement_id": "req_abc",
  "requirement": {
    "name": "Email/Password Login",
    "scenarios": [...]
  },
  "options": {
    "includeTests": true,
    "includeDocumentation": true
  }
}
```

**AI Output:**
```json
{
  "tasks": [
    {
      "title": "Implement login API endpoint",
      "description": "Create POST /api/auth/login endpoint with:\n- Email/password validation\n- bcrypt password verification\n- JWT token generation\n- Session creation",
      "complexity": 5,
      "estimatedHours": 6,
      "type": "implementation"
    },
    {
      "title": "Add login integration tests",
      "description": "Test scenarios:\n- Valid credentials\n- Invalid password\n- Non-existent email\n- Account lockout",
      "complexity": 4,
      "estimatedHours": 4,
      "type": "testing"
    },
    {
      "title": "Document login API",
      "description": "Add API documentation:\n- Endpoint description\n- Request/response examples\n- Error codes\n- Security notes",
      "complexity": 2,
      "estimatedHours": 2,
      "type": "documentation"
    }
  ]
}
```

**API Endpoint:**
```bash
POST /api/ai/suggest-tasks

Response: Array<TaskSuggestionSchema>
```

**Cost**: ~$0.02-0.05 per requirement

### 4. Orphan Task Analysis

Suggest spec requirements for orphan tasks.

**How it works:**

1. User clicks "Suggest Spec" on orphan task
2. AI analyzes task details
3. AI examines existing capabilities
4. AI suggests best fit or new capability

**Input:**
```json
{
  "task": {
    "title": "Add OAuth2 support for Google",
    "description": "Allow users to login with Google account"
  },
  "existingCapabilities": [
    {
      "id": "user-authentication",
      "name": "User Authentication",
      "requirements": ["Email/Password Login", "Password Reset"]
    }
  ]
}
```

**AI Output:**
```json
{
  "suggestions": [
    {
      "type": "add_to_existing",
      "confidence": 0.85,
      "capability_id": "user-authentication",
      "newRequirement": {
        "name": "OAuth Provider Integration",
        "content": "Users can authenticate via OAuth providers",
        "scenarios": [
          {
            "name": "Google OAuth",
            "when": "user clicks 'Login with Google'",
            "then": "OAuth flow initiates",
            "and": ["user authenticated", "account linked or created"]
          }
        ]
      },
      "rationale": "OAuth is authentication method, fits existing capability"
    },
    {
      "type": "create_new_capability",
      "confidence": 0.60,
      "newCapability": {
        "name": "Social Authentication",
        "purpose": "Third-party authentication providers",
        "requirements": [...]
      },
      "rationale": "Could expand to Facebook, GitHub, etc."
    }
  ]
}
```

**API Endpoint:**
```bash
POST /api/tasks/:task_id/suggest-spec

Response: SpecSuggestionSchema
```

**Cost**: ~$0.03-0.08 per task

### 5. Task Validation

Validate task completion against WHEN/THEN scenarios.

**How it works:**

1. User marks task complete
2. AI receives requirement scenarios
3. AI analyzes task implementation
4. AI validates each scenario

**Input:**
```json
{
  "requirement": {
    "name": "Email/Password Login",
    "scenarios": [
      {
        "when": "user submits valid credentials",
        "then": "user is authenticated",
        "and": ["session created", "redirected to dashboard"]
      }
    ]
  },
  "task": {
    "title": "Implement login endpoint",
    "completionNotes": "Created POST /api/auth/login with JWT",
    "implementationDetails": "Added bcrypt verification, session management"
  }
}
```

**AI Output:**
```json
{
  "overallStatus": "passed",
  "scenarioResults": [
    {
      "scenarioName": "Valid credentials",
      "status": "passed",
      "details": "Implementation correctly validates credentials and creates session",
      "suggestions": []
    },
    {
      "scenarioName": "Invalid password",
      "status": "failed",
      "details": "Missing rate limiting for failed attempts",
      "suggestions": [
        "Add rate limiting middleware",
        "Implement exponential backoff"
      ]
    }
  ],
  "coverage": 0.85,
  "recommendations": [
    "Add tests for concurrent login attempts",
    "Document session expiry behavior"
  ]
}
```

**API Endpoint:**
```bash
POST /api/tasks/:task_id/validate-spec

Response: ValidationResultSchema
```

**Cost**: ~$0.05-0.10 per validation

### 6. Spec Refinement

Improve specs with AI suggestions.

**How it works:**

1. User selects spec to refine
2. AI analyzes existing requirements
3. AI suggests improvements
4. User reviews and applies

**Input:**
```json
{
  "capability": {
    "name": "User Authentication",
    "requirements": [...]
  },
  "feedback": "Add security considerations"
}
```

**AI Output:**
```json
{
  "refinements": [
    {
      "type": "add_scenarios",
      "requirement_id": "req_login",
      "scenarios": [
        {
          "name": "Brute force protection",
          "when": "5 failed attempts in 1 minute",
          "then": "account temporarily locked",
          "and": ["CAPTCHA required for unlock"]
        }
      ]
    },
    {
      "type": "add_requirement",
      "requirement": {
        "name": "Security Logging",
        "content": "All authentication events are logged",
        "scenarios": [...]
      }
    },
    {
      "type": "modify_scenario",
      "requirement_id": "req_login",
      "scenario_id": "scen_valid",
      "changes": {
        "and_clauses": [
          "session created",
          "redirected to dashboard",
          "login event logged with IP and timestamp"
        ]
      }
    }
  ]
}
```

**API Endpoint:**
```bash
POST /api/ai/refine-spec

Response: SpecRefinementSchema
```

**Cost**: ~$0.08-0.20 per refinement

### 7. PRD Regeneration

Regenerate PRD from current spec state.

**How it works:**

1. User triggers PRD sync
2. AI collects all capabilities and requirements
3. AI generates cohesive PRD document
4. AI maintains original structure and tone

**Input:**
```json
{
  "originalPRD": "# Original PRD content...",
  "capabilities": [...],
  "syncMode": "merge"  // or "replace"
}
```

**AI Output:**
```markdown
# User Authentication System

*Auto-generated from specifications - Last updated: 2025-01-20*

## Overview
Comprehensive authentication system with email/password login,
OAuth integration, and security features.

## Capabilities

### User Authentication
Secure user login and session management.

#### Email/Password Login
Users authenticate using email and password credentials.

**Scenarios:**
- WHEN user submits valid credentials THEN authenticated and redirected
- WHEN user submits invalid password THEN error shown and attempt logged

**Implementation:**
- [x] Implement login API endpoint (completed)
- [x] Add login integration tests (completed)
- [ ] Document login API (in progress)

[... continues for all capabilities ...]

## Statistics
- Capabilities: 3
- Requirements: 12
- Scenarios: 36
- Tasks: 25 (20 completed, 5 in progress)
- Coverage: 83%
```

**API Endpoint:**
```bash
POST /api/projects/:project_id/prds/:prd_id/sync

Response: Regenerated PRD markdown
```

**Cost**: ~$0.15-0.40 per PRD

## Streaming & Real-Time Updates

### Streaming API

For long-running operations, use streaming:

```typescript
import { streamObject } from 'ai';

const { partialObjectStream } = await streamObject({
  model: anthropic('claude-3-5-sonnet-20241022'),
  schema: PRDAnalysisSchema,
  prompt: 'Analyze this PRD...',
});

for await (const partialObject of partialObjectStream) {
  // Update UI with partial results
  console.log(partialObject.capabilities?.length ?? 0);
}
```

**Benefits:**
- Real-time progress updates
- Better user experience
- Early cancellation support
- Reduced perceived latency

### Progress Indicators

Show progress during AI operations:

```typescript
const [progress, setProgress] = useState({
  stage: 'analyzing',  // analyzing, extracting, generating
  percent: 0,
  message: 'Analyzing PRD...'
});

// Update based on stream events
```

## Error Handling

### Common Errors

**API Key Invalid**
```json
{
  "error": "Invalid API key",
  "code": "AUTH_ERROR",
  "suggestion": "Check ANTHROPIC_API_KEY in .env"
}
```

**Rate Limit Exceeded**
```json
{
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT",
  "retry_after": 60,
  "suggestion": "Wait 60 seconds before retrying"
}
```

**Token Limit Exceeded**
```json
{
  "error": "Response too long",
  "code": "TOKEN_LIMIT",
  "suggestion": "Try analyzing smaller sections"
}
```

### Retry Strategy

Automatic retries with exponential backoff:

```typescript
const retryConfig = {
  maxRetries: 3,
  initialDelay: 1000,  // 1 second
  maxDelay: 10000,     // 10 seconds
  backoffFactor: 2
};
```

## Best Practices

### Prompt Engineering

✅ **Effective Prompts:**
```
Analyze this PRD and extract capabilities. For each capability:
1. Identify distinct functional area
2. List specific requirements
3. Generate WHEN/THEN/AND scenarios
4. Suggest implementation tasks with complexity 1-10

PRD Content:
[content here]
```

❌ **Poor Prompts:**
```
Read this and tell me what to build
```

### Context Management

**Provide Rich Context:**
- Include existing capabilities when analyzing tasks
- Reference project description
- Note technical constraints
- Mention team preferences

**Example:**
```json
{
  "context": {
    "projectType": "React SPA with Node.js backend",
    "database": "PostgreSQL",
    "existingCapabilities": [...],
    "team": {
      "size": 5,
      "techStack": ["TypeScript", "React", "Node.js"]
    }
  }
}
```

### Cost Optimization

**Reduce Costs:**

1. **Cache Results**
   ```typescript
   const cacheKey = `prd-analysis-${prdId}`;
   const cached = await cache.get(cacheKey);
   if (cached) return cached;
   ```

2. **Batch Operations**
   - Analyze multiple tasks together
   - Generate tasks for multiple requirements

3. **Use Cheaper Models for Simple Tasks**
   - GPT-3.5 for task suggestions
   - Claude Haiku for validation

4. **Limit Response Length**
   ```typescript
   maxTokens: 2000  // Instead of 4096
   ```

## Monitoring & Analytics

### AI Usage Dashboard

Track AI usage in real-time:

- Total cost: $12.45
- Requests today: 45
- Average cost per request: $0.28
- Most expensive operation: PRD Analysis ($0.35)

### Cost Alerts

Set up alerts:

```json
{
  "alerts": [
    {
      "type": "daily_budget",
      "threshold": 10.00,
      "action": "email"
    },
    {
      "type": "per_request",
      "threshold": 1.00,
      "action": "log"
    }
  ]
}
```

## Troubleshooting

### AI Not Responding

**Problem**: AI requests timeout

**Solutions:**
1. Check API key is valid
2. Verify network connectivity
3. Try smaller input
4. Check provider status page

### Poor Quality Results

**Problem**: AI generates irrelevant content

**Solutions:**
1. Improve prompt with more context
2. Use higher capability model
3. Provide examples of desired output
4. Adjust temperature parameter

### High Costs

**Problem**: AI usage costs too high

**Solutions:**
1. Review cost dashboard for expensive operations
2. Implement caching
3. Use cheaper models where appropriate
4. Set budget limits

## Next Steps

- [Monitor costs in detail](./cost-tracking.md)
- [Understand full workflows](./workflows.md)
- [Review PRD analysis](./prds.md)
- [Explore task validation](./tasks.md)

---

**Related**: [Cost Tracking](./cost-tracking.md) | [Workflows](./workflows.md) | [Configuration](/docs/configuration/environment-variables)
