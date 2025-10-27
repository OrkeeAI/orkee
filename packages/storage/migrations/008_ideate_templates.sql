-- Migration: 008_ideate_templates
-- Description: Seed default PRD quickstart templates
-- Created: 2025-01-27

-- ============================================================================
-- Default PRD Quickstart Templates
-- ============================================================================

-- Template 1: SaaS Application
INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_saas',
    'SaaS Application',
    'Perfect for building web-based software as a service applications with subscription models, user management, and team collaboration features.',
    'saas',
    '["What problem does this SaaS solve for users?", "What makes your SaaS unique in the market?", "Who is your primary target customer?", "What pricing model will you use (freemium, tiered, per-seat)?"]',
    '["User authentication and authorization", "Subscription billing and payments", "Team/workspace management", "Admin dashboard", "User settings and profiles", "Email notifications", "API access", "Activity logging and audit trails"]',
    '{"authentication": ["billing", "teams"], "billing": ["dashboard", "user-profiles"], "teams": ["workspace-management", "permissions"], "dashboard": ["analytics", "reporting"]}',
    1,
    datetime('now', 'utc')
);

-- Template 2: Mobile App
INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_mobile',
    'Mobile App (iOS/Android)',
    'Ideal for native or cross-platform mobile applications with offline capabilities, push notifications, and device integration.',
    'mobile',
    '["What core functionality do users need on mobile?", "Will this work offline or require constant connectivity?", "What device features will you use (camera, GPS, etc)?", "Is this iOS-only, Android-only, or cross-platform?"]',
    '["User onboarding flow", "Push notifications", "Offline mode and sync", "Device permissions handling", "In-app purchases (optional)", "Social sharing", "User profiles", "Settings and preferences", "Pull-to-refresh", "Deep linking"]',
    '{"onboarding": ["auth", "user-profile"], "auth": ["user-profile", "settings"], "offline-mode": ["sync-engine", "local-storage"], "push-notifications": ["user-settings", "notification-center"], "in-app-purchases": ["billing-system"]}',
    1,
    datetime('now', 'utc')
);

-- Template 3: API/Backend Service
INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_backend',
    'API / Backend Service',
    'Best for building RESTful or GraphQL APIs, microservices, and backend infrastructure with authentication, rate limiting, and comprehensive documentation.',
    'api',
    '["What data will your API manage and expose?", "Who are the API consumers (internal apps, third parties, both)?", "What authentication method will you use (API keys, OAuth, JWT)?", "What is your expected scale and performance requirements?"]',
    '["RESTful endpoints or GraphQL schema", "Authentication and authorization (JWT, OAuth)", "Rate limiting and throttling", "API documentation (Swagger/OpenAPI)", "Versioning strategy", "Error handling and logging", "Database integration", "Caching layer", "Webhooks", "Monitoring and health checks"]',
    '{"auth": ["endpoints", "middleware"], "rate-limiting": ["auth", "middleware"], "database": ["models", "migrations"], "caching": ["database"], "monitoring": ["logging", "health-checks"], "documentation": ["endpoints"]}',
    1,
    datetime('now', 'utc')
);

-- Template 4: Marketplace/Platform
INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_marketplace',
    'Marketplace / Two-Sided Platform',
    'For building platforms connecting buyers and sellers, service providers and customers, or any two-sided marketplace with transactions and reviews.',
    'marketplace',
    '["Who are the two sides of your marketplace (buyers/sellers, hosts/guests, etc)?", "What gets exchanged (products, services, bookings, etc)?", "How will you handle payments and transactions?", "What trust and safety features are needed?"]',
    '["User profiles (buyer and seller)", "Listing creation and management", "Search and filtering", "Messaging between users", "Payment processing and escrow", "Reviews and ratings system", "Dispute resolution workflow", "Commission/fee structure", "Trust and safety features", "Analytics dashboard for sellers"]',
    '{"user-profiles": ["authentication", "verification"], "listings": ["user-profiles", "search"], "search": ["listings", "filters"], "messaging": ["user-profiles"], "payments": ["listings", "escrow"], "reviews": ["payments", "user-profiles"], "dispute-resolution": ["messaging", "admin-tools"], "analytics": ["payments", "listings"]}',
    1,
    datetime('now', 'utc')
);

-- Template 5: Internal Tool / Dashboard
INSERT INTO prd_quickstart_templates (
    id,
    name,
    description,
    project_type,
    one_liner_prompts,
    default_features,
    default_dependencies,
    is_system,
    created_at
) VALUES (
    'tpl_internal',
    'Internal Tool / Dashboard',
    'Perfect for building internal enterprise tools, admin panels, data visualization dashboards, and business intelligence applications.',
    'internal-tool',
    '["What business process or workflow does this tool support?", "Who will use this tool (which departments or roles)?", "What data sources will it connect to?", "What actions can users perform through this tool?"]',
    '["Role-based access control (RBAC)", "Data visualization and charts", "Reporting and exports", "Bulk data operations", "Audit logs", "Search and filtering", "Data import/export", "Notifications and alerts", "Workflow automation", "Integration with existing systems"]',
    '{"rbac": ["authentication", "permissions"], "data-viz": ["data-access", "charting-library"], "reporting": ["data-access", "export-engine"], "bulk-operations": ["data-validation", "queuing"], "audit-logs": ["user-actions", "logging"], "automation": ["triggers", "actions"], "integrations": ["api-client", "webhooks"]}',
    1,
    datetime('now', 'utc')
);
