# ---- Build Stage ----
    FROM rust:1.83 as builder

    WORKDIR /app
    
    # Copy manifests first (for caching)
    COPY Cargo.toml Cargo.lock ./
    COPY src ./src
    
    # Build the app in release mode
    RUN cargo build --release
    
    # ---- Runtime Stage ----
    FROM debian:bookworm-slim
    
    WORKDIR /app
    
    # Install OpenSSL runtime (for any HTTP libs or TLS needs)
    RUN apt-get update && \
        apt-get install -y libssl3 ca-certificates && \
        apt-get clean && \
        rm -rf /var/lib/apt/lists/*
    
    # Copy the compiled binary from builder
    COPY --from=builder /app/target/release/hng13-stage1-string-analyzer-service .
    
    # Expose the port (Railway will map this automatically)
    EXPOSE 8080
    
    # Run the binary
    CMD ["./hng13-stage1-string-analyzer-service"]
    