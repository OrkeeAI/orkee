-- Rollback: 012_extend_templates_schema
-- Removes all added columns from prd_quickstart_templates

ALTER TABLE prd_quickstart_templates DROP COLUMN default_problem_statement;
ALTER TABLE prd_quickstart_templates DROP COLUMN default_target_audience;
ALTER TABLE prd_quickstart_templates DROP COLUMN default_value_proposition;

ALTER TABLE prd_quickstart_templates DROP COLUMN default_ui_considerations;
ALTER TABLE prd_quickstart_templates DROP COLUMN default_ux_principles;

ALTER TABLE prd_quickstart_templates DROP COLUMN default_tech_stack_quick;

ALTER TABLE prd_quickstart_templates DROP COLUMN default_mvp_scope;

ALTER TABLE prd_quickstart_templates DROP COLUMN default_research_findings;
ALTER TABLE prd_quickstart_templates DROP COLUMN default_technical_specs;

ALTER TABLE prd_quickstart_templates DROP COLUMN default_competitors;

ALTER TABLE prd_quickstart_templates DROP COLUMN default_similar_projects;
