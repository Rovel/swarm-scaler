# First stage: Build the Rust binary
FROM rust:1.70 as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Pre-build dependencies to cache them
RUN cargo fetch

# Copy the rest of the application source code
COPY . .

# Build the application in release mode
RUN cargo build --release

# Second stage: Create a minimal image with Alpine
FROM alpine:3.18

# Install necessary dependencies to run the Rust binary
RUN apk add --no-cache libc6-compat

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/rust_scaler /usr/local/bin/rust_scaler

# Expose the UDP port for peer-to-peer communication
EXPOSE 4000/udp

# Run the application
CMD ["rust_scaler"]
