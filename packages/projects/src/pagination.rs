// ABOUTME: Pagination utilities for list endpoints
// ABOUTME: Provides standardized query parameters and response wrappers

use serde::{Deserialize, Serialize};

/// Default page size for paginated queries
pub const DEFAULT_PAGE_SIZE: i64 = 20;

/// Maximum page size to prevent performance issues
pub const MAX_PAGE_SIZE: i64 = 100;

/// Minimum page number (1-indexed)
pub const MIN_PAGE: i64 = 1;

/// Query parameters for pagination
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-indexed, defaults to 1)
    #[serde(default = "default_page")]
    pub page: i64,

    /// Number of items per page (defaults to DEFAULT_PAGE_SIZE, max MAX_PAGE_SIZE)
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    MIN_PAGE
}

fn default_limit() -> i64 {
    DEFAULT_PAGE_SIZE
}

impl PaginationParams {
    /// Create new pagination params with defaults
    pub fn new() -> Self {
        Self {
            page: MIN_PAGE,
            limit: DEFAULT_PAGE_SIZE,
        }
    }

    /// Create pagination params with custom values
    pub fn with_page_and_limit(page: i64, limit: i64) -> Self {
        Self { page, limit }
    }

    /// Validate and normalize pagination parameters
    /// Returns (limit, offset) suitable for SQL queries
    pub fn validate(&self) -> (i64, i64) {
        // Ensure page is at least 1
        let page = self.page.max(MIN_PAGE);

        // Clamp limit between 1 and MAX_PAGE_SIZE
        let limit = self.limit.clamp(1, MAX_PAGE_SIZE);

        // Calculate offset (0-indexed for SQL)
        let offset = (page - 1) * limit;

        (limit, offset)
    }

    /// Get SQL LIMIT clause value
    pub fn limit(&self) -> i64 {
        self.validate().0
    }

    /// Get SQL OFFSET clause value
    pub fn offset(&self) -> i64 {
        self.validate().1
    }

    /// Get the current page number
    pub fn page(&self) -> i64 {
        self.page.max(MIN_PAGE)
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata about pagination state
#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    /// Current page number (1-indexed)
    pub page: i64,

    /// Items per page
    #[serde(rename = "pageSize")]
    pub page_size: i64,

    /// Total number of items across all pages
    #[serde(rename = "totalItems")]
    pub total_items: i64,

    /// Total number of pages
    #[serde(rename = "totalPages")]
    pub total_pages: i64,

    /// Whether there is a next page
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,

    /// Whether there is a previous page
    #[serde(rename = "hasPreviousPage")]
    pub has_previous_page: bool,
}

impl PaginationMeta {
    /// Create pagination metadata from params and total count
    pub fn new(params: &PaginationParams, total_items: i64) -> Self {
        let page = params.page();
        let page_size = params.limit();
        let total_pages = if page_size > 0 {
            (total_items + page_size - 1) / page_size
        } else {
            0
        };

        Self {
            page,
            page_size,
            total_items,
            total_pages,
            has_next_page: page < total_pages,
            has_previous_page: page > MIN_PAGE,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    /// The data items for the current page
    pub data: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, params: &PaginationParams, total_items: i64) -> Self {
        Self {
            data,
            pagination: PaginationMeta::new(params, total_items),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pagination_params() {
        let params = PaginationParams::default();
        assert_eq!(params.page(), 1);
        assert_eq!(params.limit(), DEFAULT_PAGE_SIZE);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn test_pagination_params_validation() {
        // Test negative page
        let params = PaginationParams::with_page_and_limit(-5, 10);
        assert_eq!(params.page(), 1);
        assert_eq!(params.offset(), 0);

        // Test zero page
        let params = PaginationParams::with_page_and_limit(0, 10);
        assert_eq!(params.page(), 1);
        assert_eq!(params.offset(), 0);

        // Test oversized limit
        let params = PaginationParams::with_page_and_limit(1, 200);
        assert_eq!(params.limit(), MAX_PAGE_SIZE);

        // Test negative limit
        let params = PaginationParams::with_page_and_limit(1, -5);
        assert_eq!(params.limit(), 1);
    }

    #[test]
    fn test_pagination_offset_calculation() {
        // Page 1
        let params = PaginationParams::with_page_and_limit(1, 20);
        assert_eq!(params.offset(), 0);

        // Page 2
        let params = PaginationParams::with_page_and_limit(2, 20);
        assert_eq!(params.offset(), 20);

        // Page 3 with limit 10
        let params = PaginationParams::with_page_and_limit(3, 10);
        assert_eq!(params.offset(), 20);
    }

    #[test]
    fn test_pagination_meta() {
        let params = PaginationParams::with_page_and_limit(1, 20);
        let meta = PaginationMeta::new(&params, 100);

        assert_eq!(meta.page, 1);
        assert_eq!(meta.page_size, 20);
        assert_eq!(meta.total_items, 100);
        assert_eq!(meta.total_pages, 5);
        assert!(meta.has_next_page);
        assert!(!meta.has_previous_page);
    }

    #[test]
    fn test_pagination_meta_last_page() {
        let params = PaginationParams::with_page_and_limit(5, 20);
        let meta = PaginationMeta::new(&params, 100);

        assert_eq!(meta.page, 5);
        assert!(!meta.has_next_page);
        assert!(meta.has_previous_page);
    }

    #[test]
    fn test_pagination_meta_partial_page() {
        let params = PaginationParams::with_page_and_limit(1, 20);
        let meta = PaginationMeta::new(&params, 15);

        assert_eq!(meta.total_pages, 1);
        assert!(!meta.has_next_page);
        assert!(!meta.has_previous_page);
    }

    #[test]
    fn test_paginated_response() {
        let data = vec!["item1".to_string(), "item2".to_string()];
        let params = PaginationParams::with_page_and_limit(1, 20);
        let response = PaginatedResponse::new(data.clone(), &params, 50);

        assert_eq!(response.data.len(), 2);
        assert_eq!(response.pagination.total_items, 50);
        assert_eq!(response.pagination.total_pages, 3);
    }
}
