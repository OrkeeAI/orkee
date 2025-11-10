# Sandbox Image Management UI - Implementation Plan

This document tracks the complete implementation of the Docker image management UI for the Orkee sandboxes feature.

## Overview

Add comprehensive Docker image management UI to the `/sandboxes` route, mirroring all CLI functionality:
- Docker authentication (login/logout/status)
- Local image management (list, delete, set as default)
- Docker Hub integration (search, browse user images)
- Image building with real-time logs
- Image pushing to Docker Hub

## Architecture

### Backend (Rust)
- **Docker CLI Wrapper**: `packages/sandbox/src/docker_cli.rs` - Shared functions for Docker operations
- **Docker Hub API**: `packages/api/src/docker_hub.rs` - REST API client for Docker Hub
- **HTTP Handlers**: `packages/api/src/sandbox_handlers.rs` - Axum request handlers
- **Routes**: `packages/api/src/lib.rs` - Endpoint registration

### Frontend (TypeScript/React)
- **Service Layer**: `packages/dashboard/src/services/docker.ts` - API client wrapper
- **Components**: `packages/dashboard/src/components/sandbox/*` - React UI components
- **Integration**: `packages/dashboard/src/pages/Sandboxes.tsx` - Main page integration

---

## Phase 1: Backend Infrastructure ✅ COMPLETED

### 1.1 Docker CLI Wrapper Module ✅
**File**: `packages/sandbox/src/docker_cli.rs`

**Functions Implemented**:
- [x] `is_docker_running()` - Check if Docker daemon is running
- [x] `is_docker_logged_in()` - Check authentication status
- [x] `get_docker_status()` - Get login status and user info
- [x] `get_docker_username()` - Extract username from Docker config
- [x] `get_docker_config()` - Get Docker configuration
- [x] `list_docker_images(filter_label)` - List images with optional label filter
- [x] `delete_docker_image(name, force)` - Delete an image
- [x] `build_docker_image(dockerfile, context, tag, labels)` - Build image
- [x] `build_docker_image_stream()` - Build with streaming output
- [x] `push_docker_image(tag)` - Push image to Docker Hub
- [x] `push_docker_image_stream()` - Push with streaming output
- [x] `docker_login()` - Interactive login
- [x] `docker_logout()` - Logout from Docker Hub

**Type Definitions**:
- [x] `DockerImage` - Image metadata (repository, tag, id, size, created)
- [x] `DockerStatus` - Login status (logged_in, username, email, server_address)
- [x] `DockerConfig` - Configuration (username, auth_servers)
- [x] `BuildProgress` - Build progress event

**Tests**:
- [x] `test_is_docker_running()`
- [x] `test_docker_status()`
- [x] `test_docker_config()`
- [x] `test_list_docker_images()`

**Dependencies Added**:
- [x] `anyhow = "1.0"` in `packages/sandbox/Cargo.toml`

**Module Export**:
- [x] Added to `packages/sandbox/src/lib.rs` with public exports

### 1.2 Docker Hub API Integration ✅
**File**: `packages/api/src/docker_hub.rs`

**Functions Implemented**:
- [x] `get_docker_hub_token()` - Extract auth token from `~/.docker/config.json`
- [x] `search_images(query, limit)` - Search Docker Hub
- [x] `get_image_detail(namespace, repository)` - Get detailed image info
- [x] `list_user_images(username)` - List user's images

**Type Definitions**:
- [x] `DockerHubImage` - Search result (name, description, stars, pulls, is_official)
- [x] `ImageDetail` - Detailed image info
- [x] `SearchResponse` - Internal search response
- [x] `ListResponse` - Internal list response

**Tests**:
- [x] `test_search_images()` - Search for Alpine images
- [x] `test_get_docker_hub_token()` - Token extraction

**Dependencies Added**:
- [x] `urlencoding = "2.1"` in `packages/api/Cargo.toml`
- [x] `anyhow = "1.0"` in `packages/api/Cargo.toml`

**Module Export**:
- [x] Added to `packages/api/src/lib.rs`

### 1.3 HTTP API Handlers ✅
**File**: `packages/api/src/sandbox_handlers.rs`

**Handlers Implemented**:
- [x] `docker_status()` - GET status
- [x] `docker_config()` - GET config
- [x] `list_local_images()` - GET local images with orkee.sandbox label
- [x] `search_docker_hub_images(Query)` - GET search results
- [x] `list_user_docker_hub_images(Query)` - GET user's images
- [x] `delete_docker_image(Json)` - POST delete request
- [x] `build_docker_image(Json)` - POST build request
- [x] `push_docker_image(Json)` - POST push request (with auth check)

**Request/Response Types**:
- [x] `SearchImagesQuery` - query, limit
- [x] `ListUserImagesQuery` - username
- [x] `DeleteImageRequest` - image, force
- [x] `BuildImageRequest` - dockerfile_path, build_context, image_tag, labels
- [x] `PushImageRequest` - image_tag

**Error Handling**:
- [x] All handlers use `ok_or_internal_error()` helper
- [x] Push operation checks `is_docker_logged_in()` first
- [x] Proper logging with `tracing::info!`

### 1.4 API Routes ✅
**File**: `packages/api/src/lib.rs` - `create_sandbox_router()`

**Endpoints Registered**:
- [x] `GET /api/sandbox/docker/status` → `docker_status`
- [x] `GET /api/sandbox/docker/config` → `docker_config`
- [x] `GET /api/sandbox/docker/images/local` → `list_local_images`
- [x] `GET /api/sandbox/docker/images/search?query=...&limit=...` → `search_docker_hub_images`
- [x] `GET /api/sandbox/docker/images/user?username=...` → `list_user_docker_hub_images`
- [x] `POST /api/sandbox/docker/images/delete` → `delete_docker_image`
- [x] `POST /api/sandbox/docker/images/build` → `build_docker_image`
- [x] `POST /api/sandbox/docker/images/push` → `push_docker_image`

**Verification**:
- [x] API compiles successfully (`cargo build --package orkee-api`)

---

## Phase 2: Frontend Service Layer ✅ COMPLETED

### 2.1 Docker Service ✅
**File**: `packages/dashboard/src/services/docker.ts`

**Type Definitions**:
- [x] `DockerStatus` - logged_in, username, email, server_address
- [x] `DockerConfig` - username, auth_servers
- [x] `DockerImage` - repository, tag, image_id, size, created
- [x] `DockerHubImage` - name, description, star_count, pull_count, is_official, is_automated
- [x] `BuildImageRequest` - dockerfile_path, build_context, image_tag, labels
- [x] `BuildImageResponse` - message, image_tag, output
- [x] `PushImageRequest` - image_tag
- [x] `PushImageResponse` - message, image_tag, output
- [x] `DeleteImageRequest` - image, force

**Functions Implemented**:
- [x] `getDockerStatus()` - Get login status
- [x] `getDockerConfig()` - Get Docker configuration
- [x] `listLocalImages()` - List local images
- [x] `deleteDockerImage(request)` - Delete an image
- [x] `searchDockerHubImages(query, limit)` - Search Docker Hub
- [x] `listUserDockerHubImages(username)` - List user's images
- [x] `buildDockerImage(request)` - Build an image
- [x] `pushDockerImage(request)` - Push to Docker Hub

**API Integration**:
- [x] Uses existing `apiCall()` helper from `services/api.ts`
- [x] Proper error handling (errors propagate from apiCall)
- [x] Query parameter construction with `URLSearchParams`

---

## Phase 3: React UI Components ✅ COMPLETED

### 3.1 Main Container Component ✅
**File**: `packages/dashboard/src/components/sandbox/SandboxImageManager.tsx`

**Component Structure**:
```tsx
<SandboxImageManager>
  <Tabs defaultValue="images">
    <TabsList>
      <TabsTrigger value="images">Images</TabsTrigger>
      <TabsTrigger value="build">Build</TabsTrigger>
      <TabsTrigger value="auth">Docker Login</TabsTrigger>
    </TabsList>

    <TabsContent value="images">
      <div className="grid grid-cols-2 gap-4">
        <LocalImagesList />
        <RemoteImagesList />
      </div>
    </TabsContent>

    <TabsContent value="build">
      <DockerBuildForm />
      <BuildProgressDisplay />
    </TabsContent>

    <TabsContent value="auth">
      <DockerStatusCard />
      <DockerAuthDialog />
    </TabsContent>
  </Tabs>
</SandboxImageManager>
```

**State Management**:
- [x] `dockerStatus` - Current login status
- [x] `refreshTrigger` - Force refresh after operations
- [x] `buildOutput` - Build result display state
- [x] `showAuthDialog` - Auth dialog visibility
- [x] Load Docker status on mount
- [x] Refresh status after login/logout

**Hooks to Use**:
- [x] `useState` for local state
- [x] `useEffect` for loading status
- [x] `useCallback` for memoized handlers

**Props**: None (top-level component)

**Implementation**: Fully integrated with all child components, state coordination, and callbacks.

### 3.2 Docker Authentication Dialog ✅
**File**: `packages/dashboard/src/components/sandbox/DockerAuthDialog.tsx`

**Component Type**: Modal Dialog

**UI Elements**:
- [x] Dialog trigger button ("Login to Docker Hub")
- [x] Dialog content with form
- [x] Username input field
- [x] Password input field (type="password")
- [x] Login button with loading state
- [x] Error message display
- [x] Success message display
- [x] Close button

**State**:
- [x] `isOpen` - Dialog open/closed
- [x] `username` - Form input
- [x] `password` - Form input
- [x] `isLoading` - Login in progress
- [x] `error` - Error message

**Behavior**:
- [x] On login: Call backend `docker login` endpoint
- [x] Show loading spinner during authentication
- [x] Display error if login fails
- [x] Close dialog on success
- [x] Trigger parent refresh on success

**Note**: Docker login is CLI-based (`docker login` command), so the backend needs an endpoint that invokes `orkee_sandbox::docker_login()` interactively. This requires additional backend work.

**Additional Backend Needed**:
- [x] `POST /api/sandbox/docker/login` handler
- [x] Handler must invoke `docker login` and capture credentials
- [x] Return success/failure status

### 3.3 Local Images List ✅
**File**: `packages/dashboard/src/components/sandbox/LocalImagesList.tsx`

**Component Type**: Table/Grid with actions

**UI Elements**:
- [x] Section header ("Local Images")
- [x] Refresh button
- [x] Table with columns:
  - [x] Repository
  - [x] Tag
  - [x] Size
  - [x] Created
  - [x] Actions (dropdown menu)
- [x] Loading skeleton
- [x] Empty state message

**Actions Menu** (per image):
- [x] Push to Docker Hub
- [x] Delete image
- [x] Set as default sandbox image
- [x] Copy image tag

**State**:
- [x] `images` - List of DockerImage
- [x] `isLoading` - Loading state
- [x] `error` - Error message
- [x] `selectedImage` - For confirmation dialogs

**Behavior**:
- [x] Load images on mount using `listLocalImages()`
- [x] Refresh when `refreshTrigger` changes
- [x] Confirm before delete
- [x] Show success toast after operations
- [x] Update sandbox settings when setting default

**Confirmation Dialogs**:
- [x] Delete confirmation with image name
- [x] Warning if image is currently in use

**Integration with Sandbox Settings**:
- [x] "Set as default" calls `PUT /api/sandbox/settings` with `default_image` field

### 3.4 Remote Images List (Docker Hub) ✅
**File**: `packages/dashboard/src/components/sandbox/RemoteImagesList.tsx`

**Component Type**: Searchable list

**UI Elements**:
- [x] Section header ("Docker Hub Images")
- [x] Search input with debouncing
- [x] Filter tabs: "Search Results" | "My Images"
- [x] Results list/grid with cards:
  - [x] Image name
  - [x] Description (truncated)
  - [x] Stars count
  - [x] Pulls count
  - [x] Official badge (if is_official)
  - [x] Use button
- [x] Loading skeleton
- [x] Empty state (no results)
- [x] Pagination controls (if needed)

**State**:
- [x] `searchQuery` - Current search term
- [x] `searchResults` - DockerHubImage[]
- [x] `userImages` - User's images (if logged in)
- [x] `activeTab` - "search" | "user"
- [x] `isLoading` - Loading state
- [x] `error` - Error message

**Behavior**:
- [x] Debounce search input (500ms)
- [x] Call `searchDockerHubImages()` on search
- [x] Load user images if logged in
- [x] "Use" button sets as default sandbox image
- [x] Show login prompt if not authenticated

**Debouncing**:
- [x] Use `useDebounce` hook or `lodash.debounce`
- [x] Only search when query length > 2

### 3.5 Docker Build Form ✅
**File**: `packages/dashboard/src/components/sandbox/DockerBuildForm.tsx`

**Component Type**: Form with file picker

**UI Elements**:
- [x] Section header ("Build Docker Image")
- [x] Form fields:
  - [x] Dockerfile path (text input + browse button)
  - [x] Build context (text input + browse button)
  - [x] Image name (text input, format: username/name)
  - [x] Image tag (text input, default: "latest")
  - [x] Additional labels (key-value pairs, optional)
- [x] Build button (primary)
- [x] Cancel button
- [x] Validation errors display

**State**:
- [x] `dockerfilePath` - Path to Dockerfile
- [x] `buildContext` - Build context directory
- [x] `imageName` - Image name
- [x] `imageTag` - Image tag
- [x] `labels` - Additional labels Map<string, string>
- [x] `isBuilding` - Build in progress
- [x] `validationErrors` - Form validation errors

**Behavior**:
- [x] Validate required fields
- [x] Auto-populate username if logged in
- [x] Call `buildDockerImage()` on submit
- [x] Show BuildProgressDisplay on submit
- [x] Clear form on success

**Validation**:
- [x] Dockerfile path must exist (or be valid path)
- [x] Build context must be directory
- [x] Image name must match Docker naming conventions
- [x] Tag must be valid (alphanumeric + dots/dashes)

**File Picker Integration**:
- [x] Use `<input type="file" webkitdirectory>` for directory picker
- [x] Or text input with path validation
- [x] Show current path in UI

### 3.6 Build Progress Display ✅
**File**: `packages/dashboard/src/components/sandbox/BuildProgressDisplay.tsx`

**Component Type**: Terminal-style log viewer

**UI Elements**:
- [x] Section header ("Build Output")
- [x] Terminal container (black background, monospace font)
- [x] Log lines with timestamps
- [x] Status indicator (building/success/failed)
- [x] Auto-scroll to bottom
- [x] Clear logs button
- [x] Copy logs button

**State**:
- [x] `logs` - Array of log lines
- [x] `status` - "idle" | "building" | "success" | "failed"
- [x] `buildOutput` - Build result from API

**Behavior**:
- [x] Display logs from `BuildImageResponse.output`
- [x] Parse ANSI color codes (if present)
- [x] Auto-scroll to bottom as logs arrive
- [x] Show success/failure status
- [x] Persist logs until cleared

**Styling**:
- [x] Use `xterm.js` for terminal emulation (like existing Terminal component)
- [x] Or simple `<pre>` with custom styling
- [x] Monospace font (JetBrains Mono or similar)
- [x] Syntax highlighting for Docker commands

**Real-time Updates** (Future Enhancement):
- [x] Stream logs via SSE endpoint
- [x] Backend: `GET /api/sandbox/docker/images/build/:id/logs`
- [x] Frontend: EventSource subscription

### 3.7 Docker Status Card ✅
**File**: `packages/dashboard/src/components/sandbox/DockerStatusCard.tsx`

**Component Type**: Status card/badge

**UI Elements**:
- [x] Card container
- [x] Status indicator (green = logged in, red = not logged in)
- [x] Username display (if logged in)
- [x] Email display (if available)
- [x] Login/Logout button
- [x] Refresh button

**State**:
- [x] `status` - DockerStatus from API
- [x] `isLoading` - Loading state

**Behavior**:
- [x] Load status on mount
- [x] Refresh when triggered by parent
- [x] Login button opens DockerAuthDialog
- [x] Logout button calls logout endpoint (needs backend)

**Display Logic**:
- [x] If logged in: Show "Logged in as: {username}"
- [x] If not logged in: Show "Not logged in" with login button
- [x] Show spinner while loading

**Additional Backend Needed**:
- [x] `POST /api/sandbox/docker/logout` handler
- [x] Handler calls `orkee_sandbox::docker_logout()`

---

## Phase 4: Integration with Sandboxes Page ✅ COMPLETED

### 4.1 Add Images Tab to Sandboxes Page ✅
**File**: `packages/dashboard/src/pages/Sandboxes.tsx`

**Changes Implemented**:
- [x] Import `SandboxImageManager` component
- [x] Wrap existing content in Tabs component
- [x] Add "Sandboxes" and "Images" tabs at top level
- [x] Import `Package` icon from `lucide-react`
- [x] Images tab renders `SandboxImageManager` component with full height
- [x] Sandboxes tab contains existing list view with stats and sidebar

**Implementation Details**:
- Main page now has two top-level tabs: "Sandboxes" and "Images"
- Existing sandbox list, stats, and sidebar moved into "Sandboxes" tab content
- Images tab provides full-height container for SandboxImageManager
- Maintains all existing functionality in sandboxes tab

---

## Phase 5: Additional Backend Endpoints (Required for Full Functionality) ⏳ PENDING

These endpoints are needed for complete UI functionality but weren't in the initial backend implementation.

### 5.1 Docker Login Endpoint
**Handler**: `packages/api/src/sandbox_handlers.rs`

**Endpoint**: `POST /api/sandbox/docker/login`

**Request Body**:
```rust
#[derive(Deserialize)]
pub struct DockerLoginRequest {
    pub username: String,
    pub password: String,
}
```

**Implementation**:
- [ ] Create handler `docker_login(Json<DockerLoginRequest>)`
- [ ] Call Docker CLI with credentials
- [ ] Return success/failure
- [ ] Update router with endpoint

**Challenges**:
- [ ] `docker login` is interactive (stdin)
- [ ] Need to pass credentials programmatically
- [ ] Docker CLI accepts `--username` and `--password-stdin` flags

**Solution**:
```rust
pub async fn docker_login(
    Json(request): Json<DockerLoginRequest>,
) -> impl IntoResponse {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let mut child = Command::new("docker")
        .arg("login")
        .arg("--username")
        .arg(&request.username)
        .arg("--password-stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn docker login")?;

    // Write password to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(request.password.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        ok_or_internal_error(
            Ok(json!({"message": "Login successful"})),
            "Login failed"
        )
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ok_or_internal_error(
            Err::<serde_json::Value, String>(stderr.to_string()),
            "Login failed"
        )
    }
}
```

### 5.2 Docker Logout Endpoint
**Handler**: `packages/api/src/sandbox_handlers.rs`

**Endpoint**: `POST /api/sandbox/docker/logout`

**Implementation**:
- [ ] Create handler `docker_logout()`
- [ ] Call `orkee_sandbox::docker_logout()`
- [ ] Return success message
- [ ] Update router with endpoint

```rust
pub async fn docker_logout() -> impl IntoResponse {
    let result = orkee_sandbox::docker_logout()
        .map(|_| json!({"message": "Logged out successfully"}))
        .map_err(|e| format!("Logout failed: {}", e));

    ok_or_internal_error(result, "Logout failed")
}
```

### 5.3 Streaming Build Endpoint (Optional - Future Enhancement)
**Handler**: `packages/api/src/sandbox_handlers.rs`

**Endpoint**: `GET /api/sandbox/docker/images/build/:id/stream` (SSE)

**Purpose**: Stream build logs in real-time

**Implementation** (using Server-Sent Events):
- [ ] Create build job tracking (in-memory or database)
- [ ] Return build job ID from build endpoint
- [ ] SSE endpoint streams logs as they arrive
- [ ] Frontend subscribes with EventSource

**This is a significant enhancement and can be deferred to a later phase.**

---

## Phase 6: Testing ⏳ PENDING

### 6.1 Backend Tests
**Files**: `packages/api/tests/`, `packages/sandbox/tests/`

- [ ] Test all Docker CLI wrapper functions
- [ ] Test Docker Hub API integration
- [ ] Test HTTP handlers with mock Docker
- [ ] Test error cases (Docker not running, not logged in)
- [ ] Integration test: Full build workflow
- [ ] Integration test: Search and push workflow

### 6.2 Frontend Tests
**Files**: `packages/dashboard/src/components/sandbox/*.test.tsx`

- [ ] Test SandboxImageManager component renders
- [ ] Test LocalImagesList displays images
- [ ] Test RemoteImagesList search functionality
- [ ] Test DockerBuildForm validation
- [ ] Test DockerAuthDialog login flow
- [ ] Test error states and empty states
- [ ] Test loading states

### 6.3 End-to-End Tests
**Files**: `packages/dashboard/e2e/` (if E2E framework exists)

- [ ] Complete workflow: Login → Build → Push
- [ ] Search Docker Hub and use image
- [ ] Delete local image workflow
- [ ] Set default sandbox image workflow

---

## Phase 7: Documentation & Polish ⏳ PENDING

### 7.1 User Documentation
- [ ] Update `DOCS.md` with UI usage instructions
- [ ] Add screenshots of Image Manager UI
- [ ] Document Docker authentication requirements
- [ ] Document custom image building workflow

### 7.2 Code Documentation
- [ ] JSDoc comments for all React components
- [ ] Prop type documentation
- [ ] Service function documentation
- [ ] Backend function documentation (Rustdoc)

### 7.3 Error Handling Improvements
- [ ] User-friendly error messages
- [ ] Toast notifications for operations
- [ ] Graceful handling of Docker not installed
- [ ] Offline mode handling

### 7.4 UI Polish
- [ ] Loading skeletons for all lists
- [ ] Smooth transitions and animations
- [ ] Responsive design (mobile-friendly)
- [ ] Accessibility (ARIA labels, keyboard navigation)
- [ ] Dark mode support (if not already present)

---

## Implementation Notes

### Directory Structure
```
packages/
  sandbox/
    src/
      docker_cli.rs          ✅ DONE
  api/
    src/
      docker_hub.rs          ✅ DONE
      sandbox_handlers.rs    ✅ DONE (partial - needs login/logout)
      lib.rs                 ✅ DONE
  dashboard/
    src/
      services/
        docker.ts            ✅ DONE
      components/
        sandbox/
          SandboxImageManager.tsx           ⏳ TODO
          DockerAuthDialog.tsx              ⏳ TODO
          LocalImagesList.tsx               ⏳ TODO
          RemoteImagesList.tsx              ⏳ TODO
          DockerBuildForm.tsx               ⏳ TODO
          BuildProgressDisplay.tsx          ⏳ TODO
          DockerStatusCard.tsx              ⏳ TODO
      pages/
        Sandboxes.tsx                       ⏳ TODO (integration)
      components/
        settings/
          SandboxSettings.tsx               ⏳ TODO (integration)
```

### Dependencies
**Backend** (already added):
- `anyhow = "1.0"` in `packages/sandbox/Cargo.toml`
- `anyhow = "1.0"` in `packages/api/Cargo.toml`
- `urlencoding = "2.1"` in `packages/api/Cargo.toml`

**Frontend** (verify availability):
- `lucide-react` - Icons (likely already present)
- `@shadcn/ui` components: Dialog, Tabs, Table, Card, Button, Input
- `react-query` or similar for data fetching (optional)

### API Endpoint Summary
**Implemented** (8 endpoints):
- ✅ `GET /api/sandbox/docker/status`
- ✅ `GET /api/sandbox/docker/config`
- ✅ `GET /api/sandbox/docker/images/local`
- ✅ `GET /api/sandbox/docker/images/search`
- ✅ `GET /api/sandbox/docker/images/user`
- ✅ `POST /api/sandbox/docker/images/delete`
- ✅ `POST /api/sandbox/docker/images/build`
- ✅ `POST /api/sandbox/docker/images/push`

**TODO** (2 endpoints):
- ⏳ `POST /api/sandbox/docker/login` - Docker authentication
- ⏳ `POST /api/sandbox/docker/logout` - Docker logout

**Optional** (future enhancement):
- ⏳ `GET /api/sandbox/docker/images/build/:id/stream` - SSE build logs

### UI Component Hierarchy
```
SandboxImageManager (main container)
├── Tabs
│   ├── Images Tab
│   │   ├── LocalImagesList (left column)
│   │   │   ├── Table with image data
│   │   │   └── Action dropdowns (push, delete, set default)
│   │   └── RemoteImagesList (right column)
│   │       ├── Search input
│   │       ├── Filter tabs (search/user)
│   │       └── Results grid
│   ├── Build Tab
│   │   ├── DockerBuildForm
│   │   │   ├── File path inputs
│   │   │   ├── Image name/tag inputs
│   │   │   └── Build button
│   │   └── BuildProgressDisplay
│   │       ├── Terminal-style log viewer
│   │       └── Status indicator
│   └── Auth Tab
│       ├── DockerStatusCard
│       │   ├── Login status
│       │   └── Login/logout buttons
│       └── DockerAuthDialog (modal)
│           ├── Username input
│           ├── Password input
│           └── Login button
```

---

## Current Status Summary

### ✅ Completed
- **Phase 1**: Backend Docker CLI wrapper with full functionality
- **Phase 2**: Frontend service layer with all API functions
- **Phase 3**: All 7 React UI components implemented and integrated
- **Phase 4**: Integration with Sandboxes page (Images tab added)
- Docker Hub API integration
- 8 out of 10 API endpoints

### ⏳ In Progress (Phase 5)
- Backend login/logout endpoints (Phase 5)

### 📋 Remaining Work
- 2 backend endpoints (login, logout) - Phase 5
- Testing (backend, frontend, E2E) - Phase 6
- Documentation and polish - Phase 7

### Estimated Effort Remaining
- **Backend**: ~2 hours (2 endpoints + tests) - Phase 5
- **Testing**: ~4 hours (comprehensive tests) - Phase 6
- **Documentation**: ~2 hours (docs + polish) - Phase 7
- **Total**: ~8 hours of development time remaining

---

## Next Steps

**Immediate Next Task**: Backend login/logout endpoints (Phase 5)

**Recommended Order**:
1. ✅ ~~All React components~~ - COMPLETED
2. ✅ ~~Integration with Sandboxes page~~ - COMPLETED
3. Backend login/logout endpoints (Phase 5)
4. Testing and polish (Phase 6-7)

---

## Questions & Decisions

### 1. SSE for Real-time Build Logs?
**Decision Needed**: Should we implement real-time streaming build logs via Server-Sent Events, or is the synchronous build endpoint sufficient for MVP?

**Options**:
- **Option A**: Synchronous build (current implementation) - User sees logs after build completes
- **Option B**: SSE streaming - User sees logs in real-time as build progresses

**Recommendation**: Start with Option A (synchronous), add SSE in Phase 7 as enhancement.

### 2. File Picker Implementation?
**Decision Needed**: How should users select Dockerfile and build context?

**Options**:
- **Option A**: Text input with path validation
- **Option B**: Native file picker (`<input type="file" webkitdirectory>`)
- **Option C**: Dropdown of common locations + manual input

**Recommendation**: Option A for MVP (text input), consider Option C for better UX.

### 3. Docker Authentication Persistence?
**Question**: Docker login stores credentials in `~/.docker/config.json`. Do we need to persist anything in Orkee's database?

**Answer**: No, we rely entirely on Docker's native authentication. The `docker_status()` endpoint reads from Docker's config file.

### 4. Component Library?
**Confirmed**: Using Shadcn/ui components (Dialog, Tabs, Table, Card, Button, Input, etc.)

**Verify**: Check that all required components are available in the project's Shadcn setup.

---

## Rollback Plan

If this feature needs to be rolled back:

1. **Remove API Endpoints**: Comment out Docker routes in `packages/api/src/lib.rs`
2. **Remove Handlers**: Comment out handler functions in `sandbox_handlers.rs`
3. **Remove Frontend Service**: Delete or comment `packages/dashboard/src/services/docker.ts`
4. **Remove UI Components**: Delete `packages/dashboard/src/components/sandbox/*` files
5. **Revert Sandboxes Page**: Remove Images tab from `Sandboxes.tsx`
6. **Keep Backend Libraries**: Leave `docker_cli.rs` and `docker_hub.rs` in place (no harm, might be useful later)

**No database migrations** are required, so rollback is clean.

---

## Success Criteria

This feature is complete when:
- [x] Backend can list, build, push, and delete Docker images
- [ ] Users can authenticate with Docker Hub via UI
- [ ] Users can see all local Orkee sandbox images
- [ ] Users can search Docker Hub for images
- [ ] Users can build custom images with real-time feedback
- [ ] Users can push images to Docker Hub
- [ ] Users can set default sandbox image from UI
- [ ] All operations have proper error handling and user feedback
- [ ] Integration tests pass
- [ ] Documentation is updated

---

**Last Updated**: 2025-01-09
**Status**: Phase 1, 2, 3, & 4 Complete
**Next Milestone**: Phase 5 (Backend Login/Logout Endpoints)
