# Agent Instructions

You are an autonomous coding agent working on a software project.

## Your Task

1. Read the PRD context provided in the system prompt (project description and user stories)
2. Read the project's CLAUDE.md if it exists for codebase conventions
3. Ensure you're on the main branch and it's up to date: `git checkout main && git pull` (or `master`)
4. You will be assigned ONE user story per iteration
5. Create a feature branch for this story: `git checkout -b <story-id>-<short-description>`
6. **Write tests first** (TDD) - tests should fail initially
7. Implement the feature to make tests pass
8. Run quality checks (typecheck + lint + tests)
9. If checks pass, commit ALL changes with message: `feat: [Story ID] - [Story Title]`
10. Push the branch and create a PR: `git push -u origin <branch> && gh pr create --fill`
11. Merge the PR: `gh pr merge --squash --delete-branch`
12. Return to main branch: `git checkout main && git pull`

## Quality Requirements

- ALL commits must pass quality checks (typecheck, lint, tests)
- Do NOT commit broken code
- Keep changes focused and minimal
- Follow existing code patterns

## Test-Driven Development (TDD) - REQUIRED

**Every story MUST include tests.** No PR should be merged without new tests.

### TDD Workflow

1. **Identify what to test** - Based on acceptance criteria, determine key behaviors
2. **Write failing tests first** - Create test file with tests that fail
3. **Run tests** - Confirm tests fail as expected
4. **Implement the feature** - Write only enough code to make tests pass
5. **Run tests** - Confirm all tests pass
6. **Refactor** if needed while keeping tests green

## Branch Strategy

Use a **PR-per-story** workflow:
1. Start from main branch (always pull latest first)
2. Create a feature branch for each story (e.g., `us-004-auth-backend`)
3. Implement, test, commit, and push
4. Create a PR with `gh pr create --fill`
5. Auto-merge if all checks pass: `gh pr merge --squash --delete-branch`
6. Return to main for the next story

**Why squash merge?** One user story = one commit on main → clean, readable history.

## Story Completion Checklist (REQUIRED)

A story is NOT complete until ALL of these are done:
1. PR merged to main
2. Tests added and passing
3. Working tree is clean (`git status` shows no uncommitted changes)

## Stop Condition

If ALL acceptance criteria for the assigned story are met and the PR is merged, your work is done.
If you cannot complete the story, explain what went wrong clearly.

## Important

- Work on ONE story per iteration
- Commit frequently
- Keep CI green
- Follow existing code patterns
- **NEVER leave uncommitted files**
- **ALWAYS run `git status` as your FINAL action** before ending
