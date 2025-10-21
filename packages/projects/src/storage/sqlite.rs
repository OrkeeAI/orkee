use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{migrate::MigrateDatabase, Row};
use tracing::{debug, info, warn};

use super::{
    compress_data, decompress_data, generate_project_id, ConflictType, DatabaseSnapshot,
    ImportConflict, ImportResult, ProjectFilter, ProjectStorage, StorageCapabilities,
    StorageConfig, StorageError, StorageInfo, StorageProvider, StorageResult,
};
use crate::types::{
    Priority, Project, ProjectCreateInput, ProjectStatus, ProjectUpdateInput, TaskSource,
};

/// SQLite implementation of ProjectStorage
pub struct SqliteStorage {
    pool: SqlitePool,
    config: StorageConfig,
}

impl SqliteStorage {
    /// Create a new SqliteStorage instance
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        let database_path = match &config.provider {
            StorageProvider::Sqlite { path } => path,
            _ => return Err(StorageError::InvalidFormat),
        };

        // Ensure parent directory exists
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent).map_err(StorageError::Io)?;
        }

        let database_url = format!("sqlite:{}", database_path.display());

        // Create database if it doesn't exist
        if !sqlx::Sqlite::database_exists(&database_url)
            .await
            .map_err(StorageError::Sqlx)?
        {
            debug!("Creating database at: {}", database_url);
            sqlx::Sqlite::create_database(&database_url)
                .await
                .map_err(StorageError::Sqlx)?;
        }

        // Configure connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.busy_timeout_seconds))
            .connect(&database_url)
            .await
            .map_err(StorageError::Sqlx)?;

        // Configure SQLite settings (after pool creation, before migrations)
        if config.enable_wal {
            sqlx::query("PRAGMA journal_mode = WAL")
                .execute(&pool)
                .await
                .map_err(StorageError::Sqlx)?;
        }

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        sqlx::query("PRAGMA temp_store = memory")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        sqlx::query("PRAGMA mmap_size = 268435456")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let storage = Self { pool, config };
        Ok(storage)
    }

    /// Convert a database row to a Project
    fn row_to_project(&self, row: &SqliteRow) -> StorageResult<Project> {
        let tags_json: Option<String> = row.try_get("tags")?;
        let manual_tasks_json: Option<String> = row.try_get("manual_tasks")?;
        let mcp_servers_json: Option<String> = row.try_get("mcp_servers")?;
        let git_repository_json: Option<String> = row.try_get("git_repository")?;

        let tags = if let Some(json) = tags_json {
            Some(serde_json::from_str(&json)?)
        } else {
            None
        };

        let manual_tasks = if let Some(json) = manual_tasks_json {
            Some(serde_json::from_str(&json)?)
        } else {
            None
        };

        let mcp_servers = if let Some(json) = mcp_servers_json {
            Some(serde_json::from_str(&json)?)
        } else {
            None
        };

        let git_repository = if let Some(json) = git_repository_json {
            Some(serde_json::from_str(&json)?)
        } else {
            None
        };

        let status_str: String = row.try_get("status")?;
        let status = match status_str.as_str() {
            "planning" => ProjectStatus::Planning,
            "building" => ProjectStatus::Building,
            "review" => ProjectStatus::Review,
            "launched" => ProjectStatus::Launched,
            "on-hold" => ProjectStatus::OnHold,
            "archived" => ProjectStatus::Archived,
            // Legacy compatibility
            "pre-launch" => ProjectStatus::Planning,
            _ => ProjectStatus::Planning,
        };

        let priority_str: String = row.try_get("priority")?;
        let priority = match priority_str.as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            _ => Priority::Medium,
        };

        let task_source_str: Option<String> = row.try_get("task_source")?;
        let task_source = task_source_str.and_then(|s| match s.as_str() {
            "manual" => Some(TaskSource::Manual),
            "taskmaster" => Some(TaskSource::Taskmaster),
            _ => None,
        });

        let created_at_str: String = row.try_get("created_at")?;
        let updated_at_str: String = row.try_get("updated_at")?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_| StorageError::Database("Invalid created_at timestamp".to_string()))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|_| StorageError::Database("Invalid updated_at timestamp".to_string()))?
            .with_timezone(&Utc);

        Ok(Project {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            project_root: row.try_get("project_root")?,
            description: row.try_get("description")?,
            status,
            priority,
            rank: row.try_get("rank")?,
            setup_script: row.try_get("setup_script")?,
            dev_script: row.try_get("dev_script")?,
            cleanup_script: row.try_get("cleanup_script")?,
            task_source,
            tags,
            manual_tasks,
            mcp_servers,
            git_repository,
            created_at,
            updated_at,
        })
    }

    /// Convert project status to string
    fn status_to_string(status: &ProjectStatus) -> &'static str {
        match status {
            ProjectStatus::Planning => "planning",
            ProjectStatus::Building => "building",
            ProjectStatus::Review => "review",
            ProjectStatus::Launched => "launched",
            ProjectStatus::OnHold => "on-hold",
            ProjectStatus::Archived => "archived",
        }
    }

    /// Convert priority to string
    fn priority_to_string(priority: &Priority) -> &'static str {
        match priority {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
        }
    }

    /// Convert task source to string
    fn task_source_to_string(task_source: &TaskSource) -> &'static str {
        match task_source {
            TaskSource::Manual => "manual",
            TaskSource::Taskmaster => "taskmaster",
        }
    }
}

#[async_trait]
impl ProjectStorage for SqliteStorage {
    async fn initialize(&self) -> StorageResult<()> {
        info!("Initializing SQLite storage with migrations");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(StorageError::Migration)?;

        // Run post-migration optimizations
        sqlx::query("ANALYZE")
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        info!("SQLite storage initialized successfully");
        Ok(())
    }

    async fn create_project(&self, input: ProjectCreateInput) -> StorageResult<Project> {
        let id = generate_project_id();
        let now = Utc::now();

        let tags_json = input.tags.as_ref().map(serde_json::to_string).transpose()?;
        let manual_tasks_json = input
            .manual_tasks
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let mcp_servers_json = input
            .mcp_servers
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let status_str = Self::status_to_string(&input.status.unwrap_or(ProjectStatus::Planning));
        let priority_str = Self::priority_to_string(&input.priority.unwrap_or(Priority::Medium));
        let task_source_str = input.task_source.as_ref().map(Self::task_source_to_string);

        let result = sqlx::query(
            r#"
            INSERT INTO projects (
                id, name, project_root, description, status, priority, rank,
                setup_script, dev_script, cleanup_script, task_source,
                tags, manual_tasks, mcp_servers, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.project_root)
        .bind(&input.description)
        .bind(status_str)
        .bind(priority_str)
        .bind(input.rank)
        .bind(&input.setup_script)
        .bind(&input.dev_script)
        .bind(&input.cleanup_script)
        .bind(task_source_str)
        .bind(&tags_json)
        .bind(&manual_tasks_json)
        .bind(&mcp_servers_json)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                debug!("Created project '{}' with ID {}", input.name, id);
                self.get_project(&id).await?.ok_or(StorageError::NotFound)
            }
            Err(sqlx::Error::Database(db_err)) => {
                // SQLite UNIQUE constraint violation
                if let Some(code) = db_err.code() {
                    if code == "2067" || code == "1555" {
                        // SQLITE_CONSTRAINT_UNIQUE
                        let message = db_err.message();
                        if message.contains("name") {
                            return Err(StorageError::DuplicateName(input.name));
                        } else if message.contains("project_root") {
                            return Err(StorageError::DuplicatePath(input.project_root));
                        }
                    }
                }
                Err(StorageError::Sqlx(sqlx::Error::Database(db_err)))
            }
            Err(e) => Err(StorageError::Sqlx(e)),
        }
    }

    async fn get_project(&self, id: &str) -> StorageResult<Option<Project>> {
        let row = sqlx::query("SELECT * FROM projects WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        match row {
            Some(row) => Ok(Some(self.row_to_project(&row)?)),
            None => Ok(None),
        }
    }

    async fn get_project_by_name(&self, name: &str) -> StorageResult<Option<Project>> {
        let row = sqlx::query("SELECT * FROM projects WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        match row {
            Some(row) => Ok(Some(self.row_to_project(&row)?)),
            None => Ok(None),
        }
    }

    async fn get_project_by_path(&self, path: &str) -> StorageResult<Option<Project>> {
        let row = sqlx::query("SELECT * FROM projects WHERE project_root = ?")
            .bind(path)
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        match row {
            Some(row) => Ok(Some(self.row_to_project(&row)?)),
            None => Ok(None),
        }
    }

    async fn list_projects(&self) -> StorageResult<Vec<Project>> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM projects 
            WHERE status != 'deleted'
            ORDER BY 
                CASE WHEN rank IS NULL THEN 1 ELSE 0 END,
                rank ASC,
                name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        let mut projects = Vec::new();
        for row in rows {
            projects.push(self.row_to_project(&row)?);
        }

        debug!("Retrieved {} projects", projects.len());
        Ok(projects)
    }

    async fn update_project(&self, id: &str, input: ProjectUpdateInput) -> StorageResult<Project> {
        let mut query_parts = Vec::new();

        if input.name.is_some() {
            query_parts.push("name = ?");
        }
        if input.project_root.is_some() {
            query_parts.push("project_root = ?");
        }
        if input.description.is_some() {
            query_parts.push("description = ?");
        }
        if input.status.is_some() {
            query_parts.push("status = ?");
        }
        if input.priority.is_some() {
            query_parts.push("priority = ?");
        }
        if input.rank.is_some() {
            query_parts.push("rank = ?");
        }
        if input.setup_script.is_some() {
            query_parts.push("setup_script = ?");
        }
        if input.dev_script.is_some() {
            query_parts.push("dev_script = ?");
        }
        if input.cleanup_script.is_some() {
            query_parts.push("cleanup_script = ?");
        }
        if input.task_source.is_some() {
            query_parts.push("task_source = ?");
        }
        if input.tags.is_some() {
            query_parts.push("tags = ?");
        }
        if input.manual_tasks.is_some() {
            query_parts.push("manual_tasks = ?");
        }
        if input.mcp_servers.is_some() {
            query_parts.push("mcp_servers = ?");
        }

        if query_parts.is_empty() {
            return self.get_project(id).await?.ok_or(StorageError::NotFound);
        }

        query_parts.push("updated_at = ?");

        let query_str = format!(
            "UPDATE projects SET {} WHERE id = ?",
            query_parts.join(", ")
        );

        let mut query = sqlx::query(&query_str);

        if let Some(ref name) = input.name {
            query = query.bind(name);
        }
        if let Some(ref project_root) = input.project_root {
            query = query.bind(project_root);
        }
        if let Some(ref description) = input.description {
            query = query.bind(description);
        }
        if let Some(ref status) = input.status {
            query = query.bind(Self::status_to_string(status));
        }
        if let Some(ref priority) = input.priority {
            query = query.bind(Self::priority_to_string(priority));
        }
        if let Some(rank) = input.rank {
            query = query.bind(rank);
        }
        if let Some(ref setup_script) = input.setup_script {
            query = query.bind(setup_script);
        }
        if let Some(ref dev_script) = input.dev_script {
            query = query.bind(dev_script);
        }
        if let Some(ref cleanup_script) = input.cleanup_script {
            query = query.bind(cleanup_script);
        }
        if let Some(ref task_source) = input.task_source {
            query = query.bind(Self::task_source_to_string(task_source));
        }
        if let Some(ref tags) = input.tags {
            let tags_json = serde_json::to_string(tags)?;
            query = query.bind(tags_json);
        }
        if let Some(ref manual_tasks) = input.manual_tasks {
            let manual_tasks_json = serde_json::to_string(manual_tasks)?;
            query = query.bind(manual_tasks_json);
        }
        if let Some(ref mcp_servers) = input.mcp_servers {
            let mcp_servers_json = serde_json::to_string(mcp_servers)?;
            query = query.bind(mcp_servers_json);
        }

        query = query.bind(Utc::now().to_rfc3339()).bind(id);

        let result = query.execute(&self.pool).await;

        match result {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    return Err(StorageError::NotFound);
                }
                debug!("Updated project with ID {}", id);
                self.get_project(id).await?.ok_or(StorageError::NotFound)
            }
            Err(sqlx::Error::Database(db_err)) => {
                // SQLite UNIQUE constraint violation
                if let Some(code) = db_err.code() {
                    if code == "2067" || code == "1555" {
                        // SQLITE_CONSTRAINT_UNIQUE
                        let message = db_err.message();
                        if message.contains("name") {
                            return Err(StorageError::DuplicateName(
                                input.name.unwrap_or_default(),
                            ));
                        } else if message.contains("project_root") {
                            return Err(StorageError::DuplicatePath(
                                input.project_root.unwrap_or_default(),
                            ));
                        }
                    }
                }
                Err(StorageError::Sqlx(sqlx::Error::Database(db_err)))
            }
            Err(e) => Err(StorageError::Sqlx(e)),
        }
    }

    async fn delete_project(&self, id: &str) -> StorageResult<()> {
        let result = sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        debug!("Deleted project with ID {}", id);
        Ok(())
    }

    async fn list_projects_with_filter(
        &self,
        filter: ProjectFilter,
    ) -> StorageResult<Vec<Project>> {
        let mut where_conditions = vec!["status != 'deleted'"];
        let mut query_params: Vec<String> = Vec::new();

        if let Some(status) = &filter.status {
            where_conditions.push("status = ?");
            query_params.push(Self::status_to_string(status).to_string());
        }

        if let Some(priority) = &filter.priority {
            where_conditions.push("priority = ?");
            query_params.push(Self::priority_to_string(priority).to_string());
        }

        if let Some(task_source) = &filter.task_source {
            where_conditions.push("task_source = ?");
            query_params.push(Self::task_source_to_string(task_source).to_string());
        }

        if let Some(updated_after) = &filter.updated_after {
            where_conditions.push("updated_at >= ?");
            query_params.push(updated_after.to_rfc3339());
        }

        let where_clause = where_conditions.join(" AND ");
        let limit_clause = filter
            .limit
            .map(|l| format!(" LIMIT {}", l))
            .unwrap_or_default();
        let offset_clause = filter
            .offset
            .map(|o| format!(" OFFSET {}", o))
            .unwrap_or_default();

        let query_str = format!(
            r#"
            SELECT * FROM projects 
            WHERE {}
            ORDER BY 
                CASE WHEN rank IS NULL THEN 1 ELSE 0 END,
                rank ASC,
                name ASC
            {}{}
            "#,
            where_clause, limit_clause, offset_clause
        );

        let mut query = sqlx::query(&query_str);
        for param in &query_params {
            query = query.bind(param);
        }

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let mut projects = Vec::new();
        for row in rows {
            projects.push(self.row_to_project(&row)?);
        }

        Ok(projects)
    }

    async fn search_projects(&self, query: &str) -> StorageResult<Vec<Project>> {
        if !self.config.enable_fts {
            warn!("Full-text search is disabled, falling back to LIKE search");
            let rows = sqlx::query(
                r#"
                SELECT * FROM projects 
                WHERE (name LIKE ? OR description LIKE ? OR project_root LIKE ?)
                AND status != 'deleted'
                ORDER BY 
                    CASE WHEN rank IS NULL THEN 1 ELSE 0 END,
                    rank ASC,
                    name ASC
                "#,
            )
            .bind(format!("%{}%", query))
            .bind(format!("%{}%", query))
            .bind(format!("%{}%", query))
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

            let mut projects = Vec::new();
            for row in rows {
                projects.push(self.row_to_project(&row)?);
            }
            return Ok(projects);
        }

        // Use FTS5 for better search
        let rows = sqlx::query(
            r#"
            SELECT p.* FROM projects p
            JOIN projects_fts fts ON p.rowid = fts.rowid
            WHERE projects_fts MATCH ?
            AND p.status != 'deleted'
            ORDER BY fts.rank
            "#,
        )
        .bind(query)
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        let mut projects = Vec::new();
        for row in rows {
            projects.push(self.row_to_project(&row)?);
        }

        debug!(
            "Found {} projects matching query '{}'",
            projects.len(),
            query
        );
        Ok(projects)
    }

    async fn bulk_update(
        &self,
        updates: Vec<(String, ProjectUpdateInput)>,
    ) -> StorageResult<Vec<Project>> {
        let mut updated_projects = Vec::new();
        // This is a simplified version - in a real implementation you'd want to batch these queries
        for (id, update_input) in updates {
            let project = self.update_project(&id, update_input).await?;
            updated_projects.push(project);
        }
        Ok(updated_projects)
    }

    async fn get_storage_info(&self) -> StorageResult<StorageInfo> {
        let count_row =
            sqlx::query("SELECT COUNT(*) as count FROM projects WHERE status != 'deleted'")
                .fetch_one(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
        let total_projects: i64 = count_row.try_get("count")?;

        let last_modified_row =
            sqlx::query("SELECT MAX(updated_at) as last_modified FROM projects")
                .fetch_one(&self.pool)
                .await
                .map_err(StorageError::Sqlx)?;
        let last_modified_str: Option<String> = last_modified_row.try_get("last_modified")?;
        let last_modified = last_modified_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        // Get database file size
        let db_path = match &self.config.provider {
            StorageProvider::Sqlite { path } => path,
            _ => return Err(StorageError::InvalidFormat),
        };

        let size_bytes = std::fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);

        Ok(StorageInfo {
            provider: "SQLite".to_string(),
            version: "1".to_string(),
            total_projects: total_projects as usize,
            last_modified,
            size_bytes,
            capabilities: StorageCapabilities {
                full_text_search: self.config.enable_fts,
                real_time_sync: false,
                offline_mode: true,
                multi_user: false,
                cloud_backup: true,
            },
        })
    }

    async fn export_snapshot(&self) -> StorageResult<Vec<u8>> {
        let projects = self.list_projects().await?;

        let snapshot = DatabaseSnapshot {
            version: 1,
            exported_at: Utc::now(),
            projects,
        };

        let json_data = serde_json::to_vec(&snapshot)?;
        let compressed_data = compress_data(&json_data)?;

        debug!(
            "Exported snapshot with {} projects ({} bytes compressed)",
            snapshot.projects.len(),
            compressed_data.len()
        );

        Ok(compressed_data)
    }

    async fn import_snapshot(&self, data: &[u8]) -> StorageResult<ImportResult> {
        let json_data = decompress_data(data)?;
        let snapshot: DatabaseSnapshot = serde_json::from_slice(&json_data)?;

        debug!(
            "Importing snapshot with {} projects",
            snapshot.projects.len()
        );

        let mut imported = 0;
        let mut skipped = 0;
        let mut conflicts = Vec::new();

        for project in snapshot.projects {
            // Check for conflicts
            let existing_by_name = self.get_project_by_name(&project.name).await?;
            let existing_by_path = self.get_project_by_path(&project.project_root).await?;

            if existing_by_name.is_some() {
                conflicts.push(ImportConflict {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                    conflict_type: ConflictType::DuplicateName,
                });
                skipped += 1;
                continue;
            }

            if existing_by_path.is_some() {
                conflicts.push(ImportConflict {
                    project_id: project.id.clone(),
                    project_name: project.name.clone(),
                    conflict_type: ConflictType::DuplicatePath,
                });
                skipped += 1;
                continue;
            }

            // Import project
            let create_input = ProjectCreateInput {
                name: project.name,
                project_root: project.project_root,
                description: project.description,
                status: Some(project.status),
                priority: Some(project.priority),
                rank: project.rank,
                setup_script: project.setup_script,
                dev_script: project.dev_script,
                cleanup_script: project.cleanup_script,
                task_source: project.task_source,
                tags: project.tags,
                manual_tasks: project.manual_tasks,
                mcp_servers: project.mcp_servers,
            };

            match self.create_project(create_input).await {
                Ok(_) => imported += 1,
                Err(_) => {
                    conflicts.push(ImportConflict {
                        project_id: project.id,
                        project_name: "Unknown".to_string(),
                        conflict_type: ConflictType::VersionConflict,
                    });
                    skipped += 1;
                }
            }
        }

        info!(
            "Import completed: {} imported, {} skipped, {} conflicts",
            imported,
            skipped,
            conflicts.len()
        );

        Ok(ImportResult {
            projects_imported: imported,
            projects_skipped: skipped,
            conflicts,
        })
    }

    async fn get_encryption_mode(
        &self,
    ) -> StorageResult<Option<crate::security::encryption::EncryptionMode>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT encryption_mode FROM encryption_settings WHERE id = 1")
                .fetch_optional(&self.pool)
                .await?;

        match row {
            Some((mode_str,)) => {
                let mode = mode_str.parse().map_err(|_| {
                    StorageError::Database(format!("Invalid encryption mode: {}", mode_str))
                })?;
                Ok(Some(mode))
            }
            None => Ok(None),
        }
    }

    async fn get_encryption_settings(
        &self,
    ) -> StorageResult<
        Option<(
            crate::security::encryption::EncryptionMode,
            Option<Vec<u8>>,
            Option<Vec<u8>>,
        )>,
    > {
        let row: Option<(String, Option<Vec<u8>>, Option<Vec<u8>>)> = sqlx::query_as(
            "SELECT encryption_mode, password_salt, password_hash FROM encryption_settings WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((mode_str, salt, hash)) => {
                let mode = mode_str.parse().map_err(|_| {
                    StorageError::Database(format!("Invalid encryption mode: {}", mode_str))
                })?;
                Ok(Some((mode, salt, hash)))
            }
            None => Ok(None),
        }
    }

    async fn set_encryption_mode(
        &self,
        mode: crate::security::encryption::EncryptionMode,
        salt: Option<&[u8]>,
        hash: Option<&[u8]>,
    ) -> StorageResult<()> {
        let mode_str = mode.to_string();

        sqlx::query(
            "INSERT OR REPLACE INTO encryption_settings (id, encryption_mode, password_salt, password_hash, updated_at)
             VALUES (1, ?, ?, ?, datetime('now'))"
        )
        .bind(&mode_str)
        .bind(salt)
        .bind(hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn check_password_lockout(&self) -> StorageResult<()> {
        let row: Option<(i64, Option<String>)> = sqlx::query_as(
            "SELECT attempt_count, locked_until FROM password_attempts WHERE id = 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((attempt_count, Some(locked_until_str))) => {
                // Parse the locked_until timestamp
                let locked_until = DateTime::parse_from_rfc3339(&locked_until_str)
                    .map_err(|e| {
                        StorageError::Database(format!("Invalid locked_until timestamp: {}", e))
                    })?
                    .with_timezone(&Utc);

                let now = Utc::now();

                // Check if still locked
                if locked_until > now {
                    let remaining_seconds = (locked_until - now).num_seconds();
                    let remaining_minutes = (remaining_seconds + 59) / 60; // Round up

                    return Err(StorageError::Database(format!(
                        "Account locked due to too many failed password attempts. {} failed attempts. Try again in {} minute{}.",
                        attempt_count,
                        remaining_minutes,
                        if remaining_minutes == 1 { "" } else { "s" }
                    )));
                }

                // Lockout has expired, allow access
                Ok(())
            }
            Some((_, None)) => {
                // Not locked
                Ok(())
            }
            None => {
                // No row exists, initialize it
                sqlx::query(
                    "INSERT INTO password_attempts (id, attempt_count) VALUES (1, 0)
                     ON CONFLICT(id) DO NOTHING",
                )
                .execute(&self.pool)
                .await?;
                Ok(())
            }
        }
    }

    async fn record_failed_password_attempt(&self) -> StorageResult<()> {
        const MAX_ATTEMPTS: i64 = 5;
        const LOCKOUT_MINUTES: i64 = 15;

        // Get current attempt count
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT attempt_count FROM password_attempts WHERE id = 1")
                .fetch_optional(&self.pool)
                .await?;

        let new_count = match row {
            Some((count,)) => count + 1,
            None => {
                // Initialize if not exists
                sqlx::query(
                    "INSERT INTO password_attempts (id, attempt_count) VALUES (1, 0)
                     ON CONFLICT(id) DO NOTHING",
                )
                .execute(&self.pool)
                .await?;
                1
            }
        };

        // Update attempt count and last_attempt_at
        // If we've hit the max attempts, set locked_until
        let now = Utc::now();
        let locked_until = if new_count >= MAX_ATTEMPTS {
            Some(
                (now + chrono::Duration::minutes(LOCKOUT_MINUTES))
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )
        } else {
            None
        };

        sqlx::query(
            "UPDATE password_attempts
             SET attempt_count = ?,
                 last_attempt_at = ?,
                 locked_until = ?,
                 updated_at = datetime('now', 'utc')
             WHERE id = 1",
        )
        .bind(new_count)
        .bind(now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .bind(locked_until)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn reset_password_attempts(&self) -> StorageResult<()> {
        sqlx::query(
            "UPDATE password_attempts
             SET attempt_count = 0,
                 last_attempt_at = NULL,
                 locked_until = NULL,
                 updated_at = datetime('now', 'utc')
             WHERE id = 1",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProjectStatus;

    async fn create_test_storage() -> SqliteStorage {
        use std::path::PathBuf;

        // Use in-memory database for tests - more reliable than temp files
        let db_path = PathBuf::from(":memory:");

        let config = StorageConfig {
            provider: StorageProvider::Sqlite { path: db_path },
            enable_wal: false, // WAL mode doesn't work with :memory:
            enable_fts: true,
            max_connections: 1, // Single connection for in-memory
            busy_timeout_seconds: 10,
        };

        let storage = SqliteStorage::new(config).await.unwrap();
        storage.initialize().await.unwrap();
        storage
    }

    #[tokio::test]
    async fn test_create_and_get_project() {
        let storage = create_test_storage().await;

        let input = ProjectCreateInput {
            name: "Test Project".to_string(),
            project_root: "/tmp/test".to_string(),
            description: Some("A test project".to_string()),
            status: Some(ProjectStatus::Planning),
            priority: Some(Priority::High),
            rank: Some(1),
            setup_script: Some("npm install".to_string()),
            dev_script: Some("npm run dev".to_string()),
            cleanup_script: None,
            task_source: None,
            tags: Some(vec!["rust".to_string(), "test".to_string()]),
            manual_tasks: None,
            mcp_servers: None,
        };

        let project = storage.create_project(input).await.unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.status, ProjectStatus::Planning);
        assert_eq!(project.priority, Priority::High);

        let retrieved = storage.get_project(&project.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Test Project");
        assert_eq!(
            retrieved.tags,
            Some(vec!["rust".to_string(), "test".to_string()])
        );
    }

    #[tokio::test]
    async fn test_duplicate_name_error() {
        let storage = create_test_storage().await;

        let input1 = ProjectCreateInput {
            name: "Duplicate".to_string(),
            project_root: "/tmp/dup1".to_string(),
            description: None,
            status: None,
            priority: None,
            rank: None,
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            task_source: None,
            tags: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        storage.create_project(input1).await.unwrap();

        let input2 = ProjectCreateInput {
            name: "Duplicate".to_string(),
            project_root: "/tmp/dup2".to_string(),
            description: None,
            status: None,
            priority: None,
            rank: None,
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            task_source: None,
            tags: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        let result = storage.create_project(input2).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::DuplicateName(name) => assert_eq!(name, "Duplicate"),
            _ => panic!("Expected DuplicateName error"),
        }
    }

    #[tokio::test]
    async fn test_search_projects() {
        let storage = create_test_storage().await;

        let input1 = ProjectCreateInput {
            name: "React App".to_string(),
            project_root: "/tmp/react".to_string(),
            description: Some("A React application".to_string()),
            status: None,
            priority: None,
            rank: None,
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            task_source: None,
            tags: Some(vec!["react".to_string(), "javascript".to_string()]),
            manual_tasks: None,
            mcp_servers: None,
        };

        let input2 = ProjectCreateInput {
            name: "Rust CLI".to_string(),
            project_root: "/tmp/rust".to_string(),
            description: Some("A Rust command line tool".to_string()),
            status: None,
            priority: None,
            rank: None,
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            task_source: None,
            tags: Some(vec!["rust".to_string(), "cli".to_string()]),
            manual_tasks: None,
            mcp_servers: None,
        };

        storage.create_project(input1).await.unwrap();
        storage.create_project(input2).await.unwrap();

        let results = storage.search_projects("React").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "React App");

        let results = storage.search_projects("Rust").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Rust CLI");
    }
}
