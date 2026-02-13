# PRD Sync Rules

Synchronizes the prd.json execution state back to the original PRD markdown file.

---

## Sync Rules

### 1. Completion Status

For each story in prd.json where `passes: true`:
- Find the matching story in the PRD markdown (by story ID)
- Mark ALL acceptance criteria checkboxes as checked: `- [x]`

For stories where `passes: false`:
- Leave checkboxes unchecked: `- [ ]`

### 2. Story Notes

If a story has content in its `notes` field:
- Add or update a "**Notes:**" section after acceptance criteria
- Preserve existing notes, append new ones if different

Example:
```markdown
### US-004: Implement auth backend
**Description:** As a developer...

**Acceptance Criteria:**
- [x] Better-Auth installed
- [x] Email/password enabled
- [x] Typecheck passes

**Notes:** Migrated from @convex-dev/auth to @convex-dev/better-auth. Uses triggers.user.onCreate for subscription creation.
```

### 3. New Stories

If prd.json contains stories not in the markdown PRD:
- Add them to the appropriate phase section
- Format using standard PRD story structure
- Mark with "(Added during implementation)" in the description

### 4. Modified Stories

If acceptance criteria differ between JSON and markdown:
- Prefer the JSON version (agent may have refined criteria)
- Note the change in the story's notes

### 5. Priority Changes

If story priorities were reordered significantly:
- Update the order in the markdown to match
- Note in the document header if major restructuring occurred

---

## Summary Output Format

After syncing, output a summary:

```
## PRD Sync Summary

**Source:** prd.json
**Target:** [target PRD file]

### Completed Stories (passes: true)
- US-001: Story title
- US-002: Story title

### Pending Stories (passes: false)
- US-038: Story title

### Stories with Notes
- US-004: "Implementation notes..."

### Changes Made
- Updated N stories to checked status
- Added notes to M stories
- N new stories added
```
