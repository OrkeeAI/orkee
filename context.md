# Context Tab Implementation Plan

## 📊 Overall Progress Dashboard

### Phase Completion Status

| Phase | Status | Tasks | Progress | Jump To |
|-------|--------|-------|----------|----------|
| **Phase 1** | ✅ Complete | 6/6 | ██████████ 100% | [→ Phase 1](#phase-1-basic-context-generation-week-1) |
| **Phase 2** | ✅ Complete | 4/4 | ██████████ 100% | [→ Phase 2](#phase-2-tree-sitter-integration-week-2) |
| **Phase 3** | ⏳ Pending | 0/5 | ░░░░░░░░░░ 0% | [→ Phase 3](#phase-3-openspec-integration-week-3) |
| **Phase 4** | ⏳ Pending | 0/5 | ░░░░░░░░░░ 0% | [→ Phase 4](#phase-4-advanced-features-week-4) |

**Overall Progress**: 10/20 tasks complete (50%)

**Legend**: ✅ Complete | 🔄 In Progress | ⏳ Pending | ❌ Blocked

---

## 🗺️ Table of Contents

### Introduction
- [What is this document?](#what-is-this-document)
- [Why Context Matters](#why-context-matters)
- [Architecture Overview](#architecture-overview)

### Implementation Phases
- [Phase 1: Basic Context Generation](#phase-1-basic-context-generation-week-1) (Week 1 - 7 tasks)
  - [Task 1.1: Database Migration](#task-11-create-database-migration)
  - [Task 1.2: Rust Types](#task-12-create-rust-types)
  - [Task 1.3: Context Tab Component](#task-13-create-context-tab-component)
  - [Task 1.4: Context Builder Component](#task-14-create-context-builder-component)
  - [Task 1.5: Context API Handler](#task-15-create-context-api-handler)
  - [Task 1.6: Wire Up Routes](#task-16-wire-up-routes)
  - [Phase 1 Success Criteria](#success-criteria-for-phase-1)

- [Phase 2: Tree-sitter Integration](#phase-2-tree-sitter-integration-week-2) (Week 2 - 4 tasks)
  - [Task 2.1: Add Dependencies](#task-21-add-dependencies)
  - [Task 2.2: AST Analyzer Module](#task-22-create-ast-analyzer-module)
  - [Task 2.3: AST Explorer Component](#task-23-create-ast-explorer-component)
  - [Task 2.4: Dependency Graph Builder](#task-24-create-dependency-graph-builder)
  - [Phase 2 Success Criteria](#success-criteria-for-phase-2)

- [Phase 3: OpenSpec Integration](#phase-3-openspec-integration-week-3) (Week 3 - 5 tasks)
  - [Task 3.1: Database Schema](#task-31-update-database-schema)
  - [Task 3.2: Spec-Aware Context Builder](#task-32-create-spec-aware-context-builder)
  - [Task 3.3: Context Template System](#task-33-create-context-template-system)
  - [Task 3.4: Validation Dashboard](#task-34-create-validation-dashboard)
  - [Phase 3 Success Criteria](#success-criteria-for-phase-3)

- [Phase 4: Advanced Features](#phase-4-advanced-features-week-4) (Week 4 - 5 tasks)
  - [Task 4.1: Incremental Parsing](#task-41-implement-incremental-parsing)
  - [Task 4.2: Context History](#task-42-track-context-usage)
  - [Task 4.3: Multi-language Support](#task-43-add-more-language-parsers)
  - [Phase 4 Success Criteria](#success-criteria-for-phase-4)

### Appendices
- [Glossary](#glossary)
- [Questions This Solves](#questions-this-solves)
- [Next Steps After Implementation](#next-steps-after-implementation)
- [Resources and References](#resources-and-references)

---

## 📚 How to Use This Document

### For Developers

1. **Start with the [Overall Progress Dashboard](#overall-progress-dashboard)** to see current status
2. **Navigate to your assigned phase** using the quick links
3. **Check the Phase Overview** for goals, dependencies, and timeline
4. **Review the Tasks Summary table** to understand effort estimates
5. **Work through tasks sequentially** and check them off as you complete
6. **Update the phase status** when all tasks are done

### Tracking Progress

**Task Status Icons**:
- ✅ **Complete** - Task finished and tested
- 🔄 **In Progress** - Currently being worked on
- ⏳ **Pending** - Not started yet
- ❌ **Blocked** - Waiting on dependencies

**How to Update**:
1. Check off task checkboxes as you complete them: `- [x]`
2. Update the phase overview progress: `Progress: X/Y tasks`
3. Update the overall progress dashboard table
4. Change status icons from ⏳ to 🔄 to ✅

### Navigation Tips

- Use **Back to Top** links to return to the progress dashboard
- Use **Previous/Next Phase** links to move between phases
- Use the **Table of Contents** to jump to specific sections
- Search for task numbers (e.g., "Task 2.3") to find specific tasks

---

## What is this document?

This document outlines the implementation plan for adding a "Context" tab to the Orkee project management system. If you're new to this project:

- **Orkee** is an AI agent orchestration platform for managing software projects
- **OpenSpec** is our integrated spec-driven development system (PRDs → Specs → Tasks)
- **Context Tab** will provide intelligent code context for AI agents (like Claude, GPT-4)

## Why Context Matters

When AI agents work on your codebase, they need the right context - not too much (token limits), not too little (missing information). The Context tab solves this by:

1. **Smart Selection**: Choose relevant files and code snippets for the task at hand
2. **Spec Integration**: Link code context to specifications and requirements
3. **Token Optimization**: Stay within AI model limits while maximizing useful information
4. **Code Intelligence**: Use AST parsing to understand code structure and dependencies

## Architecture Overview

```
User Interface (React)
    ↓
Context Tab Component
    ↓
API Layer (HTTP/REST)
    ↓
Rust Backend (Axum)
    ↓
Tree-sitter AST Parser
    ↓
SQLite Database
```

---

## PHASE 1: Basic Context Generation (Week 1)

[⬆️ Back to Top](#context-tab-implementation-plan) | [➡️ Next Phase](#phase-2-tree-sitter-integration-week-2)

### 🎯 Phase 1 Overview

**Goal**: Build the foundation for gathering and displaying code context

**Status**: ✅ Complete | **Progress**: 6/6 tasks (100%)

**Timeline**: Week 1 (5 days) | **Estimated Effort**: ~40 hours

**Dependencies**: 
- SQLite database setup
- Existing Orkee project structure
- React dashboard framework

### 📝 Phase 1 Tasks Summary

| Task | Status | Estimated Hours | Files |
|------|--------|----------------|-------|
| 1.1 Database Migration | ✅ Complete | 4h | `migrations/20250122000000_context.sql` |
| 1.2 Rust Types | ✅ Complete | 3h | `context/types.rs` |
| 1.3 Context Tab Component | ✅ Complete | 6h | `ContextTab.tsx` |
| 1.4 Context Builder Component | ✅ Complete | 8h | `ContextBuilder.tsx` |
| 1.5 Context API Handler | ✅ Complete | 10h | `context_handlers.rs` |
| 1.6 Wire Up Routes | ✅ Complete | 3h | `api/mod.rs` |

### ✅ Success Criteria for Phase 1

- [x] Database tables created and migrated
- [x] Basic file selection UI working
- [x] Context generation API endpoint functional
- [x] Token counting implemented
- [x] Files can be included/excluded
- [x] Generated context can be copied to clipboard
- [x] Phase 1 complete and tested

---

### Day 1-2: Database Setup

#### Task 1.1: Create Database Migration
- [x] Create migration file

Create file: `packages/projects/migrations/005_context.sql`

```sql
-- Store context configurations for projects
CREATE TABLE context_configurations (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    include_patterns TEXT DEFAULT '[]',  -- JSON array: ["src/**/*.ts", "lib/**/*.js"]
    exclude_patterns TEXT DEFAULT '[]',  -- JSON array: ["node_modules", "*.test.ts"]
    max_tokens INTEGER DEFAULT 100000,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Store generated context snapshots
CREATE TABLE context_snapshots (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    configuration_id TEXT REFERENCES context_configurations(id),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    content TEXT NOT NULL,  -- The actual generated context
    file_count INTEGER,
    total_tokens INTEGER,
    metadata TEXT,  -- JSON object with file list, generation time, etc.
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Track which files/folders users commonly include
CREATE TABLE context_usage_patterns (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    inclusion_count INTEGER DEFAULT 0,
    last_used TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(project_id, file_path)
);

CREATE INDEX idx_context_configs_project ON context_configurations(project_id);
CREATE INDEX idx_context_snapshots_project ON context_snapshots(project_id);
CREATE INDEX idx_context_patterns_project ON context_usage_patterns(project_id);
```

#### Task 1.2: Create Rust Types
- [x] Create Rust types module

Create file: `packages/projects/src/context/types.rs`

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ContextConfiguration {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub description: Option<String>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_tokens: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextGenerationRequest {
    pub project_id: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_tokens: Option<i32>,
    pub save_configuration: bool,
    pub configuration_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedContext {
    pub content: String,
    pub file_count: usize,
    pub total_tokens: usize,
    pub files_included: Vec<String>,
    pub truncated: bool,
}
```

### Day 3: Frontend Components

#### Task 1.3: Create Context Tab Component
- [x] Create ContextTab component

Create file: `packages/dashboard/src/components/ContextTab.tsx`

```tsx
import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { FileTree, Package, Settings, History } from 'lucide-react';
import { ContextBuilder } from './context/ContextBuilder';
import { ContextTemplates } from './context/ContextTemplates';
import { ContextHistory } from './context/ContextHistory';

interface ContextTabProps {
  projectId: string;
}

export function ContextTab({ projectId }: ContextTabProps) {
  const [generatedContext, setGeneratedContext] = useState<string>('');
  const [tokenCount, setTokenCount] = useState(0);

  return (
    <div className="space-y-4">
      {/* Header with quick actions */}
      <Card>
        <CardHeader>
          <CardTitle>Context Generation</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <Button variant="outline">
              <Package className="mr-2 h-4 w-4" />
              Quick Generate
            </Button>
            <Button variant="outline">
              Save Template
            </Button>
            <Button variant="outline">
              Copy to Clipboard
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Main tabbed interface */}
      <Tabs defaultValue="builder" className="space-y-4">
        <TabsList>
          <TabsTrigger value="builder">
            <FileTree className="mr-2 h-4 w-4" />
            Builder
          </TabsTrigger>
          <TabsTrigger value="templates">
            <Settings className="mr-2 h-4 w-4" />
            Templates
          </TabsTrigger>
          <TabsTrigger value="history">
            <History className="mr-2 h-4 w-4" />
            History
          </TabsTrigger>
        </TabsList>

        <TabsContent value="builder">
          <ContextBuilder
            projectId={projectId}
            onContextGenerated={(content, tokens) => {
              setGeneratedContext(content);
              setTokenCount(tokens);
            }}
          />
        </TabsContent>

        <TabsContent value="templates">
          <ContextTemplates projectId={projectId} />
        </TabsContent>

        <TabsContent value="history">
          <ContextHistory projectId={projectId} />
        </TabsContent>
      </Tabs>

      {/* Token counter */}
      {tokenCount > 0 && (
        <Card>
          <CardContent className="pt-6">
            <div className="flex justify-between items-center">
              <span>Total Tokens:</span>
              <span className="font-bold">{tokenCount.toLocaleString()}</span>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
```

#### Task 1.4: Create Context Builder Component
- [x] Create ContextBuilder component

Create file: `packages/dashboard/src/components/context/ContextBuilder.tsx`

```tsx
import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useProjectFiles } from '@/hooks/useProjectFiles';

interface ContextBuilderProps {
  projectId: string;
  onContextGenerated: (content: string, tokens: number) => void;
}

export function ContextBuilder({ projectId, onContextGenerated }: ContextBuilderProps) {
  const { data: files, isLoading } = useProjectFiles(projectId);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [excludePatterns, setExcludePatterns] = useState<string[]>([
    'node_modules',
    '*.test.ts',
    '*.spec.ts',
    '.git',
    'dist',
    'build'
  ]);

  const toggleFile = (filePath: string) => {
    const newSelected = new Set(selectedFiles);
    if (newSelected.has(filePath)) {
      newSelected.delete(filePath);
    } else {
      newSelected.add(filePath);
    }
    setSelectedFiles(newSelected);
  };

  const generateContext = async () => {
    // Call API to generate context
    const response = await fetch(`/api/projects/${projectId}/context/generate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        files: Array.from(selectedFiles),
        exclude_patterns: excludePatterns
      })
    });

    const result = await response.json();
    onContextGenerated(result.content, result.total_tokens);
  };

  return (
    <div className="grid grid-cols-2 gap-4">
      {/* File selector */}
      <Card>
        <CardHeader>
          <CardTitle>Select Files</CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-[400px]">
            {files?.map(file => (
              <div key={file.path} className="flex items-center space-x-2 py-1">
                <Checkbox
                  checked={selectedFiles.has(file.path)}
                  onCheckedChange={() => toggleFile(file.path)}
                />
                <span className="text-sm">{file.path}</span>
                <span className="text-xs text-muted-foreground">
                  ({file.size} bytes)
                </span>
              </div>
            ))}
          </ScrollArea>
        </CardContent>
      </Card>

      {/* Context preview */}
      <Card>
        <CardHeader>
          <CardTitle>Context Preview</CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-[400px]">
            <pre className="text-xs">
              {/* Preview will be shown here */}
            </pre>
          </ScrollArea>
          <Button
            className="mt-4 w-full"
            onClick={generateContext}
            disabled={selectedFiles.size === 0}
          >
            Generate Context
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
```

### Day 4-5: Backend API

#### Task 1.5: Create Context API Handler
- [x] Create context API handlers

Create file: `packages/projects/src/api/context_handlers.rs`

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::{
    context::types::{ContextGenerationRequest, GeneratedContext},
    error::ApiError,
    state::AppState,
};

pub async fn generate_context(
    Path(project_id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<ContextGenerationRequest>,
) -> Result<Json<GeneratedContext>, ApiError> {
    // 1. Get project from database
    let project = state
        .project_manager
        .get_project(&project_id)
        .await
        .map_err(|_| ApiError::NotFound)?;

    // 2. Read files from project directory
    let project_path = PathBuf::from(&project.project_root);
    let mut context_content = String::new();
    let mut file_count = 0;
    let mut files_included = Vec::new();

    // 3. Walk directory and collect files
    for pattern in &request.include_patterns {
        // Simple implementation - in production use glob crate
        let file_path = project_path.join(pattern);
        if file_path.exists() && file_path.is_file() {
            if let Ok(content) = fs::read_to_string(&file_path) {
                context_content.push_str(&format!(
                    "\n=== {} ===\n{}\n",
                    pattern, content
                ));
                file_count += 1;
                files_included.push(pattern.clone());
            }
        }
    }

    // 4. Calculate tokens (simple approximation)
    let total_tokens = context_content.len() / 4; // Rough estimate

    // 5. Save to database if requested
    if request.save_configuration {
        // Save configuration logic here
    }

    Ok(Json(GeneratedContext {
        content: context_content,
        file_count,
        total_tokens,
        files_included,
        truncated: false,
    }))
}

pub async fn list_project_files(
    Path(project_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<FileInfo>>, ApiError> {
    // Implementation to list all files in project
    // Use walkdir crate for recursive directory traversal
    Ok(Json(vec![]))
}
```

#### Task 1.6: Wire Up Routes
- [x] Wire up context routes

Add to `packages/projects/src/api/mod.rs`:

```rust
pub fn context_routes() -> Router<AppState> {
    Router::new()
        .route("/api/projects/:id/context/generate", post(generate_context))
        .route("/api/projects/:id/files", get(list_project_files))
        .route("/api/projects/:id/context/configurations", get(list_configurations))
        .route("/api/projects/:id/context/configurations", post(save_configuration))
}
```

---

## PHASE 2: Tree-sitter Integration (Week 2)

[⬆️ Back to Top](#context-tab-implementation-plan) | [⬅️ Previous Phase](#phase-1-basic-context-generation-week-1) | [➡️ Next Phase](#phase-3-openspec-integration-week-3)

### 🎯 Phase 2 Overview

**Goal**: Add code intelligence using AST parsing

**Status**: ✅ Complete | **Progress**: 4/4 tasks (100%)

**Timeline**: Week 2 (5 days) | **Estimated Effort**: ~35 hours

**Dependencies**:
- Phase 1 complete (database and basic UI)
- Tree-sitter Rust crates
- Language grammar packages

### 📝 Phase 2 Tasks Summary

| Task | Status | Estimated Hours | Files |
|------|--------|----------------|-------|
| 2.1 Add Dependencies | ✅ Complete | 2h | `Cargo.toml` |
| 2.2 AST Analyzer Module | ✅ Complete | 12h | `ast_analyzer.rs` |
| 2.3 AST Explorer Component | ✅ Complete | 10h | `ASTExplorer.tsx` |
| 2.4 Dependency Graph Builder | ✅ Complete | 8h | `dependency_graph.rs` |
| **Testing & Integration** | ⏳ Pending | 3h | Various |

### ✅ Success Criteria for Phase 2

- [x] Tree-sitter parsers integrated for TS/JS/Python/Rust
- [x] AST symbols extracted from source files
- [x] Dependency graph built from imports/exports
- [x] UI can display code structure
- [x] Users can select specific functions/classes
- [ ] Context includes only selected symbols (requires API integration)
- [ ] Phase 2 complete and tested (requires backend endpoint implementation)

### Day 1-2: Tree-sitter Setup

#### Task 2.1: Add Dependencies
- [x] Add Tree-sitter dependencies

Update `packages/projects/Cargo.toml`:

```toml
[dependencies]
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-typescript = "0.20"
tree-sitter-javascript = "0.20"
tree-sitter-python = "0.20"
```

#### Task 2.2: Create AST Analyzer Module
- [x] Create AST analyzer module

Create file: `packages/projects/src/context/ast_analyzer.rs`

```rust
use tree_sitter::{Parser, Query, QueryCursor, Node};
use std::collections::HashMap;

pub struct AstAnalyzer {
    parser: Parser,
    language: tree_sitter::Language,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line_start: usize,
    pub line_end: usize,
    pub children: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Variable,
    Import,
}

impl AstAnalyzer {
    pub fn new_typescript() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::language_typescript();
        parser.set_language(language).unwrap();
        Self { parser, language }
    }

    pub fn extract_symbols(&mut self, source_code: &str) -> Vec<Symbol> {
        let tree = self.parser.parse(source_code, None).unwrap();
        let root_node = tree.root_node();

        let mut symbols = Vec::new();
        self.walk_tree(&root_node, source_code, &mut symbols);
        symbols
    }

    fn walk_tree(&self, node: &Node, source: &str, symbols: &mut Vec<Symbol>) {
        match node.kind() {
            "function_declaration" | "method_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = &source[name_node.byte_range()];
                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Function,
                        line_start: node.start_position().row,
                        line_end: node.end_position().row,
                        children: vec![],
                    });
                }
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = &source[name_node.byte_range()];
                    symbols.push(Symbol {
                        name: name.to_string(),
                        kind: SymbolKind::Class,
                        line_start: node.start_position().row,
                        line_end: node.end_position().row,
                        children: vec![],
                    });
                }
            }
            _ => {}
        }

        // Recurse through children
        for child in node.children(&mut node.walk()) {
            self.walk_tree(&child, source, symbols);
        }
    }

    pub fn build_dependency_graph(&mut self, source_code: &str) -> HashMap<String, Vec<String>> {
        // Extract imports and build dependency map
        let mut dependencies = HashMap::new();

        // Parse imports and track what each file depends on
        // Implementation here...

        dependencies
    }
}
```

### Day 3: Frontend AST Explorer

#### Task 2.3: Create AST Explorer Component
- [x] Create AST explorer component

Create file: `packages/dashboard/src/components/context/ASTExplorer.tsx`

```tsx
import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { TreeView } from '@/components/ui/tree-view';
import { Checkbox } from '@/components/ui/checkbox';
import { Badge } from '@/components/ui/badge';
import { Function, Box, FileCode, Variable } from 'lucide-react';

interface ASTExplorerProps {
  projectId: string;
  filePath: string;
  onSymbolsSelected: (symbols: string[]) => void;
}

interface Symbol {
  name: string;
  kind: 'function' | 'class' | 'interface' | 'variable';
  lineStart: number;
  lineEnd: number;
  children: Symbol[];
}

export function ASTExplorer({ projectId, filePath, onSymbolsSelected }: ASTExplorerProps) {
  const [symbols, setSymbols] = useState<Symbol[]>([]);
  const [selectedSymbols, setSelectedSymbols] = useState<Set<string>>(new Set());

  const loadSymbols = async () => {
    const response = await fetch(
      `/api/projects/${projectId}/ast/symbols?file=${filePath}`
    );
    const data = await response.json();
    setSymbols(data.symbols);
  };

  const getSymbolIcon = (kind: string) => {
    switch (kind) {
      case 'function':
        return <Function className="h-4 w-4" />;
      case 'class':
        return <Box className="h-4 w-4" />;
      case 'interface':
        return <FileCode className="h-4 w-4" />;
      case 'variable':
        return <Variable className="h-4 w-4" />;
      default:
        return null;
    }
  };

  const renderSymbol = (symbol: Symbol, path: string = '') => {
    const fullPath = path ? `${path}.${symbol.name}` : symbol.name;

    return (
      <div key={fullPath} className="ml-4">
        <div className="flex items-center gap-2 py-1">
          <Checkbox
            checked={selectedSymbols.has(fullPath)}
            onCheckedChange={(checked) => {
              const newSelected = new Set(selectedSymbols);
              if (checked) {
                newSelected.add(fullPath);
              } else {
                newSelected.delete(fullPath);
              }
              setSelectedSymbols(newSelected);
              onSymbolsSelected(Array.from(newSelected));
            }}
          />
          {getSymbolIcon(symbol.kind)}
          <span className="text-sm">{symbol.name}</span>
          <Badge variant="secondary" className="text-xs">
            L{symbol.lineStart}-{symbol.lineEnd}
          </Badge>
        </div>
        {symbol.children.map(child => renderSymbol(child, fullPath))}
      </div>
    );
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Code Structure</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {symbols.map(symbol => renderSymbol(symbol))}
        </div>
      </CardContent>
    </Card>
  );
}
```

### Day 4-5: Dependency Analysis

#### Task 2.4: Create Dependency Graph Builder
- [x] Create dependency graph module

Create file: `packages/projects/src/context/dependency_graph.rs`

```rust
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    // Map from file path to its dependencies
    edges: HashMap<String, HashSet<String>>,
    // Map from file path to symbols it exports
    exports: HashMap<String, Vec<String>>,
    // Map from file path to symbols it imports
    imports: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            exports: HashMap::new(),
            imports: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, from: String, to: String) {
        self.edges
            .entry(from)
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    pub fn get_dependencies(&self, file: &str, depth: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((file.to_string(), 0));

        while let Some((current, current_depth)) = queue.pop_front() {
            if current_depth > depth || visited.contains(&current) {
                continue;
            }

            visited.insert(current.clone());
            result.push(current.clone());

            if let Some(deps) = self.edges.get(&current) {
                for dep in deps {
                    queue.push_back((dep.clone(), current_depth + 1));
                }
            }
        }

        result
    }

    pub fn get_dependents(&self, file: &str) -> Vec<String> {
        // Find all files that depend on this file
        let mut dependents = Vec::new();

        for (source, targets) in &self.edges {
            if targets.contains(file) {
                dependents.push(source.clone());
            }
        }

        dependents
    }
}
```

---

## PHASE 3: OpenSpec Integration (Week 3)

[⬆️ Back to Top](#context-tab-implementation-plan) | [⬅️ Previous Phase](#phase-2-tree-sitter-integration-week-2) | [➡️ Next Phase](#phase-4-advanced-features-week-4)

### 🎯 Phase 3 Overview

**Goal**: Tightly integrate context with specs, PRDs, and tasks

**Status**: ⏳ Pending | **Progress**: 0/5 tasks (0%)

**Timeline**: Week 3 (5 days) | **Estimated Effort**: ~38 hours

**Dependencies**:
- Phase 2 complete (AST parsing working)
- OpenSpec tables (from SPEC_TASK.md)
- Existing spec capabilities and requirements

### 📝 Phase 3 Tasks Summary

| Task | Status | Estimated Hours | Files |
|------|--------|----------------|-------|
| 3.1 Database Schema | ⏳ Pending | 4h | `migrations/006_context_spec_integration.sql` |
| 3.2 Spec-Aware Context Builder | ⏳ Pending | 10h | `spec_context.rs` |
| 3.3 Context Template System | ⏳ Pending | 12h | `ContextTemplates.tsx` |
| 3.4 Validation Dashboard | ⏳ Pending | 8h | `SpecValidation.tsx` |
| **Testing & Integration** | ⏳ Pending | 4h | Various |

### ✅ Success Criteria for Phase 3

- [ ] Context configs can be linked to spec capabilities
- [ ] AST symbols mapped to spec requirements
- [ ] Templates for PRD/Spec/Task contexts
- [ ] Validation shows which specs have implementing code
- [ ] Context auto-includes relevant code for specs
- [ ] WHEN/THEN scenarios validated against code
- [ ] Context tracking linked to AI usage logs (enhancement)
- [ ] Phase 3 complete and tested

### Day 1-2: Link Context to Specs

#### Task 3.1: Update Database Schema
- [ ] Create spec integration migration

Create file: `packages/projects/migrations/006_context_spec_integration.sql`

```sql
-- Link context configurations to spec capabilities
ALTER TABLE context_configurations
ADD COLUMN spec_capability_id TEXT REFERENCES spec_capabilities(id);

-- Map AST symbols to spec requirements
CREATE TABLE ast_spec_mappings (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    project_id TEXT NOT NULL REFERENCES projects(id),
    file_path TEXT NOT NULL,
    symbol_name TEXT NOT NULL,
    symbol_type TEXT NOT NULL, -- function, class, interface, etc.
    line_number INTEGER,
    requirement_id TEXT REFERENCES spec_requirements(id),
    confidence REAL DEFAULT 0.0, -- AI confidence in mapping
    verified INTEGER DEFAULT 0,  -- SQLite uses INTEGER for booleans (0/1)
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Context templates for different spec scenarios
CREATE TABLE context_templates (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),
    name TEXT NOT NULL,
    description TEXT,
    template_type TEXT NOT NULL, -- 'prd', 'capability', 'task', 'validation'
    include_patterns TEXT DEFAULT '[]',  -- JSON array
    exclude_patterns TEXT DEFAULT '[]',  -- JSON array
    ast_filters TEXT,  -- JSON object: {"include_types": ["function", "class"]}
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Track which context was used for each AI operation (optional enhancement)
ALTER TABLE ai_usage_logs
ADD COLUMN context_snapshot_id TEXT REFERENCES context_snapshots(id);

CREATE INDEX idx_ast_spec_mappings_project ON ast_spec_mappings(project_id);
CREATE INDEX idx_ast_spec_mappings_requirement ON ast_spec_mappings(requirement_id);
CREATE INDEX idx_ai_usage_logs_context ON ai_usage_logs(context_snapshot_id);
```

**Enhancement: Context Tracking for AI Operations**

The `context_snapshot_id` column added to `ai_usage_logs` enables powerful analytics:

```sql
-- Example: Find which contexts produce the best AI results
SELECT 
    cs.id,
    COUNT(*) as usage_count,
    AVG(CAST(aul.total_tokens as REAL)) as avg_tokens,
    COUNT(CASE WHEN aul.error IS NULL THEN 1 END) as successful_operations
FROM context_snapshots cs
JOIN ai_usage_logs aul ON cs.id = aul.context_snapshot_id
GROUP BY cs.id
ORDER BY successful_operations DESC;

-- Example: Token efficiency per context configuration
SELECT 
    cc.name as config_name,
    AVG(CAST(aul.total_tokens as REAL)) as avg_tokens_per_operation,
    COUNT(*) as total_operations
FROM context_configurations cc
JOIN context_snapshots cs ON cc.id = cs.configuration_id
JOIN ai_usage_logs aul ON cs.id = aul.context_snapshot_id
GROUP BY cc.id, cc.name
ORDER BY avg_tokens_per_operation ASC;
```

This enables:
- **Context Effectiveness Tracking**: Which context configurations lead to best AI results?
- **Token Efficiency Analysis**: Optimize context size vs quality tradeoff
- **Cost Attribution**: Track AI costs per context type
- **Performance Metrics**: Identify high-performing context patterns

#### Task 3.2: Create Spec-Aware Context Builder
- [ ] Create spec context builder module

Create file: `packages/projects/src/context/spec_context.rs`

```rust
use crate::openspec::types::{SpecCapability, SpecRequirement};
use crate::context::ast_analyzer::{AstAnalyzer, Symbol};

pub struct SpecContextBuilder {
    analyzer: AstAnalyzer,
}

impl SpecContextBuilder {
    pub fn new() -> Self {
        Self {
            analyzer: AstAnalyzer::new_typescript(),
        }
    }

    /// Generate context for a specific capability
    pub async fn build_capability_context(
        &mut self,
        capability: &SpecCapability,
        project_root: &str,
    ) -> String {
        let mut context = String::new();

        // 1. Add capability description
        context.push_str(&format!("## Capability: {}\n\n", capability.name));
        context.push_str(&format!("Purpose: {}\n\n", capability.purpose));

        // 2. Add requirements
        context.push_str("### Requirements:\n");
        for req in &capability.requirements {
            context.push_str(&format!("- {}: {}\n", req.name, req.content));
        }

        // 3. Find and add implementing code
        context.push_str("\n### Implementation:\n");
        let implementations = self.find_implementations(capability, project_root).await;
        for (file, symbols) in implementations {
            context.push_str(&format!("\n#### File: {}\n", file));
            for symbol in symbols {
                context.push_str(&format!("- {} (lines {}-{})\n",
                    symbol.name, symbol.line_start, symbol.line_end));
            }
        }

        context
    }

    /// Find code that implements a spec requirement
    pub async fn find_implementations(
        &mut self,
        capability: &SpecCapability,
        project_root: &str,
    ) -> HashMap<String, Vec<Symbol>> {
        // Use AST analysis to find functions/classes that match requirement names
        // This is where ML/embedding similarity could be used
        HashMap::new()
    }

    /// Validate that code matches spec scenarios
    pub async fn validate_spec_scenarios(
        &mut self,
        requirement: &SpecRequirement,
        code: &str,
    ) -> ValidationResult {
        // Parse code and check if it handles WHEN/THEN scenarios
        ValidationResult {
            passed: false,
            details: vec![],
        }
    }
}
```

### Day 3: Context Templates

#### Task 3.3: Create Context Template System
- [ ] Create context templates component

Create file: `packages/dashboard/src/components/context/ContextTemplates.tsx`

```tsx
import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { useSpecs } from '@/hooks/useSpecs';
import { usePRDs } from '@/hooks/usePRDs';

interface Template {
  id: string;
  name: string;
  description: string;
  type: 'prd' | 'capability' | 'task' | 'validation';
  includePatterns: string[];
  excludePatterns: string[];
}

const DEFAULT_TEMPLATES: Template[] = [
  {
    id: 'prd-full',
    name: 'Full PRD Context',
    description: 'Include all code related to a PRD',
    type: 'prd',
    includePatterns: ['src/**/*.ts', 'lib/**/*.ts', 'README.md'],
    excludePatterns: ['**/*.test.ts', 'node_modules'],
  },
  {
    id: 'task-focused',
    name: 'Task Implementation',
    description: 'Context for implementing a specific task',
    type: 'task',
    includePatterns: ['src/**/*.ts'],
    excludePatterns: ['**/*.test.ts', '**/*.spec.ts'],
  },
  {
    id: 'validation',
    name: 'Spec Validation',
    description: 'Context for validating spec implementation',
    type: 'validation',
    includePatterns: ['src/**/*.ts', 'tests/**/*.test.ts'],
    excludePatterns: ['node_modules'],
  },
];

export function ContextTemplates({ projectId }: { projectId: string }) {
  const [selectedTemplate, setSelectedTemplate] = useState<Template>();
  const [linkedSpec, setLinkedSpec] = useState<string>();
  const { data: specs } = useSpecs(projectId);
  const { data: prds } = usePRDs(projectId);

  const applyTemplate = async () => {
    if (!selectedTemplate) return;

    // Generate context based on template
    const response = await fetch(`/api/projects/${projectId}/context/from-template`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        template_id: selectedTemplate.id,
        spec_id: linkedSpec,
      }),
    });

    const result = await response.json();
    // Handle generated context
  };

  return (
    <div className="space-y-4">
      {/* Template selector */}
      <Card>
        <CardHeader>
          <CardTitle>Context Templates</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Select
            value={selectedTemplate?.id}
            onValueChange={(id) =>
              setSelectedTemplate(DEFAULT_TEMPLATES.find(t => t.id === id))
            }
          >
            <SelectTrigger>
              <SelectValue placeholder="Select a template" />
            </SelectTrigger>
            <SelectContent>
              {DEFAULT_TEMPLATES.map(template => (
                <SelectItem key={template.id} value={template.id}>
                  <div>
                    <div className="font-medium">{template.name}</div>
                    <div className="text-xs text-muted-foreground">
                      {template.description}
                    </div>
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          {/* Link to spec */}
          {selectedTemplate?.type === 'capability' && (
            <Select value={linkedSpec} onValueChange={setLinkedSpec}>
              <SelectTrigger>
                <SelectValue placeholder="Link to spec capability" />
              </SelectTrigger>
              <SelectContent>
                {specs?.map(spec => (
                  <SelectItem key={spec.id} value={spec.id}>
                    {spec.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}

          <Button onClick={applyTemplate} disabled={!selectedTemplate}>
            Apply Template
          </Button>
        </CardContent>
      </Card>

      {/* Template details */}
      {selectedTemplate && (
        <Card>
          <CardHeader>
            <CardTitle>Template Configuration</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <div>
              <span className="text-sm font-medium">Include Patterns:</span>
              <div className="flex flex-wrap gap-1 mt-1">
                {selectedTemplate.includePatterns.map(pattern => (
                  <Badge key={pattern} variant="secondary">
                    {pattern}
                  </Badge>
                ))}
              </div>
            </div>
            <div>
              <span className="text-sm font-medium">Exclude Patterns:</span>
              <div className="flex flex-wrap gap-1 mt-1">
                {selectedTemplate.excludePatterns.map(pattern => (
                  <Badge key={pattern} variant="outline">
                    {pattern}
                  </Badge>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
```

### Day 4-5: Code-to-Spec Validation

#### Task 3.4: Create Validation Dashboard
- [ ] Create spec validation component

Create file: `packages/dashboard/src/components/context/SpecValidation.tsx`

```tsx
import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { CheckCircle, XCircle, AlertCircle } from 'lucide-react';

interface ValidationResult {
  requirement: string;
  status: 'passed' | 'failed' | 'warning';
  details: string[];
  codeReferences: Array<{
    file: string;
    line: number;
    snippet: string;
  }>;
}

export function SpecValidation({ projectId, specId }: { projectId: string; specId: string }) {
  const [validationResults, setValidationResults] = useState<ValidationResult[]>([]);
  const [isValidating, setIsValidating] = useState(false);

  const runValidation = async () => {
    setIsValidating(true);

    const response = await fetch(
      `/api/projects/${projectId}/context/validate-spec/${specId}`,
      { method: 'POST' }
    );

    const results = await response.json();
    setValidationResults(results.validations);
    setIsValidating(false);
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'passed':
        return <CheckCircle className="h-5 w-5 text-green-500" />;
      case 'failed':
        return <XCircle className="h-5 w-5 text-red-500" />;
      case 'warning':
        return <AlertCircle className="h-5 w-5 text-yellow-500" />;
      default:
        return null;
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Spec Implementation Validation</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <Button onClick={runValidation} disabled={isValidating}>
          {isValidating ? 'Validating...' : 'Run Validation'}
        </Button>

        {validationResults.map((result, index) => (
          <Alert key={index}>
            <div className="flex items-start gap-3">
              {getStatusIcon(result.status)}
              <div className="flex-1">
                <div className="font-medium">{result.requirement}</div>
                <AlertDescription>
                  {result.details.map((detail, i) => (
                    <div key={i} className="text-sm mt-1">{detail}</div>
                  ))}
                </AlertDescription>
                {result.codeReferences.length > 0 && (
                  <div className="mt-2 space-y-1">
                    <div className="text-xs font-medium">Code References:</div>
                    {result.codeReferences.map((ref, i) => (
                      <div key={i} className="text-xs text-muted-foreground">
                        {ref.file}:{ref.line}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </Alert>
        ))}
      </CardContent>
    </Card>
  );
}
```

---

## PHASE 4: Advanced Features (Week 4)

[⬆️ Back to Top](#context-tab-implementation-plan) | [⬅️ Previous Phase](#phase-3-openspec-integration-week-3)

### 🎯 Phase 4 Overview

**Goal**: Polish, optimize, and add power features

**Status**: ⏳ Pending | **Progress**: 0/5 tasks (0%)

**Timeline**: Week 4 (5 days) | **Estimated Effort**: ~42 hours

**Dependencies**:
- Phase 3 complete (spec integration working)
- All core features functional
- Performance baseline established

### 📝 Phase 4 Tasks Summary

| Task | Status | Estimated Hours | Files |
|------|--------|----------------|-------|
| 4.1 Incremental Parsing | ⏳ Pending | 10h | `incremental_parser.rs`, `batch_processor.rs` |
| 4.2 Context History | ⏳ Pending | 12h | `ContextHistory.tsx`, `history_service.rs` |
| 4.3 Multi-language Support | ⏳ Pending | 12h | `language_support.rs` |
| **Testing & Integration** | ⏳ Pending | 4h | Various |
| **Documentation** | ⏳ Pending | 4h | README, API docs |

### ✅ Success Criteria for Phase 4

- [ ] Incremental parsing with SHA256 caching working
- [ ] Context history tracking and visualization
- [ ] Multi-language support (TS/JS/Python/Rust/Go/Java)
- [ ] Batch processing for large codebases
- [ ] Token tree visualization
- [ ] Performance optimizations complete
- [ ] All documentation updated
- [ ] Phase 4 complete and tested

### Day 1-2: Performance Optimization

#### Task 4.1: Implement Incremental Parsing
- [ ] Create incremental parser module

Create file: `packages/projects/src/context/incremental_parser.rs`

```rust
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use sha2::{Sha256, Digest};
use tree_sitter::{Parser, Tree};
use crate::context::ast_analyzer::Symbol;

pub struct IncrementalParser {
    cache: HashMap<String, ParsedFile>,
    parsers: HashMap<String, Parser>,
}

struct ParsedFile {
    content_hash: String,
    tree: Tree,
    symbols: Vec<Symbol>,
    last_modified: SystemTime,
    dependencies: Vec<String>,
}

impl IncrementalParser {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();

        // Initialize parsers for different languages
        let mut ts_parser = Parser::new();
        ts_parser.set_language(tree_sitter_typescript::language_typescript()).unwrap();
        parsers.insert("typescript".to_string(), ts_parser);

        let mut rust_parser = Parser::new();
        rust_parser.set_language(tree_sitter_rust::language()).unwrap();
        parsers.insert("rust".to_string(), rust_parser);

        Self {
            cache: HashMap::new(),
            parsers,
        }
    }

    pub fn parse_file(&mut self, path: &PathBuf) -> Result<&ParsedFile, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Calculate content hash
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        // Check cache
        let path_str = path.to_string_lossy().to_string();
        if let Some(cached) = self.cache.get(&path_str) {
            if cached.content_hash == hash {
                return Ok(cached);
            }
        }

        // Determine language from extension
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let language = match extension {
            "ts" | "tsx" => "typescript",
            "rs" => "rust",
            _ => return Err(format!("Unsupported file extension: {}", extension)),
        };

        // Parse with appropriate parser
        let parser = self.parsers.get_mut(language)
            .ok_or_else(|| format!("No parser for language: {}", language))?;

        let tree = parser.parse(&content, None)
            .ok_or_else(|| "Failed to parse file".to_string())?;

        // Extract symbols and dependencies
        let symbols = extract_symbols(&tree, &content);
        let dependencies = extract_dependencies(&tree, &content);

        // Store in cache
        let parsed = ParsedFile {
            content_hash: hash,
            tree,
            symbols,
            last_modified: SystemTime::now(),
            dependencies,
        };

        self.cache.insert(path_str.clone(), parsed);
        Ok(self.cache.get(&path_str).unwrap())
    }

    pub fn invalidate_stale_entries(&mut self, max_age_secs: u64) {
        let now = SystemTime::now();
        self.cache.retain(|_, file| {
            if let Ok(elapsed) = now.duration_since(file.last_modified) {
                elapsed.as_secs() < max_age_secs
            } else {
                false
            }
        });
    }
}

fn extract_symbols(tree: &Tree, content: &str) -> Vec<Symbol> {
    // Implementation from ast_analyzer
    vec![]
}

fn extract_dependencies(tree: &Tree, content: &str) -> Vec<String> {
    // Extract import/require statements
    vec![]
}
```

#### Task 4.1b: Add Batch Processing
- [ ] Create batch processor module

Create file: `packages/projects/src/context/batch_processor.rs`

```rust
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use std::path::PathBuf;
use super::incremental_parser::IncrementalParser;

pub struct BatchProcessor {
    parser: IncrementalParser,
    max_concurrent: usize,
}

impl BatchProcessor {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            parser: IncrementalParser::new(),
            max_concurrent,
        }
    }

    pub async fn process_directory(&mut self, dir: PathBuf) -> Vec<ProcessedFile> {
        let (tx, mut rx) = mpsc::channel(100);
        let mut tasks = JoinSet::new();

        // Walk directory and queue files
        let files = walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                let path = e.path();
                matches!(path.extension().and_then(|s| s.to_str()),
                    Some("ts") | Some("tsx") | Some("rs") | Some("js") | Some("jsx"))
            })
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>();

        // Process in batches
        for chunk in files.chunks(self.max_concurrent) {
            for path in chunk {
                let tx_clone = tx.clone();
                let path_clone = path.clone();

                tasks.spawn(async move {
                    // Process file and send result
                    let result = process_single_file(path_clone).await;
                    let _ = tx_clone.send(result).await;
                });
            }

            // Wait for batch to complete before starting next
            while let Some(result) = tasks.join_next().await {
                // Handle result
            }
        }

        drop(tx);

        // Collect all results
        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }

        results
    }
}

async fn process_single_file(path: PathBuf) -> ProcessedFile {
    // Process individual file
    ProcessedFile {
        path,
        symbols: vec![],
        dependencies: vec![],
        tokens: 0,
    }
}

pub struct ProcessedFile {
    pub path: PathBuf,
    pub symbols: Vec<String>,
    pub dependencies: Vec<String>,
    pub tokens: usize,
}
```

### Day 3: Context History

#### Task 4.2: Track Context Usage
- [ ] Create context history component

Create file: `packages/dashboard/src/components/context/ContextHistory.tsx`

```tsx
import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  BarChart, Bar, LineChart, Line, XAxis, YAxis,
  CartesianGrid, Tooltip, ResponsiveContainer
} from 'recharts';
import { Clock, TrendingUp, FileText, Hash } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

interface ContextSnapshot {
  id: string;
  createdAt: Date;
  tokenCount: number;
  fileCount: number;
  taskId?: string;
  taskSuccess?: boolean;
  configuration: {
    name: string;
    includePatterns: string[];
  };
  filesIncluded: string[];
}

interface UsageStats {
  totalContextsGenerated: number;
  averageTokens: number;
  successRate: number;
  mostUsedFiles: Array<{ file: string; count: number }>;
  tokenUsageOverTime: Array<{ date: string; tokens: number }>;
}

export function ContextHistory({ projectId }: { projectId: string }) {
  const [snapshots, setSnapshots] = useState<ContextSnapshot[]>([]);
  const [stats, setStats] = useState<UsageStats | null>(null);
  const [selectedSnapshot, setSelectedSnapshot] = useState<ContextSnapshot | null>(null);

  useEffect(() => {
    loadHistory();
    loadStats();
  }, [projectId]);

  const loadHistory = async () => {
    const response = await fetch(`/api/projects/${projectId}/context/history`);
    const data = await response.json();
    setSnapshots(data.snapshots);
  };

  const loadStats = async () => {
    const response = await fetch(`/api/projects/${projectId}/context/stats`);
    const data = await response.json();
    setStats(data);
  };

  const restoreContext = async (snapshotId: string) => {
    await fetch(`/api/projects/${projectId}/context/restore`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ snapshot_id: snapshotId }),
    });
  };

  return (
    <div className="space-y-4">
      {/* Statistics Overview */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Total Contexts</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.totalContextsGenerated}</div>
              <p className="text-xs text-muted-foreground">Generated this month</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Average Tokens</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {stats.averageTokens.toLocaleString()}
              </div>
              <p className="text-xs text-muted-foreground">Per context</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Success Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.successRate}%</div>
              <p className="text-xs text-muted-foreground">Tasks completed</p>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Token Usage Chart */}
      {stats && (
        <Card>
          <CardHeader>
            <CardTitle>Token Usage Over Time</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={stats.tokenUsageOverTime}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="date" />
                <YAxis />
                <Tooltip />
                <Line
                  type="monotone"
                  dataKey="tokens"
                  stroke="#8884d8"
                  strokeWidth={2}
                />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      )}

      {/* Most Used Files */}
      {stats && (
        <Card>
          <CardHeader>
            <CardTitle>Most Frequently Included Files</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={stats.mostUsedFiles.slice(0, 10)}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="file" angle={-45} textAnchor="end" height={100} />
                <YAxis />
                <Tooltip />
                <Bar dataKey="count" fill="#82ca9d" />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      )}

      {/* Context History List */}
      <Card>
        <CardHeader>
          <CardTitle>Recent Contexts</CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-[400px]">
            <div className="space-y-2">
              {snapshots.map(snapshot => (
                <div
                  key={snapshot.id}
                  className="flex items-center justify-between p-3 border rounded-lg hover:bg-accent cursor-pointer"
                  onClick={() => setSelectedSnapshot(snapshot)}
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">
                        {snapshot.configuration.name}
                      </span>
                      {snapshot.taskSuccess !== undefined && (
                        <Badge variant={snapshot.taskSuccess ? 'success' : 'destructive'}>
                          {snapshot.taskSuccess ? 'Success' : 'Failed'}
                        </Badge>
                      )}
                    </div>
                    <div className="flex items-center gap-4 mt-1 text-xs text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {formatDistanceToNow(snapshot.createdAt, { addSuffix: true })}
                      </span>
                      <span className="flex items-center gap-1">
                        <Hash className="h-3 w-3" />
                        {snapshot.tokenCount.toLocaleString()} tokens
                      </span>
                      <span className="flex items-center gap-1">
                        <FileText className="h-3 w-3" />
                        {snapshot.fileCount} files
                      </span>
                    </div>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      restoreContext(snapshot.id);
                    }}
                  >
                    Restore
                  </Button>
                </div>
              ))}
            </div>
          </ScrollArea>
        </CardContent>
      </Card>

      {/* Snapshot Details Modal */}
      {selectedSnapshot && (
        <Card>
          <CardHeader>
            <CardTitle>Context Details</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div>
                <span className="font-medium">Files Included:</span>
                <div className="mt-1 max-h-[200px] overflow-y-auto">
                  {selectedSnapshot.filesIncluded.map(file => (
                    <div key={file} className="text-sm text-muted-foreground">
                      {file}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
```

#### Task 4.2b: Backend Support for History
- [ ] Create history service module

Create file: `packages/projects/src/context/history_service.rs`

```rust
use sqlx::{SqlitePool, Row};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub id: String,
    pub project_id: String,
    pub content: String,
    pub token_count: i32,
    pub file_count: i32,
    pub configuration_id: Option<String>,
    pub task_id: Option<String>,
    pub task_success: Option<bool>,
    pub files_included: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextStats {
    pub total_contexts_generated: i32,
    pub average_tokens: f64,
    pub success_rate: f64,
    pub most_used_files: Vec<FileUsage>,
    pub token_usage_over_time: Vec<TokenUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUsage {
    pub file: String,
    pub count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub date: String,
    pub tokens: i32,
}

pub struct HistoryService {
    pool: SqlitePool,
}

impl HistoryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn save_snapshot(&self, snapshot: &ContextSnapshot) -> Result<String, sqlx::Error> {
        let id = generate_id();

        sqlx::query!(
            r#"
            INSERT INTO context_snapshots
            (id, project_id, content, file_count, total_tokens, metadata, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
            "#,
            id,
            snapshot.project_id,
            snapshot.content,
            snapshot.file_count,
            snapshot.token_count,
            serde_json::to_string(&snapshot.files_included).unwrap()
        )
        .execute(&self.pool)
        .await?;

        // Track file usage patterns
        for file in &snapshot.files_included {
            self.track_file_usage(&snapshot.project_id, file).await?;
        }

        Ok(id)
    }

    async fn track_file_usage(&self, project_id: &str, file_path: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO context_usage_patterns (project_id, file_path, inclusion_count, last_used)
            VALUES (?1, ?2, 1, CURRENT_TIMESTAMP)
            ON CONFLICT(project_id, file_path)
            DO UPDATE SET
                inclusion_count = inclusion_count + 1,
                last_used = CURRENT_TIMESTAMP
            "#,
            project_id,
            file_path
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_stats(&self, project_id: &str) -> Result<ContextStats, sqlx::Error> {
        // Total contexts
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM context_snapshots WHERE project_id = ?",
            project_id
        )
        .fetch_one(&self.pool)
        .await?;

        // Average tokens
        let avg_tokens = sqlx::query_scalar!(
            "SELECT AVG(total_tokens) FROM context_snapshots WHERE project_id = ?",
            project_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0.0);

        // Success rate (contexts linked to successful tasks)
        let success_rate = self.calculate_success_rate(project_id).await?;

        // Most used files
        let most_used = sqlx::query!(
            r#"
            SELECT file_path, inclusion_count
            FROM context_usage_patterns
            WHERE project_id = ?
            ORDER BY inclusion_count DESC
            LIMIT 10
            "#,
            project_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|r| FileUsage {
            file: r.file_path,
            count: r.inclusion_count,
        })
        .collect();

        // Token usage over time (last 30 days)
        let token_timeline = self.get_token_timeline(project_id).await?;

        Ok(ContextStats {
            total_contexts_generated: total as i32,
            average_tokens: avg_tokens,
            success_rate,
            most_used_files: most_used,
            token_usage_over_time: token_timeline,
        })
    }

    async fn calculate_success_rate(&self, project_id: &str) -> Result<f64, sqlx::Error> {
        // Implementation depends on task tracking
        Ok(75.0) // Placeholder
    }

    async fn get_token_timeline(&self, project_id: &str) -> Result<Vec<TokenUsage>, sqlx::Error> {
        // Get daily token usage for last 30 days
        let rows = sqlx::query!(
            r#"
            SELECT
                DATE(created_at) as date,
                SUM(total_tokens) as tokens
            FROM context_snapshots
            WHERE project_id = ?
            AND created_at > datetime('now', '-30 days')
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
            project_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| TokenUsage {
            date: r.date.unwrap_or_default(),
            tokens: r.tokens.unwrap_or(0) as i32,
        }).collect())
    }
}

fn generate_id() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
```

### Day 4-5: Multi-language Support

#### Task 4.3: Add More Language Parsers
- [ ] Create multi-language parser module

Create file: `packages/projects/src/context/language_support.rs`

```rust
use tree_sitter::{Parser, Language, Query};
use std::collections::HashMap;
use lazy_static::lazy_static;

pub struct LanguageConfig {
    pub language: Language,
    pub file_extensions: Vec<&'static str>,
    pub symbol_query: String,
    pub import_query: String,
    pub comment_delimiters: (String, String),
}

lazy_static! {
    static ref LANGUAGE_CONFIGS: HashMap<String, LanguageConfig> = {
        let mut configs = HashMap::new();

        // TypeScript/JavaScript
        configs.insert("typescript".to_string(), LanguageConfig {
            language: tree_sitter_typescript::language_typescript(),
            file_extensions: vec!["ts", "tsx"],
            symbol_query: r#"
                (function_declaration name: (identifier) @function)
                (class_declaration name: (type_identifier) @class)
                (interface_declaration name: (type_identifier) @interface)
                (variable_declarator name: (identifier) @variable)
            "#.to_string(),
            import_query: r#"
                (import_statement source: (string) @import)
                (export_statement source: (string) @export)
            "#.to_string(),
            comment_delimiters: ("//".to_string(), "/* */".to_string()),
        });

        // Python
        configs.insert("python".to_string(), LanguageConfig {
            language: tree_sitter_python::language(),
            file_extensions: vec!["py"],
            symbol_query: r#"
                (function_definition name: (identifier) @function)
                (class_definition name: (identifier) @class)
                (assignment left: (identifier) @variable)
            "#.to_string(),
            import_query: r#"
                (import_statement) @import
                (import_from_statement) @import_from
            "#.to_string(),
            comment_delimiters: ("#".to_string(), "'''".to_string()),
        });

        // Rust
        configs.insert("rust".to_string(), LanguageConfig {
            language: tree_sitter_rust::language(),
            file_extensions: vec!["rs"],
            symbol_query: r#"
                (function_item name: (identifier) @function)
                (struct_item name: (type_identifier) @struct)
                (enum_item name: (type_identifier) @enum)
                (trait_item name: (type_identifier) @trait)
                (impl_item type: (type_identifier) @impl)
            "#.to_string(),
            import_query: r#"
                (use_declaration) @use
                (extern_crate_declaration) @extern
            "#.to_string(),
            comment_delimiters: ("//".to_string(), "/* */".to_string()),
        });

        // Go
        configs.insert("go".to_string(), LanguageConfig {
            language: tree_sitter_go::language(),
            file_extensions: vec!["go"],
            symbol_query: r#"
                (function_declaration name: (identifier) @function)
                (method_declaration name: (field_identifier) @method)
                (type_declaration (type_spec name: (type_identifier) @type))
            "#.to_string(),
            import_query: r#"
                (import_declaration) @import
            "#.to_string(),
            comment_delimiters: ("//".to_string(), "/* */".to_string()),
        });

        // Java
        configs.insert("java".to_string(), LanguageConfig {
            language: tree_sitter_java::language(),
            file_extensions: vec!["java"],
            symbol_query: r#"
                (method_declaration name: (identifier) @method)
                (class_declaration name: (identifier) @class)
                (interface_declaration name: (identifier) @interface)
                (field_declaration declarator: (variable_declarator name: (identifier) @field))
            "#.to_string(),
            import_query: r#"
                (import_declaration) @import
            "#.to_string(),
            comment_delimiters: ("//".to_string(), "/* */".to_string()),
        });

        configs
    };
}

pub struct MultiLanguageParser {
    parsers: HashMap<String, Parser>,
    current_language: Option<String>,
}

impl MultiLanguageParser {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();

        for (lang_name, config) in LANGUAGE_CONFIGS.iter() {
            let mut parser = Parser::new();
            parser.set_language(config.language).unwrap();
            parsers.insert(lang_name.clone(), parser);
        }

        Self {
            parsers,
            current_language: None,
        }
    }

    pub fn detect_language(&self, file_path: &str) -> Option<String> {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())?;

        for (lang_name, config) in LANGUAGE_CONFIGS.iter() {
            if config.file_extensions.contains(&extension) {
                return Some(lang_name.clone());
            }
        }

        None
    }

    pub fn parse_file(&mut self, file_path: &str, content: &str) -> Result<ParsedFile, String> {
        let language = self.detect_language(file_path)
            .ok_or_else(|| format!("Unsupported file type: {}", file_path))?;

        let parser = self.parsers.get_mut(&language)
            .ok_or_else(|| format!("No parser for language: {}", language))?;

        let tree = parser.parse(content, None)
            .ok_or_else(|| "Failed to parse file".to_string())?;

        let config = LANGUAGE_CONFIGS.get(&language).unwrap();

        // Extract symbols using language-specific queries
        let symbols = self.extract_symbols_with_query(
            &tree,
            content,
            &config.symbol_query,
            &language
        )?;

        // Extract imports/dependencies
        let imports = self.extract_imports_with_query(
            &tree,
            content,
            &config.import_query,
            &language
        )?;

        Ok(ParsedFile {
            path: file_path.to_string(),
            language,
            symbols,
            imports,
            tree,
        })
    }

    fn extract_symbols_with_query(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        query_str: &str,
        language: &str,
    ) -> Result<Vec<Symbol>, String> {
        let config = LANGUAGE_CONFIGS.get(language).unwrap();
        let query = Query::new(config.language, query_str)
            .map_err(|e| format!("Invalid query: {:?}", e))?;

        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        let mut symbols = Vec::new();
        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let name = &source[node.byte_range()];
                let kind = match capture.index {
                    0 => SymbolKind::Function,
                    1 => SymbolKind::Class,
                    2 => SymbolKind::Interface,
                    3 => SymbolKind::Variable,
                    _ => SymbolKind::Unknown,
                };

                symbols.push(Symbol {
                    name: name.to_string(),
                    kind,
                    line_start: node.start_position().row,
                    line_end: node.end_position().row,
                    children: vec![],
                });
            }
        }

        Ok(symbols)
    }

    fn extract_imports_with_query(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        query_str: &str,
        language: &str,
    ) -> Result<Vec<String>, String> {
        let config = LANGUAGE_CONFIGS.get(language).unwrap();
        let query = Query::new(config.language, query_str)
            .map_err(|e| format!("Invalid query: {:?}", e))?;

        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        let mut imports = Vec::new();
        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let import_text = &source[node.byte_range()];
                imports.push(import_text.to_string());
            }
        }

        Ok(imports)
    }
}

#[derive(Debug)]
pub struct ParsedFile {
    pub path: String,
    pub language: String,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<String>,
    pub tree: tree_sitter::Tree,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line_start: usize,
    pub line_end: usize,
    pub children: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Struct,
    Enum,
    Trait,
    Variable,
    Method,
    Field,
    Unknown,
}

// Utility function for language-aware token counting
pub fn estimate_tokens(content: &str, language: &str) -> usize {
    // Language-specific token estimation
    // Different languages have different token densities
    let multiplier = match language {
        "python" => 0.3,     // Python is concise
        "java" => 0.4,       // Java is verbose
        "rust" => 0.35,      // Rust is moderately dense
        "typescript" => 0.35, // TS similar to Rust
        "go" => 0.3,         // Go is concise
        _ => 0.25,           // Default estimate
    };

    (content.len() as f64 * multiplier) as usize
}
```

#### Task 4.3b: Language-Aware Context Formatting

Create file: `packages/projects/src/context/formatter.rs`

```rust
use crate::context::language_support::{ParsedFile, Symbol, SymbolKind};

pub struct ContextFormatter {
    include_imports: bool,
    include_comments: bool,
    max_line_length: usize,
}

impl ContextFormatter {
    pub fn new() -> Self {
        Self {
            include_imports: true,
            include_comments: false,
            max_line_length: 120,
        }
    }

    pub fn format_context(&self, files: Vec<ParsedFile>) -> String {
        let mut output = String::new();

        // Group files by language
        let mut by_language: std::collections::HashMap<String, Vec<ParsedFile>> =
            std::collections::HashMap::new();

        for file in files {
            by_language
                .entry(file.language.clone())
                .or_insert_with(Vec::new)
                .push(file);
        }

        // Format each language group
        for (language, files) in by_language {
            output.push_str(&format!("\n## {} Files\n\n", language));

            for file in files {
                output.push_str(&self.format_file(&file));
            }
        }

        output
    }

    fn format_file(&self, file: &ParsedFile) -> String {
        let mut output = String::new();

        // File header
        output.push_str(&format!("\n### File: {}\n", file.path));
        output.push_str(&format!("Language: {}\n", file.language));

        // Symbol summary
        if !file.symbols.is_empty() {
            output.push_str("\n#### Symbols:\n");
            for symbol in &file.symbols {
                let icon = match symbol.kind {
                    SymbolKind::Function | SymbolKind::Method => "ƒ",
                    SymbolKind::Class => "C",
                    SymbolKind::Interface => "I",
                    SymbolKind::Struct => "S",
                    SymbolKind::Enum => "E",
                    SymbolKind::Trait => "T",
                    SymbolKind::Variable | SymbolKind::Field => "v",
                    _ => "?",
                };
                output.push_str(&format!(
                    "- {} {} (lines {}-{})\n",
                    icon,
                    symbol.name,
                    symbol.line_start + 1,
                    symbol.line_end + 1
                ));
            }
        }

        // Dependencies/Imports
        if self.include_imports && !file.imports.is_empty() {
            output.push_str("\n#### Dependencies:\n");
            for import in &file.imports {
                let truncated = if import.len() > self.max_line_length {
                    format!("{}...", &import[..self.max_line_length])
                } else {
                    import.clone()
                };
                output.push_str(&format!("- {}\n", truncated));
            }
        }

        output.push_str("\n---\n");
        output
    }
}
```

### Success Criteria for Phase 4

- [ ] Incremental parsing reduces regeneration time by 80%
- [ ] Context history shows usage patterns
- [ ] Support for 5+ programming languages
- [ ] Caching reduces API calls
- [ ] Analytics dashboard shows effectiveness

---

## Integration Points

### With Existing OpenSpec System

#### OpenSpec Context Bridge

Create file: `packages/projects/src/context/openspec_bridge.rs`

```rust
use crate::openspec::types::{PRD, SpecCapability, SpecRequirement, SpecTask};
use crate::context::types::ContextConfiguration;
use crate::context::spec_context::SpecContextBuilder;
use sqlx::SqlitePool;

pub struct OpenSpecContextBridge {
    pool: SqlitePool,
    spec_builder: SpecContextBuilder,
}

impl OpenSpecContextBridge {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            spec_builder: SpecContextBuilder::new(),
        }
    }

    /// Generate context for a specific PRD
    pub async fn generate_prd_context(
        &mut self,
        prd_id: &str,
        project_root: &str,
    ) -> Result<String, String> {
        // 1. Load PRD from database
        let prd = self.load_prd(prd_id).await?;

        // 2. Load all associated capabilities
        let capabilities = self.load_capabilities_for_prd(prd_id).await?;

        // 3. Generate context that includes:
        //    - PRD overview
        //    - All capability implementations
        //    - Related test files
        //    - Documentation

        let mut context = String::new();

        // PRD Header
        context.push_str(&format!(
            r#"# PRD Context: {}

## Executive Summary
{}

## Target Capabilities
"#,
            prd.title, prd.executive_summary
        ));

        // For each capability, find implementing code
        for capability in capabilities {
            let cap_context = self.spec_builder
                .build_capability_context(&capability, project_root)
                .await;
            context.push_str(&cap_context);

            // Find test files related to this capability
            let test_files = self.find_test_files(&capability, project_root).await?;
            if !test_files.is_empty() {
                context.push_str("\n### Related Tests:\n");
                for test_file in test_files {
                    context.push_str(&format!("- {}\n", test_file));
                }
            }
        }

        Ok(context)
    }

    /// Generate context for a specific task
    pub async fn generate_task_context(
        &mut self,
        task_id: &str,
        project_root: &str,
    ) -> Result<String, String> {
        // 1. Load task details
        let task = self.load_task(task_id).await?;

        // 2. Find the requirement this task implements
        let requirement = self.load_requirement(&task.requirement_id).await?;

        // 3. Find existing implementations related to this requirement
        let related_code = self.find_requirement_implementations(
            &requirement.id,
            project_root
        ).await?;

        // 4. Build focused context
        let mut context = String::new();

        context.push_str(&format!(
            r#"# Task Context: {}

## Requirement
{}

## Acceptance Criteria
{}

## Related Code
"#,
            task.title,
            requirement.content,
            task.acceptance_criteria
        ));

        for (file, symbols) in related_code {
            context.push_str(&format!("\n### {}\n", file));
            for symbol in symbols {
                context.push_str(&format!("- {} (lines {}-{})\n",
                    symbol.name, symbol.line_start, symbol.line_end));
            }
        }

        // 5. Include WHEN/THEN scenarios
        let scenarios = self.load_scenarios(&requirement.id).await?;
        if !scenarios.is_empty() {
            context.push_str("\n## Test Scenarios\n");
            for scenario in scenarios {
                context.push_str(&format!("- WHEN {} THEN {}\n",
                    scenario.when_clause, scenario.then_clause));
            }
        }

        Ok(context)
    }

    /// Validate that code matches spec requirements
    pub async fn validate_spec_coverage(
        &mut self,
        capability_id: &str,
        project_root: &str,
    ) -> SpecValidationReport {
        let capability = self.load_capability(capability_id).await.unwrap();
        let requirements = self.load_requirements(&capability_id).await.unwrap();

        let mut report = SpecValidationReport {
            capability_name: capability.name.clone(),
            total_requirements: requirements.len(),
            implemented: 0,
            partially_implemented: 0,
            not_implemented: 0,
            details: vec![],
        };

        for requirement in requirements {
            // Check if code exists for this requirement
            let implementations = self.find_requirement_implementations(
                &requirement.id,
                project_root
            ).await.unwrap();

            let status = if implementations.is_empty() {
                RequirementStatus::NotImplemented
            } else if self.validates_scenarios(&requirement, &implementations).await {
                RequirementStatus::Implemented
            } else {
                RequirementStatus::PartiallyImplemented
            };

            match status {
                RequirementStatus::Implemented => report.implemented += 1,
                RequirementStatus::PartiallyImplemented => report.partially_implemented += 1,
                RequirementStatus::NotImplemented => report.not_implemented += 1,
            }

            report.details.push(RequirementValidation {
                requirement: requirement.content.clone(),
                status,
                code_references: implementations.keys().cloned().collect(),
            });
        }

        report
    }

    // Helper methods
    async fn load_prd(&self, prd_id: &str) -> Result<PRD, String> {
        // Load from database
        Ok(PRD::default())
    }

    async fn load_capabilities_for_prd(&self, prd_id: &str) -> Result<Vec<SpecCapability>, String> {
        // Load from database
        Ok(vec![])
    }

    async fn find_test_files(&self, capability: &SpecCapability, root: &str) -> Result<Vec<String>, String> {
        // Find test files related to capability
        Ok(vec![])
    }

    async fn validates_scenarios(&self, requirement: &SpecRequirement, implementations: &HashMap<String, Vec<Symbol>>) -> bool {
        // Check if implementations handle WHEN/THEN scenarios
        false
    }
}

#[derive(Debug)]
pub struct SpecValidationReport {
    pub capability_name: String,
    pub total_requirements: usize,
    pub implemented: usize,
    pub partially_implemented: usize,
    pub not_implemented: usize,
    pub details: Vec<RequirementValidation>,
}

#[derive(Debug)]
pub struct RequirementValidation {
    pub requirement: String,
    pub status: RequirementStatus,
    pub code_references: Vec<String>,
}

#[derive(Debug)]
pub enum RequirementStatus {
    Implemented,
    PartiallyImplemented,
    NotImplemented,
}
```

#### Frontend Integration with Specs Tab

Create file: `packages/dashboard/src/hooks/useContextForSpec.ts`

```typescript
import { useQuery, useMutation } from '@tanstack/react-query';
import { api } from '@/services/api';

interface SpecContext {
  capability: string;
  requirements: Array<{
    id: string;
    content: string;
    hasImplementation: boolean;
  }>;
  suggestedFiles: string[];
  contextSize: number;
}

export function useContextForSpec(projectId: string, specId: string) {
  return useQuery({
    queryKey: ['context', 'spec', projectId, specId],
    queryFn: async () => {
      const response = await api.get(
        `/projects/${projectId}/context/spec/${specId}`
      );
      return response.data as SpecContext;
    },
  });
}

export function useGenerateContextFromPRD(projectId: string) {
  return useMutation({
    mutationFn: async (prdId: string) => {
      const response = await api.post(
        `/projects/${projectId}/context/from-prd`,
        { prd_id: prdId }
      );
      return response.data;
    },
  });
}

export function useValidateSpecImplementation(projectId: string) {
  return useMutation({
    mutationFn: async (capabilityId: string) => {
      const response = await api.post(
        `/projects/${projectId}/context/validate-spec`,
        { capability_id: capabilityId }
      );
      return response.data;
    },
  });
}
```

#### Integration in Project Detail Page

Update file: `packages/dashboard/src/pages/ProjectDetail.tsx`

Add Context tab after Tasks tab:

```tsx
import { ContextTab } from '@/components/ContextTab';

// In the tabs section:
<TabsList className="grid w-full grid-cols-8">
  <TabsTrigger value="overview">Overview</TabsTrigger>
  <TabsTrigger value="specs">Specs</TabsTrigger>
  <TabsTrigger value="tasks">Tasks</TabsTrigger>
  <TabsTrigger value="context">Context</TabsTrigger> {/* NEW TAB */}
  <TabsTrigger value="ai-usage">AI Usage</TabsTrigger>
  <TabsTrigger value="mcp-servers">MCP Servers</TabsTrigger>
  <TabsTrigger value="agents">Agents</TabsTrigger>
  <TabsTrigger value="settings">Settings</TabsTrigger>
</TabsList>

// In the tab contents:
<TabsContent value="context">
  <ContextTab projectId={projectId} />
</TabsContent>
```

#### API Routes Integration

Add to `packages/cli/src/api/mod.rs`:

```rust
use crate::context::handlers::{
    generate_context, generate_prd_context, generate_task_context,
    validate_spec, list_project_files, get_context_history,
    get_context_stats, restore_context_snapshot
};

pub fn context_routes() -> Router<AppState> {
    Router::new()
        // Basic context generation
        .route("/api/projects/:id/context/generate", post(generate_context))
        .route("/api/projects/:id/files", get(list_project_files))

        // OpenSpec integration
        .route("/api/projects/:id/context/from-prd", post(generate_prd_context))
        .route("/api/projects/:id/context/from-task", post(generate_task_context))
        .route("/api/projects/:id/context/validate-spec", post(validate_spec))

        // History and analytics
        .route("/api/projects/:id/context/history", get(get_context_history))
        .route("/api/projects/:id/context/stats", get(get_context_stats))
        .route("/api/projects/:id/context/restore", post(restore_context_snapshot))

        // Templates
        .route("/api/projects/:id/context/templates", get(list_templates))
        .route("/api/projects/:id/context/from-template", post(apply_template))
}
```

### With AI Agents

1. **Token Optimization**
   - Stay within model limits (GPT-4: 128k, Claude: 200k)
   - Prioritize most relevant code
   - Compress when needed

2. **Structured Output**
   - Consistent formatting for AI parsing
   - Clear file boundaries
   - Include metadata (language, dependencies)

---

## Testing Strategy

### Unit Tests

Create file: `packages/projects/src/context/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_ast_extraction() {
        let code = r#"
        function hello(name: string): string {
            return `Hello, ${name}!`;
        }

        class Greeter {
            greet(name: string) {
                return hello(name);
            }
        }
        "#;

        let mut parser = MultiLanguageParser::new();
        let result = parser.parse_file("test.ts", code).unwrap();

        assert_eq!(result.symbols.len(), 2);
        assert!(result.symbols.iter().any(|s| s.name == "hello"));
        assert!(result.symbols.iter().any(|s| s.name == "Greeter"));
    }

    #[test]
    fn test_dependency_graph_building() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let main_file = temp_dir.path().join("main.ts");
        fs::write(&main_file, r#"
            import { helper } from './helper';
            import { utils } from './utils';

            export function main() {
                helper();
                utils.process();
            }
        "#).unwrap();

        let helper_file = temp_dir.path().join("helper.ts");
        fs::write(&helper_file, r#"
            import { utils } from './utils';

            export function helper() {
                utils.log();
            }
        "#).unwrap();

        let utils_file = temp_dir.path().join("utils.ts");
        fs::write(&utils_file, r#"
            export const utils = {
                process: () => {},
                log: () => {}
            };
        "#).unwrap();

        let mut graph = DependencyGraph::new();

        // Build graph
        graph.add_edge("main.ts".to_string(), "helper.ts".to_string());
        graph.add_edge("main.ts".to_string(), "utils.ts".to_string());
        graph.add_edge("helper.ts".to_string(), "utils.ts".to_string());

        // Test dependency resolution
        let deps = graph.get_dependencies("main.ts", 1);
        assert_eq!(deps.len(), 3); // main.ts, helper.ts, utils.ts

        let deps = graph.get_dependencies("helper.ts", 1);
        assert_eq!(deps.len(), 2); // helper.ts, utils.ts

        // Test dependent finding
        let dependents = graph.get_dependents("utils.ts");
        assert_eq!(dependents.len(), 2); // main.ts and helper.ts depend on utils.ts
    }

    #[test]
    fn test_context_generation_with_patterns() {
        let config = ContextConfiguration {
            id: "test".to_string(),
            project_id: "project1".to_string(),
            name: "Test Config".to_string(),
            description: None,
            include_patterns: vec!["src/**/*.ts".to_string()],
            exclude_patterns: vec!["**/*.test.ts".to_string()],
            max_tokens: 10000,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Test pattern matching
        assert!(matches_pattern("src/components/Button.ts", &config.include_patterns));
        assert!(!matches_pattern("src/components/Button.test.ts", &config.exclude_patterns));
    }

    #[tokio::test]
    async fn test_incremental_parsing() {
        let mut parser = IncrementalParser::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ts");

        // First parse
        fs::write(&file_path, "function test() {}").unwrap();
        let result1 = parser.parse_file(&file_path).unwrap();
        assert_eq!(result1.symbols.len(), 1);

        // Same content - should use cache
        let result2 = parser.parse_file(&file_path).unwrap();
        assert_eq!(result1.content_hash, result2.content_hash);

        // Modified content - should reparse
        fs::write(&file_path, "function test() {} function test2() {}").unwrap();
        let result3 = parser.parse_file(&file_path).unwrap();
        assert_ne!(result1.content_hash, result3.content_hash);
        assert_eq!(result3.symbols.len(), 2);
    }

    #[test]
    fn test_token_estimation() {
        let python_code = "def hello(): return 'world'";
        let java_code = "public class Hello { public String greet() { return \"world\"; } }";
        let typescript_code = "const hello = (): string => 'world';";

        // Test language-specific token estimation
        assert!(estimate_tokens(python_code, "python") < estimate_tokens(java_code, "java"));
        assert_eq!(
            estimate_tokens(typescript_code, "typescript"),
            (typescript_code.len() as f64 * 0.35) as usize
        );
    }

    fn matches_pattern(path: &str, patterns: &[String]) -> bool {
        // Simplified pattern matching for tests
        patterns.iter().any(|p| {
            if p.contains("**") {
                let prefix = p.split("**").next().unwrap();
                path.starts_with(prefix)
            } else {
                path == p
            }
        })
    }
}
```

### Integration Tests

Create file: `packages/dashboard/src/components/context/__tests__/ContextTab.test.tsx`

```typescript
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ContextTab } from '../ContextTab';
import { server } from '@/test/server';
import { rest } from 'msw';

describe('Context Tab', () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );

  beforeEach(() => {
    queryClient.clear();
  });

  it('should generate context for selected files', async () => {
    const mockFiles = [
      { path: 'src/index.ts', size: 1024 },
      { path: 'src/utils.ts', size: 512 },
    ];

    server.use(
      rest.get('/api/projects/:id/files', (req, res, ctx) => {
        return res(ctx.json({ success: true, data: mockFiles }));
      }),
      rest.post('/api/projects/:id/context/generate', (req, res, ctx) => {
        return res(
          ctx.json({
            success: true,
            data: {
              content: '// Generated context',
              total_tokens: 500,
              file_count: 2,
            },
          })
        );
      })
    );

    render(<ContextTab projectId="test-project" />, { wrapper });

    // Wait for files to load
    await waitFor(() => {
      expect(screen.getByText('src/index.ts')).toBeInTheDocument();
    });

    // Select files
    const checkboxes = screen.getAllByRole('checkbox');
    await userEvent.click(checkboxes[0]);
    await userEvent.click(checkboxes[1]);

    // Generate context
    const generateButton = screen.getByText('Generate Context');
    await userEvent.click(generateButton);

    // Verify result
    await waitFor(() => {
      expect(screen.getByText('500')).toBeInTheDocument(); // Token count
    });
  });

  it('should link context to specs', async () => {
    const mockSpecs = [
      { id: 'spec1', name: 'Authentication', capabilities: [] },
      { id: 'spec2', name: 'User Management', capabilities: [] },
    ];

    server.use(
      rest.get('/api/projects/:id/specs', (req, res, ctx) => {
        return res(ctx.json({ success: true, data: mockSpecs }));
      }),
      rest.post('/api/projects/:id/context/from-prd', (req, res, ctx) => {
        return res(
          ctx.json({
            success: true,
            data: { content: 'PRD context generated' },
          })
        );
      })
    );

    render(<ContextTab projectId="test-project" />, { wrapper });

    // Switch to templates tab
    const templatesTab = screen.getByText('Templates');
    await userEvent.click(templatesTab);

    // Select a template
    const templateSelect = screen.getByRole('combobox');
    await userEvent.click(templateSelect);

    const prdTemplate = await screen.findByText('Full PRD Context');
    await userEvent.click(prdTemplate);

    // Apply template
    const applyButton = screen.getByText('Apply Template');
    await userEvent.click(applyButton);

    await waitFor(() => {
      expect(screen.getByText(/PRD context generated/)).toBeInTheDocument();
    });
  });

  it('should validate spec implementation', async () => {
    const mockValidation = {
      capability_name: 'Authentication',
      total_requirements: 5,
      implemented: 3,
      partially_implemented: 1,
      not_implemented: 1,
      details: [
        {
          requirement: 'User login',
          status: 'implemented',
          code_references: ['auth/login.ts'],
        },
      ],
    };

    server.use(
      rest.post('/api/projects/:id/context/validate-spec', (req, res, ctx) => {
        return res(ctx.json({ success: true, data: mockValidation }));
      })
    );

    render(<ContextTab projectId="test-project" />, { wrapper });

    // Find and click validation button
    const validateButton = screen.getByText('Run Validation');
    await userEvent.click(validateButton);

    // Check results
    await waitFor(() => {
      expect(screen.getByText('Authentication')).toBeInTheDocument();
      expect(screen.getByText('User login')).toBeInTheDocument();
      expect(screen.getByText('auth/login.ts')).toBeInTheDocument();
    });
  });

  it('should display context history with analytics', async () => {
    const mockHistory = {
      snapshots: [
        {
          id: 'snap1',
          createdAt: new Date('2024-01-01'),
          tokenCount: 5000,
          fileCount: 10,
          configuration: { name: 'Full Context' },
          filesIncluded: ['src/index.ts'],
        },
      ],
      stats: {
        totalContextsGenerated: 25,
        averageTokens: 4500,
        successRate: 85,
        mostUsedFiles: [{ file: 'src/index.ts', count: 15 }],
        tokenUsageOverTime: [{ date: '2024-01-01', tokens: 5000 }],
      },
    };

    server.use(
      rest.get('/api/projects/:id/context/history', (req, res, ctx) => {
        return res(ctx.json({ success: true, data: mockHistory }));
      })
    );

    render(<ContextTab projectId="test-project" />, { wrapper });

    // Switch to history tab
    const historyTab = screen.getByText('History');
    await userEvent.click(historyTab);

    // Verify stats display
    await waitFor(() => {
      expect(screen.getByText('25')).toBeInTheDocument(); // Total contexts
      expect(screen.getByText('85%')).toBeInTheDocument(); // Success rate
    });
  });
});
```

### End-to-End Tests

Create file: `packages/e2e/tests/context-tab.spec.ts`

```typescript
import { test, expect } from '@playwright/test';

test.describe('Context Tab E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to project detail page
    await page.goto('/projects/test-project');

    // Click on Context tab
    await page.click('text=Context');
  });

  test('full context generation workflow', async ({ page }) => {
    // Select files
    await page.click('input[type="checkbox"]:nth-of-type(1)');
    await page.click('input[type="checkbox"]:nth-of-type(2)');

    // Configure options
    await page.fill('input[placeholder="Max tokens"]', '10000');

    // Generate context
    await page.click('button:has-text("Generate Context")');

    // Wait for generation
    await expect(page.locator('.token-count')).toBeVisible();

    // Copy to clipboard
    await page.click('button:has-text("Copy to Clipboard")');

    // Verify clipboard (if possible in test environment)
    const clipboardText = await page.evaluate(() => navigator.clipboard.readText());
    expect(clipboardText).toContain('File:');
  });

  test('spec validation workflow', async ({ page }) => {
    // Navigate to validation
    await page.click('text=Run Validation');

    // Wait for results
    await expect(page.locator('.validation-results')).toBeVisible();

    // Check for validation status indicators
    await expect(page.locator('.status-passed')).toBeVisible();
    await expect(page.locator('.status-failed')).toBeVisible();

    // Expand details
    await page.click('.validation-details-toggle');
    await expect(page.locator('.code-reference')).toBeVisible();
  });
});
```

---

## Deployment Checklist

### Backend
- [ ] Run database migrations
- [ ] Install tree-sitter dependencies
- [ ] Configure language parsers
- [ ] Set up caching (Redis optional)

### Frontend
- [ ] Install npm dependencies
- [ ] Update API endpoints
- [ ] Configure token limits per model
- [ ] Test clipboard functionality

### Documentation
- [ ] Update API documentation
- [ ] Add user guide for Context tab
- [ ] Document template creation
- [ ] Add troubleshooting guide

---

## Success Metrics

### Quantitative
- Context generation time < 2 seconds for average project
- Token optimization saves 40% without losing critical info
- 90% of generated contexts fit within model limits
- AST parsing accuracy > 95%

### Qualitative
- Developers report 50% faster task completion
- AI agents produce more accurate code
- Spec validation catches 80% of implementation gaps
- Context templates cover 90% of use cases

---

## Risks and Mitigations

### Risk 1: Large Codebases
**Problem**: Projects with 10k+ files slow down parsing
**Mitigation**:
- Implement incremental parsing
- Use file watching for cache invalidation
- Limit initial scan to modified files

### Risk 2: Token Limits
**Problem**: Context exceeds AI model limits
**Mitigation**:
- Intelligent truncation algorithms
- Summary generation for large sections
- Multiple context "chunks" if needed

### Risk 3: AST Parser Limitations
**Problem**: Some languages/frameworks not supported
**Mitigation**:
- Fallback to text-based extraction
- Plugin system for custom parsers
- Community-contributed language support

---

## Glossary for New Developers

**AST (Abstract Syntax Tree)**: A tree representation of source code structure. Lets us understand code semantically, not just as text.

**Context**: The relevant code and documentation an AI needs to understand a task. Think of it as "everything the AI needs to know."

**OpenSpec**: Our system for managing specifications. Flow: PRD (Product Requirements) → Specs (Detailed Requirements) → Tasks (Implementation).

**Token**: Unit of text for AI models. Roughly 4 characters = 1 token. Models have limits (GPT-4: 128k tokens).

**Tree-sitter**: A parser generator tool that builds fast, incremental parsers for programming languages. It creates ASTs we can analyze.

**Spec Capability**: A functional area in OpenSpec (like "authentication" or "payment processing").

**WHEN/THEN Scenarios**: Test scenarios in specs. "WHEN user logs in THEN show dashboard."

---

## Questions This Solves

1. **"How do I give AI the right context?"** → Context tab handles file selection and formatting
2. **"My prompts are too long!"** → Token optimization and smart truncation
3. **"Does our code match the spec?"** → Validation dashboard shows gaps
4. **"What code implements this feature?"** → AST mapping links code to specs
5. **"How do I generate context for a task?"** → Templates provide one-click context

---

## Next Steps After Implementation

1. **ML-based code-to-spec mapping** using embeddings
2. **Context quality scoring** based on task success
3. **Auto-context generation** for every new task
4. **Cross-project context** sharing for similar features
5. **Context versioning** tied to git commits

---

## Resources and References

- [Tree-sitter Documentation](https://tree-sitter.github.io)
- [OpenSpec Methodology](https://github.com/Fission-AI/OpenSpec)
- [Token Counting Guide](https://platform.openai.com/tokenizer)
- [AST Explorer](https://astexplorer.net) - Visualize ASTs online
- [Orkee OpenSpec Integration](SPEC_TASK.md) - Our existing spec system

---

*Last Updated: 2025-10-21*
*Status: Ready for Implementation*
*Estimated Timeline: 4 weeks*
*Team Size: 1-2 developers*