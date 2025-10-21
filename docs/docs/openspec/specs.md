---
sidebar_position: 4
---

# Creating and Managing Specs

Specifications (specs) are the heart of OpenSpec. This guide covers creating capabilities, defining requirements, writing scenarios, and managing your spec library.

## Spec Hierarchy

OpenSpec uses a three-level hierarchy:

```
Capability (e.g., "User Authentication")
  └── Requirement (e.g., "Password Reset")
       └── Scenario (e.g., "WHEN valid email THEN reset link sent")
```

### Capabilities

**Capabilities** represent major functional areas or features:

- Authentication
- User Profile Management
- Payment Processing
- Notifications
- Search & Discovery

### Requirements

**Requirements** are specific functional needs within a capability:

- User can reset forgotten password
- Profile photos are validated for size and format
- Credit card payments are processed securely
- Email notifications are sent asynchronously
- Search results are ranked by relevance

### Scenarios

**Scenarios** are testable conditions in WHEN/THEN/AND format:

```markdown
WHEN user clicks "Forgot Password" and enters valid email
THEN password reset link is sent to email
AND link expires in 1 hour
AND old reset links are invalidated
```

## Creating Specs

### Method 1: From PRD Analysis

The recommended approach (covered in [PRD documentation](./prds.md)):

1. Upload PRD
2. Analyze with AI
3. Review extracted capabilities
4. Click "Create Specs"

### Method 2: SpecBuilderWizard

Create specs manually using the 4-step wizard:

**Step 1: Choose Mode**

Three modes available:
- **PRD-Driven**: Select existing PRD to base specs on
- **Manual**: Create from scratch
- **Task-Driven**: Generate specs from existing tasks

**Step 2: Define Capability**

```markdown
Capability Name: User Authentication
Purpose: Provide secure user authentication and session management
```

**Step 3: Add Requirements**

For each requirement:

```markdown
Requirement Name: Password Reset

Description:
Users who forget their password can request a reset link via email.
The link is time-limited and single-use for security.

Scenarios:
1. Valid Email
   WHEN: user enters registered email address
   THEN: reset link is sent to email
   AND: link expires in 1 hour
   AND: previous reset links are invalidated

2. Invalid Email
   WHEN: user enters unregistered email
   THEN: generic success message is shown (security)
   AND: no email is sent

3. Expired Link
   WHEN: user clicks reset link after 1 hour
   THEN: error message is displayed
   AND: user can request new link
```

**Step 4: Validation**

Review spec summary:
- Capability name and purpose
- Requirement count
- Scenario count per requirement
- Validation warnings (if any)

Click **"Create Spec"** to save.

### Method 3: API

Create specs programmatically:

```bash
curl -X POST http://localhost:4001/api/projects/PROJECT_ID/specs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "user-authentication",
    "purpose": "Secure user authentication and session management",
    "spec_markdown": "# User Authentication\n...",
    "requirements": [
      {
        "name": "Password Reset",
        "content": "Users can reset forgotten passwords",
        "scenarios": [
          {
            "name": "Valid email",
            "when_clause": "user enters registered email",
            "then_clause": "reset link is sent",
            "and_clauses": ["link expires in 1 hour"]
          }
        ]
      }
    ]
  }'
```

## Writing Effective Scenarios

### WHEN/THEN/AND Format

**WHEN** - Describes the trigger/condition
**THEN** - Describes the expected outcome
**AND** - Describes additional outcomes (optional, can have multiple)

### Good vs Bad Scenarios

✅ **Good - Specific and Testable:**
```markdown
WHEN user submits registration form with valid email and password
THEN account is created in database
AND confirmation email is sent
AND user is redirected to dashboard
AND welcome event is logged
```

❌ **Bad - Vague and Untestable:**
```markdown
WHEN user signs up
THEN it works
```

✅ **Good - Covers Edge Cases:**
```markdown
WHEN user submits form with email that already exists
THEN error message "Email already registered" is shown
AND form remains populated with entered data
AND no database changes occur
```

❌ **Bad - Ignores Edge Cases:**
```markdown
WHEN user signs up
THEN account is created
```

### Scenario Templates

**Happy Path:**
```markdown
WHEN [valid input/normal flow]
THEN [expected success outcome]
AND [side effects]
```

**Error Handling:**
```markdown
WHEN [invalid input]
THEN [error message shown]
AND [system state unchanged]
```

**Edge Cases:**
```markdown
WHEN [boundary condition]
THEN [appropriate behavior]
AND [data integrity maintained]
```

**Performance:**
```markdown
WHEN [load condition]
THEN [response within SLA]
AND [resources properly released]
```

## Managing Capabilities

### Viewing Capabilities

**Via Dashboard:**
1. Navigate to **Specs** tab
2. Click **"Specifications"** section
3. Browse capability cards

Each card shows:
- Capability name
- Purpose summary
- Requirement count
- Status (active, deprecated, archived)
- Last updated

**Via API:**
```bash
GET /api/projects/PROJECT_ID/specs
```

### Updating Capabilities

**Edit via SpecDetailsView:**
1. Click capability card
2. Click **"Edit"**
3. Modify purpose or requirements
4. Click **"Save Changes"**

**Update via API:**
```bash
PUT /api/projects/PROJECT_ID/specs/SPEC_ID
Content-Type: application/json

{
  "purpose": "Updated purpose statement",
  "spec_markdown": "# Updated content\n..."
}
```

### Capability Status

Capabilities have three statuses:

**Active** (default)
- Currently in use
- Tasks can link to requirements
- Appears in all views

**Deprecated**
- Functionality being phased out
- Existing links remain
- Highlighted in yellow

**Archived**
- No longer in use
- Hidden from main views
- Can be restored if needed

Change status:
```bash
PUT /api/projects/PROJECT_ID/specs/SPEC_ID
Content-Type: application/json

{
  "status": "deprecated"
}
```

## Managing Requirements

### Adding Requirements

**Via SpecBuilderWizard:**
- Edit existing capability
- Click **"Add Requirement"**
- Fill in name, content, scenarios
- Save changes

**Via API:**
```bash
POST /api/projects/PROJECT_ID/specs/SPEC_ID/requirements
Content-Type: application/json

{
  "name": "Email Verification",
  "content": "New accounts must verify email address",
  "position": 3,
  "scenarios": [...]
}
```

### Updating Requirements

**Via SpecDetailsView:**
1. Open capability
2. Go to **Requirements** tab
3. Click requirement to edit
4. Modify content or scenarios
5. Save

**Via API:**
```bash
PUT /api/projects/PROJECT_ID/specs/SPEC_ID/requirements/REQ_ID
Content-Type: application/json

{
  "content": "Updated requirement description",
  "scenarios": [...]
}
```

### Deleting Requirements

:::warning
Deleting a requirement also deletes:
- All scenarios for that requirement
- All task links to that requirement
- All validation results

This cannot be undone. Consider archiving the parent capability instead.
:::

Delete via API:
```bash
DELETE /api/projects/PROJECT_ID/specs/SPEC_ID/requirements/REQ_ID
```

## Spec Validation

### Validation Rules

OpenSpec validates specs against these rules:

**Capability Level:**
- ✅ Name is kebab-case (e.g., "user-auth")
- ✅ Name is 3-50 characters
- ✅ Purpose is provided
- ✅ Has at least one requirement

**Requirement Level:**
- ✅ Name is 3-100 characters
- ✅ Content is provided
- ✅ Has at least one scenario

**Scenario Level:**
- ✅ Name is provided
- ✅ WHEN clause is provided
- ✅ THEN clause is provided
- ✅ AND clauses (if present) are not empty

### Running Validation

**Via API:**
```bash
POST /api/projects/PROJECT_ID/specs/validate
Content-Type: application/json

{
  "name": "new-capability",
  "purpose": "...",
  "requirements": [...]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "valid": true,
    "errors": [],
    "warnings": [
      "Requirement 'User Login' has only 1 scenario (recommend 3+)"
    ],
    "stats": {
      "requirements": 5,
      "scenarios": 12,
      "averageScenariosPerRequirement": 2.4
    }
  }
}
```

### Validation Warnings

Warnings don't prevent creation but suggest improvements:

- "Purpose section is short (recommend 2+ sentences)"
- "Requirement has only 1 scenario (recommend 3+)"
- "No error handling scenarios found"
- "No edge case scenarios found"

## Spec Markdown Format

Specs are stored as markdown in `spec_markdown` field:

```markdown
# User Authentication

## Purpose
Provide secure user authentication and session management for the application.

## Requirements

### Password Reset
Users who forget their password can request a reset link via email.

**Scenarios:**

#### Valid Email
- **WHEN**: user enters registered email address
- **THEN**: reset link is sent to email
- **AND**:
  - link expires in 1 hour
  - previous reset links are invalidated

#### Invalid Email
- **WHEN**: user enters unregistered email
- **THEN**: generic success message is shown
- **AND**:
  - no email is sent (security measure)

### Email Verification
New accounts must verify their email address before full access.

**Scenarios:**
...
```

## Best Practices

### Organizing Capabilities

✅ **Do:**
- Group related functionality
- Use clear, descriptive names
- Keep capabilities focused (single responsibility)
- Separate concerns (auth vs profile vs payments)

❌ **Don't:**
- Create kitchen-sink capabilities
- Mix unrelated features
- Use technical jargon in names
- Nest capabilities (keep hierarchy flat)

### Writing Requirements

✅ **Do:**
- Start with user perspective ("User can...")
- Include acceptance criteria
- Cover happy path and errors
- Consider performance and security

❌ **Don't:**
- Describe implementation details
- Mix multiple concerns in one requirement
- Skip error scenarios
- Use ambiguous language

### Scenario Coverage

Aim for comprehensive coverage:

**Minimum per Requirement:**
- 1 happy path scenario
- 1 error scenario
- 1 edge case scenario

**Recommended per Requirement:**
- 2-3 happy path variations
- 2-3 error scenarios
- 1-2 edge cases
- 1 performance/security scenario (if applicable)

### Versioning Strategy

**Capability Versions:**
- Increment `version` on major changes
- Track version in `spec_capabilities.version`
- Keep old versions for audit trail

**Change Management:**
- Use change proposals for modifications
- Require approval for breaking changes
- Document rationale in change records

## Database Schema Reference

### spec_capabilities

```sql
CREATE TABLE spec_capabilities (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    prd_id TEXT,              -- Source PRD (optional)
    name TEXT NOT NULL,        -- kebab-case
    purpose_markdown TEXT,
    spec_markdown TEXT NOT NULL,
    design_markdown TEXT,      -- Optional design docs
    requirement_count INTEGER DEFAULT 0,
    version INTEGER DEFAULT 1,
    status TEXT DEFAULT 'active',
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (prd_id) REFERENCES prds(id)
);
```

### spec_requirements

```sql
CREATE TABLE spec_requirements (
    id TEXT PRIMARY KEY,
    capability_id TEXT NOT NULL,
    name TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    position INTEGER DEFAULT 0,  -- Ordering
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    FOREIGN KEY (capability_id) REFERENCES spec_capabilities(id) ON DELETE CASCADE
);
```

### spec_scenarios

```sql
CREATE TABLE spec_scenarios (
    id TEXT PRIMARY KEY,
    requirement_id TEXT NOT NULL,
    name TEXT NOT NULL,
    when_clause TEXT NOT NULL,
    then_clause TEXT NOT NULL,
    and_clauses TEXT,  -- JSON array
    position INTEGER DEFAULT 0,
    created_at TIMESTAMP,
    FOREIGN KEY (requirement_id) REFERENCES spec_requirements(id) ON DELETE CASCADE
);
```

## API Reference

### List Capabilities

```bash
GET /api/projects/:project_id/specs
```

### Get Capability

```bash
GET /api/projects/:project_id/specs/:spec_id
```

### Create Capability

```bash
POST /api/projects/:project_id/specs
```

### Update Capability

```bash
PUT /api/projects/:project_id/specs/:spec_id
```

### Delete Capability

```bash
DELETE /api/projects/:project_id/specs/:spec_id
```

### Validate Spec

```bash
POST /api/projects/:project_id/specs/validate
```

### Get Requirements

```bash
GET /api/projects/:project_id/specs/:spec_id/requirements
```

## Next Steps

- [Link tasks to requirements](./tasks.md)
- [Understand full workflows](./workflows.md)
- [Manage changes with proposals](./workflows.md#change-proposals)
- [Validate tasks against scenarios](./tasks.md#validation)

---

**Related**: [PRD Management](./prds.md) | [Task Integration](./tasks.md) | [Workflows](./workflows.md)
