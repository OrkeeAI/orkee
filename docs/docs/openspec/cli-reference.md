---
sidebar_position: 7
---

# CLI Reference

The `orkee spec` command provides command-line tools for managing OpenSpec changes, specifications, and workflows.

## Global Options

All `spec` commands support these options:

- `--help` - Show help information
- `--version` - Show version information

## Commands Overview

| Command | Description |
|---------|-------------|
| `spec list` | List active changes or specifications |
| `spec show` | Show details of a specific change |
| `spec validate` | Validate changes against OpenSpec format |
| `spec archive` | Archive a completed change and apply deltas |
| `spec export` | Export specs to filesystem |
| `spec import` | Import specs from filesystem |

---

## `spec list`

List all active changes or specifications for a project.

### Usage

```bash
orkee spec list [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--project <ID>` | Filter by project ID | Auto-detect from current directory |
| `--json` | Output as JSON | false |

### Examples

```bash
# List changes for current project (auto-detected)
orkee spec list

# List changes for specific project
orkee spec list --project my-project-id

# Output as JSON for scripting
orkee spec list --json
```

### Output

```
Changes for project: my-app

ID                    Status      Created              Tasks    Completed
add-user-auth-1       Draft       2025-01-20 10:30    12       0%
refactor-api-2        Review      2025-01-21 14:15    8        25%
fix-validation-3      Approved    2025-01-22 09:00    5        100%
```

---

## `spec show`

Show detailed information about a specific change, including proposal, tasks, and deltas.

### Usage

```bash
orkee spec show <CHANGE_ID> [OPTIONS]
```

### Arguments

- `<CHANGE_ID>` - The ID of the change to show

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--json` | Output as JSON | false |
| `--deltas-only` | Show only the spec deltas | false |

### Examples

```bash
# Show full change details
orkee spec show add-user-auth-1

# Show only spec deltas
orkee spec show add-user-auth-1 --deltas-only

# Output as JSON
orkee spec show add-user-auth-1 --json
```

### Output

```markdown
Change: add-user-auth-1
Status: Draft
Created: 2025-01-20 10:30:15
PRD: user-auth-requirements.md

## Proposal

Add user authentication system with JWT tokens and secure password hashing.

## Tasks

1. [ ] Create user model with encrypted password field
2. [ ] Implement JWT token generation and validation
3. [ ] Add login and registration endpoints
4. [ ] Create password reset flow

## Deltas

### ADDED: user-authentication

#### Requirement: User Registration
The system SHALL allow users to create accounts with email and password.

##### Scenario: Successful registration
- **WHEN** valid email and password are provided
- **THEN** account is created with hashed password
- **AND** verification email is sent
```

---

## `spec validate`

Validate changes against OpenSpec format requirements.

### Usage

```bash
orkee spec validate [CHANGE_ID] [OPTIONS]
```

### Arguments

- `[CHANGE_ID]` - Optional change ID to validate. If omitted, validates all changes.

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--strict` | Use strict validation (enforces normative language) | false |
| `--project <ID>` | Project ID (for validating all changes) | Auto-detect |

### Examples

```bash
# Validate specific change
orkee spec validate add-user-auth-1

# Validate with strict mode (enforces SHALL/MUST)
orkee spec validate add-user-auth-1 --strict

# Validate all changes in project
orkee spec validate --project my-project-id
```

### Validation Rules

#### Required Format

1. **Requirement Headers**: Must use `### Requirement: [Name]`
2. **Scenario Headers**: Must use `#### Scenario: [Name]` (exactly 4 hashtags)
3. **Scenario Format**: Must use WHEN/THEN bullet format
4. **Delta Operations**: Must start with `## ADDED`, `## MODIFIED`, or `## REMOVED Requirements`

#### Strict Mode (--strict)

Additionally enforces:
- **Normative Language**: Requirements must use SHALL or MUST (not should/may)
- **Complete Scenarios**: Every requirement must have at least one scenario

### Output

```
Validating change: add-user-auth-1

✅ Delta format valid
✅ Requirement headers correct
✅ Scenario headers correct (4 hashtags)
✅ WHEN/THEN format valid

Validation passed!
```

Or with errors:

```
Validating change: add-user-auth-1

❌ Validation errors found:

Line 15: Scenario headers must use exactly 4 hashtags (####)
Line 23: Scenarios must use '- **WHEN** ...' and '- **THEN** ...' format
Line 30: Requirements must use SHALL or MUST (strict mode)

Validation failed with 3 errors.
```

---

## `spec archive`

Archive a completed change and apply its deltas to create or update capabilities.

### Usage

```bash
orkee spec archive <CHANGE_ID> [OPTIONS]
```

### Arguments

- `<CHANGE_ID>` - The ID of the change to archive

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-y, --yes` | Skip confirmation prompt | false |
| `--skip-specs` | Skip updating specs (for tooling-only changes) | false |

### Examples

```bash
# Archive change with confirmation
orkee spec archive add-user-auth-1

# Archive without confirmation
orkee spec archive add-user-auth-1 --yes

# Archive without applying spec changes
orkee spec archive add-user-auth-1 --skip-specs
```

### Confirmation Prompt

```
Archive change add-user-auth-1 and apply deltas? [y/N]
```

### What Happens

When you archive a change:

1. **Validation**: Change is validated against OpenSpec format
2. **Delta Application**: Each delta is applied:
   - `ADDED` - Creates new capability with requirements and scenarios
   - `MODIFIED` - Updates existing capability
   - `REMOVED` - Marks capability as deprecated
3. **Status Update**: Change status set to "Archived"
4. **Timestamp**: Archive timestamp recorded

### Output

```
Validating change add-user-auth-1...
✅ Validation passed

Applying deltas...
✅ Created capability: user-authentication
✅ Created 4 requirements
✅ Created 8 scenarios

Change add-user-auth-1 archived successfully
```

### Errors

```
❌ Cannot archive change: Validation failed
❌ Cannot archive change: Already archived
❌ Cannot archive change: Status must be 'Approved' or 'Completed'
```

---

## `spec export`

Export OpenSpec structure from database to filesystem for version control.

### Usage

```bash
orkee spec export --project <PROJECT_ID> [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--project <ID>` | Project ID (required) | None |
| `--path <PATH>` | Directory to export to | `./` |

### Examples

```bash
# Export to current directory
orkee spec export --project my-project-id

# Export to specific path
orkee spec export --project my-project-id --path ./openspec-export
```

### Output Structure

```
./openspec/
├── project.md              # Project context and conventions
├── AGENTS.md               # OpenSpec agent instructions
├── specs/                  # Active specifications
│   ├── user-authentication/
│   │   ├── spec.md         # Requirements and scenarios
│   │   └── design.md       # Design notes (if present)
│   └── api-endpoints/
│       └── spec.md
├── changes/                # Active change proposals
│   ├── add-user-auth-1.md
│   └── refactor-api-2.md
└── archive/                # Archived changes
    └── fix-validation-3.md
```

### Files Created

- **project.md** - Generated from project metadata
- **AGENTS.md** - Stub with link to OpenSpec documentation
- **specs/[name]/spec.md** - Capability requirements and scenarios
- **specs/[name]/design.md** - Design notes (if present in database)
- **changes/[id].md** - Active change proposals with deltas
- **archive/[id].md** - Archived changes

---

## `spec import`

Import OpenSpec structure from filesystem into database.

### Usage

```bash
orkee spec import --project <PROJECT_ID> [OPTIONS]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `--project <ID>` | Project ID (required) | None |
| `--path <PATH>` | Directory to import from | `./` |
| `--force` | Overwrite existing data | false |

### Examples

```bash
# Import from current directory
orkee spec import --project my-project-id

# Import from specific path
orkee spec import --project my-project-id --path ./openspec-export

# Force overwrite of existing data
orkee spec import --project my-project-id --force
```

### Conflict Resolution

Without `--force`, import uses "Prefer Remote" strategy:
- If specification exists in DB but not in files → Keep in DB
- If specification exists in files but not in DB → Import from files
- If specification exists in both → Skip (keep DB version)

With `--force`, import uses "Overwrite" strategy:
- All imported files overwrite database versions

### Output

```
Importing OpenSpec data from ./openspec/...

✅ Imported 3 specifications
✅ Imported 2 active changes
✅ Imported 1 archived change

Import completed successfully
```

---

## Project Auto-Detection

Most commands auto-detect the project ID from your current directory by:

1. Looking for `.git`, `package.json`, `Cargo.toml`, or similar project markers
2. Searching up the directory tree to find project root
3. Querying database for project with matching path

If auto-detection fails, use `--project <ID>` explicitly.

### Example

```bash
# In project directory
cd ~/projects/my-app
orkee spec list              # Auto-detects my-app project

# Outside project directory
cd ~/
orkee spec list --project my-app-id  # Must specify project
```

---

## Common Workflows

### Creating and Archiving Changes

```bash
# 1. Create change in dashboard or via PRD analysis
# 2. Review and validate
orkee spec show add-user-auth-1
orkee spec validate add-user-auth-1 --strict

# 3. Archive when complete
orkee spec archive add-user-auth-1 --yes
```

### Exporting for Version Control

```bash
# Export specs before making changes
orkee spec export --project my-project-id --path ./openspec

# Commit to git
cd openspec
git add .
git commit -m "Export OpenSpec state before refactor"
```

### Importing in Sandbox

```bash
# In new environment/sandbox
orkee spec import --project my-project-id --path ./openspec

# Verify import
orkee spec list
```

---

## Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| "Project not found" | Invalid project ID | Use `orkee projects list` to find ID |
| "Change not found" | Invalid change ID | Use `spec list` to see available changes |
| "Validation failed" | OpenSpec format errors | Run `spec validate` to see specific issues |
| "Already archived" | Change already archived | Cannot archive twice |
| "Path not found" | Invalid export/import path | Check directory exists |

---

## See Also

- [OpenSpec Overview](./overview.md) - Introduction to OpenSpec concepts
- [Workflows](./workflows.md) - End-to-end development workflows
- [API Reference](../api-reference/openspec.md) - REST API endpoints
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions
