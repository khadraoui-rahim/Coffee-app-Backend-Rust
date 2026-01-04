# Coffee Menu Backend API

A RESTful backend API for a coffee menu application built with Rust, Axum, and PostgreSQL.

## Features

- Full CRUD operations for coffee products
- RESTful API design with JSON responses
- PostgreSQL database with SQLx for type-safe queries
- Automated database migrations
- Docker containerization for easy deployment
- Comprehensive validation and error handling
- Property-based testing for correctness guarantees

## Technology Stack

- **Language**: Rust
- **Web Framework**: Axum 0.7
- **Database**: PostgreSQL 15+
- **Database Driver**: SQLx (compile-time checked queries)
- **Runtime**: Tokio (async)
- **Serialization**: Serde
- **Containerization**: Docker & Docker Compose

## Prerequisites

- Rust 1.75+ (for local development)
- Docker and Docker Compose (for containerized deployment)
- PostgreSQL 15+ (for local development without Docker)

## Quick Start with Docker

1. Clone the repository and navigate to the project directory:
   ```bash
   cd coffee_app-backend
   ```

2. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

3. Start the services with Docker Compose:
   ```bash
   docker-compose up --build
   ```

4. The API will be available at `http://localhost:8080`

## Local Development Setup

1. Install Rust from [rustup.rs](https://rustup.rs/)

2. Install PostgreSQL and create a database:
   ```bash
   createdb coffee_db
   ```

3. Copy and configure environment variables:
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials
   ```

4. Run database migrations:
   ```bash
   cargo install sqlx-cli
   sqlx migrate run
   ```

5. Build and run the application:
   ```bash
   cargo run
   ```

## API Endpoints

### Create Coffee
```bash
POST /api/coffees
Content-Type: application/json

{
  "name": "Caffe Mocha",
  "coffee_type": "Deep Foam",
  "price": 453,
  "rating": 4.8,
  "temperature": "hot",
  "description": "Rich chocolate and espresso blend",
  "size": "medium",
  "liked": false
}
```

### Get All Coffees
```bash
GET /api/coffees
```

### Get Coffee by ID
```bash
GET /api/coffees/{id}
```

### Update Coffee
```bash
PUT /api/coffees/{id}
Content-Type: application/json

{
  "name": "Updated Name",
  "price": 500
}
```

### Delete Coffee
```bash
DELETE /api/coffees/{id}
```

## Data Model

Coffee products include:
- `id`: Unique identifier (auto-generated)
- `name`: Coffee name
- `coffee_type`: Type of coffee (e.g., "Espresso", "Latte")
- `price`: Price in cents (integer)
- `rating`: Rating from 0.0 to 5.0
- `temperature`: "hot", "cold", or "both"
- `description`: Product description
- `size`: Size (e.g., "small", "medium", "large")
- `liked`: Boolean favorite status
- `created_at`: Timestamp (auto-generated)
- `updated_at`: Timestamp (auto-updated)

## Validation Rules

- **Price**: Must be greater than 0
- **Rating**: Must be between 0.0 and 5.0
- **Temperature**: Must be "hot", "cold", or "both"

## Development

### Running Tests
```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run property-based tests
cargo test property
```

### Building for Production
```bash
cargo build --release
```

### Database Migrations
```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## Project Structure

```
coffee_app-backend/
├── Cargo.toml              # Rust dependencies
├── Dockerfile              # Docker build configuration
├── docker-compose.yml      # Docker services configuration
├── README.md               # This file
├── .env.example            # Environment variables template
├── migrations/             # Database migrations
│   └── *.sql
├── src/                    # Source code
│   ├── main.rs            # Application entry point
│   ├── db.rs              # Database connection
│   ├── models.rs          # Data models
│   ├── handlers.rs        # HTTP handlers
│   ├── routes.rs          # Route configuration
│   └── error.rs           # Error handling
└── tests/                 # Test suites
    ├── unit/
    ├── property/
    └── integration/
```

## Error Responses

All errors return JSON with an `error` field:

```json
{
  "error": "Descriptive error message"
}
```

HTTP Status Codes:
- `200 OK`: Successful GET/PUT
- `201 Created`: Successful POST
- `204 No Content`: Successful DELETE
- `400 Bad Request`: Validation error
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server error

## License

This project is part of a coffee shop application suite.

## Contributing

This is a learning/demonstration project. Feel free to use it as a reference for building Rust APIs with Axum and PostgreSQL.
