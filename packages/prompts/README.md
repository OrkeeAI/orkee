# Orkee Prompts

Centralized AI prompt management for Orkee. This package provides a unified source of truth for all AI prompts used throughout the application.

## Architecture

### Hybrid Approach

- **JSON Files**: System prompts (PRD, research) stored in version-controlled JSON
- **Database**: Expert personas remain in SQLite (runtime customizable)
- **Unified Access**: PromptManager provides single interface for both sources

### Directory Structure

```
/packages/prompts/
├── schema.json           # JSON schema for prompt validation
├── system/              # System-level prompts
│   ├── prd.json        # PRD generation system prompt
│   └── research.json   # Research analysis system prompt
├── prd/                # PRD section prompts
│   ├── overview.json
│   ├── features.json
│   ├── ux.json
│   ├── technical.json
│   ├── roadmap.json
│   ├── dependencies.json
│   ├── risks.json
│   ├── research.json
│   └── complete.json
└── research/           # Research & analysis prompts
    ├── competitor-analysis.json
    ├── feature-extraction.json
    ├── ui-pattern.json
    ├── gap-analysis.json
    ├── lessons-learned.json
    └── research-synthesis.json
```

## Prompt Structure

Each prompt file follows this schema:

```json
{
  "id": "unique-identifier",
  "name": "Human-readable name",
  "category": "prd|research|expert|system",
  "template": "Prompt text with {{parameter}} placeholders",
  "parameters": ["list", "of", "parameters"],
  "outputSchema": {
    "expectedField": "type description"
  },
  "metadata": {
    "version": "1.0.0",
    "lastModified": "2025-10-30",
    "description": "Brief description"
  }
}
```

## Usage

### TypeScript

```typescript
import { PromptManager } from '@orkee/prompts';

const manager = new PromptManager();

// Load a PRD prompt
const prompt = await manager.getPrompt('overview', {
  description: 'A mobile app for tracking water intake'
});

// Get system prompt
const systemPrompt = await manager.getSystemPrompt('prd');
```

### Rust

```rust
use orkee_prompts::PromptManager;

let manager = PromptManager::new()?;

// Load a PRD prompt
let prompt = manager.get_prompt("overview", &[
    ("description", "A mobile app for tracking water intake")
])?;

// Get system prompt
let system_prompt = manager.get_system_prompt("prd")?;
```

## Parameter Substitution

Templates use `{{parameterName}}` syntax. The PromptManager automatically replaces these with provided values:

```typescript
// Template: "Based on this description: {{description}}"
// Parameters: { description: "My app" }
// Result: "Based on this description: My app"
```

## Benefits

1. **Version Control**: All system prompts tracked in Git
2. **Single Source of Truth**: No duplication between TypeScript/Rust
3. **Type Safety**: JSON schema validation + TypeScript interfaces
4. **Easy Editing**: Standard JSON format, no code changes needed
5. **Flexibility**: Database-backed expert personas remain customizable

## Migration Notes

This package eliminates the previous duplication where prompts were defined in:
- `packages/dashboard/src/services/ai/prompts.ts` (TypeScript)
- `packages/ideate/src/prompts.rs` (Rust)
- `packages/ideate/src/research_prompts.rs` (Rust)

Now both TypeScript and Rust read from the same JSON source files.
