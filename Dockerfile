# Build stage
FROM rust:1.86-slim AS builder
WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy source code
COPY Cargo.toml ./
COPY crates/ ./crates/

# Build the CLI
RUN cargo build --release -p rsendmail-cli

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* && \
    useradd -r -s /bin/false rsendmail && \
    mkdir /data && \
    chown rsendmail:rsendmail /data

# Copy the binary
COPY --from=builder /usr/src/app/target/release/rsendmail-cli /usr/local/bin/rsendmail

USER rsendmail
WORKDIR /data
ENTRYPOINT ["rsendmail"]
