# Template Field Synchronization - Implementation Plan

## Overview
Extend PRD quickstart templates to pre-fill all Guided Mode input fields, not just dependencies. Users can create, edit, and manage custom templates via new Template Manager UI.

---

## Phase 1: Database Schema - Add Template Fields for All Sections

**Status:** ✅ Complete

**Migration File:** `012_extend_templates_schema.sql` (and corresponding `.down.sql`)

**Objective:** Extend `prd_quickstart_templates` table with defaults for every Guided Mode section

**Checklist:**
- [x] Create `012_extend_templates_schema.sql` with new columns
- [x] Create `012_extend_templates_schema.down.sql` rollback
- [x] Add JSON validation CHECK constraints
- [x] Update seed data with template defaults
- [x] Test migration files created successfully

**Summary:**
Created migration 012 with 12 new columns across 5 sections:
- Overview: problem_statement, target_audience, value_proposition
- UX: ui_considerations, ux_principles
- Technical: tech_stack_quick
- Roadmap: mvp_scope (JSON array)
- Research: research_findings, technical_specs, competitors (JSON array), similar_projects (JSON array)

All 5 system templates (SaaS, Mobile, API, Marketplace, Internal Tool) seeded with realistic defaults for each section.

**New Columns to Add:**

### Overview Section
- `default_problem_statement` (TEXT)
- `default_target_audience` (TEXT)
- `default_value_proposition` (TEXT)

### UX Section
- `default_ui_considerations` (TEXT)
- `default_ux_principles` (TEXT)

### Technical Section
- `default_tech_stack_quick` (TEXT) - free-form text with key technologies

### Roadmap Section
- `default_mvp_scope` (TEXT - JSON array of strings)

### Risks Section
- (Leave empty as per requirements)

### Research Section
- `default_research_findings` (TEXT)
- `default_technical_specs` (TEXT)
- `default_competitors` (TEXT - JSON array of competitor names/descriptions)
- `default_similar_projects` (TEXT - JSON array of similar project references)

**JSON Validation:**
- Add CHECK constraints for JSON array fields:
  - `default_mvp_scope`
  - `default_competitors`
  - `default_similar_projects`

**Deliverables:**
- `012_extend_templates_schema.sql` - Add all columns with constraints
- `012_extend_templates_schema.down.sql` - Drop new columns
- Update seed data in `008_ideate_templates.sql` (or create new seed migration if needed)

---

## Phase 2: Backend - Update Template Application Logic

**Status:** ✅ Complete

**Summary:**
- Updated `PRDTemplate` struct with 12 new fields across all sections
- Updated `CreateTemplateInput` with same 12 fields
- Enhanced `get_templates_by_category`, `get_template`, and `create_template` to handle all fields
- Added `update_template` method for editing user-created templates (system templates protected)
- Implemented `apply_template_to_session` method that creates all 6 sections with template defaults
- All template-created sections marked with `ai_generated = 1`
- Proper JSON parsing for array fields (mvp_scope, competitors, similar_projects)

**Files to Modify:**
- `/packages/ideate/src/manager.rs` - `apply_template_to_session` method
- `/packages/ideate/src/types.rs` - Update `PRDTemplate` struct if needed

**Objective:** Enhance `apply_template_to_session` to populate all 6 sections with template defaults

**Checklist:**
- [x] Update `PRDTemplate` struct with new fields
- [x] Update `CreateTemplateInput` struct with new fields
- [x] Update `get_templates_by_category` to fetch all new fields
- [x] Update `get_template` to fetch all new fields
- [x] Update `create_template` to insert all new fields
- [x] Add `update_template` method for editing templates
- [x] Create `apply_template_to_session` method in manager.rs
- [x] Add overview section creation logic
- [x] Add UX section creation logic
- [x] Add technical section creation logic
- [x] Add roadmap section creation logic
- [x] Add research section creation logic
- [x] Add dependencies section creation logic
- [x] Add JSON parsing for array fields
- [x] Add error handling and logging

**Changes:**
- Modify `apply_template_to_session` to:
  - Create `ideate_overview` with default_problem_statement, default_target_audience, default_value_proposition
  - Create `ideate_ux` with default_ui_considerations, default_ux_principles
  - Create `ideate_technical` with default_tech_stack_quick
  - Create `ideate_roadmap` with default_mvp_scope (parsed from JSON)
  - Create `ideate_research` with default_research_findings, default_technical_specs, default_competitors, default_similar_projects
  - Mark all sections with `ai_generated = 1` to indicate template origin
  - Skip sections where all fields are NULL (don't create empty sections)

**Error Handling:**
- Proper error handling for malformed JSON
- Logging for debugging template application

**Deliverables:**
- Updated `apply_template_to_session` method with comprehensive section creation

---

## Phase 3: Backend - Template CRUD API Endpoints

**Status:** ✅ Complete

**Summary:**
- Added 4 new handlers: `create_template`, `update_template`, `delete_template`, `get_template`
- All handlers properly use TemplateManager methods
- System templates protected from modification (Forbidden error on update/delete)
- Added routes to ideate router:
  - POST /ideate/templates - Create
  - PUT /ideate/templates/{id} - Update
  - DELETE /ideate/templates/{id} - Delete
  - GET /ideate/templates/{id} - Get single (already existed)
- Proper error handling and logging

**Files to Modify:**
- `/packages/api/src/ideate_handlers.rs` - Add new handlers
- `/packages/ideate/src/templates.rs` - Add/update manager methods

**Objective:** Expose endpoints for creating, reading, updating, and deleting templates

**Checklist:**
- [x] Add `create_template` handler
- [x] Add `update_template` handler
- [x] Add `delete_template` handler
- [x] Add `get_template` handler to return all new fields
- [x] Add input validation (via CreateTemplateInput struct)
- [x] Add permission checks (system templates read-only in update_template)
- [x] Add error responses (Forbidden for system templates)
- [x] Add routes to router

**Endpoints:**

| Method | Endpoint | Purpose | Auth |
|--------|----------|---------|------|
| GET | `/api/ideate/templates` | List all templates | Public |
| GET | `/api/ideate/templates/:id` | Get single template | Public |
| POST | `/api/ideate/templates` | Create new template | User |
| PUT | `/api/ideate/templates/:id` | Update existing template | Owner |
| DELETE | `/api/ideate/templates/:id` | Delete template | Owner |

**Request/Response Schemas:**

**POST/PUT Request Body:**
```json
{
  "name": "string",
  "description": "string",
  "project_type": "saas|mobile|api|marketplace|internal-tool",
  "default_problem_statement": "string",
  "default_target_audience": "string",
  "default_value_proposition": "string",
  "default_ui_considerations": "string",
  "default_ux_principles": "string",
  "default_tech_stack_quick": "string",
  "default_mvp_scope": ["string"],
  "default_research_findings": "string",
  "default_technical_specs": "string",
  "default_competitors": ["string"],
  "default_similar_projects": ["string"]
}
```

**Validation Rules:**
- System templates (is_system=1) cannot be modified or deleted
- Only user-created templates can be updated/deleted
- JSON array fields must be valid JSON
- name and project_type are required

**Deliverables:**
- New handler functions: `create_template`, `update_template`, `delete_template`
- Updated `get_template` handler to return all new fields
- Input validation and error responses
- Proper permission checks

---

## Phase 4: Frontend - Template Manager Tab on /templates Page

**Status:** 🔄 In Progress

**Files to Create/Modify:**
- `/packages/dashboard/src/pages/templates.tsx` - Add new tab
- `/packages/dashboard/src/components/TemplateManager.tsx` - New component
- `/packages/dashboard/src/components/TemplateEditor.tsx` - New form component
- `/packages/dashboard/src/components/TemplateList.tsx` - New list component
- `/packages/dashboard/src/hooks/useTemplates.ts` - New hooks for CRUD

**Objective:** Create full CRUD UI for template management

**Checklist:**
- [x] Create `useQuickstartTemplates.ts` hooks (create, update, delete, list)
- [x] Add template methods to ideateService
- [x] Add PRDTemplate and CreateTemplateInput types to ideateService
- [ ] Create `TemplateList.tsx` component
- [ ] Create `TemplateEditor.tsx` form component with tabs
- [ ] Create `TemplateManager.tsx` main component
- [ ] Add tab to `/pages/templates.tsx`
- [ ] Add system template read-only UI
- [ ] Add duplicate template functionality
- [ ] Add search/filter by project type
- [ ] Test all CRUD operations

**UI Structure:**

### Template Manager Tab
- List view of all templates (system + user-created)
- Visual distinction between system and user templates (badge/icon)
- Search/filter by project type
- Action buttons: Edit, Duplicate, Delete
- "New Template" button

### Template Editor Modal/Form
- Tabs for each section:
  - Overview
  - UX
  - Technical
  - Roadmap
  - Research
- Form fields matching template schema:
  - **Overview Tab:**
    - Problem Statement (textarea)
    - Target Audience (textarea)
    - Value Proposition (textarea)
  - **UX Tab:**
    - UI Considerations (textarea)
    - UX Principles (textarea)
  - **Technical Tab:**
    - Tech Stack Quick (textarea, free-form)
  - **Roadmap Tab:**
    - MVP Scope (list editor - add/remove items)
  - **Research Tab:**
    - Research Findings (textarea)
    - Technical Specs (textarea)
    - Competitors (list editor)
    - Similar Projects (list editor)
- Save/Cancel buttons
- Validation feedback
- System templates shown as read-only

### Template Creation Flow
- "New Template" button opens form
- Select project_type (dropdown)
- Enter template name and description
- Fill in section defaults (all optional)
- Save as user-created template

### Template Duplication
- "Duplicate" action on any template
- Creates new user-owned copy with "-copy" suffix
- Opens editor for immediate customization

**Deliverables:**
- `TemplateManager.tsx` - Main component with tab integration
- `TemplateEditor.tsx` - Form with all section tabs
- `TemplateList.tsx` - List view with actions
- `useTemplates.ts` - React Query hooks for CRUD operations
- Proper loading/error states and toast notifications

---

## Phase 5: End-to-End Testing

**Status:** ⏳ Pending

**Objective:** Verify complete workflow from template creation to session pre-fill

**Checklist:**
- [ ] Test create custom template
- [ ] Test edit user-created template
- [ ] Test duplicate template
- [ ] Test delete template
- [ ] Test session pre-fill with custom template
- [ ] Test partial template data
- [ ] Test JSON array fields
- [ ] Verify database state
- [ ] Create documentation/screenshots

**Test Scenarios:**

### 1. Create Custom Template
- [ ] Create new template with all fields populated
- [ ] Verify saved in database with correct values
- [ ] Verify appears in template list
- [ ] Verify marked as user-created (is_system=0)

### 2. Edit Template
- [ ] Modify existing user-created template fields
- [ ] Verify changes persist in database
- [ ] Verify system templates shown as read-only (no edit button)
- [ ] Verify cannot edit system templates

### 3. Duplicate Template
- [ ] Duplicate system template
- [ ] Verify new copy is user-owned
- [ ] Verify can edit the copy
- [ ] Verify original unchanged

### 4. Delete Template
- [ ] Delete user-created template
- [ ] Verify removed from list
- [ ] Verify cannot delete system templates (no delete button)

### 5. Session Pre-fill
- [ ] Create Guided Mode session
- [ ] Select custom template
- [ ] Navigate through all 7 sections
- [ ] Verify all fields pre-filled with template defaults
- [ ] Verify can edit pre-filled values
- [ ] Verify `ai_generated` flag set correctly for template-filled sections

### 6. Template with Partial Data
- [ ] Create template with only some fields filled
- [ ] Create session with template
- [ ] Verify empty fields remain empty (not filled with defaults)
- [ ] Verify only non-empty sections created

### 7. JSON Array Fields
- [ ] Create template with MVP scope items
- [ ] Create template with competitors list
- [ ] Create template with similar projects list
- [ ] Verify arrays properly parsed and displayed in session

**Deliverables:**
- Test checklist with pass/fail results
- Screenshots of template manager UI
- Verification of database state
- Sample templates created for testing

---

## Implementation Sequence

1. **Phase 1** → Create migration with new columns
2. **Phase 2** → Update backend application logic
3. **Phase 3** → Add API endpoints
4. **Phase 4** → Build frontend UI
5. **Phase 5** → Run full test suite

**Estimated Effort:**
- Phase 1: 30 min (migration)
- Phase 2: 1 hour (backend logic)
- Phase 3: 1.5 hours (API endpoints)
- Phase 4: 2-3 hours (frontend UI)
- Phase 5: 1 hour (testing)

**Total: ~6-7 hours**

---

## Files to Create/Modify Summary

### Database
- Create: `012_extend_templates_schema.sql`
- Create: `012_extend_templates_schema.down.sql`

### Backend
- Modify: `/packages/ideate/src/manager.rs`
- Modify: `/packages/ideate/src/types.rs` (if needed)
- Modify: `/packages/api/src/ideate_handlers.rs`
- Modify: `/packages/ideate/src/templates.rs`

### Frontend
- Modify: `/packages/dashboard/src/pages/templates.tsx`
- Create: `/packages/dashboard/src/components/TemplateManager.tsx`
- Create: `/packages/dashboard/src/components/TemplateEditor.tsx`
- Create: `/packages/dashboard/src/components/TemplateList.tsx`
- Create: `/packages/dashboard/src/hooks/useTemplates.ts`

---

## Notes

- System templates (is_system=1) are read-only in UI
- User-created templates can be fully modified and deleted
- Template duplication creates a new user-owned copy
- Session pre-fill marks template-sourced sections with `ai_generated = 1`
- Empty template fields don't create sections in sessions
- All JSON fields have validation constraints in database
