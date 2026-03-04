use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    pub page: u32,
    
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    pub limit: u32,
    
    pub cursor: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

impl PaginationQuery {
    pub fn validate_and_normalize(&self) -> ValidatedPagination {
        let limit = if self.limit > 100 { 100 } else { self.limit };
        let page = if self.page < 1 { 1 } else { self.page };
        let offset = (page - 1) * limit;
        
        ValidatedPagination {
            page,
            limit,
            offset,
            cursor: self.cursor.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedPagination {
    pub page: u32,
    pub limit: u32,
    pub offset: u32,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl PaginationMeta {
    pub fn new(total: u64, page: u32, limit: u32) -> Self {
        let total_pages = if limit > 0 {
            ((total as f64) / (limit as f64)).ceil() as u32
        } else {
            0
        };
        
        let has_next = page < total_pages;
        let has_prev = page > 1;
        
        Self {
            total,
            page,
            limit,
            total_pages,
            has_next,
            has_prev,
            next_cursor: None,
        }
    }
    
    pub fn with_cursor(mut self, cursor: Option<String>) -> Self {
        self.next_cursor = cursor;
        self
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: u64, page: u32, limit: u32) -> Self {
        Self {
            data,
            pagination: PaginationMeta::new(total, page, limit),
        }
    }
    
    pub fn with_cursor(data: Vec<T>, total: u64, page: u32, limit: u32, cursor: Option<String>) -> Self {
        Self {
            data,
            pagination: PaginationMeta::new(total, page, limit).with_cursor(cursor),
        }
    }
}

// Helper functions for offset-based pagination
pub fn calculate_offset(page: u32, limit: u32) -> u32 {
    (page - 1) * limit
}

pub fn build_pagination_meta(total: u64, page: u32, limit: u32) -> PaginationMeta {
    PaginationMeta::new(total, page, limit)
}

// Helper functions for cursor-based pagination
pub fn build_cursor_where_clause(cursor: &Option<String>, column: &str) -> String {
    match cursor {
        Some(c) => format!("{} > '{}'", column, c),
        None => "1=1".to_string(),
    }
}

pub fn generate_next_cursor<T>(results: &[T], get_id: impl Fn(&T) -> String) -> Option<String> {
    results.last().map(|item| get_id(item))
}
