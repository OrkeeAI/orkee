// ABOUTME: PRD (Product Requirements Document) module
// ABOUTME: Provides types and database operations for managing product requirements documents

pub mod db;
pub mod types;

// Re-export main types for convenience
pub use db::{
    create_prd, delete_prd, get_prd, get_prds_by_project, get_prds_by_project_paginated,
    hard_delete_prd, restore_prd, update_prd, DbError, DbResult,
};
pub use types::{PRDSource, PRDStatus, PRD};
