# Docker Authentication Implementation Plan

## 📋 Execution Overview

**Goal**: Enable Docker authentication for pulling private sandbox images

**Phases**:
- ✅ **Phase 0**: Quick Fix (5 min) - Get sandboxes working with public images
- 🔨 **Phase 1**: Auth System Extension (1-2 hours) - Add Docker to OAuth providers
- 🔨 **Phase 2**: Docker Provider Integration (2-3 hours) - Add authenticated image pulls
- 🔨 **Phase 3**: Configuration CLI (1 hour) - Add sandbox config commands
- 🔨 **Phase 4**: Testing (2 hours) - Comprehensive test coverage
- 📝 **Phase 5**: Documentation (1 hour) - User guides and examples

**Total Estimated Time**: 7-9 hours

---

## Phase 0: Quick Fix (IMMEDIATE - 5 minutes)

**Goal**: Make sandboxes work immediately by defaulting to a public image

### ✅ Done When
- Sandboxes can be created without specifying an image
- Default image is `alpine:latest` (public, always available)
- No more 404 errors for `orkee/sandbox:latest`

### 🧪 Test First (TDD)
```rust
// File: packages/api/tests/sandbox_handlers_test.rs
#[tokio::test]
async fn test_sandbox_creation_without_image_uses_default() {
    let db = setup_test_db().await;

    let request = CreateSandboxRequestBody {
        name: "test".to_string(),
        image: None,  // No image specified
        ..Default::default()
    };

    let response = create_sandbox(State(db), Json(request)).await;

    // Should succeed with alpine:latest
    assert!(response.is_ok());
    let sandbox = response.unwrap();
    assert_eq!(sandbox.config.image, Some("alpine:latest".to_string()));
}
```

### 📝 Implementation

**File**: `packages/api/src/sandbox_handlers.rs`

**Location**: Line ~226-232 in `create_sandbox()` function

**Change**:
```rust
// BEFORE (line 232):
image: body.image,

// AFTER:
image: body.image.or(Some("alpine:latest".to_string())),
```

**Complete Context**:
```rust
// Build the sandbox request
let request = CreateSandboxRequest {
    name: body.name,
    provider: body.provider.unwrap_or(settings.default_provider),
    agent_id: body.agent_id.unwrap_or_else(|| "claude".to_string()),
    user_id: "default-user".to_string(),
    project_id: None,
    image: body.image.or(Some("alpine:latest".to_string())),  // ✅ CHANGE THIS LINE
    cpu_cores: body.cpu_cores,
    memory_mb: body.memory_mb,
    storage_gb: body.disk_gb,
    // ... rest stays the same
};
```

### ✅ Verify

**Test manually**:
```bash
# Start server
ORKEE_DEV_MODE=true cargo run --bin orkee -- dashboard --api-port 4001

# Create sandbox without image
curl -X POST http://localhost:4001/api/sandboxes \
  -H "Content-Type: application/json" \
  -d '{"name":"test-sandbox"}'

# Should succeed and pull alpine:latest
```

**Expected result**:
```json
{
  "success": true,
  "data": {
    "id": "sbx_...",
    "name": "test-sandbox",
    "status": "running",
    // ... alpine:latest was pulled and container created
  }
}
```

**Commit**:
```bash
git add packages/api/src/sandbox_handlers.rs
git commit -m "fix(sandbox): default to alpine:latest when no image specified

- Prevents 404 errors when pulling non-existent orkee/sandbox:latest
- Uses public alpine:latest as fallback
- Sandboxes now work out of the box without configuration"
```

---

## Phase 1: Extend Authentication System (1-2 hours)

**Goal**: Add Docker as an OAuth provider so users can run `orkee auth login docker`

### ✅ Done When
- `orkee auth login docker` command exists
- Docker credentials are stored encrypted in database
- Can retrieve and decrypt credentials for Docker provider

### 🧪 Tests First (TDD)

Create test file: `packages/auth/tests/docker_provider_tests.rs`

```rust
use orkee_auth::OAuthProvider;
use std::str::FromStr;

#[test]
fn test_docker_provider_exists() {
    // Docker should be in the list of all providers
    assert!(OAuthProvider::all().contains(&OAuthProvider::Docker));
}

#[test]
fn test_docker_provider_from_string() {
    // Should parse "docker" string
    let provider = OAuthProvider::from_str("docker").unwrap();
    assert_eq!(provider, OAuthProvider::Docker);
}

#[test]
fn test_docker_provider_display() {
    // Should format as "docker"
    assert_eq!(format!("{}", OAuthProvider::Docker), "docker");
}

#[tokio::test]
async fn test_docker_login_stores_credentials() {
    let db = setup_test_db().await;

    // Simulate login
    let username = "testuser";
    let password = "testtoken";
    let registry = "docker.io";

    // This function doesn't exist yet - we'll create it
    import_docker_credentials_for_test(&db, username, password, registry)
        .await
        .unwrap();

    // Verify stored in database
    let tokens = db.token_storage.list_tokens().await.unwrap();
    let docker_token = tokens.iter()
        .find(|t| t.provider.as_deref() == Some("docker"))
        .expect("Docker token should be stored");

    // Verify fields
    assert_eq!(docker_token.account_email.as_deref(), Some(username));

    // Verify encrypted (starts with "encrypted:" prefix)
    assert!(docker_token.token.contains("encrypted:") || docker_token.token.len() > 50);

    // Verify metadata has registry
    let metadata: serde_json::Value = serde_json::from_str(
        docker_token.metadata.as_ref().unwrap()
    ).unwrap();
    assert_eq!(metadata["registry"], registry);
}
```

**Run tests (should fail)**:
```bash
cargo test -p orkee-auth docker_provider
# Expected: All tests fail - Docker provider doesn't exist yet
```

### 📝 Implementation

#### Step 1.1: Add Docker to OAuthProvider Enum

**File**: `packages/auth/src/oauth/provider.rs`

**Add Docker variant**:
```rust
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Claude,
    OpenAI,
    Google,
    XAI,
    Docker,  // ✅ ADD THIS LINE
}

impl OAuthProvider {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Claude,
            Self::OpenAI,
            Self::Google,
            Self::XAI,
            Self::Docker,  // ✅ ADD THIS LINE
        ]
    }
}

impl FromStr for OAuthProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "openai" => Ok(Self::OpenAI),
            "google" => Ok(Self::Google),
            "xai" => Ok(Self::XAI),
            "docker" => Ok(Self::Docker),  // ✅ ADD THIS LINE
            _ => Err(format!("Unknown OAuth provider: {}", s)),
        }
    }
}

impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::OpenAI => write!(f, "openai"),
            Self::Google => write!(f, "google"),
            Self::XAI => write!(f, "xai"),
            Self::Docker => write!(f, "docker"),  // ✅ ADD THIS LINE
        }
    }
}
```

#### Step 1.2: Add Docker Login Command

**File**: `packages/cli/src/bin/cli/auth.rs`

**Add imports** (at top of file):
```rust
use serde_json::json;
```

**Add function** (before `login_command`):
```rust
async fn import_docker_credentials(db: &DbState) -> anyhow::Result<()> {
    use dialoguer::{Input, Password};

    println!("🐋 Docker Hub Authentication");
    println!("Create a Personal Access Token at: https://hub.docker.com/settings/security\n");

    // Prompt for registry
    let registry: String = Input::new()
        .with_prompt("Registry URL")
        .default("docker.io".to_string())
        .interact_text()?;

    // Prompt for username
    let username: String = Input::new()
        .with_prompt("Docker Hub username")
        .interact_text()?;

    // Prompt for password/token (hidden)
    let password = Password::new()
        .with_prompt("Password or Personal Access Token")
        .interact()?;

    // Create metadata with registry info
    let metadata = json!({
        "registry": registry
    });

    // Import token using OAuthManager
    db.oauth_manager
        .import_token(
            "default-user",
            OAuthProvider::Docker,
            &format!("Docker Hub ({})", registry),
            &password,
            Some(&username),  // Store username in account_email
            None,             // No scope for now
            Some(&metadata.to_string()),
        )
        .await?;

    println!("✅ Docker credentials saved successfully!");
    println!("   Registry: {}", registry);
    println!("   Username: {}", username);

    Ok(())
}
```

**Update `login_command`** (add Docker case):
```rust
pub async fn login_command(provider: &str) -> anyhow::Result<()> {
    let db = DbState::init().await?;

    let oauth_provider = OAuthProvider::from_str(provider)?;

    match oauth_provider {
        OAuthProvider::Claude => import_claude_token().await?,
        OAuthProvider::OpenAI => import_openai_token().await?,
        OAuthProvider::Google => import_google_token().await?,
        OAuthProvider::XAI => import_xai_token().await?,
        OAuthProvider::Docker => import_docker_credentials(&db).await?,  // ✅ ADD THIS LINE
    }

    Ok(())
}
```

#### Step 1.3: Update Token Manager Validation

**File**: `packages/auth/src/oauth/manager.rs`

**Find `import_token` method** (around line 54-80) and add Docker case:

```rust
pub async fn import_token(
    &self,
    user_id: &str,
    provider: OAuthProvider,
    name: &str,
    token: &str,
    account_email: Option<&str>,
    scope: Option<&str>,
    metadata: Option<&str>,
) -> Result<TokenGeneration, AuthError> {
    // Validate token format based on provider
    match provider {
        OAuthProvider::Claude => {
            // Existing validation
        }
        OAuthProvider::OpenAI => {
            // Existing validation
        }
        OAuthProvider::Google => {
            // Existing validation
        }
        OAuthProvider::XAI => {
            // Existing validation
        }
        OAuthProvider::Docker => {  // ✅ ADD THIS BLOCK
            // Docker accepts any non-empty token
            // Can be password or Personal Access Token
            if token.trim().is_empty() {
                return Err(AuthError::InvalidToken(
                    "Docker password/token cannot be empty".to_string()
                ));
            }
            info!("Importing Docker Hub credentials");
        }
    }

    // ... rest of method stays the same
}
```

### ✅ Verify

**Run tests** (should now pass):
```bash
cargo test -p orkee-auth docker_provider
# Expected: All tests pass
```

**Test CLI command**:
```bash
cargo run --bin orkee -- auth login docker
# Follow prompts:
# Registry: docker.io
# Username: your-dockerhub-username
# Password: your-token-or-password
# Expected: "✅ Docker credentials saved successfully!"
```

**Verify in database**:
```bash
sqlite3 ~/.orkee/orkee.db "SELECT provider, account_email, name FROM oauth_tokens WHERE provider='docker'"
# Expected: Shows your Docker credentials (password is encrypted)
```

**Commit**:
```bash
git add packages/auth/src/oauth/provider.rs \
        packages/cli/src/bin/cli/auth.rs \
        packages/auth/src/oauth/manager.rs \
        packages/auth/tests/docker_provider_tests.rs

git commit -m "feat(auth): add Docker Hub authentication support

- Add Docker to OAuthProvider enum
- Implement 'orkee auth login docker' command
- Store credentials encrypted with registry metadata
- Support both passwords and Personal Access Tokens

Tests:
- Provider enum parsing and display
- Credential storage and encryption
- Metadata storage for registry URL"
```

---

## Phase 2: Docker Provider Integration (2-3 hours)

**Goal**: Make DockerProvider use stored credentials when pulling images

### ✅ Done When
- DockerProvider can retrieve Docker credentials from database
- Credentials are decrypted before use
- Image pulls use authentication when credentials available
- Public images still work without authentication (fallback)

### 🧪 Tests First (TDD)

Create test file: `packages/sandbox/tests/docker_auth_tests.rs`

```rust
use orkee_sandbox::DockerProvider;
use orkee_security::ApiKeyEncryption;

#[tokio::test]
async fn test_pull_public_image_without_auth() {
    // Should work without credentials
    let provider = DockerProvider::new().unwrap();
    let result = provider.pull_image("alpine:latest").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pull_with_valid_credentials() {
    let (token_storage, encryption) = setup_test_auth().await;

    // Store valid test credentials
    store_docker_credentials(
        &token_storage,
        &encryption,
        "testuser",
        "testtoken",
        "docker.io"
    ).await.unwrap();

    // Create provider with auth
    let provider = DockerProvider::with_auth(
        Some(Arc::new(token_storage)),
        Some(Arc::new(encryption))
    ).unwrap();

    // Should use auth for private images
    // Note: This will fail if testuser/testtoken are invalid
    // In real test, use mock or test account
    let result = provider.pull_image("testuser/private-image:latest").await;

    // Verify auth was attempted (even if it fails due to fake creds)
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(!err_msg.contains("No credentials")); // Auth was tried
}

#[test]
fn test_registry_parsing() {
    let provider = DockerProvider::new().unwrap();

    assert_eq!(provider.parse_registry("alpine:latest"), "docker.io");
    assert_eq!(provider.parse_registry("user/image:tag"), "docker.io");
    assert_eq!(provider.parse_registry("gcr.io/project/image"), "gcr.io");
    assert_eq!(
        provider.parse_registry("123.dkr.ecr.us-east-1.amazonaws.com/img"),
        "123.dkr.ecr.us-east-1.amazonaws.com"
    );
}

#[tokio::test]
async fn test_credential_decryption() {
    let encryption = ApiKeyEncryption::from_machine_id().unwrap();
    let original = "my-docker-token";

    // Encrypt
    let encrypted = encryption.encrypt(original).unwrap();

    // Decrypt
    let decrypted = encryption.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, original);
}
```

**Run tests** (should fail):
```bash
cargo test -p orkee-sandbox docker_auth
# Expected: Tests fail - methods don't exist yet
```

### 📝 Implementation

#### Step 2.1: Update DockerProvider Struct

**File**: `packages/sandbox/src/providers/docker.rs`

**Add imports** (at top of file):
```rust
use bollard::auth::DockerCredentials;
use orkee_security::{ApiKeyEncryption, TokenStorage};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json;
```

**Update struct** (find `pub struct DockerProvider`):
```rust
pub struct DockerProvider {
    client: Docker,
    label_prefix: String,
    token_storage: Option<Arc<TokenStorage>>,    // ✅ ADD THIS
    encryption: Option<Arc<ApiKeyEncryption>>,   // ✅ ADD THIS
}
```

**Update constructors**:
```rust
impl DockerProvider {
    // Keep existing new() for backward compatibility
    pub fn new() -> Result<Self, ProviderError> {
        Self::with_auth(None, None)
    }

    // ✅ ADD THIS: New constructor with auth support
    pub fn with_auth(
        token_storage: Option<Arc<TokenStorage>>,
        encryption: Option<Arc<ApiKeyEncryption>>,
    ) -> Result<Self, ProviderError> {
        let client = Docker::connect_with_local_defaults()
            .map_err(|e| ProviderError::Connection(format!("Failed to connect to Docker: {}", e)))?;

        Ok(Self {
            client,
            label_prefix: "orkee".to_string(),
            token_storage,
            encryption,
        })
    }
}
```

#### Step 2.2: Add Helper Methods

**Add to `impl DockerProvider`**:

```rust
    // ✅ ADD THIS: Parse registry from image name
    fn parse_registry(&self, image: &str) -> String {
        if image.contains('/') {
            let parts: Vec<&str> = image.split('/').collect();
            // Check if first part looks like a registry (has '.' or ':')
            if parts[0].contains('.') || parts[0].contains(':') {
                return parts[0].to_string();
            }
        }
        // Default to Docker Hub
        "docker.io".to_string()
    }

    // ✅ ADD THIS: Get credentials for a registry
    async fn get_docker_credentials(
        &self,
        registry: &str,
    ) -> Result<Option<DockerCredentials>, ProviderError> {
        // Check if we have auth components
        let (token_storage, encryption) = match (&self.token_storage, &self.encryption) {
            (Some(ts), Some(enc)) => (ts, enc),
            _ => return Ok(None), // No auth available
        };

        // Get all tokens
        let tokens = token_storage
            .list_tokens()
            .await
            .map_err(|e| ProviderError::Configuration(format!("Failed to list tokens: {}", e)))?;

        // Find Docker token for this registry
        let docker_token = tokens.iter().find(|t| {
            // Check if it's a Docker token
            if t.provider.as_deref() != Some("docker") {
                return false;
            }

            // Check if registry matches
            if let Some(metadata_str) = &t.metadata {
                if let Ok(metadata) = serde_json::from_str::<HashMap<String, String>>(metadata_str) {
                    if let Some(token_registry) = metadata.get("registry") {
                        return token_registry == registry;
                    }
                }
            }

            false
        });

        let token = match docker_token {
            Some(t) => t,
            None => return Ok(None), // No credentials for this registry
        };

        // Decrypt the password
        let decrypted_password = encryption
            .decrypt(&token.token)
            .map_err(|e| {
                ProviderError::Configuration(format!(
                    "Failed to decrypt Docker credentials. \
                     This may happen if encryption password changed. \
                     Please run 'orkee auth login docker' again. Error: {}",
                    e
                ))
            })?;

        // Build Docker credentials
        Ok(Some(DockerCredentials {
            username: token.account_email.clone(),
            password: Some(decrypted_password),
            serveraddress: Some(registry.to_string()),
            ..Default::default()
        }))
    }
```

#### Step 2.3: Update pull_image Method

**Find existing `pull_image` method** (around line 608-640) and **replace** with:

```rust
    async fn pull_image(&self, image: &str) -> Result<(), ProviderError> {
        use futures_util::stream::StreamExt;

        info!("Pulling Docker image: {}", image);

        // Parse registry from image name
        let registry = self.parse_registry(image);
        debug!("Detected registry: {} for image: {}", registry, image);

        // Try to get credentials for this registry
        let credentials = self.get_docker_credentials(&registry).await?;

        if credentials.is_some() {
            info!("Using authenticated pull for image: {}", image);
        } else {
            info!("Using unauthenticated pull for image: {}", image);
        }

        // Create image options
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        // Pull with or without authentication
        let mut stream = self.client.create_image(Some(options), None, credentials);

        // Stream progress
        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(status) = info.status {
                        debug!("Pull status: {}", status);
                    }
                    if let Some(error) = info.error {
                        return Err(ProviderError::Image(format!("Docker pull error: {}", error)));
                    }
                }
                Err(e) => {
                    // Parse error for better messages
                    use bollard::errors::Error as BollardError;
                    let error_msg = match &e {
                        BollardError::DockerResponseServerError { status_code: 401, .. } => {
                            format!(
                                "Authentication failed for image '{}'. \
                                 Please run 'orkee auth login docker' to set credentials.",
                                image
                            )
                        }
                        BollardError::DockerResponseServerError { status_code: 404, .. } => {
                            format!(
                                "Image '{}' not found. Please check:\n\
                                 - Image name spelling\n\
                                 - Image exists on registry\n\
                                 - You have access to this image",
                                image
                            )
                        }
                        BollardError::DockerResponseServerError { status_code: 429, .. } => {
                            format!(
                                "Docker Hub rate limit exceeded. \
                                 Authenticate to get higher limits: 'orkee auth login docker'"
                            )
                        }
                        _ => format!("Failed to pull image '{}': {}", image, e),
                    };

                    return Err(ProviderError::Image(error_msg));
                }
            }
        }

        info!("Successfully pulled image: {}", image);
        Ok(())
    }
```

#### Step 2.4: Wire Auth into DbState

**File**: `packages/projects/src/db.rs`

**Find `DbState::new` method** (around line 38-98) and update the Docker provider initialization:

```rust
impl DbState {
    pub fn new(pool: SqlitePool) -> Result<Self, StorageError> {
        // ... existing code ...

        let token_storage = Arc::new(TokenStorage::new(pool.clone()));

        // ✅ ADD THIS: Get encryption instance
        let encryption = Arc::new(
            orkee_security::ApiKeyEncryption::from_machine_id()
                .map_err(|e| StorageError::Encryption(format!("Failed to create encryption: {}", e)))?
        );

        // Initialize sandbox manager
        let sandbox_storage = Arc::new(orkee_sandbox::SandboxStorage::new(pool.clone()));
        let sandbox_manager = Arc::new(orkee_sandbox::SandboxManager::new(
            sandbox_storage,
            Arc::new(tokio::sync::RwLock::new(orkee_sandbox::SettingsManager::new(
                pool.clone(),
            ))),
        ));

        // ✅ UPDATE THIS: Register Docker provider WITH auth
        let docker_provider = Arc::new(
            orkee_sandbox::DockerProvider::with_auth(
                Some(token_storage.clone()),  // ✅ Pass token storage
                Some(encryption.clone()),      // ✅ Pass encryption
            )
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to initialize Docker provider: {}", e);
                panic!("Docker provider initialization failed: {}", e)
            }),
        ) as Arc<dyn orkee_sandbox::SandboxProvider>;

        // ... rest stays the same ...
    }
}
```

### ✅ Verify

**Run tests** (should pass):
```bash
cargo test -p orkee-sandbox docker_auth
# Expected: All tests pass
```

**Manual test with authentication**:
```bash
# 1. Login to Docker Hub
cargo run --bin orkee -- auth login docker
# Enter your credentials

# 2. Start server
ORKEE_DEV_MODE=true cargo run --bin orkee -- dashboard --api-port 4001

# 3. Create sandbox with your private image (if you have one)
curl -X POST http://localhost:4001/api/sandboxes \
  -H "Content-Type: application/json" \
  -d '{"name":"test","image":"your-username/your-private-image:latest"}'

# Should pull successfully with your credentials
```

**Manual test without authentication** (verify fallback):
```bash
# Should still work for public images
curl -X POST http://localhost:4001/api/sandboxes \
  -H "Content-Type: application/json" \
  -d '{"name":"test-public","image":"nginx:alpine"}'

# Should succeed without credentials
```

**Commit**:
```bash
git add packages/sandbox/src/providers/docker.rs \
        packages/projects/src/db.rs \
        packages/sandbox/tests/docker_auth_tests.rs

git commit -m "feat(sandbox): add Docker registry authentication

- Add auth support to DockerProvider via with_auth() constructor
- Retrieve and decrypt Docker credentials from database
- Parse registry from image name (docker.io, gcr.io, etc.)
- Use credentials for authenticated image pulls
- Graceful fallback to unauthenticated for public images
- Enhanced error messages with actionable guidance

Tests:
- Public image pulls without auth
- Credential retrieval and decryption
- Registry parsing for various formats
- Error handling for auth failures"
```

---

## Phase 3: Configuration CLI (1 hour)

**Goal**: Add `orkee sandbox config` commands to manage default images

### ✅ Done When
- `orkee sandbox config show` displays current settings
- `orkee sandbox config set-image <image>` sets global default
- Settings are persisted to database

### 🧪 Tests First (TDD)

```rust
// File: packages/cli/tests/sandbox_config_tests.rs

#[tokio::test]
async fn test_set_default_image() {
    let db = setup_test_db().await;

    // Set default image
    set_default_image(&db, "myuser/custom-sandbox:v1").await.unwrap();

    // Verify in database
    let settings = db.sandbox_settings.get_sandbox_settings().await.unwrap();
    assert_eq!(settings.default_image, Some("myuser/custom-sandbox:v1".to_string()));
}

#[tokio::test]
async fn test_show_config_displays_settings() {
    let db = setup_test_db().await;

    // Should show current settings
    let output = show_sandbox_config(&db).await.unwrap();
    assert!(output.contains("default_provider"));
    assert!(output.contains("default_image"));
}
```

### 📝 Implementation

**Create new file**: `packages/cli/src/bin/cli/sandbox.rs`

```rust
use anyhow::Result;
use orkee_projects::DbState;

pub async fn sandbox_command(subcommand: &str, args: &[String]) -> Result<()> {
    let db = DbState::init().await?;

    match subcommand {
        "config" => config_command(&db, args).await?,
        _ => {
            eprintln!("Unknown sandbox subcommand: {}", subcommand);
            eprintln!("Available commands: config");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn config_command(db: &DbState, args: &[String]) -> Result<()> {
    if args.is_empty() {
        eprintln!("Usage: orkee sandbox config <show|set-image>");
        std::process::exit(1);
    }

    match args[0].as_str() {
        "show" => show_config(db).await?,
        "set-image" => {
            if args.len() < 2 {
                eprintln!("Usage: orkee sandbox config set-image <image>");
                std::process::exit(1);
            }
            set_default_image(db, &args[1]).await?;
        }
        _ => {
            eprintln!("Unknown config command: {}", args[0]);
            eprintln!("Available: show, set-image");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn show_config(db: &DbState) -> Result<()> {
    let settings = db.sandbox_settings.get_sandbox_settings().await?;

    println!("📦 Sandbox Configuration\n");
    println!("Provider: {}", settings.default_provider);
    println!("Image:    {}", settings.default_image.unwrap_or_else(|| "alpine:latest (default)".to_string()));
    println!("CPU:      {} cores", settings.default_cpu_cores);
    println!("Memory:   {} MB", settings.default_memory_mb);
    println!("Storage:  {} GB", settings.default_storage_gb);

    Ok(())
}

async fn set_default_image(db: &DbState, image: &str) -> Result<()> {
    use orkee_sandbox::SandboxSettings;

    // Get current settings
    let mut settings = db.sandbox_settings.get_sandbox_settings().await?;

    // Update image
    settings.default_image = Some(image.to_string());

    // Save
    db.sandbox_settings
        .update_sandbox_settings(&settings, Some("cli"))
        .await?;

    println!("✅ Default sandbox image set to: {}", image);

    Ok(())
}
```

**Register in**: `packages/cli/src/bin/cli/mod.rs`

```rust
pub mod auth;
pub mod sandbox;  // ✅ ADD THIS

// In main CLI app builder, add:
.subcommand(
    Command::new("sandbox")
        .about("Manage sandbox configuration")
        .subcommand(Command::new("config")
            .about("View or update sandbox configuration")
            .subcommand(Command::new("show").about("Show current configuration"))
            .subcommand(Command::new("set-image")
                .about("Set default sandbox image")
                .arg(Arg::new("image").required(true))))
)

// In match statement:
"sandbox" => {
    if let Some(matches) = matches.subcommand_matches("sandbox") {
        if let Some(("config", config_matches)) = matches.subcommand() {
            if config_matches.subcommand_matches("show").is_some() {
                sandbox::sandbox_command("config", &["show".to_string()]).await?;
            } else if let Some(set_matches) = config_matches.subcommand_matches("set-image") {
                let image = set_matches.get_one::<String>("image").unwrap();
                sandbox::sandbox_command("config", &["set-image".to_string(), image.clone()]).await?;
            }
        }
    }
}
```

### ✅ Verify

```bash
# Show current config
cargo run --bin orkee -- sandbox config show

# Set custom image
cargo run --bin orkee -- sandbox config set-image myuser/custom-sandbox:v1.0

# Verify it was saved
cargo run --bin orkee -- sandbox config show
```

**Commit**:
```bash
git add packages/cli/src/bin/cli/sandbox.rs \
        packages/cli/src/bin/cli/mod.rs

git commit -m "feat(cli): add sandbox configuration commands

- Add 'orkee sandbox config show' to view settings
- Add 'orkee sandbox config set-image' to set default image
- Persist changes to database
- Display current configuration in friendly format"
```

---

## Phase 4: Testing (2 hours)

**Goal**: Comprehensive test coverage for all Docker authentication functionality

### 📝 Test Files to Create

1. **Unit Tests**: Each component tested in isolation
2. **Integration Tests**: Full flow testing
3. **Manual Test Plan**: Real-world usage scenarios

See test code sections in Phases 1-3 above for complete test implementations.

**Run full test suite**:
```bash
cargo test --workspace
# All tests should pass
```

---

## Phase 5: Documentation (1 hour)

**Goal**: User-facing documentation for Docker authentication

### 📝 Tasks

- [ ] Update main README with Docker auth section
- [ ] Create docs/docker-authentication.md user guide
- [ ] Add troubleshooting guide
- [ ] Document Personal Access Token creation

---

## Reference: Common Errors

### 404 Image Not Found
```
Error: Image 'user/image:latest' not found
```
**Solution**: Check spelling, verify image exists, ensure access

### 401 Authentication Failed
```
Error: Authentication failed for image
```
**Solution**: Run `orkee auth login docker` with valid credentials

### 429 Rate Limit
```
Error: Docker Hub rate limit exceeded
```
**Solution**: Authenticate to get 200 pulls/6hr instead of 100

### Decryption Failed
```
Error: Failed to decrypt Docker credentials
```
**Solution**: Re-run `orkee auth login docker` if encryption password changed

---

## Quick Reference: Key Files

| File | Purpose |
|------|---------|
| `packages/auth/src/oauth/provider.rs` | OAuth provider enum |
| `packages/cli/src/bin/cli/auth.rs` | Login commands |
| `packages/sandbox/src/providers/docker.rs` | Docker API integration |
| `packages/projects/src/db.rs` | Component wiring |
| `packages/api/src/sandbox_handlers.rs` | HTTP endpoints |

---

## Cargo Dependencies

**No new dependencies needed!** All required crates already in use:
- `bollard` - Docker API
- `serde_json` - Metadata storage
- `orkee_security` - Encryption
- `orkee_auth` - OAuth system
