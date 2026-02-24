use serde::Deserialize;

/// SQL query builder for constructing parameterized queries
/// Builds a single SQL query with filters, sorting, and pagination
pub struct SQLQueryBuilder {
    base_query: String,
    where_clauses: Vec<String>,
    params: Vec<String>,
    order_clause: Option<String>,
    limit: u32,
    offset: u32,
}

impl SQLQueryBuilder {
    /// Creates a new SQLQueryBuilder with default values
    pub fn new() -> Self {
        Self {
            base_query: "SELECT * FROM coffees".to_string(),
            where_clauses: Vec::new(),
            params: Vec::new(),
            order_clause: None,
            limit: 10,
            offset: 0,
        }
    }
    
    /// Adds a search filter for partial name matching (case-insensitive)
    /// Uses ILIKE for PostgreSQL case-insensitive pattern matching
    pub fn add_search_filter(&mut self, search: &str) {
        let param_index = self.params.len() + 1;
        self.where_clauses.push(format!("name ILIKE ${}", param_index));
        self.params.push(format!("%{}%", search));
    }
    
    /// Adds a type filter for exact type matching (case-insensitive)
    /// Uses ILIKE for PostgreSQL case-insensitive matching
    pub fn add_type_filter(&mut self, type_val: &str) {
        let param_index = self.params.len() + 1;
        self.where_clauses.push(format!("coffee_type ILIKE ${}", param_index));
        self.params.push(type_val.to_string());
    }
    
    /// Adds price range filters (min and/or max)
    /// Both bounds are inclusive
    pub fn add_price_range(&mut self, min: Option<f64>, max: Option<f64>) {
        if let Some(min_price) = min {
            let param_index = self.params.len() + 1;
            self.where_clauses.push(format!("price >= ${}", param_index));
            self.params.push(min_price.to_string());
        }
        
        if let Some(max_price) = max {
            let param_index = self.params.len() + 1;
            self.where_clauses.push(format!("price <= ${}", param_index));
            self.params.push(max_price.to_string());
        }
    }
    
    /// Sets the sort order for the query
    /// Adds an ORDER BY clause with the specified field and order
    pub fn set_sort(&mut self, field: SortField, order: SortOrder) {
        let field_name = match field {
            SortField::Price => "price",
            SortField::Rating => "rating",
        };
        
        let order_str = match order {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        };
        
        self.order_clause = Some(format!("{} {}", field_name, order_str));
    }
    
    /// Sets pagination parameters
    /// Calculates LIMIT and OFFSET based on page number and limit
    pub fn set_pagination(&mut self, page: u32, limit: u32) {
        self.limit = limit;
        self.offset = (page - 1) * limit;
    }
    
    /// Builds the final SQL query string with all parameters
    /// Returns a tuple of (query_string, parameters)
    pub fn build(&self) -> (String, Vec<String>) {
        let mut query = self.base_query.clone();
        
        // Add WHERE clauses if any filters were added
        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }
        
        // Add ORDER BY clause if sorting was specified
        if let Some(ref order) = self.order_clause {
            query.push_str(" ORDER BY ");
            query.push_str(order);
        }
        
        // Add LIMIT and OFFSET for pagination directly (not as bound parameters)
        // PostgreSQL requires these to be integers, not text
        query.push_str(&format!(" LIMIT {}", self.limit));
        query.push_str(&format!(" OFFSET {}", self.offset));
        
        // Return only the filter parameters (not limit/offset)
        (query, self.params.clone())
    }
}

/// Query parameters extracted from HTTP request
/// All fields are optional to support flexible querying
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    /// Search term for partial name matching (case-insensitive)
    pub search: Option<String>,
    /// Filter by coffee type (case-insensitive exact match)
    pub type_filter: Option<String>,
    /// Minimum price filter (inclusive)
    pub min_price: Option<f64>,
    /// Maximum price filter (inclusive)
    pub max_price: Option<f64>,
    /// Sort field: "price" or "rating"
    pub sort: Option<String>,
    /// Sort order: "asc" or "desc"
    pub order: Option<String>,
    /// Page number (1-indexed, defaults to 1)
    pub page: Option<u32>,
    /// Items per page (defaults to 10)
    pub limit: Option<u32>,
}

/// Sort field options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Price,
    Rating,
}

/// Sort order options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Validated and normalized query parameters
/// All validation rules have been applied and defaults set
#[derive(Debug)]
pub struct ValidatedQuery {
    /// Normalized search term (trimmed, None if empty)
    pub search: Option<String>,
    /// Normalized type filter (trimmed, None if empty)
    pub type_filter: Option<String>,
    /// Minimum price filter (validated as positive)
    pub min_price: Option<f64>,
    /// Maximum price filter (validated as positive and >= min_price)
    pub max_price: Option<f64>,
    /// Sort field (None means no sorting)
    pub sort_field: Option<SortField>,
    /// Sort order (defaults based on sort field)
    pub sort_order: SortOrder,
    /// Page number (validated as positive, defaults to 1)
    pub page: u32,
    /// Items per page (validated as positive, defaults to 10)
    pub limit: u32,
}

/// Validation error type
#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Query parameter validator
pub struct QueryValidator;

impl QueryValidator {
    /// Validates and normalizes query parameters
    /// Returns ValidatedQuery on success or ValidationError on failure
    pub fn validate(params: QueryParams) -> Result<ValidatedQuery, ValidationError> {
        // Validate and normalize search parameter
        let search = Self::normalize_string(params.search);
        
        // Validate and normalize type_filter parameter
        let type_filter = Self::normalize_string(params.type_filter);
        
        // Validate price parameters
        let min_price = if let Some(price) = params.min_price {
            Self::validate_price(price, "min_price")?;
            Some(price)
        } else {
            None
        };
        
        let max_price = if let Some(price) = params.max_price {
            Self::validate_price(price, "max_price")?;
            Some(price)
        } else {
            None
        };
        
        // Validate min_price <= max_price
        if let (Some(min), Some(max)) = (min_price, max_price) {
            if min > max {
                return Err(ValidationError {
                    message: "min_price cannot be greater than max_price".to_string(),
                });
            }
        }
        
        // Validate and map sort field
        let sort_field = if let Some(sort_str) = params.sort {
            Some(Self::parse_sort_field(&sort_str)?)
        } else {
            None
        };
        
        // Validate and map sort order, applying defaults based on sort field
        let sort_order = if let Some(order_str) = params.order {
            Self::parse_sort_order(&order_str)?
        } else {
            // Default order depends on sort field
            match sort_field {
                Some(SortField::Price) => SortOrder::Asc,
                Some(SortField::Rating) => SortOrder::Desc,
                None => SortOrder::Asc, // Default when no sort specified
            }
        };
        
        // Validate pagination parameters
        let page = if let Some(p) = params.page {
            Self::validate_pagination_param(p, "page")?;
            p
        } else {
            1 // Default page
        };
        
        let limit = if let Some(l) = params.limit {
            Self::validate_pagination_param(l, "limit")?;
            l
        } else {
            10 // Default limit
        };
        
        Ok(ValidatedQuery {
            search,
            type_filter,
            min_price,
            max_price,
            sort_field,
            sort_order,
            page,
            limit,
        })
    }
    
    /// Normalizes string parameters by trimming whitespace
    /// Returns None if the string is empty or whitespace-only
    fn normalize_string(s: Option<String>) -> Option<String> {
        s.and_then(|s| {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
    }
    
    /// Validates that a price is positive (not negative or zero)
    fn validate_price(price: f64, param_name: &str) -> Result<(), ValidationError> {
        if price <= 0.0 {
            return Err(ValidationError {
                message: format!("{} must be a positive number", param_name),
            });
        }
        if price.is_nan() || price.is_infinite() {
            return Err(ValidationError {
                message: format!("{} must be a valid number", param_name),
            });
        }
        Ok(())
    }
    
    /// Parses sort field string to SortField enum
    fn parse_sort_field(s: &str) -> Result<SortField, ValidationError> {
        match s.to_lowercase().as_str() {
            "price" => Ok(SortField::Price),
            "rating" => Ok(SortField::Rating),
            _ => Err(ValidationError {
                message: format!("Invalid sort field '{}'. Must be 'price' or 'rating'", s),
            }),
        }
    }
    
    /// Parses sort order string to SortOrder enum
    fn parse_sort_order(s: &str) -> Result<SortOrder, ValidationError> {
        match s.to_lowercase().as_str() {
            "asc" => Ok(SortOrder::Asc),
            "desc" => Ok(SortOrder::Desc),
            _ => Err(ValidationError {
                message: format!("Invalid sort order '{}'. Must be 'asc' or 'desc'", s),
            }),
        }
    }
    
    /// Validates pagination parameters (page and limit)
    /// Must be positive (not zero or negative)
    fn validate_pagination_param(value: u32, param_name: &str) -> Result<(), ValidationError> {
        if value == 0 {
            return Err(ValidationError {
                message: format!("{} must be a positive number (greater than 0)", param_name),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_builder_basic_query() {
        let builder = SQLQueryBuilder::new();
        let (query, params) = builder.build();
        
        assert!(query.contains("SELECT * FROM coffees"));
        assert!(query.contains("LIMIT"));
        assert!(query.contains("OFFSET"));
        assert_eq!(params.len(), 0); // No filter parameters, limit/offset are in query string
    }

    #[test]
    fn test_sql_builder_with_search() {
        let mut builder = SQLQueryBuilder::new();
        builder.add_search_filter("espresso");
        let (query, params) = builder.build();
        
        assert!(query.contains("WHERE"));
        assert!(query.contains("name ILIKE $1"));
        assert_eq!(params[0], "%espresso%");
    }

    #[test]
    fn test_sql_builder_with_type_filter() {
        let mut builder = SQLQueryBuilder::new();
        builder.add_type_filter("latte");
        let (query, params) = builder.build();
        
        assert!(query.contains("WHERE"));
        assert!(query.contains("coffee_type ILIKE $1"));
        assert_eq!(params[0], "latte");
    }

    #[test]
    fn test_sql_builder_with_price_range() {
        let mut builder = SQLQueryBuilder::new();
        builder.add_price_range(Some(5.0), Some(10.0));
        let (query, params) = builder.build();
        
        assert!(query.contains("WHERE"));
        assert!(query.contains("price >= $1"));
        assert!(query.contains("price <= $2"));
        assert_eq!(params[0], "5");
        assert_eq!(params[1], "10");
    }

    #[test]
    fn test_sql_builder_with_sorting() {
        let mut builder = SQLQueryBuilder::new();
        builder.set_sort(SortField::Price, SortOrder::Asc);
        let (query, _) = builder.build();
        
        assert!(query.contains("ORDER BY price ASC"));
    }

    #[test]
    fn test_sql_builder_with_pagination() {
        let mut builder = SQLQueryBuilder::new();
        builder.set_pagination(2, 20);
        let (query, _params) = builder.build();
        
        assert!(query.contains("LIMIT 20"));
        assert!(query.contains("OFFSET 20")); // page 2 * 20 items = 20
    }

    #[test]
    fn test_sql_builder_combined_filters() {
        let mut builder = SQLQueryBuilder::new();
        builder.add_search_filter("coffee");
        builder.add_type_filter("espresso");
        builder.add_price_range(Some(3.0), Some(8.0));
        builder.set_sort(SortField::Rating, SortOrder::Desc);
        builder.set_pagination(1, 10);
        
        let (query, params) = builder.build();
        
        assert!(query.contains("WHERE"));
        assert!(query.contains("name ILIKE $1"));
        assert!(query.contains("AND"));
        assert!(query.contains("coffee_type ILIKE $2"));
        assert!(query.contains("price >= $3"));
        assert!(query.contains("price <= $4"));
        assert!(query.contains("ORDER BY rating DESC"));
        assert!(query.contains("LIMIT"));
        assert!(query.contains("OFFSET"));
        
        assert_eq!(params[0], "%coffee%");
        assert_eq!(params[1], "espresso");
        assert_eq!(params[2], "3");
        assert_eq!(params[3], "8");
    }

    #[test]
    fn test_normalize_string_with_whitespace() {
        assert_eq!(
            QueryValidator::normalize_string(Some("  test  ".to_string())),
            Some("test".to_string())
        );
    }

    #[test]
    fn test_normalize_string_empty() {
        assert_eq!(
            QueryValidator::normalize_string(Some("   ".to_string())),
            None
        );
    }

    #[test]
    fn test_normalize_string_none() {
        assert_eq!(QueryValidator::normalize_string(None), None);
    }

    #[test]
    fn test_validate_price_positive() {
        assert!(QueryValidator::validate_price(10.0, "price").is_ok());
    }

    #[test]
    fn test_validate_price_zero() {
        assert!(QueryValidator::validate_price(0.0, "price").is_err());
    }

    #[test]
    fn test_validate_price_negative() {
        assert!(QueryValidator::validate_price(-5.0, "price").is_err());
    }

    #[test]
    fn test_parse_sort_field_valid() {
        assert_eq!(
            QueryValidator::parse_sort_field("price").unwrap(),
            SortField::Price
        );
        assert_eq!(
            QueryValidator::parse_sort_field("RATING").unwrap(),
            SortField::Rating
        );
    }

    #[test]
    fn test_parse_sort_field_invalid() {
        assert!(QueryValidator::parse_sort_field("invalid").is_err());
    }

    #[test]
    fn test_parse_sort_order_valid() {
        assert_eq!(
            QueryValidator::parse_sort_order("asc").unwrap(),
            SortOrder::Asc
        );
        assert_eq!(
            QueryValidator::parse_sort_order("DESC").unwrap(),
            SortOrder::Desc
        );
    }

    #[test]
    fn test_parse_sort_order_invalid() {
        assert!(QueryValidator::parse_sort_order("invalid").is_err());
    }

    #[test]
    fn test_validate_pagination_param_valid() {
        assert!(QueryValidator::validate_pagination_param(1, "page").is_ok());
        assert!(QueryValidator::validate_pagination_param(100, "limit").is_ok());
    }

    #[test]
    fn test_validate_pagination_param_zero() {
        assert!(QueryValidator::validate_pagination_param(0, "page").is_err());
    }

    #[test]
    fn test_validate_full_query_with_defaults() {
        let params = QueryParams {
            search: None,
            type_filter: None,
            min_price: None,
            max_price: None,
            sort: None,
            order: None,
            page: None,
            limit: None,
        };

        let validated = QueryValidator::validate(params).unwrap();
        assert_eq!(validated.page, 1);
        assert_eq!(validated.limit, 10);
        assert_eq!(validated.sort_order, SortOrder::Asc);
    }

    #[test]
    fn test_validate_price_range_valid() {
        let params = QueryParams {
            search: None,
            type_filter: None,
            min_price: Some(5.0),
            max_price: Some(10.0),
            sort: None,
            order: None,
            page: None,
            limit: None,
        };

        let validated = QueryValidator::validate(params).unwrap();
        assert_eq!(validated.min_price, Some(5.0));
        assert_eq!(validated.max_price, Some(10.0));
    }

    #[test]
    fn test_validate_price_range_invalid() {
        let params = QueryParams {
            search: None,
            type_filter: None,
            min_price: Some(10.0),
            max_price: Some(5.0),
            sort: None,
            order: None,
            page: None,
            limit: None,
        };

        assert!(QueryValidator::validate(params).is_err());
    }

    #[test]
    fn test_validate_sort_defaults() {
        // Price sort defaults to ascending
        let params = QueryParams {
            search: None,
            type_filter: None,
            min_price: None,
            max_price: None,
            sort: Some("price".to_string()),
            order: None,
            page: None,
            limit: None,
        };

        let validated = QueryValidator::validate(params).unwrap();
        assert_eq!(validated.sort_field, Some(SortField::Price));
        assert_eq!(validated.sort_order, SortOrder::Asc);

        // Rating sort defaults to descending
        let params = QueryParams {
            search: None,
            type_filter: None,
            min_price: None,
            max_price: None,
            sort: Some("rating".to_string()),
            order: None,
            page: None,
            limit: None,
        };

        let validated = QueryValidator::validate(params).unwrap();
        assert_eq!(validated.sort_field, Some(SortField::Rating));
        assert_eq!(validated.sort_order, SortOrder::Desc);
    }
}
