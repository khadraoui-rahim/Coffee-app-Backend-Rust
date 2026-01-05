# Build stage
FROM rust:latest as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage - use rust image to allow testing
FROM rust:latest

WORKDIR /app

# Install PostgreSQL client libraries
RUN apt-get update && \
    apt-get install -y libpq-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy everything for testing
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations

# Copy the binary from builder
COPY --from=builder /app/target/release/coffee-api /usr/local/bin/coffee-api

# Expose port
EXPOSE 8080

# Run the binary
CMD ["coffee-api"]
