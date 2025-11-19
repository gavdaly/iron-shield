# Stage 1: Build the backend and frontend
FROM rust:1.76 AS builder

WORKDIR /app

# Copy over the manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Copy over the source code
COPY ./src ./src
COPY ./build.rs ./build.rs
COPY ./frontend ./frontend
COPY ./templates ./templates

# Build the backend and frontend
RUN cargo build --release

# Stage 2: Create the final image
FROM debian:bullseye-slim
WORKDIR /app

# Copy the backend binary
COPY --from=builder /app/target/release/iron_shield .

# Copy the frontend assets
COPY --from=builder /app/frontend/dist ./frontend/dist

# Copy templates
COPY --from=builder /app/templates ./templates

# Set the path to the templates
ENV TEMPLATES_DIR=/app/templates

# Expose the port
EXPOSE 3000

# Run the app
CMD ["/app/iron_shield"]
