# Contributing to Orkee

First off, thank you for considering contributing to Orkee! It's people like you that make Orkee such a great tool for AI agent orchestration.

## We Develop with Github

We use GitHub to host code, to track issues and feature requests, as well as accept pull requests.

## We Use [Github Flow](https://guides.github.com/introduction/flow/index.html)

Pull requests are the best way to propose changes to the codebase. We actively welcome your pull requests:

1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the test suite passes.
5. Make sure your code lints.
6. Issue that pull request!

## Any contributions you make will be under the MIT Software License

In short, when you submit code changes, your submissions are understood to be under the same [MIT License](http://choosealicense.com/licenses/mit/) that covers the project. Feel free to contact the maintainers if that's a concern.

## Report bugs using Github's [issues](https://github.com/OrkeeAI/orkee/issues)

We use GitHub issues to track public bugs. Report a bug by [opening a new issue](https://github.com/OrkeeAI/orkee/issues/new); it's that easy!

## Write bug reports with detail, background, and sample code

**Great Bug Reports** tend to have:

- A quick summary and/or background
- Steps to reproduce
  - Be specific!
  - Give sample code if you can
- What you expected would happen
- What actually happens
- Notes (possibly including why you think this might be happening, or stuff you tried that didn't work)

## Development Process

### Prerequisites

- Node.js v18+
- pnpm v8+
- Rust (latest stable)
- Git

### Setting Up Your Development Environment

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/OrkeeAI/orkee.git
   cd orkee
   ```

2. Install dependencies:
   ```bash
   pnpm install
   ```

3. Start development servers:
   ```bash
   turbo dev
   ```

### Project Structure

Orkee is a monorepo with four main packages:
- **CLI Server** (`packages/cli/`) - Rust Axum HTTP server
- **Dashboard** (`packages/dashboard/`) - React SPA
- **TUI** (`packages/tui/`) - Terminal interface
- **Projects** (`packages/projects/`) - Shared Rust library

### Making Changes

1. Create a feature branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following our coding standards (see below)

3. Run tests:
   ```bash
   turbo test
   ```

4. Run linting:
   ```bash
   turbo lint
   ```

5. Commit your changes:
   ```bash
   git commit -m "feat: add amazing feature"
   ```

### Commit Message Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation only changes
- `style:` Changes that don't affect code meaning
- `refactor:` Code change that neither fixes a bug nor adds a feature
- `perf:` Performance improvement
- `test:` Adding missing tests
- `chore:` Changes to build process or auxiliary tools

Examples:
```
feat: add project export functionality
fix: resolve race condition in health check polling
docs: update API endpoint documentation
```

## Coding Standards

### Rust Code
- Follow standard Rust formatting (`cargo fmt`)
- Ensure no warnings with `cargo clippy`
- Write tests for new functionality
- Document public APIs with doc comments

### TypeScript/React Code
- Follow the existing code style
- Use TypeScript strict mode
- Prefer functional components with hooks
- Write meaningful component and variable names
- Add JSDoc comments for complex functions

### General Guidelines
- Keep changes focused and atomic
- Write clear, self-documenting code
- Add comments only when necessary to explain "why" not "what"
- Update relevant documentation
- Maintain backward compatibility when possible

## Testing

### Running Tests

```bash
# Run all tests
turbo test

# Run Rust tests only
cd packages/cli && cargo test
cd packages/projects && cargo test

# Run Dashboard tests
cd packages/dashboard && pnpm test
```

### Writing Tests

- Write unit tests for new functions/methods
- Add integration tests for API endpoints
- Include edge cases and error scenarios
- Aim for meaningful test coverage, not 100%

## Pull Request Process

1. Update the README.md with details of changes to the interface, if applicable
2. Update the CLAUDE.md file if you've made changes that affect AI agent interactions
3. The PR will be merged once you have the sign-off of at least one maintainer

### PR Checklist

Before submitting a PR, ensure:
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code follows the project's style guidelines
- [ ] Commit messages follow conventional commits
- [ ] Documentation is updated if needed
- [ ] PR description clearly describes the changes

## Community

### Code of Conduct

Please note we have a code of conduct. Please follow it in all your interactions with the project:

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

### Getting Help

If you need help, you can:
- Check the [documentation](./README.md)
- Open a [discussion](https://github.com/OrkeeAI/orkee/discussions)
- Ask in an issue (label it as a question)

## Recognition

Contributors who make significant contributions may be:
- Added to the contributors list
- Given credit in release notes
- Invited to become project maintainers

## License

By contributing to Orkee, you agree that your contributions will be licensed under its MIT License.

---

Thank you again for your contribution! 🎉