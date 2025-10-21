---
sidebar_position: 2
---

# Getting Started with OpenSpec

This guide walks you through your first OpenSpec workflow, from uploading a PRD to validating completed tasks.

## Prerequisites

Before you begin, ensure you have:

- ✅ Orkee installed and running
- ✅ At least one project created
- ✅ Anthropic API key configured (required for AI features)

### Configure AI Provider

OpenSpec uses AI for PRD analysis and task generation. Set up your API key:

```bash
# Add to ~/.orkee/.env or your environment
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# Restart Orkee to apply changes
orkee dashboard --restart
```

:::tip Optional Providers
While Anthropic (Claude) is recommended, you can also use OpenAI:
```bash
export OPENAI_API_KEY="sk-proj-..."
```
:::

## Workflow 1: PRD → Spec → Task

This is the traditional top-down approach, starting with requirements.

### Step 1: Create a PRD

Open your project in the Orkee dashboard and navigate to the **Specs** tab.

Create a simple PRD in markdown format. For example, a user authentication system:

```markdown
# User Authentication System

## Overview
Build a secure user authentication system with email/password login.

## Requirements

### Core Authentication
- Users can register with email and password
- Users can log in with valid credentials
- Passwords are hashed with bcrypt
- Sessions persist across browser restarts

### Security Features
- Rate limiting on login attempts (5 per minute)
- Account lockout after 3 failed attempts
- Password reset via email
- Email verification for new accounts

### User Experience
- Clear error messages for invalid credentials
- "Remember me" option for extended sessions
- Logout from all devices option
```

### Step 2: Upload and Analyze PRD

1. Click **"Upload PRD"** in the Specs tab
2. Paste your markdown or upload a `.md` file
3. Click **"Analyze with AI"**
4. Wait for AI analysis to complete

The AI will extract:
- **Capabilities** - High-level functional areas (e.g., "Core Authentication", "Security Features")
- **Requirements** - Specific needs with WHEN/THEN scenarios
- **Suggested Tasks** - Implementation items with complexity scores

### Step 3: Review Extracted Capabilities

The analysis results show:

**Capability: Core Authentication**
- Requirement: User Registration
  - Scenario: WHEN user provides valid email and password THEN account is created
  - Scenario: WHEN email already exists THEN error message is shown
- Requirement: User Login
  - Scenario: WHEN credentials are valid THEN user is authenticated
  - Scenario: WHEN credentials are invalid THEN error message is shown

**Capability: Security Features**
- Requirement: Rate Limiting
  - Scenario: WHEN 5 login attempts in 1 minute THEN further attempts are blocked
- Requirement: Account Lockout
  - Scenario: WHEN 3 failed login attempts THEN account is locked for 30 minutes

### Step 4: Create Specs

Click **"Create Specs"** to save the extracted capabilities to your project. This creates:

- Database records for each capability
- Requirements with WHEN/THEN scenarios
- Full audit trail in `prd_spec_sync_history`

### Step 5: Generate Tasks

The AI suggests tasks based on requirements:

- "Implement user registration API endpoint" (Complexity: 5/10)
- "Create bcrypt password hashing utility" (Complexity: 3/10)
- "Build login form with validation" (Complexity: 4/10)
- "Add rate limiting middleware" (Complexity: 6/10)
- "Implement account lockout logic" (Complexity: 7/10)

Click **"Generate Tasks"** to create these in your project. Each task is automatically linked to its spec requirement.

### Step 6: Implement and Validate

As you complete tasks:

1. **Validate** against scenarios using the TaskSpecLinker
2. **Update** task status when validation passes
3. **Track** requirement completion automatically

The SyncDashboard shows:
- Requirements coverage (% of requirements with completed tasks)
- Orphan task count
- Spec overview with status

## Workflow 2: Task → Spec → PRD

This is the bottom-up approach, starting with implementation.

### Step 1: Create a Manual Task

Sometimes developers create tasks before specs exist:

1. Go to your project's Tasks view
2. Click **"New Task"**
3. Create: "Add OAuth 2.0 support for Google login"

This task is now an **orphan** (no spec link).

### Step 2: Detect Orphan

The SyncDashboard automatically detects orphan tasks:

- Navigate to **Specs > Coverage** tab
- View **"Orphan Tasks"** section
- See your OAuth task listed

### Step 3: AI Suggests Spec

Click **"Suggest Spec"** next to the orphan task. The AI analyzes the task and suggests:

**Option 1: Add to Existing Capability**
- Capability: "Core Authentication"
- New Requirement: "OAuth Provider Integration"
- Scenarios:
  - WHEN user clicks "Login with Google" THEN OAuth flow initiates
  - WHEN OAuth succeeds THEN user account is created or linked
  - WHEN OAuth fails THEN error message is shown

**Option 2: Create New Capability**
- Capability: "Social Authentication"
- Requirement: "Google OAuth Integration"
- (similar scenarios)

### Step 4: Create Change Proposal

Select your preferred option and click **"Create Change Proposal"**. This creates:

- A change record in `spec_changes` table
- Delta record describing the addition
- Tasks list referencing your orphan task
- Status: "draft"

### Step 5: Review and Approve

Review the proposal:

1. Navigate to **Specs > Changes** (planned feature)
2. View proposal details
3. Edit if needed
4. Click **"Approve"**

Approval triggers:
- Spec capability/requirement creation
- Task link to new requirement
- PRD regeneration with new section
- Sync history entry

### Step 6: Verify PRD Update

The PRD now includes a new section:

```markdown
### OAuth Provider Integration
Users can authenticate using external OAuth providers for streamlined signup.

**Scenarios:**
- When user clicks "Login with Google", OAuth flow initiates
- When OAuth succeeds, user account is created or linked
- When OAuth fails, error message is shown

**Implementation:**
- Task: Add OAuth 2.0 support for Google login
```

## Understanding the UI

### Specs Tab Structure

The Specs tab has three sections:

1. **PRD View** - Upload, view, and analyze PRDs
2. **Specifications** - Browse capabilities and requirements
3. **Coverage** - Monitor orphan tasks and sync status

### Key Components

**PRDUploadDialog** (3 tabs):
- Upload: Paste markdown or upload file
- Preview: Rendered markdown view
- Analysis: AI-extracted capabilities and tasks

**SpecBuilderWizard** (4 steps):
- Mode: Choose PRD-driven, Manual, or Task-driven
- Capability: Define capability name and purpose
- Requirements: Add requirements with scenarios
- Validation: Review and create spec

**TaskSpecLinker**:
- Search requirements across all capabilities
- Link tasks to requirements
- Validate task completion
- View scenario details

**SyncDashboard**:
- Orphan Tasks: Find unlinked tasks
- PRD Sync: Manual sync triggers
- Spec Overview: Capability status at a glance

## Best Practices

### Writing Good PRDs

✅ **Do:**
- Use clear, concise language
- Include specific requirements
- Describe user journeys
- List acceptance criteria

❌ **Don't:**
- Mix implementation details with requirements
- Use ambiguous language
- Skip security considerations
- Forget edge cases

### Creating Effective Scenarios

✅ **Do:**
```markdown
WHEN user submits form with valid data
THEN account is created
AND confirmation email is sent
AND user is redirected to dashboard
```

❌ **Don't:**
```markdown
WHEN stuff happens
THEN it works
```

### Managing Orphan Tasks

**Regular Review**: Check the SyncDashboard weekly for orphan tasks

**Batch Processing**: Group similar orphans into single capabilities

**Team Coordination**: Discuss orphans in team meetings to decide on spec structure

## Common Issues

### AI Analysis Fails

**Problem**: "AI analysis failed" error

**Solution**:
1. Check `ANTHROPIC_API_KEY` is set
2. Verify API key is valid
3. Check AI usage dashboard for errors
4. Try again with smaller PRD

### Tasks Not Linking

**Problem**: Generated tasks don't link to requirements

**Solution**:
1. Verify task generation completed successfully
2. Check `task_spec_links` table in database
3. Use TaskSpecLinker to manually link
4. Report bug if issue persists

### PRD Not Regenerating

**Problem**: PRD doesn't update after spec changes

**Solution**:
1. Click manual sync in SyncDashboard
2. Check `prd_spec_sync_history` for errors
3. Verify change was approved
4. Regenerate PRD manually

## Next Steps

Now that you understand the basics:

- [Learn about PRD management](./prds.md) - Upload, version, and analyze PRDs
- [Explore spec creation](./specs.md) - Create capabilities and requirements manually
- [Master task integration](./tasks.md) - Link tasks and validate completion
- [Understand workflows](./workflows.md) - Advanced sync and change management
- [Monitor AI costs](./cost-tracking.md) - Track and optimize AI usage

---

**Need Help?** Check the [troubleshooting section](#common-issues) or [open an issue](https://github.com/OrkeeAI/orkee/issues).
