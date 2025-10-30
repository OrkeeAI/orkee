-- Migration: 012_extend_templates_schema
-- Description: Add template fields for all Guided Mode sections (Overview, UX, Technical, Roadmap, Research)
-- Created: 2025-01-29

-- ============================================================================
-- Extend prd_quickstart_templates with section-specific default fields
-- ============================================================================

ALTER TABLE prd_quickstart_templates ADD COLUMN default_problem_statement TEXT;
ALTER TABLE prd_quickstart_templates ADD COLUMN default_target_audience TEXT;
ALTER TABLE prd_quickstart_templates ADD COLUMN default_value_proposition TEXT;

ALTER TABLE prd_quickstart_templates ADD COLUMN default_ui_considerations TEXT;
ALTER TABLE prd_quickstart_templates ADD COLUMN default_ux_principles TEXT;

ALTER TABLE prd_quickstart_templates ADD COLUMN default_tech_stack_quick TEXT;

ALTER TABLE prd_quickstart_templates ADD COLUMN default_mvp_scope TEXT
  CHECK (json_valid(default_mvp_scope) OR default_mvp_scope IS NULL);

ALTER TABLE prd_quickstart_templates ADD COLUMN default_research_findings TEXT;
ALTER TABLE prd_quickstart_templates ADD COLUMN default_technical_specs TEXT;

ALTER TABLE prd_quickstart_templates ADD COLUMN default_competitors TEXT
  CHECK (json_valid(default_competitors) OR default_competitors IS NULL);

ALTER TABLE prd_quickstart_templates ADD COLUMN default_similar_projects TEXT
  CHECK (json_valid(default_similar_projects) OR default_similar_projects IS NULL);

-- ============================================================================
-- Populate defaults for existing system templates
-- ============================================================================

-- Template 1: SaaS Application
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the core problem or pain point your SaaS solves for users',
  default_target_audience = 'Define your primary users: business teams, enterprises, SMBs, or specific verticals',
  default_value_proposition = 'Explain the unique value and competitive advantage of your SaaS solution',
  default_ui_considerations = 'Clean, professional interface with intuitive navigation. Dashboard-centric design with real-time data updates. Mobile-responsive for on-the-go access.',
  default_ux_principles = 'Simplicity and efficiency. Minimize clicks to complete tasks. Provide contextual help and onboarding. Enable power users with keyboard shortcuts and advanced features.',
  default_tech_stack_quick = 'Frontend: React/Vue/Angular with TypeScript. Backend: Node.js/Python/Go. Database: PostgreSQL or MongoDB. Hosting: AWS/GCP/Azure. Authentication: OAuth 2.0 or SAML.',
  default_mvp_scope = '["User authentication and authorization", "Subscription billing and payments", "Team/workspace management", "Admin dashboard", "User settings and profiles"]',
  default_research_findings = 'Market analysis shows growing demand for specialized SaaS solutions. Key competitors: [list competitors]. Market size: [estimate]. Growth rate: [percentage].',
  default_technical_specs = 'API-first architecture. RESTful or GraphQL endpoints. Real-time sync with WebSockets. Horizontal scalability with microservices.',
  default_competitors = '["Competitor A - brief description", "Competitor B - brief description", "Competitor C - brief description"]',
  default_similar_projects = '["Similar project 1 - what to learn from it", "Similar project 2 - what to learn from it"]'
WHERE id = 'tpl_saas';

-- Template 2: Mobile App
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the core problem your mobile app solves and why mobile is the right platform',
  default_target_audience = 'Define your users: iOS only, Android only, or both. Age range, tech-savviness, use cases.',
  default_value_proposition = 'Explain the unique value of your mobile app compared to web alternatives',
  default_ui_considerations = 'Touch-friendly interface with large tap targets. Minimal text, icon-driven navigation. Bottom tab bar for main features. Native platform conventions (iOS vs Android).',
  default_ux_principles = 'Mobile-first thinking. Offline-first where possible. Fast load times. Minimal data usage. Intuitive gestures and animations.',
  default_tech_stack_quick = 'React Native or Flutter for cross-platform. Native Swift (iOS) or Kotlin (Android) for platform-specific. Firebase or similar for backend. SQLite for local storage.',
  default_mvp_scope = '["User onboarding flow", "Core feature 1", "Core feature 2", "Push notifications", "User profiles"]',
  default_research_findings = 'Mobile-first users expect fast, lightweight apps. Key competitors: [list]. App store trends: [insights]. User retention rates: [data].',
  default_technical_specs = 'Native APIs for camera, GPS, contacts. Offline sync with backend. Push notification service integration. App store deployment pipeline.',
  default_competitors = '["Competitor A - features and ratings", "Competitor B - features and ratings"]',
  default_similar_projects = '["Open source project 1 - architecture lessons", "Open source project 2 - design patterns"]'
WHERE id = 'tpl_mobile';

-- Template 3: API/Backend Service
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the data or service your API exposes and the problem it solves for consumers',
  default_target_audience = 'Define API consumers: internal apps, third-party developers, mobile clients, or all',
  default_value_proposition = 'Explain why developers should use your API over alternatives',
  default_ui_considerations = 'N/A - API only. Focus on developer experience: clear documentation, SDKs, interactive API explorer.',
  default_ux_principles = 'Developer experience first. Consistent API design. Clear error messages. Comprehensive documentation. Easy authentication and rate limiting.',
  default_tech_stack_quick = 'Node.js/Python/Go/Rust for API server. Express/FastAPI/Gin for framework. PostgreSQL/MongoDB for data. Redis for caching. Docker for deployment.',
  default_mvp_scope = '["Core endpoints", "Authentication (API keys or OAuth)", "Rate limiting", "API documentation", "Error handling"]',
  default_research_findings = 'API market trends: [insights]. Popular API patterns: REST vs GraphQL. Developer preferences: [data]. Monetization models: [options].',
  default_technical_specs = 'RESTful or GraphQL architecture. JWT or API key authentication. Versioning strategy (URL or header). Webhook support for events. OpenAPI/GraphQL schema.',
  default_competitors = '["Competitor API 1 - endpoints and features", "Competitor API 2 - pricing and limits"]',
  default_similar_projects = '["Open API 1 - design patterns", "Open API 2 - best practices"]'
WHERE id = 'tpl_backend';

-- Template 4: Marketplace/Platform
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the problem your marketplace solves by connecting two sides',
  default_target_audience = 'Define both sides: buyers/sellers, hosts/guests, service providers/customers. Include demographics and behaviors.',
  default_value_proposition = 'Explain the unique value for both sides of the marketplace',
  default_ui_considerations = 'Dual-sided interface design. Seller dashboard for listings and analytics. Buyer interface for discovery and checkout. Admin dashboard for moderation.',
  default_ux_principles = 'Trust and transparency. Easy listing creation for sellers. Intuitive search and filtering for buyers. Secure payment flow. Clear dispute resolution.',
  default_tech_stack_quick = 'Frontend: React for web, React Native for mobile. Backend: Node.js/Python. Database: PostgreSQL. Payment: Stripe/PayPal. Search: Elasticsearch.',
  default_mvp_scope = '["User profiles (buyer and seller)", "Listing creation and management", "Search and filtering", "Payment processing", "Reviews and ratings"]',
  default_research_findings = 'Marketplace trends: [insights]. Successful models: [examples]. Commission structures: [analysis]. User acquisition: [strategies].',
  default_technical_specs = 'Dual-sided authentication. Escrow payment system. Real-time notifications. Search and recommendation engine. Dispute resolution workflow.',
  default_competitors = '["Competitor 1 - commission, features, market share", "Competitor 2 - commission, features, market share"]',
  default_similar_projects = '["Successful marketplace 1 - what worked", "Successful marketplace 2 - lessons learned"]'
WHERE id = 'tpl_marketplace';

-- Template 5: Internal Tool / Dashboard
UPDATE prd_quickstart_templates SET
  default_problem_statement = 'Describe the business process or workflow this internal tool streamlines',
  default_target_audience = 'Define users: which departments, roles, or teams. Their technical proficiency.',
  default_value_proposition = 'Explain time/cost savings and efficiency gains for the organization',
  default_ui_considerations = 'Enterprise-grade interface. Data-dense but organized. Keyboard shortcuts for power users. Customizable dashboards. Print-friendly reports.',
  default_ux_principles = 'Efficiency and power. Minimize clicks for common tasks. Bulk operations. Keyboard navigation. Audit trails for compliance.',
  default_tech_stack_quick = 'Frontend: React with TypeScript. Backend: Node.js or Python. Database: PostgreSQL. Authentication: LDAP/Active Directory integration. Hosting: On-premise or cloud.',
  default_mvp_scope = '["Role-based access control (RBAC)", "Core data views", "Key workflows", "Reporting and exports", "Audit logs"]',
  default_research_findings = 'Internal tool adoption: [factors]. ROI metrics: [examples]. Integration needs: [systems]. Compliance requirements: [standards].',
  default_technical_specs = 'Enterprise authentication (LDAP, SAML). Data export formats (CSV, Excel, PDF). API for third-party integrations. Audit logging for compliance.',
  default_competitors = '["Commercial tool 1 - features and cost", "Commercial tool 2 - features and cost"]',
  default_similar_projects = '["Internal tool 1 - architecture", "Internal tool 2 - lessons learned"]'
WHERE id = 'tpl_internal';
