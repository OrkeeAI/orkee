// ABOUTME: Integration tests for tag storage operations
// ABOUTME: Tests CRUD operations, pagination, archiving, and deletion validation

use orkee_tags::{TagCreateInput, TagStorage, TagUpdateInput};
use sqlx::SqlitePool;

/// Helper to create an in-memory database for testing
async fn create_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Create tags table
    sqlx::query(
        r#"
        CREATE TABLE tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT,
            description TEXT,
            created_at TEXT NOT NULL,
            archived_at TEXT
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create tasks table for FK validation tests
    sqlx::query(
        r#"
        CREATE TABLE tasks (
            id TEXT PRIMARY KEY,
            tag_id TEXT,
            title TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

#[tokio::test]
async fn test_create_tag() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool);

    let input = TagCreateInput {
        name: "Feature".to_string(),
        color: Some("#ff0000".to_string()),
        description: Some("Feature work".to_string()),
    };

    let tag = storage.create_tag(input).await.unwrap();

    assert_eq!(tag.name, "Feature");
    assert_eq!(tag.color, Some("#ff0000".to_string()));
    assert_eq!(tag.description, Some("Feature work".to_string()));
    assert!(tag.id.starts_with("tag-"));
    assert!(tag.archived_at.is_none());
}

#[tokio::test]
async fn test_get_tag() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool);

    let input = TagCreateInput {
        name: "Bug".to_string(),
        color: None,
        description: None,
    };

    let created = storage.create_tag(input).await.unwrap();
    let retrieved = storage.get_tag(&created.id).await.unwrap();

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.name, "Bug");
}

#[tokio::test]
async fn test_get_tag_by_name() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool);

    let input = TagCreateInput {
        name: "Refactor".to_string(),
        color: None,
        description: None,
    };

    storage.create_tag(input).await.unwrap();

    let found = storage.get_tag_by_name("Refactor").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Refactor");

    let not_found = storage.get_tag_by_name("NonExistent").await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_list_tags() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool);

    // Create multiple tags
    for name in &["Feature", "Bug", "Docs"] {
        let input = TagCreateInput {
            name: name.to_string(),
            color: None,
            description: None,
        };
        storage.create_tag(input).await.unwrap();
    }

    let tags = storage.list_tags(false).await.unwrap();
    assert_eq!(tags.len(), 3);

    // Check alphabetical ordering
    assert_eq!(tags[0].name, "Bug");
    assert_eq!(tags[1].name, "Docs");
    assert_eq!(tags[2].name, "Feature");
}

#[tokio::test]
async fn test_list_tags_paginated() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool);

    // Create 5 tags
    for i in 0..5 {
        let input = TagCreateInput {
            name: format!("Tag{}", i),
            color: None,
            description: None,
        };
        storage.create_tag(input).await.unwrap();
    }

    // Test pagination
    let (page1, total) = storage
        .list_tags_paginated(false, Some(2), Some(0))
        .await
        .unwrap();
    assert_eq!(page1.len(), 2);
    assert_eq!(total, 5);

    let (page2, _) = storage
        .list_tags_paginated(false, Some(2), Some(2))
        .await
        .unwrap();
    assert_eq!(page2.len(), 2);

    // Ensure different tags on different pages
    assert_ne!(page1[0].id, page2[0].id);
}

#[tokio::test]
async fn test_update_tag() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool);

    let input = TagCreateInput {
        name: "Original".to_string(),
        color: None,
        description: None,
    };

    let created = storage.create_tag(input).await.unwrap();

    let update = TagUpdateInput {
        name: Some("Updated".to_string()),
        color: Some("#00ff00".to_string()),
        description: Some("New description".to_string()),
        archived_at: None,
    };

    let updated = storage.update_tag(&created.id, update).await.unwrap();

    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.color, Some("#00ff00".to_string()));
    assert_eq!(updated.description, Some("New description".to_string()));
}

#[tokio::test]
async fn test_archive_tag() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool);

    let input = TagCreateInput {
        name: "ToArchive".to_string(),
        color: None,
        description: None,
    };

    let created = storage.create_tag(input).await.unwrap();
    assert!(created.archived_at.is_none());

    let archived = storage.archive_tag(&created.id).await.unwrap();
    assert!(archived.archived_at.is_some());

    // Archived tags should not appear in default listing
    let tags = storage.list_tags(false).await.unwrap();
    assert_eq!(tags.len(), 0);

    // But should appear when including archived
    let all_tags = storage.list_tags(true).await.unwrap();
    assert_eq!(all_tags.len(), 1);
}

#[tokio::test]
async fn test_unarchive_tag() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool);

    let input = TagCreateInput {
        name: "ToUnarchive".to_string(),
        color: None,
        description: None,
    };

    let created = storage.create_tag(input).await.unwrap();
    let archived = storage.archive_tag(&created.id).await.unwrap();
    assert!(archived.archived_at.is_some());

    let unarchived = storage.unarchive_tag(&created.id).await.unwrap();
    assert!(unarchived.archived_at.is_none());

    // Should appear in default listing again
    let tags = storage.list_tags(false).await.unwrap();
    assert_eq!(tags.len(), 1);
}

#[tokio::test]
async fn test_delete_unused_tag() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool.clone());

    let input = TagCreateInput {
        name: "Unused".to_string(),
        color: None,
        description: None,
    };

    let created = storage.create_tag(input).await.unwrap();

    // Should successfully delete unused tag
    storage.delete_tag(&created.id).await.unwrap();

    // Verify it's gone
    let result = storage.get_tag(&created.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_tag_in_use_fails() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool.clone());

    let input = TagCreateInput {
        name: "InUse".to_string(),
        color: None,
        description: None,
    };

    let created = storage.create_tag(input).await.unwrap();

    // Create a task using this tag
    sqlx::query("INSERT INTO tasks (id, tag_id, title) VALUES (?, ?, ?)")
        .bind("task-1")
        .bind(&created.id)
        .bind("Test task")
        .execute(&pool)
        .await
        .unwrap();

    // Delete should fail
    let result = storage.delete_tag(&created.id).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("tasks are using it"));

    // Tag should still exist
    let still_exists = storage.get_tag(&created.id).await;
    assert!(still_exists.is_ok());
}

#[tokio::test]
async fn test_partial_update() {
    let pool = create_test_db().await;
    let storage = TagStorage::new(pool);

    let input = TagCreateInput {
        name: "Original".to_string(),
        color: Some("#ff0000".to_string()),
        description: Some("Original description".to_string()),
    };

    let created = storage.create_tag(input).await.unwrap();

    // Only update the color
    let update = TagUpdateInput {
        name: None,
        color: Some("#0000ff".to_string()),
        description: None,
        archived_at: None,
    };

    let updated = storage.update_tag(&created.id, update).await.unwrap();

    assert_eq!(updated.name, "Original"); // Unchanged
    assert_eq!(updated.color, Some("#0000ff".to_string())); // Changed
    assert_eq!(
        updated.description,
        Some("Original description".to_string())
    ); // Unchanged
}
