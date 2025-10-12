---
sidebar_position: 2
title: Contributing Guide
---

# Contributing to Orkee

First off, thank you for considering contributing to Orkee! It's people like you that make Orkee such a great tool for AI agent orchestration.

## Quick Start for Contributors

Ready to contribute? Here's how to get started in 5 minutes:

1. **Fork and clone** the repository
2. **Install dependencies** with `bun install`
3. **Start developing** with `turbo dev`
4. **Make your changes** following our guidelines
5. **Submit a pull request**

The rest of this guide provides detailed information on each step.

## Table of Contents

- [Development Workflow](#development-workflow)
- [GitHub Flow](#github-flow)
- [Commit Message Conventions](#commit-message-conventions)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Bug Reports](#bug-reports)
- [Community Guidelines](#community-guidelines)

## Development Workflow

### Prerequisites

Make sure you have these tools installed before starting:

- **Node.js** v18 or higher
- **bun** v1.0+ (our package manager)
- **Rust** (latest stable version)
- **Git** for version control

### Setting Up Your Development Environment

1. **Fork the repository** on GitHub and clone your fork:
   ```bash
   git clone https://github.com/YOUR-USERNAME/orkee.git
   cd orkee
   ```

2. **Add the upstream remote** to stay in sync with the main repository:
   ```bash
   git remote add upstream https://github.com/OrkeeAI/orkee.git
   ```

3. **Install dependencies** using bun:
   ```bash
   bun install
   ```

4. **Start the development servers** for all packages:
   ```bash
   turbo dev
   ```
   This starts both the CLI server (port 4001) and dashboard (port 5173).

5. **Verify everything works** by opening http://localhost:5173 in your browser.

### Understanding the Project Structure

Orkee is organized as a monorepo with five main packages:

```
orkee/
├── packages/
│   ├── cli/          # Rust Axum HTTP server (REST API backend)
│   ├── dashboard/    # React SPA with Vite (web interface)
│   ├── tui/          # Ratatui terminal interface
│   ├── projects/     # Shared Rust library for project management
│   └── cloud/        # Optional cloud sync functionality
├── docs/             # Docusaurus documentation site
└── turbo.json        # Turborepo configuration
```

**Key characteristics:**
- **CLI Server** (`packages/cli/`): Provides REST API endpoints at port 4001
- **Dashboard** (`packages/dashboard/`): React frontend with Shadcn/ui and Tailwind CSS
- **TUI** (`packages/tui/`): Standalone terminal interface using the projects library
- **Projects** (`packages/projects/`): Core library shared between CLI and TUI
- **Cloud** (`packages/cloud/`): Optional features for Orkee Cloud integration

### Working on Specific Packages

You can develop and test individual packages using Turborepo filters:

```bash
# Dashboard only
turbo dev --filter=@orkee/dashboard

# CLI server only
turbo dev --filter=@orkee/cli

# Build specific package
turbo build --filter=@orkee/dashboard

# Test specific package
turbo test --filter=@orkee/cli
```

### Running the CLI Directly

For CLI-specific development, you can use cargo commands directly:

```bash
cd packages/cli

# Start dashboard (API + UI)
cargo run --bin orkee -- dashboard

# Use local dashboard in development
cargo run --bin orkee -- dashboard --dev

# Custom ports
cargo run --bin orkee -- dashboard --api-port 8080 --ui-port 3000

# Launch TUI
cargo run --bin orkee -- tui

# Project management commands
cargo run --bin orkee -- projects list
cargo run --bin orkee -- projects add
cargo run --bin orkee -- projects show <id>

# Run tests
cargo test

# Production build
cargo build --release
```

## GitHub Flow

We use [GitHub Flow](https://guides.github.com/introduction/flow/) for all changes. This is a lightweight, branch-based workflow.

### Making Changes

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

   Use descriptive branch names that indicate the purpose:
   - `feature/add-project-export` - for new features
   - `fix/health-check-race-condition` - for bug fixes
   - `docs/api-endpoint-updates` - for documentation
   - `refactor/simplify-auth-flow` - for refactoring

2. **Make your changes** following our [coding standards](#coding-standards).

3. **Write or update tests** to cover your changes.

4. **Run tests** to ensure nothing broke:
   ```bash
   turbo test
   ```

5. **Run linting** to catch style issues:
   ```bash
   turbo lint
   ```

6. **Commit your changes** using [conventional commits](#commit-message-conventions):
   ```bash
   git add .
   git commit -m "feat: add project export functionality"
   ```

7. **Keep your branch up to date** with the main repository:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

8. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

9. **Open a pull request** on GitHub.

## Commit Message Conventions

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. This creates a standardized commit history that's easy to follow and enables automated changelog generation.

### Commit Message Format

```
<type>: <description>

[optional body]

[optional footer]
```

### Commit Types

- **feat**: A new feature for users
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Code style changes (formatting, missing semicolons, etc.)
- **refactor**: Code changes that neither fix bugs nor add features
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **chore**: Changes to build process, dependencies, or tooling

### Examples

**Simple feature addition:**
```
feat: add project export functionality
```

**Bug fix with description:**
```
fix: resolve race condition in health check polling

The health check was occasionally returning stale status due to
concurrent access to the health state. Added proper synchronization
using Arc<RwLock>.
```

**Breaking change:**
```
feat!: change project API response format

BREAKING CHANGE: The `/api/projects` endpoint now returns projects
in a `data` field instead of directly as an array. Update client
code to access `response.data` instead of using the response directly.
```

**Documentation update:**
```
docs: update API endpoint documentation

Added examples for all project management endpoints and clarified
the response format.
```

**Multiple changes in one commit (avoid this if possible):**
```
feat: implement project tagging system

- Add tags field to Project model
- Create API endpoints for tag management
- Update dashboard to display tags
- Add tests for tag functionality
```

### Tips for Good Commit Messages

- **Use the imperative mood**: "add feature" not "added feature"
- **Keep the first line under 50 characters** when possible
- **Capitalize the first letter** of the description
- **Don't end the description with a period**
- **Use the body** to explain _what_ and _why_, not _how_
- **Reference issues** in the footer: `Closes #123` or `Fixes #456`

## Coding Standards

### General Principles

1. **Simplicity over cleverness**: Write clear, maintainable code
2. **Consistency**: Match the style of surrounding code
3. **Don't repeat yourself**: Extract common logic into reusable functions
4. **Self-documenting**: Use descriptive names instead of comments when possible
5. **Comments for "why"**: Explain reasoning, not implementation

### Rust Code Style

We follow standard Rust conventions with these guidelines:

#### Formatting
```bash
# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

#### Linting
```bash
# Run Clippy
cargo clippy

# Fail on warnings
cargo clippy -- -D warnings
```

#### Code Organization

**Good example:**
```rust
// ABOUTME: This file manages project CRUD operations
// ABOUTME: Handles SQLite storage and Git integration

use crate::models::Project;
use std::path::PathBuf;

pub struct ProjectManager {
    storage: SqliteStorage,
}

impl ProjectManager {
    /// Creates a new project with the given configuration.
    ///
    /// Returns an error if the project path is invalid or already exists.
    pub fn create_project(&self, config: ProjectConfig) -> Result<Project> {
        self.validate_path(&config.path)?;
        self.storage.insert(config)
    }

    fn validate_path(&self, path: &PathBuf) -> Result<()> {
        // Validation logic
    }
}
```

**Key points:**
- Every file starts with a 2-line ABOUTME comment
- Public APIs have doc comments (`///`)
- Private helpers don't need doc comments if the name is clear
- Use `Result` for operations that can fail
- Prefer `&str` for read-only strings, `String` for owned strings

#### Error Handling

Use `anyhow` for application errors and `thiserror` for library errors:

```rust
use anyhow::{Context, Result};

fn load_project(id: &str) -> Result<Project> {
    let project = storage.get(id)
        .context("Failed to load project from database")?;

    Ok(project)
}
```

#### Testing

Write tests in the same file or in a `tests/` subdirectory:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_success() {
        let manager = ProjectManager::new_in_memory();
        let config = ProjectConfig {
            name: "test".to_string(),
            path: PathBuf::from("/tmp/test"),
        };

        let result = manager.create_project(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_invalid_path() {
        let manager = ProjectManager::new_in_memory();
        let config = ProjectConfig {
            name: "test".to_string(),
            path: PathBuf::from(""),
        };

        let result = manager.create_project(config);
        assert!(result.is_err());
    }
}
```

### TypeScript/React Code Style

We use TypeScript in strict mode with modern React patterns.

#### File Organization

**Component file structure:**
```tsx
// ABOUTME: This file displays project details and manages project actions
// ABOUTME: Includes edit, delete, and sync functionality

import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';

interface ProjectDetailProps {
  projectId: string;
}

export function ProjectDetail({ projectId }: ProjectDetailProps) {
  const [project, setProject] = useState<Project | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadProject();
  }, [projectId]);

  const loadProject = async () => {
    // Load project logic
  };

  return (
    <Card>
      {/* Component JSX */}
    </Card>
  );
}
```

#### TypeScript Best Practices

**Use explicit types for props and state:**
```tsx
// Good
interface ButtonProps {
  label: string;
  onClick: () => void;
  disabled?: boolean;
}

// Avoid
const Button = (props: any) => { /* ... */ }
```

**Prefer type inference for local variables:**
```tsx
// Good
const projects = await fetchProjects();
const hasProjects = projects.length > 0;

// Avoid unnecessary annotations
const projects: Project[] = await fetchProjects();
const hasProjects: boolean = projects.length > 0;
```

**Use discriminated unions for state:**
```tsx
type FetchState<T> =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: T }
  | { status: 'error'; error: string };

const [state, setState] = useState<FetchState<Project[]>>({ status: 'idle' });
```

#### React Patterns

**Functional components with hooks:**
```tsx
export function ProjectList() {
  const [projects, setProjects] = useState<Project[]>([]);
  const { connection } = useConnection();

  useEffect(() => {
    if (connection.isConnected) {
      loadProjects();
    }
  }, [connection.isConnected]);

  return (
    <div className="grid gap-4">
      {projects.map(project => (
        <ProjectCard key={project.id} project={project} />
      ))}
    </div>
  );
}
```

**Custom hooks for reusable logic:**
```tsx
function useProjects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadProjects = useCallback(async () => {
    try {
      setLoading(true);
      const data = await api.projects.list();
      setProjects(data);
      setError(null);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadProjects();
  }, [loadProjects]);

  return { projects, loading, error, reload: loadProjects };
}
```

#### Styling with Tailwind CSS

Use Tailwind utility classes and maintain consistent spacing:

```tsx
<div className="flex flex-col gap-4 p-6">
  <h1 className="text-2xl font-bold">Projects</h1>
  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {/* Cards */}
  </div>
</div>
```

#### API Client Patterns

Centralize API calls in service files:

```typescript
// services/projects.ts
export const projectsApi = {
  list: async (): Promise<Project[]> => {
    const response = await api.get('/api/projects');
    return response.data;
  },

  get: async (id: string): Promise<Project> => {
    const response = await api.get(`/api/projects/${id}`);
    return response.data;
  },

  create: async (data: ProjectCreateInput): Promise<Project> => {
    const response = await api.post('/api/projects', data);
    return response.data;
  },
};
```

### Documentation Standards

#### Code Comments

**When to comment:**
- Complex algorithms or business logic
- Non-obvious design decisions
- Workarounds for known issues
- Public API documentation

**When not to comment:**
- Self-explanatory code
- What the code does (should be clear from names)
- Redundant information

**Good comments:**
```rust
// We use Arc<RwLock> here because the health state needs to be shared
// across multiple threads but writes are infrequent compared to reads.
let health_state = Arc::new(RwLock::new(HealthState::default()));

// Retry up to 3 times with exponential backoff to handle transient
// network errors without overwhelming the server.
for attempt in 0..3 {
    match send_request().await {
        Ok(response) => return Ok(response),
        Err(e) if attempt < 2 => {
            tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempt))).await;
        }
        Err(e) => return Err(e),
    }
}
```

**Bad comments:**
```rust
// Bad: States the obvious
let mut count = 0; // Initialize count to 0

// Bad: Redundant
// Get project by ID
fn get_project(id: &str) -> Result<Project> {
    // ...
}

// Bad: Outdated or misleading
// TODO: This is a temporary hack (written 2 years ago)
// This uses the new API (when there's no old API)
```

#### README and Documentation Updates

When adding features or making significant changes:

1. **Update README.md** if public interfaces change
2. **Update CLAUDE.md** if the change affects AI agent interactions
3. **Add migration notes** if breaking changes are introduced
4. **Update API documentation** in the docs package

## Testing Requirements

We take testing seriously. All contributions must include appropriate tests.

### Test Coverage Expectations

- **New features**: Comprehensive test coverage including edge cases
- **Bug fixes**: At least one test demonstrating the fix
- **Refactoring**: Existing tests must pass; add tests for new edge cases
- **Documentation**: No tests required

### Running Tests

```bash
# Run all tests across all packages
turbo test

# Run Rust tests
cd packages/cli && cargo test
cd packages/projects && cargo test

# Run Rust tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_project

# Run Dashboard tests
cd packages/dashboard && bun test

# Run with coverage
cargo tarpaulin --out Html
```

### Writing Good Tests

#### Rust Tests

**Unit tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let manager = ProjectManager::new_in_memory();
        let config = create_test_config("test-project");

        let result = manager.create_project(config);

        assert!(result.is_ok());
        let project = result.unwrap();
        assert_eq!(project.name, "test-project");
    }

    #[test]
    fn test_duplicate_project_fails() {
        let manager = ProjectManager::new_in_memory();
        let config = create_test_config("duplicate");

        manager.create_project(config.clone()).unwrap();
        let result = manager.create_project(config);

        assert!(result.is_err());
    }

    fn create_test_config(name: &str) -> ProjectConfig {
        ProjectConfig {
            name: name.to_string(),
            path: PathBuf::from(format!("/tmp/{}", name)),
        }
    }
}
```

**Integration tests:**
```rust
// tests/integration/project_api.rs
use axum::http::StatusCode;
use axum_test::TestServer;

#[tokio::test]
async fn test_create_project_endpoint() {
    let app = create_test_app().await;
    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/api/projects")
        .json(&json!({
            "name": "test-project",
            "path": "/tmp/test"
        }))
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: ApiResponse<Project> = response.json();
    assert!(body.success);
    assert_eq!(body.data.name, "test-project");
}
```

#### TypeScript/React Tests

**Component tests with React Testing Library:**
```tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { ProjectCard } from './ProjectCard';

describe('ProjectCard', () => {
  const mockProject = {
    id: '1',
    name: 'Test Project',
    path: '/tmp/test',
    description: 'A test project',
  };

  it('renders project information', () => {
    render(<ProjectCard project={mockProject} />);

    expect(screen.getByText('Test Project')).toBeInTheDocument();
    expect(screen.getByText('/tmp/test')).toBeInTheDocument();
  });

  it('calls onEdit when edit button is clicked', () => {
    const onEdit = jest.fn();
    render(<ProjectCard project={mockProject} onEdit={onEdit} />);

    fireEvent.click(screen.getByRole('button', { name: /edit/i }));

    expect(onEdit).toHaveBeenCalledWith(mockProject.id);
  });
});
```

**API service tests:**
```typescript
import { projectsApi } from './projects';
import { api } from './api';

jest.mock('./api');

describe('projectsApi', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('fetches projects list', async () => {
    const mockProjects = [{ id: '1', name: 'Test' }];
    (api.get as jest.Mock).mockResolvedValue({ data: mockProjects });

    const result = await projectsApi.list();

    expect(api.get).toHaveBeenCalledWith('/api/projects');
    expect(result).toEqual(mockProjects);
  });
});
```

### Test Organization

- **Unit tests**: In the same file as the code (for Rust) or `__tests__` directory
- **Integration tests**: In `tests/` directory at package root
- **End-to-end tests**: In `e2e/` directory at repository root
- **Test utilities**: In `tests/helpers/` or `tests/fixtures/`

## Pull Request Process

### Before Submitting

Complete this checklist before opening a PR:

- [ ] Code compiles without warnings
- [ ] All tests pass (`turbo test`)
- [ ] Code is properly formatted (`cargo fmt`, `bun run lint`)
- [ ] No linting errors (`cargo clippy`, `turbo lint`)
- [ ] Commit messages follow [conventional commits](#commit-message-conventions)
- [ ] Documentation is updated if needed
- [ ] ABOUTME comments added to new files
- [ ] Tests cover new functionality or bug fixes

### Creating a Pull Request

1. **Push your branch** to your fork on GitHub.

2. **Open a pull request** against the `main` branch of the upstream repository.

3. **Fill out the PR template** with:
   - **Description**: Clear explanation of what changed and why
   - **Motivation**: The problem you're solving or feature you're adding
   - **Testing**: How you tested the changes
   - **Screenshots**: For UI changes (before/after)
   - **Breaking changes**: Any API or behavior changes
   - **Related issues**: Use `Closes #123` or `Fixes #456`

4. **Example PR description:**
   ```markdown
   ## Description
   Adds project export functionality allowing users to export projects as JSON files.

   ## Motivation
   Users requested the ability to back up and share project configurations.
   Closes #234

   ## Changes
   - Added export endpoint at `/api/projects/:id/export`
   - Implemented `ProjectExporter` service
   - Added export button to project detail page
   - Included tests for export functionality

   ## Testing
   - Unit tests for ProjectExporter
   - Integration test for API endpoint
   - Manual testing with 5+ projects
   - Verified JSON structure and file download

   ## Screenshots
   [Screenshot of export button and download]

   ## Checklist
   - [x] Code compiles and tests pass
   - [x] Documentation updated
   - [x] Conventional commit messages used
   ```

### Review Process

1. **Automated checks** run on your PR (tests, linting, formatting).
2. **Maintainer review**: A maintainer will review your code and provide feedback.
3. **Address feedback**: Make changes based on review comments.
4. **Approval**: Once approved, a maintainer will merge your PR.

### After Your PR is Merged

1. **Delete your feature branch** (GitHub offers to do this automatically).
2. **Update your fork** with the latest from upstream:
   ```bash
   git checkout main
   git pull upstream main
   git push origin main
   ```
3. **Celebrate!** 🎉 You've contributed to Orkee.

## Bug Reports

Found a bug? We want to hear about it!

### Before Submitting a Bug Report

1. **Check existing issues**: Your bug might already be reported
2. **Update to latest version**: The bug might be fixed
3. **Reproduce the bug**: Make sure it's consistent

### Creating a Bug Report

Open a [new issue](https://github.com/OrkeeAI/orkee/issues/new) with these details:

**Great bug reports include:**

- **Quick summary**: One-line description of the bug
- **Environment**:
  - OS and version (macOS 14.1, Ubuntu 22.04, Windows 11, etc.)
  - Node.js version (`node --version`)
  - Orkee version (`orkee --version`)
  - Rust version (`rustc --version`)
- **Steps to reproduce**:
  1. Start Orkee dashboard
  2. Create a new project
  3. Click export button
  4. Observe error in console
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Logs and errors**: Copy relevant error messages
- **Screenshots**: If applicable
- **Additional context**: Any other relevant information

**Example bug report:**
```markdown
## Bug: Export button throws error on projects with tags

### Environment
- OS: macOS 14.1
- Node.js: v20.10.0
- Orkee: v0.2.1
- Browser: Chrome 120.0

### Steps to Reproduce
1. Create a project with tags ["frontend", "react"]
2. Navigate to project detail page
3. Click the "Export" button
4. Check browser console

### Expected Behavior
Project exports as JSON file with all fields including tags.

### Actual Behavior
Export fails with error: "Cannot serialize undefined property 'tags'"

### Error Message
```
TypeError: Cannot serialize undefined property 'tags'
    at ProjectExporter.serialize (exporter.ts:45)
    at handleExport (ProjectDetail.tsx:89)
```

### Screenshots
[Screenshot of error in console]

### Additional Context
This only happens with projects that have tags. Projects without tags export successfully.
```

## Community Guidelines

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. All participants are expected to:

- **Use welcoming and inclusive language**
- **Be respectful** of differing viewpoints and experiences
- **Accept constructive criticism gracefully**
- **Focus on what is best** for the community
- **Show empathy** towards other community members

Unacceptable behavior includes:

- Harassment, discrimination, or personal attacks
- Trolling, insulting comments, or inflammatory remarks
- Public or private harassment
- Publishing others' private information
- Other conduct that could reasonably be considered inappropriate

### Reporting Issues

If you experience or witness unacceptable behavior, please report it to the maintainers at [conduct@orkee.ai](mailto:conduct@orkee.ai).

### Getting Help

Need help contributing?

- **Documentation**: Check the [main README](https://github.com/OrkeeAI/orkee/blob/main/README.md)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/OrkeeAI/orkee/discussions)
- **Issues**: Open an issue with the "question" label
- **Community**: Join our community chat (link TBD)

### Recognition

We value all contributions! Contributors who make significant contributions may:

- Be added to the **contributors list** in README.md
- Receive **credit in release notes** for their contributions
- Be **invited to become project maintainers** based on sustained contributions

## License

By contributing to Orkee, you agree that your contributions will be licensed under the [MIT License](https://github.com/OrkeeAI/orkee/blob/main/LICENSE).

---

Thank you for contributing to Orkee! Your efforts help make AI agent orchestration better for everyone. 🎉

If you have questions about this guide or the contribution process, don't hesitate to ask in [GitHub Discussions](https://github.com/OrkeeAI/orkee/discussions).
