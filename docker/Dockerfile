# Use the official Rust image as the base image
FROM rust:1.87 as builder

WORKDIR /app

# Copy your project's Cargo.toml and Cargo.lock
COPY ./Cargo.toml ./Cargo.lock ./

# Copy your source code
COPY ./src ./src

# Build your application for release
RUN cargo build --release

# Create a new stage with a minimal image
FROM debian:stable-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the build artifact from the builder stage
COPY --from=builder /app/target/release/fusionauth-ignite /usr/local/bin/fusionauth-ignite

# Set the startup command to run your binary
CMD ["fusionauth-ignite"]
