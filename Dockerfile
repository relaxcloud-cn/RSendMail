# 第一阶段：依赖缓存层
FROM rust:1.86-slim as cacher
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock* ./
COPY crates/ ./crates/
RUN rm -f Cargo.lock && \
    # Create dummy source files to cache dependencies
    echo "fn main() {}" > crates/rsendmail-cli/src/main.rs && \
    echo "" > crates/rsendmail-core/src/lib.rs && \
    cargo fetch

# 第二阶段：构建层
FROM rust:1.86-slim as builder
WORKDIR /usr/src/app
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY Cargo.toml Cargo.lock* ./
COPY crates/ ./crates/
RUN rm -f Cargo.lock && \
    cargo build --release -p rsendmail-cli --offline

# 第三阶段：运行层
FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* && \
    useradd -r -s /bin/false rsendmail && \
    mkdir /data && \
    chown rsendmail:rsendmail /data

# 只复制必要的二进制文件
COPY --from=builder /usr/src/app/target/release/rsendmail /usr/local/bin/

USER rsendmail
WORKDIR /data
ENTRYPOINT ["rsendmail"]
