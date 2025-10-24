---
sidebar_position: 9
---

# Troubleshooting Guide

Common issues and solutions for OpenSpec workflows.

## Validation Errors

### Error: "Scenarios must use '#### Scenario:' format"

**Cause:** Scenario headers don't have exactly 4 hashtags.

**Solution:** Ensure all scenario headers use exactly 4 hashtags:

```markdown
✅ Correct:
#### Scenario: Successful login

❌ Wrong:
### Scenario: Successful login
##### Scenario: Successful login
```

---

### Error: "Scenarios must use '- **WHEN** ...' format"

**Cause:** Scenario bullets don't follow WHEN/THEN/AND format.

**Solution:** Use the correct bullet format:

```markdown
✅ Correct:
#### Scenario: User registration
- **WHEN** valid email and password are provided
- **THEN** account is created
- **AND** verification email is sent

❌ Wrong:
#### Scenario: User registration
When valid email and password are provided
Then account is created
```

---

### Error: "Requirements must use SHALL or MUST"

**Cause:** Using non-normative language (should, may, could) in strict mode.

**Solution:** Replace with normative language:

```markdown
✅ Correct:
The system SHALL validate email addresses.
The application MUST hash passwords with bcrypt.

❌ Wrong:
The system should validate email addresses.
The application may hash passwords with bcrypt.
```

**Tip:** Use "SHALL" for mandatory requirements and "MUST" for critical requirements.

---

### Error: "Requirements must use '### Requirement:' format"

**Cause:** Requirement headers don't follow the correct format.

**Solution:** Use exactly 3 hashtags followed by "Requirement:":

```markdown
✅ Correct:
### Requirement: User Authentication

❌ Wrong:
## Requirement: User Authentication
### User Authentication
```

---

### Error: "Delta must start with ## ADDED, ## MODIFIED, or ## REMOVED Requirements"

**Cause:** Spec delta doesn't have a valid operation header.

**Solution:** Start each delta with a valid operation:

```markdown
✅ Correct:
## ADDED Requirements

### Requirement: User Registration
...

✅ Also correct:
## MODIFIED Requirements

### Requirement: Login Flow
...

❌ Wrong:
## Requirements

### Requirement: User Registration
...
```

---

## Archive Errors

### Error: "Cannot archive change: Already archived"

**Cause:** Attempting to archive a change that's already been archived.

**Solution:** Check change status:

```bash
orkee spec show <change-id>
```

If status is "Archived", the change cannot be archived again. If you need to modify an archived change:

1. Create a new change with the modifications
2. Archive the new change

---

### Error: "Cannot archive change: Status must be 'Approved' or 'Completed'"

**Cause:** Change is still in Draft or Review status.

**Solution:** Update the change status first:

```bash
# Via API
curl -X PUT http://localhost:4001/api/changes/<change-id>/status \
  -H "Content-Type: application/json" \
  -d '{"status": "approved"}'

# Then archive
orkee spec archive <change-id>
```

Or use the dashboard to transition through the approval workflow:
1. Draft → Review
2. Review → Approved
3. Approved → Archive

---

### Error: "Validation failed: Cannot archive invalid change"

**Cause:** Change has OpenSpec format errors.

**Solution:** Fix validation errors before archiving:

```bash
# Check what's wrong
orkee spec validate <change-id> --strict

# Fix errors in the change
# Then try archiving again
orkee spec archive <change-id>
```

---

## Task Completion Issues

### Tasks Not Showing in Dashboard

**Cause:** Tasks haven't been parsed from the markdown yet.

**Solution:** Trigger task parsing:

```bash
# Via API
curl -X POST http://localhost:4001/api/changes/<change-id>/tasks/refresh

# Or reload the change in the dashboard
```

The system automatically parses tasks when:
- A change is first created
- The tasks_markdown field is updated
- Manual refresh is triggered

---

### Task Completion Percentage Not Updating

**Cause:** Database trigger might not be firing properly.

**Solution:**

1. Check if tasks exist:
```bash
# Via API
curl http://localhost:4001/api/changes/<change-id>/tasks
```

2. If tasks exist but percentage is wrong, refresh:
```bash
curl -X POST http://localhost:4001/api/changes/<change-id>/tasks/refresh
```

3. If still wrong, check database:
```sql
SELECT
    id,
    tasks_total_count,
    tasks_completed_count,
    tasks_completion_percentage
FROM spec_changes
WHERE id = '<change-id>';
```

---

### Cannot Toggle Task Completion

**Cause:** Task might not exist or API request is malformed.

**Solution:** Verify task exists and use correct request format:

```bash
# Check task exists
curl http://localhost:4001/api/changes/<change-id>/tasks

# Toggle with correct format
curl -X POST http://localhost:4001/api/changes/<change-id>/tasks/<task-id>/toggle \
  -H "Content-Type: application/json" \
  -d '{"completed_by": "user-id"}'
```

---

## Import/Export Issues

### Error: "Path not found"

**Cause:** Export or import path doesn't exist.

**Solution:** Create the directory first:

```bash
mkdir -p ./openspec
orkee spec export --project <project-id> --path ./openspec
```

For import, ensure the directory contains a valid OpenSpec structure:
```
openspec/
├── project.md
├── AGENTS.md
├── specs/
├── changes/
└── archive/
```

---

### Export Creates Empty Directories

**Cause:** Project has no specifications or changes yet.

**Solution:** This is normal for new projects. Create some content first:

1. Upload a PRD
2. Analyze it to create changes
3. Archive changes to create specs
4. Then export will have content

---

### Import Overwrites My Changes

**Cause:** Using `--force` flag or PreferRemote strategy.

**Solution:**

Without `--force`, import uses PreferLocal strategy (keeps DB data):
```bash
orkee spec import --project <project-id> --path ./openspec
```

To preview what would be imported without changes:
```bash
# Export current state first
orkee spec export --project <project-id> --path ./current-state

# Compare with import source
diff -r ./current-state/openspec/ ./import-source/openspec/
```

---

## PRD Analysis Issues

### AI Analysis Fails

**Cause:** Missing API key or rate limit exceeded.

**Solution:**

1. Check API key is configured:
```bash
echo $ANTHROPIC_API_KEY
# or
echo $OPENAI_API_KEY
```

2. If missing, add to environment:
```bash
export ANTHROPIC_API_KEY="sk-ant-api03-..."
orkee dashboard --restart
```

3. If rate limited, wait and retry:
```
Error: Rate limit exceeded. Please retry after 60 seconds.
```

---

### Analysis Creates Invalid Specs

**Cause:** AI sometimes generates incorrect format despite instructions.

**Solution:** Validate and fix manually:

```bash
# Check what's wrong
orkee spec validate <change-id> --strict

# View the problematic delta
orkee spec show <change-id> --deltas-only

# Fix in the database or via API
# Then re-validate
orkee spec validate <change-id> --strict
```

**Prevention:** Use strict validation mode and fix issues before archiving.

---

### Change Created But No Deltas

**Cause:** PRD analysis succeeded but delta creation failed.

**Solution:** Check the change:

```bash
orkee spec show <change-id> --deltas-only
```

If no deltas exist, the PRD might not have contained extractable capabilities. Try:

1. Make PRD more specific with clear requirements
2. Add more detail about features to build
3. Re-analyze the PRD

---

## Database Issues

### Migration Failed

**Cause:** Database schema might be out of date.

**Solution:** Check which migrations have run:

```sql
SELECT * FROM _sqlx_migrations ORDER BY version;
```

Look for these OpenSpec migrations:
- `20250118000000_openspec.sql`
- `20250127000000_openspec_alignment.sql`
- `20250128000000_task_completion_tracking.sql`

If missing, migrations need to be run. Restart Orkee to trigger automatic migration.

---

### Foreign Key Constraint Failed

**Cause:** Trying to create change for non-existent project or PRD.

**Solution:** Verify the project and PRD exist:

```bash
# Check project exists
orkee projects list

# Check PRD exists (via API)
curl http://localhost:4001/api/prds?project_id=<project-id>
```

---

### Column Not Found Error

**Cause:** Database schema is out of sync with code.

**Solution:**

1. Back up your database:
```bash
cp ~/.orkee/orkee.db ~/.orkee/orkee.db.backup
```

2. Check for failed migrations:
```sql
SELECT * FROM _sqlx_migrations WHERE success = 0;
```

3. If found, may need to manually apply or restart Orkee:
```bash
orkee dashboard --restart
```

---

## Performance Issues

### Validation is Slow

**Cause:** Large change with many deltas.

**Solution:**

- Break large changes into smaller ones
- Use `--deltas-only` flag when viewing:
```bash
orkee spec show <change-id> --deltas-only
```

---

### Export Takes Too Long

**Cause:** Many specifications or large markdown content.

**Solution:**

- Export to SSD instead of network drive
- Filter by specific specs if possible (future feature)
- Check disk space:
```bash
df -h
```

---

## CLI Issues

### Command Not Found: orkee spec

**Cause:** Using older version of Orkee without spec commands.

**Solution:** Update Orkee:

```bash
npm update -g orkee
# or
cargo install --git https://github.com/OrkeeAI/orkee --force
```

Verify version:
```bash
orkee --version
```

---

### Auto-Detection Failed

**Cause:** Not in a project directory or project not registered.

**Solution:**

1. Check if in project directory:
```bash
pwd
```

2. List registered projects:
```bash
orkee projects list
```

3. If project not registered, add it:
```bash
orkee projects add --name "My Project" --path $(pwd)
```

4. Or use explicit `--project` flag:
```bash
orkee spec list --project <project-id>
```

---

## Frontend Issues

### Changes Not Updating in Dashboard

**Cause:** Dashboard might be using stale data.

**Solution:**

1. Refresh the page
2. Check server connection (health indicator in top-right)
3. If server disconnected, restart:
```bash
orkee dashboard --restart
```

---

### Task Checkboxes Not Responding

**Cause:** Optimistic update failed and UI didn't revert.

**Solution:**

1. Refresh the page to get fresh data
2. Check browser console for errors
3. Verify API is responding:
```bash
curl http://localhost:4001/api/health
```

---

### Validation Errors Not Showing

**Cause:** Validation hasn't been run yet.

**Solution:** Trigger validation:

```bash
# Via CLI
orkee spec validate <change-id>

# Or via dashboard - click "Validate" button
```

---

## Getting Help

If your issue isn't covered here:

1. **Check Logs**
```bash
# Server logs
tail -f ~/.orkee/logs/orkee.log

# Database queries (set RUST_LOG=debug)
RUST_LOG=debug orkee dashboard
```

2. **Verify Database**
```bash
# Open database
sqlite3 ~/.orkee/orkee.db

# Check spec_changes table
SELECT id, status, validation_status FROM spec_changes;
```

3. **Report Issue**

Create an issue at [https://github.com/OrkeeAI/orkee/issues](https://github.com/OrkeeAI/orkee/issues) with:
- Orkee version (`orkee --version`)
- Error message
- Steps to reproduce
- Relevant logs

---

## Common Gotchas

### ✋ Archived Changes Cannot Be Modified

Once archived, changes are immutable. Create a new change to make modifications.

### ✋ Validation is Not Automatic

Changes are not automatically validated. Run `spec validate` before archiving.

### ✋ Deltas Are Not Applied Until Archive

Creating a change doesn't update specs. You must archive the change to apply deltas.

### ✋ Task Completion Percentage is Read-Only

Don't manually update `tasks_completion_percentage`. It's calculated by database triggers.

### ✋ OpenSpec Format is Strict

The format requirements exist for a reason - they enable automated processing and validation.

---

## See Also

- [CLI Reference](./cli-reference.md) - Full command documentation
- [API Reference](../api-reference/openspec.md) - REST API details
- [Workflows](./workflows.md) - End-to-end processes
- [Getting Started](./getting-started.md) - Initial setup guide
