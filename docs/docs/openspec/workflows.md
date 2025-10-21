---
sidebar_position: 6
---

# OpenSpec Workflows

This guide covers end-to-end workflows for spec-driven development, including PRD→Spec→Task flow, Task→Spec→PRD sync, and change management processes.

## Overview

OpenSpec supports bidirectional workflows that keep PRDs, specs, and tasks synchronized:

```
┌─────────────────────────────────────────┐
│         PRD → Spec → Task               │
│    (Top-Down Requirements Flow)         │
└─────────────────────────────────────────┘
                  ↓
        ┌─────────────────┐
        │   Sync Engine   │
        └─────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│         Task → Spec → PRD               │
│   (Bottom-Up Implementation Flow)       │
└─────────────────────────────────────────┘
```

## Workflow 1: PRD → Spec → Task

The traditional top-down approach starts with requirements and flows to implementation.

### Phase 1: PRD Upload & Analysis

**Step 1: Create PRD**

Write your Product Requirements Document in markdown:

```markdown
# E-Commerce Product Catalog

## Overview
Build a product catalog system with search, filtering, and detail views.

## Requirements

### Product Listing
- Display products in grid layout
- Support pagination (20 items per page)
- Show product image, name, price, rating

### Product Search
- Full-text search across name and description
- Real-time search results
- Search suggestions as user types

### Product Filtering
- Filter by category, price range, rating
- Multiple filters can be applied
- Filter counts show available options

### Product Details
- Show full product information
- Display product images gallery
- Show related products
- Add to cart functionality
```

**Step 2: Upload to Orkee**

1. Navigate to project **Specs** tab
2. Click **"Upload PRD"**
3. Paste markdown or upload file
4. Preview rendered content
5. Click **"Save PRD"**

**Step 3: AI Analysis**

1. Click **"Analyze with AI"**
2. Wait for analysis (10-30 seconds)
3. Review extracted data:

**Extracted Capabilities:**
- Product Catalog Display (4 requirements)
- Search & Discovery (3 requirements)
- Filtering System (2 requirements)

**Extracted Requirements (example):**
- Requirement: Product Grid Layout
  - Scenario: WHEN page loads THEN products display in grid
  - Scenario: WHEN window resizes THEN grid adapts responsively
- Requirement: Full-Text Search
  - Scenario: WHEN user types query THEN results filter instantly
  - Scenario: WHEN no results THEN helpful message shows

**Suggested Tasks (example):**
- "Create ProductGrid component" (complexity: 4/10, 3 hours)
- "Implement search API endpoint" (complexity: 6/10, 6 hours)
- "Build filter UI with checkboxes" (complexity: 5/10, 4 hours)

### Phase 2: Spec Creation

**Step 4: Review & Edit**

Before creating specs:

✅ **Review Capabilities**
- Verify names are clear
- Check purpose statements
- Ensure proper grouping

✅ **Review Requirements**
- Verify scenarios are testable
- Add missing scenarios
- Clarify ambiguous requirements

✅ **Review Tasks**
- Check task descriptions
- Verify complexity estimates
- Add or remove tasks as needed

**Step 5: Create Specs**

1. Click **"Create Specs"**
2. Specs are saved to database:
   - 3 capabilities created
   - 9 requirements created
   - 27 scenarios created
3. View in **Specifications** section

### Phase 3: Task Generation

**Step 6: Generate Tasks**

1. Review suggested tasks
2. Click **"Generate Tasks"**
3. Tasks are created and linked:
   - 15 tasks created
   - All linked to source requirements
   - Added to project task list

**Step 7: Task Organization**

Organize generated tasks:

1. **Prioritize** by complexity/importance
2. **Assign** to team members
3. **Estimate** hours if not auto-estimated
4. **Schedule** in sprints/milestones

### Phase 4: Implementation & Validation

**Step 8: Implement Tasks**

For each task:

1. Review linked requirement
2. Read all WHEN/THEN scenarios
3. Implement functionality
4. Write tests for scenarios
5. Mark task complete

**Step 9: Validate Completion**

1. Click **"Validate"** on completed task
2. AI checks implementation against scenarios:
   ```json
   {
     "scenario": "WHEN user types query THEN results filter",
     "status": "passed",
     "details": "Implementation correctly filters results"
   }
   ```
3. Fix any failing scenarios
4. Re-validate until all pass

**Step 10: Track Progress**

Monitor progress in SyncDashboard:
- Requirements coverage: 75% (6/9 with completed tasks)
- Task completion: 60% (9/15 tasks done)
- Validation status: 8 passed, 1 failed

## Workflow 2: Task → Spec → PRD

The bottom-up approach starts with implementation and updates specifications.

### Phase 1: Manual Task Creation

**Step 1: Developer Creates Task**

Developer identifies need not in specs:

```
Task: Add product comparison feature
Description: Allow users to compare up to 3 products side-by-side
```

Task is created as **orphan** (no spec link).

### Phase 2: Orphan Detection

**Step 2: System Detects Orphan**

SyncDashboard automatically shows:
- Orphan Tasks: 1
- New task appears in orphan list
- Alert icon on task card

**Step 3: Review Orphan**

Click task to review:
- Task details
- When created
- By whom
- Current status

### Phase 3: Spec Suggestion

**Step 4: AI Analyzes Task**

Click **"Suggest Spec"** button. AI analyzes and suggests:

**Option 1: Add to Existing Capability**

```
Capability: Product Catalog Display
New Requirement: Product Comparison

Scenarios:
- WHEN user selects 2-3 products THEN comparison view opens
- WHEN user compares products THEN features display side-by-side
- WHEN user removes product THEN comparison updates
- WHEN only 1 product selected THEN comparison disabled

Confidence: 85%
Rationale: Fits naturally with product display features
```

**Option 2: Create New Capability**

```
Capability: Advanced Product Features
Purpose: Enhanced product interaction features

Requirement: Product Comparison
(same scenarios as above)

Confidence: 65%
Rationale: Could be expanded with wishlist, reviews, etc.
```

**Step 5: Choose Option**

Select best option based on:
- Project architecture
- Team organization
- Future roadmap

### Phase 4: Change Proposal

**Step 6: Create Change Proposal**

If significant change, create formal proposal:

1. Click **"Create Change Proposal"**
2. Review generated proposal:

```markdown
# Change Proposal: Add Product Comparison

## Proposal
Add product comparison feature to allow users to compare
up to 3 products side-by-side.

## Affected Capabilities
- Product Catalog Display

## Deltas
- Added Requirement: Product Comparison
  - 4 new scenarios
  - 1 linked task

## Tasks
- [ ] Add product comparison feature (existing task)
- [ ] Write comparison tests
- [ ] Update documentation

## Rationale
User research shows 40% of users want to compare products
before purchase decision.
```

3. Edit if needed
4. Click **"Submit for Review"**

**Step 7: Review & Approval**

Change proposals go through workflow:

1. **Draft** - Being written
2. **Review** - Awaiting approval
3. **Approved** - Ready to implement
4. **Implementing** - Work in progress
5. **Completed** - Implemented and merged

Approval triggers:
- Requirement added to capability
- Task linked to new requirement
- PRD updated with new section

### Phase 5: PRD Regeneration

**Step 8: Automatic PRD Update**

When change is approved, PRD regenerates:

```markdown
# E-Commerce Product Catalog

[existing content...]

### Product Comparison
Allow users to compare up to 3 products side-by-side to make
informed purchase decisions.

**Scenarios:**
- When user selects 2-3 products, comparison view opens
- When user compares products, features display side-by-side
- When user removes product, comparison updates
- When only 1 product selected, comparison is disabled

**Implementation:**
- Task: Add product comparison feature
```

**Step 9: Sync History**

All changes tracked in sync history:

```json
{
  "sync_id": "sync_001",
  "prd_id": "prd_abc",
  "direction": "task_to_spec",
  "changes": {
    "added_requirements": 1,
    "modified_requirements": 0,
    "linked_tasks": 1
  },
  "performed_by": "user_123",
  "performed_at": "2025-01-20T15:00:00Z"
}
```

## Workflow 3: Spec-to-Spec Sync

Synchronize changes between related specs.

### Use Case: Shared Requirements

**Scenario**: Multiple capabilities need user authentication

**Solution**: Create shared requirements

1. Create "Authentication" capability
2. Add "User Login" requirement with scenarios
3. Link from multiple capabilities:
   - Product reviews → "User must be logged in"
   - Order history → "User must be logged in"
   - Wishlist → "User must be logged in"

### Cross-Capability Dependencies

Track dependencies between capabilities:

```json
{
  "capability_id": "product-reviews",
  "dependencies": [
    {
      "capability_id": "authentication",
      "requirement_id": "user-login",
      "type": "requires"
    },
    {
      "capability_id": "product-catalog",
      "requirement_id": "product-details",
      "type": "uses"
    }
  ]
}
```

## Change Management

### Change Proposal Workflow

**1. Creation**

```bash
POST /api/:project_id/changes
{
  "prd_id": "prd_abc",
  "proposal_markdown": "# Proposal...",
  "tasks_markdown": "- [ ] Task 1...",
  "status": "draft"
}
```

**2. Review**

Reviewers check:
- Proposal rationale
- Spec deltas (additions/modifications/removals)
- Linked tasks
- Impact on existing requirements

**3. Approval**

```bash
PUT /api/:project_id/changes/:change_id/status
{
  "status": "approved",
  "approved_by": "user_123"
}
```

Approval triggers:
- Specs updated with deltas
- Tasks linked to requirements
- PRD regenerated
- Notifications sent

**4. Implementation**

Track implementation progress:
- Tasks created from proposal
- Tasks marked complete
- Requirements validated

**5. Completion**

When all tasks done:
- Change marked "completed"
- Metrics updated
- Success recorded

### Delta Management

Deltas describe spec changes:

**Addition Delta:**
```json
{
  "delta_type": "added",
  "capability_name": "Product Comparison",
  "delta_markdown": "## Product Comparison\n...",
  "requirements": [...]
}
```

**Modification Delta:**
```json
{
  "delta_type": "modified",
  "capability_id": "product-catalog",
  "delta_markdown": "### Updated Requirement...",
  "requirements": [
    {
      "id": "req_existing",
      "changes": "Added 2 new scenarios for edge cases"
    }
  ]
}
```

**Removal Delta:**
```json
{
  "delta_type": "removed",
  "capability_id": "legacy-feature",
  "delta_markdown": "Removed deprecated feature",
  "reason": "No longer supported in v2.0"
}
```

## Sync Strategies

### Manual Sync

Trigger sync manually when:
- Major milestone reached
- Before release
- After sprint completion
- On demand

```bash
POST /api/projects/:project_id/prds/:prd_id/sync
{
  "direction": "spec_to_prd",  # or "prd_to_spec"
  "mode": "merge",              # or "replace", "append"
  "dry_run": false
}
```

### Automatic Sync

Configure automatic sync:

```json
{
  "sync_enabled": true,
  "sync_triggers": [
    "change_approved",
    "requirement_completed",
    "weekly_schedule"
  ],
  "sync_direction": "bidirectional",
  "conflict_resolution": "manual"  # or "auto_merge"
}
```

### Conflict Resolution

Handle sync conflicts:

**Conflict Types:**
1. Same requirement modified in PRD and spec
2. Task linked to deleted requirement
3. Spec added without PRD update

**Resolution Strategies:**
1. **Manual Review** - Present conflicts to user
2. **Auto-Merge** - Use latest timestamp
3. **Spec Wins** - Spec always takes precedence
4. **PRD Wins** - PRD always takes precedence

## Best Practices

### Workflow Selection

**Use PRD→Spec→Task when:**
- Starting new project
- Clear requirements upfront
- Waterfall or staged approach
- Stakeholder buy-in needed

**Use Task→Spec→PRD when:**
- Agile/iterative development
- Requirements emerge from implementation
- Tight timelines
- Proof-of-concept work

### Change Management

✅ **Do:**
- Create proposals for significant changes
- Document rationale
- Get approval before implementation
- Track all changes in sync history

❌ **Don't:**
- Make direct spec edits without proposals
- Skip approval for breaking changes
- Ignore orphan tasks
- Delete requirements with linked tasks

### Sync Frequency

**Recommended:**
- Manual sync: Weekly or per sprint
- Automatic sync: On change approval
- PRD regeneration: Monthly or per release

**Avoid:**
- Syncing too frequently (noise)
- Syncing too rarely (drift)
- Automatic sync without review

## Troubleshooting

### Sync Fails

**Problem**: "Sync failed" error

**Solutions:**
1. Check sync history for errors
2. Verify all requirements exist
3. Resolve conflicts manually
4. Try dry-run first
5. Check database constraints

### Orphans Keep Growing

**Problem**: Orphan count increasing

**Solutions:**
1. Set team policy: link before approve
2. Weekly orphan review meeting
3. Use AI suggestions for common patterns
4. Create templates for frequent tasks

### Changes Not Reflecting in PRD

**Problem**: PRD doesn't update after spec changes

**Solutions:**
1. Check change status (must be "approved")
2. Manually trigger PRD sync
3. Verify sync history for errors
4. Check PRD status (can't update "approved" PRD)

## API Reference

### Create Change Proposal

```bash
POST /api/:project_id/changes
```

### Update Change Status

```bash
PUT /api/:project_id/changes/:change_id/status
```

### Create Delta

```bash
POST /api/:project_id/changes/:change_id/deltas
```

### Sync PRD

```bash
POST /api/projects/:project_id/prds/:prd_id/sync
```

### Get Sync History

```bash
GET /api/projects/:project_id/prds/:prd_id/sync-history
```

## Next Steps

- [Monitor AI costs](./cost-tracking.md)
- [Explore AI features](./ai-features.md)
- [Review PRD management](./prds.md)
- [Understand task integration](./tasks.md)

---

**Related**: [PRDs](./prds.md) | [Specs](./specs.md) | [Tasks](./tasks.md) | [AI Features](./ai-features.md)
